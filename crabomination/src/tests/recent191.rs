//! Functionality tests for `catalog::sets::decks::recent191` — the
//! `EventKind::BecomesPlotted` self-trigger (CR 702.170).

use crate::catalog;
use crate::game::two_player_game;
use crate::game::*;
use crate::mana::Color;

/// Longhorn Sharpshooter pings when plotted.
#[test]
fn longhorn_sharpshooter_burns_on_plot() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[1].life = 20;
    let card = g.add_card_to_hand(0, catalog::longhorn_sharpshooter());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Plot { card_id: card }).expect("plot Longhorn Sharpshooter");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == card), "card is plotted (exiled)");
    assert_eq!(g.players[1].life, 18, "dealt 2 to the opponent on plot");
}

/// Aloe Alchemist pumps a creature when plotted.
#[test]
fn aloe_alchemist_pumps_on_plot() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sole creature → auto-targeted
    let card = g.add_card_to_hand(0, catalog::aloe_alchemist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Plot { card_id: card }).expect("plot Aloe Alchemist");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4), "+3/+2 from the plot trigger");
    assert!(cp.keywords.contains(&crate::card::Keyword::Trample), "gained trample");
}
