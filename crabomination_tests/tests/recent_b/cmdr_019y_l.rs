//! Commander: the 2026-10-17 (`019Y6J4R`) twelfth closer batch
//! (`decks::cmdr_019y_l`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::card::{Keyword, Value};

fn resolve(g: &mut GameState, e: Effect) {
    let evs = g.resolve_effect(&e, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// CR 701.12a — Struggle for Skemfar grows your creature, then it fights the
/// second target.
#[test]
fn struggle_for_skemfar_counters_then_fights() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let sfs = g.add_card_to_hand(0, catalog::struggle_for_skemfar());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: sfs,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "the 3/3 killed the 2/2");
    assert!(g.battlefield_find(mine).is_some());
}

/// Shepherd of the Cosmos returns a two-mana permanent card.
#[test]
fn shepherd_of_the_cosmos_returns_a_cheap_permanent() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let sc = g.add_card_to_hand(0, catalog::shepherd_of_the_cosmos());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: sc, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// Infernal Sovereign: casting a spell draws a card and costs 1 life.
#[test]
fn infernal_sovereign_draws_on_each_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::infernal_sovereign());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[0].hand.len(), 1, "the bolt left, a card came");
    assert_eq!(g.players[0].life, 19);
}

/// Demonic Collusion without buyback: tutor, then to the graveyard.
#[test]
fn demonic_collusion_tutors() {
    let mut g = pod(2);
    let dc = g.add_card_to_hand(0, catalog::demonic_collusion());
    flood(&mut g, 0);
    cast(&mut g, dc, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == dc));
}

/// You Hear Something on Watch's second mode hits an attacker for 5.
#[test]
fn you_hear_something_on_watch_springs_the_trap() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    let y = g.add_card_to_hand(0, catalog::you_hear_something_on_watch());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: y,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
}

/// Sigil of Myrkul mills, then — with four creature cards in the graveyard —
/// grows and grants deathtouch (CR 603.12 reflexive trigger).
#[test]
fn sigil_of_myrkul_rewards_a_full_graveyard() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sigil_of_myrkul());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::hill_giant());
    }
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::hill_giant());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Deathtouch));
}

/// Graven Lore draws three (no snow mana: scry 0).
#[test]
fn graven_lore_draws_three() {
    let mut g = pod(2);
    let gl = g.add_card_to_hand(0, catalog::graven_lore());
    flood(&mut g, 0);
    cast(&mut g, gl, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), 3);
}

/// Wistfulness cast off green mana exiles an opposing artifact (the {G}{G}
/// trigger; no {U}{U} was spent, so no loot).
#[test]
fn wistfulness_green_exiles_an_artifact() {
    let mut g = pod(2);
    let art = ready(&mut g, 1, catalog::lotus_ring());
    let wf = g.add_card_to_hand(0, catalog::wistfulness());
    g.players[0].mana_pool.add(Color::Green, 5);
    cast(&mut g, wf, Some(Target::Permanent(art))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == art));
    assert!(g.players[0].hand.is_empty(), "no loot");
}

/// Virtus's Maneuver: the caster is a friend, the opponent a foe.
#[test]
fn virtuss_maneuver_regrows_and_edicts() {
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let vm = g.add_card_to_hand(0, catalog::virtuss_maneuver());
    flood(&mut g, 0);
    cast(&mut g, vm, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == dead));
    assert!(g.battlefield_find(bear).is_none());
}

/// Prudent Fateseer pumps the team on the first scry of the turn only.
#[test]
fn prudent_fateseer_triggers_once_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::prudent_fateseer());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    resolve(&mut g, Effect::Scry { who: PlayerRef::You, amount: Value::ONE });
    resolve(&mut g, Effect::Scry { who: PlayerRef::You, amount: Value::ONE });
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Lotus Ring: +3/+3 and vigilance; the granted ability sacrifices the
/// creature for three mana of one color.
#[test]
fn lotus_ring_turns_a_creature_into_mana() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ring = ready(&mut g, 0, catalog::lotus_ring());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: ring, target: bear }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords().contains(&Keyword::Vigilance));
    g.players[0].mana_pool = Default::default();
    activate(&mut g, bear, 0, None).expect("granted mana ability");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Feywild Trickster makes a Faerie Dragon whenever you roll.
#[test]
fn feywild_trickster_rolls_up_a_dragon() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::feywild_trickster());
    resolve(
        &mut g,
        Effect::RollDie {
            sides: 6,
            count: Value::ONE,
            modifier: Value::ZERO,
            reroll_at_most: 0,
            results: vec![],
            ignore_lowest: 0,
            on_doubles: None,
        },
    );
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Faerie Dragon"));
}

/// Infinite Hourglass ticks up in your upkeep and pumps every creature; an
/// opponent may pay {3} to tick it back down during an upkeep (CR 602.5).
#[test]
fn infinite_hourglass_ticks_and_anyone_can_rewind() {
    let mut g = pod(2);
    let ih = ready(&mut g, 0, catalog::infinite_hourglass());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    g.players[1].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ih,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("any player");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
}

#[allow(dead_code)]
fn _unused(_: Selector) {}
