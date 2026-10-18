//! Commander: the 2026-10-18 (`019Y6J4R`) seventeenth closer batch
//! (`decks::cmdr_019y_q`).

use crabomination::card::{CardId, CounterType, Keyword};
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


/// Invasion of Xerex bounces a creature on entry (CR 310.11 — a Siege enters
/// with its defense counters).
#[test]
fn invasion_of_xerex_bounces_on_entry() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ix = g.add_card_to_hand(0, catalog::invasion_of_xerex());
    flood(&mut g, 0);
    cast(&mut g, ix, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    assert_eq!(g.battlefield_find(ix).unwrap().counter_count(CounterType::Defense), 4);
}

/// Driven: trample and a card per connecting creature this turn (CR 611.2c —
/// only the creatures there as it resolves).
#[test]
fn driven_gives_trample_and_card_draw() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let dd = g.add_card_to_hand(0, catalog::driven_despair());
    flood(&mut g, 0);
    cast(&mut g, dd, None).expect("Driven");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Sunder the Gateway's second mode: incubate 2 and transform it at once.
#[test]
fn sunder_the_gateway_hatches_an_incubator() {
    let mut g = pod(2);
    let st = g.add_card_to_hand(0, catalog::sunder_the_gateway());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: st, target: None, additional_targets: vec![], mode: Some(1), x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token).expect("the token").id;
    let cp = g.computed_permanent(token).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "a 0/0 Phyrexian with two counters");
}

/// Seize the Storm's Elemental counts instants and sorceries in the graveyard.
#[test]
fn seize_the_storm_counts_spells() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    let ss = g.add_card_to_hand(0, catalog::seize_the_storm());
    flood(&mut g, 0);
    cast(&mut g, ss, None).expect("cast");
    let el = g.battlefield.iter().find(|c| c.definition.name == "Elemental").expect("Elemental").id;
    let cp = g.computed_permanent(el).unwrap();
    // Three Bolts plus Seize the Storm itself, now in the graveyard.
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Telemin Performance mills to the opponent's first creature and steals it.
#[test]
fn telemin_performance_steals_the_first_creature() {
    let mut g = pod(2);
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::forest());
    let giant = g.add_card_to_library(1, catalog::hill_giant());
    let tp = g.add_card_to_hand(0, catalog::telemin_performance());
    flood(&mut g, 0);
    cast(&mut g, tp, Some(Target::Player(1))).expect("cast");
    let gi = g.battlefield_find(giant).expect("the Giant came over");
    assert_eq!(gi.controller, 0);
    assert_eq!(g.players[1].graveyard.len(), 1, "the Forest was milled");
}

fn activate(g: &mut GameState, id: CardId, idx: usize) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// CR 702.62 — Rift Elemental pays its time counter off a suspended card in
/// exile (the last one casts it); with none anywhere it can't activate.
#[test]
fn rift_elemental_eats_suspended_time_counters() {
    let mut g = pod(2);
    let re = ready(&mut g, 0, catalog::rift_elemental());
    flood(&mut g, 0);
    assert!(activate(&mut g, re, 0).is_err(), "no time counter anywhere");
    let blade = g.add_card_to_exile(0, catalog::arc_blade());
    g.exile.iter_mut().find(|c| c.id == blade).unwrap().add_counters(CounterType::Time, 2);
    activate(&mut g, re, 0).expect("a suspended card pays");
    assert_eq!(g.computed_permanent(re).unwrap().power, 3);
    assert_eq!(g.exile.iter().find(|c| c.id == blade).unwrap().counter_count(CounterType::Time), 1);
}
