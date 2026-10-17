//! Commander: the 2026-10-17 (`019Y6J4R`) eighth closer batch
//! (`decks::cmdr_019y_h`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
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



















/// Timebender turned face up takes two time counters off a permanent.
#[test]
fn timebender_unmorphs_into_two_time_counters_off() {
    let mut g = pod(2);
    let rock = ready(&mut g, 0, catalog::sol_ring());
    g.battlefield_find_mut(rock).unwrap().add_counters(CounterType::Time, 3);
    let tb = g.add_card_to_hand(0, catalog::timebender());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: tb }).expect("face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::TurnFaceUp { card_id: tb }).expect("unmorph");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rock).unwrap().counter_count(CounterType::Time), 1);
}

/// Soldevi Excavations needs an untapped Island to sacrifice, else it's lost.
#[test]
fn soldevi_excavations_eats_an_untapped_island() {
    let mut g = pod(2);
    let island = ready(&mut g, 0, catalog::island());
    let se = g.add_card_to_hand(0, catalog::soldevi_excavations());
    g.perform_action(GameAction::PlayLand(se)).expect("land drop");
    drain_stack(&mut g);
    assert!(g.battlefield_find(se).is_some());
    assert!(g.battlefield_find(island).is_none(), "the Island went");

    let mut g = pod(2);
    let island = ready(&mut g, 0, catalog::island());
    g.battlefield_find_mut(island).unwrap().tapped = true;
    let se = g.add_card_to_hand(0, catalog::soldevi_excavations());
    g.perform_action(GameAction::PlayLand(se)).expect("land drop");
    drain_stack(&mut g);
    assert!(g.battlefield_find(se).is_none(), "a tapped Island won't do");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == se));
}

/// Glissa's first mode incubates twice at the beginning of combat.
#[test]
fn glissa_incubates_twice_at_combat() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::glissa_herald_of_predation());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Mode(0),
    ]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let inc = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Incubator").count();
    assert_eq!(inc, 2);
}
