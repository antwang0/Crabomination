//! Functionality tests for `catalog::sets::decks::recent257` (Alquist Proft).

use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Alquist Proft investigates on ETB, then converts a Clue + {X}{W}{U}{U} into
/// X cards drawn and X life.
#[test]
fn alquist_proft_investigates_then_draws_x() {
    let mut g = two_player_game();
    let proft = g.add_card_to_battlefield(0, catalog::alquist_proft_master_sleuth());
    g.fire_self_etb_triggers(proft, 0);
    drain_stack(&mut g);
    let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
    assert_eq!(clues, 1, "ETB investigated for one Clue");

    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.battlefield_find_mut(proft).unwrap().summoning_sick = false; // able to tap
    g.players[0].mana_pool.add_colorless(2); // the {X}=2 generic
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: proft,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
    })
    .expect("activate the draw-X ability with X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew X=2 cards");
    assert_eq!(g.players[0].life, life_before + 2, "gained X=2 life");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count(),
        0,
        "the Clue was sacrificed as a cost",
    );
}
