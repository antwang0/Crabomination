//! Functionality tests for `catalog::sets::decks::recent284`.

use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game, GameAction, Target, TurnStep};
use crabomination::mana::Color;

/// Scrapshooter's gift-gated ETB: promising the gift lets the opponent draw and
/// destroys a targeted artifact/enchantment.
#[test]
fn scrapshooter_gift_promised_destroys() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let art = g.add_card_to_battlefield(1, catalog::sol_ring()); // an artifact to destroy
    g.add_card_to_library(1, catalog::forest()); // opponent has a card to draw
    let opp_hand = g.players[1].hand.len();
    let scrap = g.add_card_to_hand(0, catalog::scrapshooter());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastGift {
        card_id: scrap,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Scrapshooter with gift");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == art), "artifact destroyed");
    assert_eq!(g.players[1].hand.len(), opp_hand + 1, "opponent drew the gift card");
}

/// Without the gift promised, Scrapshooter's ETB never fires.
#[test]
fn scrapshooter_no_gift_no_effect() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_library(1, catalog::forest());
    let opp_hand = g.players[1].hand.len();
    let scrap = g.add_card_to_hand(0, catalog::scrapshooter());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: scrap,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Scrapshooter without gift");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == art), "artifact survives (no gift)");
    assert_eq!(g.players[1].hand.len(), opp_hand, "opponent didn't draw (no gift)");
}
