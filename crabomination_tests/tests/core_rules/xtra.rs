//! Extra-turn spell behavior tests (Time Walk, Time Warp, etc.).
//!
//! These live in the top-level crate (rather than next to the card factories
//! in `crabomination_catalog`) because they drive the full game engine —
//! casting the spell and asserting the extra-turn bank — which is not visible
//! from the catalog crate.

use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

fn cast_and_resolve(card: crabomination::card::CardDefinition, blue: u32, generic: u32) -> GameState {
    cast_and_resolve_at(card, blue, generic, None)
}

fn cast_and_resolve_at(
    card: crabomination::card::CardDefinition,
    blue: u32,
    generic: u32,
    target: Option<Target>,
) -> GameState {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, card);
    g.players[0].mana_pool.add(Color::Blue, blue);
    g.players[0].mana_pool.add_colorless(generic);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    g
}

/// Table-driven: every plain "take an extra turn" spell banks exactly one turn.
#[test]
fn extra_turn_spells_bank_one_extra_turn() {
    // (name, card, blue, generic, target) — Time Warp is "TARGET player
    // takes an extra turn", so it casts with an explicit player target.
    let cases: Vec<(&str, crabomination::card::CardDefinition, u32, u32, Option<Target>)> = vec![
        ("Time Walk", catalog::time_walk(), 1, 1, None),
        ("Time Warp", catalog::time_warp(), 2, 3, Some(Target::Player(0))),
        ("Temporal Manipulation", catalog::temporal_manipulation(), 2, 3, None),
        ("Capture of Jingzhou", catalog::capture_of_jingzhou(), 2, 3, None),
        ("Nexus of Fate", catalog::nexus_of_fate(), 2, 5, None),
    ];
    for (name, card, blue, generic, target) in cases {
        let g = cast_and_resolve_at(card, blue, generic, target);
        assert_eq!(g.players[0].extra_turns, 1, "{name} banks one extra turn");
    }
}

/// Time Warp can grant the extra turn to the OPPONENT — "target player".
#[test]
fn time_warp_can_target_the_opponent() {
    let g = cast_and_resolve_at(catalog::time_warp(), 2, 3, Some(Target::Player(1)));
    assert_eq!(g.players[1].extra_turns, 1, "opponent banks the extra turn");
    assert_eq!(g.players[0].extra_turns, 0, "caster gets nothing");
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
