//! Commander: the Forces of the Imperium precon (40K, Inquisitor Greyfax,
//! `decks::cmdr_greyfax`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// CR 614 / 104.3 — The Golden Throne replaces a loss: it is exiled and
/// your life becomes 1; the shield is spent.
#[test]
fn the_golden_throne_saves_you_once() {
    let mut g = main_phase(2);
    let throne = g.add_card_to_battlefield(0, catalog::the_golden_throne());
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[0].eliminated);
    assert_eq!(g.players[0].life, 1);
    assert!(g.battlefield_find(throne).is_none());
    assert!(g.exile.iter().any(|c| c.id == throne));
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "no second save");
}

/// Birth of the Imperium: an Astartes per opponent, then two cards for each
/// opponent with fewer creatures than you.
#[test]
fn birth_of_the_imperium_counts_opponents() {
    let mut g = main_phase(4);
    library(&mut g, 0, 10);
    let saga = g.add_card_to_hand(0, catalog::birth_of_the_imperium());
    cast(&mut g, saga, None).expect("cast");
    assert_eq!(count(&g, 0, "Astartes Warrior"), 3, "one per opponent");
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    let eff = catalog::birth_of_the_imperium().saga_chapters[2].1.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let _ = g.resolve_effect(&eff, &ctx);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 4, "two opponents have fewer than three creatures");
}

/// Greyfax: +1/+0 and vigilance to the others; {1}, {T} taps and
/// investigates.
#[test]
fn inquisitor_greyfax_hunts_heresy() {
    let mut g = main_phase(2);
    let greyfax = g.add_card_to_battlefield(0, catalog::inquisitor_greyfax());
    g.clear_sickness(greyfax);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (3, 2));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Vigilance));
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, greyfax, 0, Some(Target::Permanent(foe))).expect("{1}, {T}");
    assert!(g.battlefield_find(foe).unwrap().tapped);
    assert_eq!(count(&g, 0, "Clue"), 1);
}

/// Assault Intercessor: an opponent's creature dying costs its controller 2.
#[test]
fn assault_intercessor_chainsword() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::assault_intercessor());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(foe))).expect("bolt");
    assert_eq!(g.players[1].life, 18);
}

/// Deny the Witch counters a spell and drains its caster by your creature
/// count.
#[test]
fn deny_the_witch_counters_and_drains() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("opponent bolts");
    let deny = g.add_card_to_hand(0, catalog::deny_the_witch());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: deny, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None })
        .expect("deny");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "countered");
    assert_eq!(g.players[1].life, 17);
}

/// Exterminatus strips opponents' indestructible, then destroys all nonland
/// permanents.
#[test]
fn exterminatus_leaves_nothing() {
    let mut g = main_phase(2);
    let foe = g.add_card_to_battlefield(1, catalog::darksteel_brute());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ex = g.add_card_to_hand(0, catalog::exterminatus());
    cast(&mut g, ex, None).expect("cast");
    assert!(g.battlefield_find(foe).is_none(), "indestructible stripped");
    assert_eq!(count(&g, 0, "Grizzly Bears"), 0);
}

/// Marneus Calgar draws once for a batch of tokens (Company Commander's
/// Soldiers at a four-seat table).
#[test]
fn marneus_calgar_draws_per_batch() {
    let mut g = main_phase(4);
    library(&mut g, 0, 5);
    g.add_card_to_battlefield(0, catalog::marneus_calgar());
    let hand = g.players[0].hand.len();
    let cc = g.add_card_to_hand(0, catalog::company_commander());
    cast(&mut g, cc, None).expect("cast");
    assert_eq!(count(&g, 0, "Soldier"), 3);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the batch of three");
}

/// Belisarius Cawl taps two artifacts for an Astartes.
#[test]
fn belisarius_cawl_ultima_founding() {
    let mut g = main_phase(2);
    let cawl = g.add_card_to_battlefield(0, catalog::belisarius_cawl());
    g.clear_sickness(cawl);
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    activate(&mut g, cawl, 0, None).expect("tap two artifacts");
    assert_eq!(count(&g, 0, "Astartes Warrior"), 1);
}

/// The Flesh Is Weak: your countered creatures become artifacts and dodge
/// the -1/-1 every nonartifact creature takes. CR 613.8 — the layer-7c set
/// reads the layer-4 type change; a compound filter read printed types.
#[test]
fn the_flesh_is_weak() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flesh = g.add_card_to_hand(0, catalog::the_flesh_is_weak());
    cast(&mut g, flesh, None).expect("cast");
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(pt(&g, mine), (3, 3), "an artifact now");
    assert_eq!(pt(&g, foe), (1, 1));
}

/// Commissar Severina Raine drains each opponent by the other attackers.
#[test]
fn commissar_leads_from_the_front() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase(2);
    let raine = g.add_card_to_battlefield(0, catalog::commissar_severina_raine());
    let bears: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    let mut attacks = vec![Attack { attacker: raine, target: AttackTarget::Player(1) }];
    attacks.extend(bears.iter().map(|&b| Attack { attacker: b, target: AttackTarget::Player(1) }));
    for a in &attacks {
        g.clear_sickness(a.attacker);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Sister Hospitaller reanimates and gains the mana value; Celestine returns
/// a creature up to the life gained at your end step.
#[test]
fn sister_hospitaller_and_celestine() {
    let mut g = main_phase(2);
    let wurm = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let sister = g.add_card_to_hand(0, catalog::sister_hospitaller());
    cast(&mut g, sister, Some(Target::Permanent(wurm))).expect("cast");
    assert!(g.battlefield_find(wurm).is_some());
    assert_eq!(g.players[0].life, 22);
    g.add_card_to_battlefield(0, catalog::celestine_the_living_saint());
    let back = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(back).is_some(), "mana value 2, two life gained");
}

/// The Vehicles' entry and attack burn (table-driven over entry).
#[test]
fn knight_paladin_and_thunderhawk_enter() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::knight_paladin());
    let kp = g.add_card_to_hand(0, catalog::knight_paladin());
    cast(&mut g, kp, None).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (16, 16));
    let th = g.add_card_to_hand(0, catalog::thunderhawk_gunship());
    cast(&mut g, th, None).expect("cast");
    assert_eq!(count(&g, 0, "Astartes Warrior"), 2);
}

/// Neyam Shai Murad trades with the player it damaged: another opponent's
/// graveyard is out of reach.
#[test]
fn neyam_trades_only_with_the_damaged_player() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase(3);
    let neyam = g.add_card_to_battlefield(0, catalog::neyam_shai_murad());
    g.clear_sickness(neyam);
    let theirs = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: neyam, target: AttackTarget::Player(1) }]))
        .expect("attack seat 1");
    while g.step != TurnStep::EndCombat && !g.is_game_over() {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 17, "Neyam connected");
    assert!(g.players[2].graveyard.iter().any(|c| c.id == theirs), "seat 2's card stays put");
}
