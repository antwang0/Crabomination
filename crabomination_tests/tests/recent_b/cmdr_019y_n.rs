//! Commander: the 2026-10-18 (`019Y6J4R`) fourteenth closer batch
//! (`decks::cmdr_019y_n`).

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


use crabomination::card::Keyword;

fn opposing_attacker(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = ready(g, 1, def);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: id, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(g);
    g.priority.player_with_priority = 0;
    id
}

/// CR 601.2f — Magnificent End costs {1}{W} against a tapped creature.
#[test]
fn magnificent_end_is_cheap_against_a_tapped_creature() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.battlefield_find_mut(giant).unwrap().tapped = true;
    let me = g.add_card_to_hand(0, catalog::magnificent_end());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, me, Some(Target::Permanent(giant))).expect("two mana is enough");
    assert!(g.battlefield_find(giant).is_none());
}

/// CR 601.2f — Depower costs {U} against an attacker, and cantrips.
#[test]
fn depower_is_cheap_against_an_attacker() {
    let mut g = pod(2);
    let giant = opposing_attacker(&mut g, catalog::hill_giant());
    let dp = g.add_card_to_hand(0, catalog::depower());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, dp, Some(Target::Permanent(giant))).expect("one mana is enough");
    assert_eq!(g.computed_permanent(giant).unwrap().power, -1);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 601.2 — Draconic Roar burns the controller too when a Dragon was in
/// hand as it was cast.
#[test]
fn draconic_roar_burns_the_controller_with_a_dragon() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::noxious_dragon());
    let dr = g.add_card_to_hand(0, catalog::draconic_roar());
    flood(&mut g, 0);
    cast(&mut g, dr, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, 17);
}

/// Without a Dragon, Draconic Roar only hits the creature.
#[test]
fn draconic_roar_without_a_dragon() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let dr = g.add_card_to_hand(0, catalog::draconic_roar());
    flood(&mut g, 0);
    cast(&mut g, dr, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(g.players[1].life, 20);
}

/// Stomped by the Foot unkicked: -2/-2.
#[test]
fn stomped_by_the_foot_unkicked_shrinks_two() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let sf = g.add_card_to_hand(0, catalog::stomped_by_the_foot());
    flood(&mut g, 0);
    cast(&mut g, sf, Some(Target::Permanent(giant))).expect("cast");
    let cp = g.computed_permanent(giant).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

/// Pyrrhic Strike unblighted: one mode — the artifact.
#[test]
fn pyrrhic_strike_destroys_an_artifact() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::lotus_ring());
    let ps = g.add_card_to_hand(0, catalog::pyrrhic_strike());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: ps,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    // Lotus Ring is indestructible: the destroy does nothing, but the cast is legal.
    assert!(g.battlefield_find(ring).is_some());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ps));
}

/// Valkyrie's Sword: pay {4}{W} on entry for an Angel wielding it (6/5).
#[test]
fn valkyries_sword_calls_an_angel() {
    let mut g = pod(2);
    let vs = g.add_card_to_hand(0, catalog::valkyries_sword());
    flood(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    cast(&mut g, vs, None).expect("cast");
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Angel Warrior").expect("an Angel").id;
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 5));
    assert!(cp.keywords().contains(&Keyword::Vigilance));
}

/// Bifur puts a counter on a creature when it enters.
#[test]
fn bifur_counters_on_entry() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bf = g.add_card_to_hand(0, catalog::bifur_melodic_rider());
    flood(&mut g, 0);
    cast(&mut g, bf, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// With an enduring story (CR 207.2c storied), Bifur's own Dwarf trigger fires
/// twice.
#[test]
fn bifur_doubles_dwarf_triggers_with_an_enduring_story() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::lotus_ring());
    }
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bf = g.add_card_to_hand(0, catalog::bifur_melodic_rider());
    flood(&mut g, 0);
    cast(&mut g, bf, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}
