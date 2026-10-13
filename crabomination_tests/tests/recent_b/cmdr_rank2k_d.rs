//! Commander: EDHREC ranks 1501-2000, fourth batch (`decks::cmdr_rank2k_d`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::ActivateAbility { card_id: id, ability_index, target, additional_targets: vec![], x_value: None, mode: None })
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    act(g, GameAction::DeclareAttackers(attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect()))
        .expect("attack");
}

/// Elfhame Druid's {G}{G} pays only for a kicked spell.
#[test]
fn elfhame_druid_mana_is_for_kicked_spells() {
    let mut g = pod(2);
    let druid = ready(&mut g, 0, catalog::elfhame_druid());
    activate(&mut g, druid, 1, None).expect("tap for {G}{G}");
    g.players[0].mana_pool.add(Color::Red, 2);
    let charger = g.add_card_to_hand(0, catalog::shatterskull_charger());
    assert!(cast(&mut g, charger, None).is_err(), "an unkicked spell can't use it");
    g.players[0].mana_pool.add_colorless(1);
    act(
        &mut g,
        GameAction::CastSpellKicked { card_id: charger, target: None, additional_targets: vec![], mode: None, x_value: None },
    )
    .expect("{1}{R}{R} + kicker {2}: the Druid's {G}{G} covers the generic");
    assert_eq!(g.battlefield_find(charger).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "kicked");
}

/// Shatterskull Charger goes back to hand at your end step without a +1/+1
/// counter; Archwing Dragon goes back at every end step.
#[test]
fn shatterskull_charger_and_archwing_dragon_return_at_end_step() {
    let mut g = pod(2);
    flood(&mut g);
    let charger = g.add_card_to_hand(0, catalog::shatterskull_charger());
    cast(&mut g, charger, None).expect("cast");
    let dragon = ready(&mut g, 0, catalog::archwing_dragon());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    for id in [charger, dragon] {
        assert!(g.players[0].hand.iter().any(|c| c.id == id), "back in hand");
    }
}

/// Rally for the Throne's adamant gains a life per creature — only with three
/// white spent.
#[test]
fn rally_for_the_throne_adamant() {
    for (white, want) in [(3, 2), (1, 0)] {
        let mut g = pod(2);
        let life = g.players[0].life;
        let rally = g.add_card_to_hand(0, catalog::rally_for_the_throne());
        g.players[0].mana_pool.add(Color::White, white);
        g.players[0].mana_pool.add_colorless(3 - white);
        cast(&mut g, rally, None).expect("cast");
        assert_eq!(g.players[0].life - life, want);
    }
}

/// Repel the Abominable stops a non-Human's damage, not a Human's.
#[test]
fn repel_the_abominable_spares_humans() {
    let mut g = pod(2);
    flood(&mut g);
    let repel = g.add_card_to_hand(0, catalog::repel_the_abominable());
    cast(&mut g, repel, None).expect("cast");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life = g.players[1].life;
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[1].life, life, "an instant is a non-Human source");
}

/// Arwen spends her indestructible counter: the target and Arwen each get a
/// +1/+1 and a lifelink counter.
#[test]
fn arwen_shares_her_counters() {
    let mut g = pod(2);
    flood(&mut g);
    let arwen = g.add_card_to_hand(0, catalog::arwen_mortal_queen());
    cast(&mut g, arwen, None).expect("cast");
    assert!(g.computed_permanent(arwen).unwrap().keywords().contains(&Keyword::Indestructible));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, arwen, 0, Some(Target::Permanent(bear))).expect("activate");
    for id in [arwen, bear] {
        let cp = g.computed_permanent(id).unwrap();
        assert!(cp.keywords().contains(&Keyword::Lifelink));
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }
    assert!(!g.computed_permanent(arwen).unwrap().keywords().contains(&Keyword::Indestructible), "the counter is spent");
    assert!(activate(&mut g, arwen, 0, Some(Target::Permanent(bear))).is_err());
}

/// Immard banks a charge counter, then spends it for one of the two modes.
#[test]
fn immard_charges_then_fires() {
    let mut g = pod(2);
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let immard = g.add_card_to_hand(0, catalog::immard_the_stormcleaver());
    cast(&mut g, immard, None).expect("cast");
    assert_eq!(g.battlefield_find(immard).unwrap().counter_count(CounterType::Charge), 1);
    g.clear_sickness(immard);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    attack(&mut g, &[immard]);
    assert_eq!(g.battlefield_find(immard).unwrap().counter_count(CounterType::Charge), 0);
    let cp = g.computed_permanent(immard).unwrap();
    let kw = cp.keywords();
    let shielded = kw.contains(&Keyword::Lifelink) && kw.contains(&Keyword::Indestructible);
    assert!(g.players[1].life == life - 4 || shielded, "removing the counter chose a mode");
}

/// Jolrael animates a land as an X/X flier, X = your hand size.
#[test]
fn jolrael_animates_a_land_by_hand_size() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::jolrael_voice_of_zhalfir());
    let land = ready(&mut g, 0, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords().contains(&Keyword::Flying));
}

/// Return of the Mole Man makes one Moloid per permanent card in your
/// graveyard.
#[test]
fn return_of_the_mole_man_counts_permanent_cards() {
    let mut g = pod(2);
    flood(&mut g);
    let mole = ready(&mut g, 0, catalog::return_of_the_mole_man());
    for def in [catalog::grizzly_bears(), catalog::forest(), catalog::lightning_bolt()] {
        g.add_card_to_graveyard(0, def);
    }
    activate(&mut g, mole, 0, None).expect("activate");
    let moloids = g.battlefield.iter().filter(|c| c.definition.name == "Moloid").count();
    assert_eq!(moloids, 3, "Bears, Forest and the sacrificed enchantment itself");
}

/// Templar Knight needs five attacking Templar Knights to tap.
#[test]
fn templar_knight_taps_five_attacking_knights() {
    let mut g = pod(2);
    let knights: Vec<_> = (0..5).map(|_| ready(&mut g, 0, catalog::templar_knight())).collect();
    let crown = g.add_card_to_library(0, catalog::thran_temporal_gateway());
    flood(&mut g);
    assert!(activate(&mut g, knights[0], 0, None).is_err(), "not attacking");
    attack(&mut g, &knights);
    flood(&mut g);
    activate(&mut g, knights[0], 0, None).expect("five attackers");
    assert!(g.battlefield_find(crown).is_some());
}

/// Viconia returns one creature card at random once four are in your
/// graveyard.
#[test]
fn viconia_returns_a_random_creature_card() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::viconia_drow_apostate());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "three is not enough");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}
