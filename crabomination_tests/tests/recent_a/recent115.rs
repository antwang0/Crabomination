//! Functionality tests for the mana-persistence staples in
//! `catalog::sets::decks::recent115`.

use crabomination::catalog;
use crabomination::game::{two_player_game};
use crabomination::mana::Color;

/// Upwelling keeps every player's unspent mana through a step/phase end.
#[test]
fn upwelling_keeps_all_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::upwelling());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add(Color::Blue, 3);
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 3, "controller keeps all colors");
    assert_eq!(g.players[1].mana_pool.total(), 3, "opponent keeps mana too");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "green preserved as green");
}

/// Omnath keeps only green; other colors still empty.
#[test]
fn omnath_keeps_only_green() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::omnath_locus_of_mana());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Red, 3);
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "green survives");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0, "red empties");
}

/// Omnath grows +1/+1 for each unspent green mana (live CDA).
#[test]
fn omnath_grows_with_unspent_green() {
    let mut g = two_player_game();
    let omnath = g.add_card_to_battlefield(0, catalog::omnath_locus_of_mana());
    assert_eq!(g.computed_permanent(omnath).unwrap().power, 1, "1/1 with an empty pool");
    g.players[0].mana_pool.add(Color::Green, 3);
    assert_eq!(g.computed_permanent(omnath).unwrap().power, 4, "1 + three unspent green");
    assert_eq!(g.computed_permanent(omnath).unwrap().toughness, 4);
}
