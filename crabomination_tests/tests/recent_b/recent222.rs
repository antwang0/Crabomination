//! Functionality tests for `catalog::sets::decks::recent222`.

use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Vizier of the Menagerie lets you cast a creature spell off the top of your
/// library.
#[test]
fn vizier_casts_creature_from_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vizier_of_the_menagerie());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // top of library
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Grizzly Bears from the top of the library");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bears && c.controller == 0), "Grizzly Bears entered from the top");
}
