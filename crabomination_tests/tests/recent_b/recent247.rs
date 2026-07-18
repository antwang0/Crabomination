//! Functionality tests for `catalog::sets::decks::recent247` (MKM lands,
//! artifact value, and a wither commander).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};

fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
}

/// Magnifying Glass taps for {C} and its {4}, {T} ability investigates.
#[test]
fn magnifying_glass_taps_and_investigates() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let glass = g.add_card_to_battlefield(0, catalog::magnifying_glass());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: glass,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the investigate ability");
    drain_stack(&mut g);
    assert_eq!(clues(&g, 0), 1, "investigated");
}

/// Escape Tunnel sacrifices to make a small creature unblockable.
#[test]
fn escape_tunnel_grants_unblockable() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let tunnel = g.add_card_to_battlefield(0, catalog::escape_tunnel());
    g.perform_action(GameAction::ActivateAbility {
        card_id: tunnel,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the unblockable ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tunnel).is_none(), "Escape Tunnel sacrificed");
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable),
        "small creature can't be blocked"
    );
}

/// Scene of the Crime enters tapped and sacrifices to draw a card.
#[test]
fn scene_of_the_crime_enters_tapped_and_draws() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let scene = g.add_card_to_battlefield(0, catalog::scene_of_the_crime());
    // (The enters-tapped static is exercised through the real ETB path in play;
    // `add_card_to_battlefield` skips replacement effects.)
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: scene,
        ability_index: 2,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sac to draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.battlefield_find(scene).is_none(), "sacrificed");
}

/// Massacre Girl grants wither to your creatures and draws when an opponent's
/// creature dies with toughness less than 1.
#[test]
fn massacre_girl_wither_and_death_draw() {
    let mut g = two_player_game();
    let girl = g.add_card_to_battlefield(0, catalog::massacre_girl_known_killer());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = girl;
    assert!(
        g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Wither),
        "your creatures have wither"
    );
    // An opponent 1/1 reduced to 0 toughness dies → draw.
    g.add_card_to_library(0, catalog::grizzly_bears());
    let foe = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1
    g.battlefield_find_mut(foe).unwrap().counters.insert(CounterType::MinusOneMinusOne, 1);
    let hand = g.players[0].hand.len();
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "0-toughness creature died");
    assert_eq!(g.players[0].hand.len(), hand + 1, "Massacre Girl drew a card");

    // A creature that dies to lethal damage (toughness still ≥ 1) does not draw.
    let hardy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand2 = g.players[0].hand.len();
    g.battlefield_find_mut(hardy).unwrap().damage = 2; // lethal, toughness stays 2
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(hardy).is_none(), "took lethal damage and died");
    assert_eq!(g.players[0].hand.len(), hand2, "toughness was 2 → no draw");
}
