//! Coveted Jewel ETB draw-three (modern_decks).

use crate::catalog;
use crate::game::{drain_stack, two_player_game, GameAction};

#[test]
fn coveted_jewel_etb_draws_three() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::coveted_jewel());
    g.players[0].mana_pool.add_colorless(6);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    // Jewel leaves hand (-1); ETB draws 3 (+3): net +2.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}
