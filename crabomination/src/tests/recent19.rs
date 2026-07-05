//! Functionality tests for the `catalog::sets::decks::recent19` batch.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;

/// Beast-Kin Ranger pumps itself +1/+0 when another creature enters.
#[test]
fn beast_kin_ranger_pumps_on_other_etb() {
    let mut g = two_player_game();
    let ranger = g.add_card_to_battlefield(0, catalog::beast_kin_ranger());
    assert_eq!(g.computed_permanent(ranger).unwrap().power, 3);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ranger).unwrap().power, 4, "+1/+0 from the new creature");
}

/// Marble Gargoyle firebreathes toughness with {W}.
#[test]
fn marble_gargoyle_pumps_toughness() {
    let mut g = two_player_game();
    let gar = g.add_card_to_battlefield(0, catalog::marble_gargoyle());
    assert!(g.computed_permanent(gar).unwrap().keywords.contains(&Keyword::Flying));
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gar, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump toughness");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(gar).unwrap().toughness, 3, "+0/+1");
}

/// Coral Colony mills X = your defenders.
#[test]
fn coral_colony_mills_by_defender_count() {
    let mut g = two_player_game();
    let colony = g.add_card_to_battlefield(0, catalog::coral_colony());
    g.clear_sickness(colony); // a defender
    g.add_card_to_battlefield(0, catalog::coral_colony()); // a second defender
    for _ in 0..4 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    let gy = g.players[1].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: colony, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Coral Colony");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy + 2, "milled 2 for two defenders");
}
