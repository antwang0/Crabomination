//! CR conformance for the Mirrodin-completion pass:
//! - CR 723 — controlling another player during their next turn (Mindslaver).
//! - CR 605.1a — a mana ability's `{T}` fires the tapped-for-mana event, and
//!   the ability itself never uses the stack.
//! - CR 500.8 / 502-506 — a turn-scoped step skip actually skips the step.
//! - CR 103.2c — a spell or ability that shuffles a library is an event.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Run to the start of the next turn.
fn end_turn(g: &mut GameState) {
    let started = g.turn_number;
    while g.turn_number == started {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

// ── CR 723 — Controlling Another Player ──

/// CR 723.1 — the effect applies to the next turn the affected player takes,
/// and CR 723.3 leaves them the active player.
#[test]
fn cr_723_1_control_applies_on_the_targets_next_turn() {
    let mut g = main_phase();
    let slaver = g.add_card_to_battlefield(0, catalog::mindslaver());
    g.clear_sickness(slaver);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: slaver, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.pending_player_control, vec![(1, 0)]);
    assert_eq!(g.acting_seat_for(1), 1, "not yet their turn");
    end_turn(&mut g);
    assert_eq!(g.active_player_idx, 1, "CR 723.3 — still the active player");
    assert_eq!(g.acting_seat_for(1), 0, "seat 0 makes their decisions");
    assert!(g.pending_player_control.is_empty(), "the pending entry was consumed");
}

/// CR 723.1 — the control ends when the next turn begins.
#[test]
fn cr_723_1_control_expires_after_that_turn() {
    let mut g = main_phase();
    g.pending_player_control.push((1, 0));
    end_turn(&mut g);
    assert_eq!(g.acting_seat_for(1), 0);
    end_turn(&mut g);
    assert_eq!(g.acting_seat_for(1), 1, "back to playing themselves");
}

/// CR 723.1a — the most recently created control effect is the one that works.
#[test]
fn cr_723_1a_later_control_overwrites_earlier() {
    let mut g = two_player_game();
    g.players.push(crabomination::player::Player::new(2, "P2"));
    g.pending_player_control.push((2, 0));
    g.pending_player_control.push((2, 1));
    g.apply_pending_player_control(2);
    assert_eq!(g.acting_seat_for(2), 1);
}

/// CR 723.4 — the controller sees everything the controlled player can,
/// starting with their hand.
#[test]
fn cr_723_4_controller_sees_the_controlled_hand() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.pending_player_control.push((1, 0));
    end_turn(&mut g);
    let view = crabomination::server::view::project(&g, 0);
    assert!(matches!(view.players[1].hand[0], crabomination::net::HandCardView::Known(_)));
    assert_eq!(view.players[1].controlled_by, Some(0));
}

// ── CR 605.1a — mana abilities ──

/// CR 605.1a / 605.3 — a mana ability resolves without using the stack, and its
/// `{T}` is a tapped-for-mana event.
#[test]
fn cr_605_1a_mana_ability_taps_without_using_the_stack() {
    let mut g = main_phase();
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    let events = g
        .perform_action(GameAction::ActivateAbility {
            card_id: swamp, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("tap for mana");
    assert!(g.stack.is_empty(), "mana abilities don't use the stack");
    assert_eq!(g.players[0].mana_pool.total(), 1);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TappedForMana { card_id, .. } if *card_id == swamp))
    );
}

/// CR 605.1a — a non-mana `{T}` ability taps without emitting the event.
#[test]
fn cr_605_1a_non_mana_tap_is_not_tapped_for_mana() {
    let mut g = main_phase();
    let tower = g.add_card_to_battlefield(0, catalog::tower_of_eons());
    g.players[0].mana_pool.add_colorless(8);
    let events = g
        .perform_action(GameAction::ActivateAbility {
            card_id: tower, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
    assert!(!events.iter().any(|e| matches!(e, GameEvent::TappedForMana { .. })));
    assert!(!g.stack.is_empty(), "a non-mana ability uses the stack");
}

// ── CR 500.8 — skipping a step ──

/// CR 500.8 — a turn-scoped skip removes the step from that turn only.
#[test]
fn cr_500_8_a_skipped_draw_step_never_happens() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    g.skipped_steps_this_turn.push((1, TurnStep::Draw));
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::Untap;
    let before = g.players[1].hand.len();
    while g.step != TurnStep::PreCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].hand.len(), before);
    // …and it's gone by the next turn.
    end_turn(&mut g);
    assert!(g.skipped_steps_this_turn.is_empty());
}

// ── CR 103.2c — shuffling is an event ──

/// CR 103.2c — a search that shuffles the library announces it, so
/// "whenever a player shuffles" triggers can see it.
#[test]
fn cr_103_2c_a_search_shuffle_is_an_event() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::psychogenic_probe());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Search {
                who: crabomination::effect::PlayerRef::Seat(1),
                filter: crabomination::card::SelectionRequirement::Creature,
                to: crabomination::effect::ZoneDest::Hand(
                    crabomination::effect::PlayerRef::Seat(1),
                ),
            },
            &crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0),
        )
        .expect("search");
    assert!(evs.iter().any(|e| matches!(e, GameEvent::LibraryShuffled { player: 1 })));
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "Psychogenic Probe saw the shuffle");
}

/// A single `{T}` for mana taps the source exactly once (regression for the
/// tapped-for-mana event riding alongside the plain tap).
#[test]
fn cr_605_1a_mana_tap_emits_both_tap_events_once() {
    let mut g = main_phase();
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    let events = g
        .perform_action(GameAction::ActivateAbility {
            card_id: swamp, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("tap");
    assert_eq!(
        events.iter().filter(|e| matches!(e, GameEvent::PermanentTapped { .. })).count(),
        1
    );
    assert_eq!(
        events.iter().filter(|e| matches!(e, GameEvent::TappedForMana { .. })).count(),
        1
    );
}
