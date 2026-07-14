//! Functionality tests for `catalog::sets::decks::recent199`.

use crate::card::CounterType;
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};

/// Growing Dread manifests dread on ETB (a face-down 2/2 appears).
#[test]
fn growing_dread_manifests_dread() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let gd = g.add_card_to_battlefield(0, catalog::growing_dread());
    g.fire_self_etb_triggers(gd, 0);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
        "a face-down manifest entered",
    );
}

/// Growing Dread puts a +1/+1 counter on a permanent turned face up.
#[test]
fn growing_dread_counters_face_up() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let gd = g.add_card_to_battlefield(0, catalog::growing_dread());
    g.fire_self_etb_triggers(gd, 0);
    drain_stack(&mut g);
    let manifest = g.battlefield.iter().find(|c| c.controller == 0 && c.face_down).unwrap().id;
    let evs = vec![GameEvent::TurnedFaceUp { card_id: manifest }];
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(manifest).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "counter placed on the face-up permanent",
    );
}

/// Entity Tracker draws when another enchantment you control enters.
#[test]
fn entity_tracker_draws_on_enchantment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::entity_tracker());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let ench = g.add_card_to_battlefield(0, catalog::growing_dread()); // an enchantment enters
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Eerie drew a card");
}
