//! Functionality tests for the MH3 batch-5 cards in `catalog::sets::mh3e`.

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// The colorless mana ability taps for {C}.
#[test]
fn bountiful_landscape_taps_for_colorless() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::bountiful_landscape());
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "one colorless produced");
}

/// The sac ability fetches an eligible basic onto the battlefield tapped.
#[test]
fn bountiful_landscape_sac_fetches_basic_tapped() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::bountiful_landscape());
    let mountain = g.add_card_to_library(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(mountain))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac to fetch");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == land) == false, "Landscape sacrificed");
    let fetched = g.battlefield.iter().find(|c| c.id == mountain).expect("Mountain fetched");
    assert!(fetched.tapped, "fetched basic enters tapped");
}

/// A non-eligible basic (Swamp — off Bountiful's F/I/M list) can't be fetched.
#[test]
fn bountiful_landscape_wont_fetch_off_color_basic() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::bountiful_landscape());
    let swamp = g.add_card_to_library(0, catalog::swamp());
    // Decider would pick the swamp, but the filter excludes it → nothing fetched.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(swamp))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac to fetch");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != swamp), "off-color Swamp not fetched");
}

/// Vega draws when you cast a spell from your graveyard (not from hand).
#[test]
fn vega_draws_on_graveyard_cast() {
    // Isolate Vega's extra draw by comparing library depletion with vs.
    // without Vega on the battlefield when flashing back the same spell.
    fn deplete_with(vega: bool) -> usize {
        let mut g = two_player_game();
        for _ in 0..20 { g.add_card_to_library(0, catalog::island()); }
        if vega {
            g.add_card_to_battlefield(0, catalog::vega_the_watcher());
        }
        let id = g.add_card_to_library(0, catalog::faithless_looting());
        let pos = g.players[0].library.iter().position(|c| c.id == id).unwrap();
        let card = g.players[0].library.remove(pos);
        g.players[0].graveyard.push(card);
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        let before = g.players[0].library.len();
        g.perform_action(GameAction::CastFlashback {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("flashback Faithless Looting");
        drain_stack(&mut g);
        before - g.players[0].library.len()
    }
    // Faithless Looting draws 2; Vega adds exactly one more.
    assert_eq!(deplete_with(false), 2, "without Vega only the spell's draws");
    assert_eq!(deplete_with(true), 3, "Vega draws one extra on the graveyard cast");
}

/// Cycling the land for its three colors draws a card.
#[test]
fn contaminated_landscape_cycles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::contaminated_landscape());
    let _ = g.add_card_to_library(0, catalog::forest());
    for c in [Color::White, Color::Blue, Color::Black] {
        g.players[0].mana_pool.add(c, 1);
    }
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle for {W}{U}{B}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "cycled land in graveyard");
    assert_eq!(g.players[0].hand.len(), before, "discard one, draw one");
}
