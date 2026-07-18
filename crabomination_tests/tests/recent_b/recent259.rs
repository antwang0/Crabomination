//! Functionality tests for `catalog::sets::decks::recent259` (Living Conundrum).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::two_player_game;

/// Living Conundrum is a 2/5 with a full library and a 10/10 flying, vigilant
/// beater once the library is empty.
#[test]
fn living_conundrum_wakes_on_empty_library() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::living_conundrum());
    g.add_card_to_library(0, catalog::island());
    let base = g.computed_permanent(id).unwrap();
    assert_eq!((base.power, base.toughness), (2, 5), "2/5 while the library has cards");
    assert!(!base.keywords.contains(&Keyword::Flying), "no flying yet");

    g.players[0].library.clear();
    let big = g.computed_permanent(id).unwrap();
    assert_eq!((big.power, big.toughness), (10, 10), "10/10 with an empty library");
    assert!(big.keywords.contains(&Keyword::Flying), "gained flying");
    assert!(big.keywords.contains(&Keyword::Vigilance), "gained vigilance");
    assert!(big.keywords.contains(&Keyword::Hexproof), "still hexproof");
}
