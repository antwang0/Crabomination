//! Functionality tests for `catalog::sets::decks::recent197`.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Seize the Secrets costs {1} less after a crime.
#[test]
fn seize_the_secrets_crime_discount() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].committed_crime_this_turn = true;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::seize_the_secrets());
    // Only {1}{U} available — the crime discount must apply for this to cast.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("crime discount makes it {1}{U}");
    drain_stack(&mut g);
    // Cast one card out of hand, drew two → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew two cards");
}

/// Take for a Ride steals a creature, untapping it and granting haste.
#[test]
fn take_for_a_ride_steals_and_hastes() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::take_for_a_ride());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Take for a Ride");
    drain_stack(&mut g);
    let v = g.battlefield_find(victim).unwrap();
    assert_eq!(v.controller, 0, "gained control");
    assert!(!v.tapped, "untapped");
    assert!(g.compute_battlefield().iter().find(|c| c.id == victim).unwrap()
        .keywords.contains(&crate::card::Keyword::Haste), "granted haste");
}

/// Silver Deputy digs a basic to the top of your library on ETB.
#[test]
fn silver_deputy_digs_a_basic() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // non-land noise
    let forest_id = g.add_card_to_library(0, catalog::forest());
    let dep = g.add_card_to_battlefield(0, catalog::silver_deputy());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest_id))]));
    g.fire_self_etb_triggers(dep, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(forest_id), "basic on top");
}
