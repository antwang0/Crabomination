//! Functionality tests for `catalog::sets::decks::recent288` — Doc Aurlock's
//! graveyard/exile/plot cost reductions.

use crabomination::catalog;
use crabomination::mana::Color;
use crabomination::game::{two_player_game, GameAction};

/// Doc Aurlock reduces Plot activation costs by {2}: Longhorn Sharpshooter's
/// {3}{R} plot cost becomes {1}{R}.
#[test]
fn doc_aurlock_discounts_plot() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doc_aurlock_grizzled_genius());
    let card = g.add_card_to_hand(0, catalog::longhorn_sharpshooter());
    // Only {1}{R} available — the full {3}{R} would be short.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Plot { card_id: card }).expect("plot at the reduced cost");
    assert!(g.exile.iter().any(|c| c.id == card), "the plotted card sits in exile");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the reduced cost drained the pool exactly");
}

/// Doc Aurlock reduces exile casts by {2}: a foretold Behold the Multiverse
/// (foretell {1}{U}) casts for just {U}.
#[test]
fn doc_aurlock_discounts_exile_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doc_aurlock_grizzled_genius());
    let card = g.add_card_to_exile(0, catalog::behold_the_multiverse());
    g.exile.iter_mut().find(|c| c.id == card).unwrap().face_down = true;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastForetold {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the foretold spell at the reduced exile cost");
    assert_eq!(g.players[0].mana_pool.total(), 0, "only one blue mana was spent");
}
