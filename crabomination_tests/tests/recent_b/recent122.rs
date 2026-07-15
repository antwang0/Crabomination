//! Functionality tests for `catalog::sets::decks::recent122`.

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Gigantosaurus is a 10/10.
#[test]
fn gigantosaurus_is_a_ten_ten() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::gigantosaurus());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (10, 10));
}

/// Cephalid Inkmage becomes unblockable under threshold.
#[test]
fn cephalid_inkmage_threshold_unblockable() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::cephalid_inkmage());
    assert!(!g.computed_permanent(mage).unwrap().keywords.contains(&crabomination::card::Keyword::Unblockable),
        "no threshold → blockable");
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    assert!(g.computed_permanent(mage).unwrap().keywords.contains(&crabomination::card::Keyword::Unblockable),
        "threshold → unblockable");
}

/// Dire Downdraft's discount lets it cast for {2}{U} against a tapped creature.
#[test]
fn dire_downdraft_discount_vs_tapped() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::dire_downdraft());
    // Only the discounted {2}{U} is available.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at the {1}-off discount");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "the bear left the battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == bear), "it went to its owner's library");
}

/// Curator of Destinies digs five with Fact or Fiction on entry and is
/// uncounterable with flying.
#[test]
fn curator_of_destinies_fact_or_fiction() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let lib_before = g.players[0].library.len();
    let curator = g.add_card_to_battlefield(0, catalog::curator_of_destinies());
    let cp = g.computed_permanent(curator).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Flying), "has flying");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::CantBeCountered), "uncounterable");
    g.fire_self_etb_triggers(curator, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 5, "five cards left the library");
    // All five land in hand + graveyard combined.
    assert_eq!(g.players[0].hand.len() + g.players[0].graveyard.len(), 5, "split into hand and graveyard");
}
