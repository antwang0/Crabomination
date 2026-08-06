//! Secrets of Strixhaven — the Special Guests (SPG) sheet.

use crabomination::catalog;
use crabomination::game::types::GameAction;
use crabomination::game::{drain_stack, two_player_game};

/// Magus of the Library taps for {C} always, but only draws while the hand
/// is exactly seven cards deep.
#[test]
fn magus_of_the_library_draws_only_at_exactly_seven_cards() {
    let mut g = two_player_game();
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_library());
    g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(magus);
    g.priority.player_with_priority = 0;
    let draw = |id| GameAction::ActivateAbility {
        card_id: id,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };

    while g.players[0].hand.len() > 6 {
        g.players[0].hand.pop();
    }
    while g.players[0].hand.len() < 6 {
        g.add_card_to_hand(0, catalog::forest());
    }
    assert!(g.perform_action(draw(magus)).is_err(), "six cards is not seven");

    g.add_card_to_hand(0, catalog::forest());
    g.perform_action(draw(magus)).expect("seven cards");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 8);
}

/// Library of Leng sends a forced discard to the top of the library.
#[test]
fn library_of_leng_puts_discards_on_top_of_the_library() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::library_of_leng());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let mut events = vec![];
    assert!(g.discard_card(0, bolt, &mut events));

    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt), "not the graveyard");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt));
    assert!(g.effective_max_hand_size(0).is_none(), "and no maximum hand size");
}
