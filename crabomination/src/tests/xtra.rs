//! Extra-turn spell behavior tests (Time Walk, Time Warp, etc.).
//!
//! These live in the top-level crate (rather than next to the card factories
//! in `crabomination_catalog`) because they drive the full game engine —
//! casting the spell and asserting the extra-turn bank — which is not visible
//! from the catalog crate.

use crate::catalog;
use crate::game::*;
use crate::mana::Color;

fn cast_and_resolve(card: crate::card::CardDefinition, blue: u32, generic: u32) -> GameState {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, card);
    g.players[0].mana_pool.add(Color::Blue, blue);
    g.players[0].mana_pool.add_colorless(generic);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    g
}

#[test]
fn time_walk_banks_one_extra_turn() {
    let g = cast_and_resolve(catalog::time_walk(), 1, 1);
    assert_eq!(g.players[0].extra_turns, 1);
}

#[test]
fn time_warp_banks_one_extra_turn() {
    let g = cast_and_resolve(catalog::time_warp(), 2, 3);
    assert_eq!(g.players[0].extra_turns, 1);
}

#[test]
fn temporal_manipulation_banks_one_extra_turn() {
    let g = cast_and_resolve(catalog::temporal_manipulation(), 2, 3);
    assert_eq!(g.players[0].extra_turns, 1);
}

#[test]
fn capture_of_jingzhou_banks_one_extra_turn() {
    let g = cast_and_resolve(catalog::capture_of_jingzhou(), 2, 3);
    assert_eq!(g.players[0].extra_turns, 1);
}

#[test]
fn nexus_of_fate_banks_one_extra_turn() {
    let g = cast_and_resolve(catalog::nexus_of_fate(), 2, 5);
    assert_eq!(g.players[0].extra_turns, 1);
}

#[test]
fn extra_turn_then_taken_keeps_active_player() {
    // The extra-turn bank is consumed in do_cleanup (CR 500.7): the
    // active player keeps the turn instead of passing.
    let mut g = cast_and_resolve(catalog::time_walk(), 1, 1);
    g.active_player_idx = 0;
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 0, "extra turn keeps the same player");
    assert_eq!(g.players[0].extra_turns, 0, "charge consumed");
}
