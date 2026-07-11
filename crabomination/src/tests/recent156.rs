//! Functionality tests for `catalog::sets::decks::recent156` (BLB Valiant Mice).
//! Valiant fires via a friendly `BecameTarget` event; the tests dispatch it
//! directly (as the existing Valiant tests do).

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;

/// Seedglaive Mentor's Valiant puts a +1/+1 counter on it the first time you
/// target it each turn.
#[test]
fn seedglaive_mentor_valiant_grows() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::seedglaive_mentor());
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: m, caster: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(m).unwrap().power, 4, "Valiant added a +1/+1 counter");
    // Only once per turn.
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: m, caster: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(m).unwrap().power, 4, "fires only once per turn");
}

/// Mouse Trapper's Valiant taps a creature an opponent controls.
#[test]
fn mouse_trapper_valiant_taps_opponent() {
    let mut g = two_player_game();
    let trapper = g.add_card_to_battlefield(0, catalog::mouse_trapper());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: trapper, caster: 0 }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "Valiant tapped the opponent's creature");
}

/// Flowerfoot Swordmaster's Valiant pumps every Mouse you control +1/+0.
#[test]
fn flowerfoot_swordmaster_valiant_pumps_mice() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::flowerfoot_swordmaster());
    let other = g.add_card_to_battlefield(0, catalog::seedglaive_mentor());
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: master, caster: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(master).unwrap().power, 2, "self pumped +1/+0");
    assert_eq!(g.computed_permanent(other).unwrap().power, 4, "other Mouse pumped +1/+0");
}

/// Whiskerquill Scribe's Valiant loots — discard a card to draw a card.
#[test]
fn whiskerquill_scribe_valiant_loots() {
    let mut g = two_player_game();
    let scribe = g.add_card_to_battlefield(0, catalog::whiskerquill_scribe());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: scribe, caster: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 1, "discarded a card");
    assert!(g.players[0].library.is_empty(), "drew the top of library");
}
