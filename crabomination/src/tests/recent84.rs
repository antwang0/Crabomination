//! Functionality tests for `catalog::sets::decks::recent84` (chosen-type batch).

use crate::card::{CounterType, CreatureType};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::two_player_game;
use crate::game::types::{Attack, AttackTarget, TurnStep};
use crate::game::*;
use crate::mana::Color;

/// Choose `ct` at the next creature-type decision, then fire `id`'s ETB.
fn enter_choosing(g: &mut GameState, id: CardId, ct: CreatureType) {
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(ct)]));
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
}

fn cast_bear_from_hand(g: &mut GameState) {
    let id = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G} 2/2 Bear
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Grizzly Bears");
    drain_stack(g);
}

#[test]
fn vanquishers_banner_anthems_and_draws_on_cast() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let banner = g.add_card_to_battlefield(0, catalog::vanquishers_banner());
    enter_choosing(&mut g, banner, CreatureType::Bear);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "chosen-type Bear gets +1/+1");

    g.add_card_to_library(0, catalog::plains());
    let hand = g.players[0].hand.len();
    cast_bear_from_hand(&mut g); // casting a Bear draws one
    assert_eq!(g.players[0].hand.len(), hand + 1, "cast-of-type drew a card (net: -Bear +draw +…)");
}

#[test]
fn kindred_discovery_draws_on_enter_and_attack() {
    let mut g = two_player_game();
    let kd = g.add_card_to_battlefield(0, catalog::kindred_discovery());
    enter_choosing(&mut g, kd, CreatureType::Bear);
    g.add_card_to_library(0, catalog::plains());
    let hand = g.players[0].hand.len();
    cast_bear_from_hand(&mut g); // a Bear enters → draw
    // hand: started H, +Bear (added to hand), -Bear (cast), +1 (library plains), + drawn = H+1
    assert_eq!(g.players[0].hand.len(), hand + 1, "Bear entering drew a card");

    // Now attack with a Bear → draw again.
    let attacker = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    g.clear_sickness(attacker);
    g.add_card_to_library(0, catalog::plains());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let before = g.players[0].hand.len();
    let events = g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "Bear attacking drew a card");
}

#[test]
fn door_of_destinies_scales_with_charge_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let door = g.add_card_to_battlefield(0, catalog::door_of_destinies());
    enter_choosing(&mut g, door, CreatureType::Bear);
    // No counters yet → no pump.
    let b = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (2, 2), "no charge counters → no anthem");

    for _ in 0..2 { cast_bear_from_hand(&mut g); } // two Bear casts → two charge counters
    assert_eq!(g.battlefield_find(door).unwrap().counters.get(&CounterType::Charge).copied(), Some(2));
    let b = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "+1/+1 per charge counter → +2/+2");
}

