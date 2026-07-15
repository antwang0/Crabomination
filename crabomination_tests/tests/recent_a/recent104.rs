//! Functionality tests for `catalog::sets::decks::recent104` — the
//! long-deferred primitive cards (Pulmonic Sliver, Twilight Prophet,
//! Goblin Welder, Gilt-Leaf Archdruid).

use crabomination::catalog;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

/// Pulmonic Sliver: a dying Sliver goes to its owner's library top instead of
/// the graveyard; a non-Sliver still dies normally.
#[test]
fn pulmonic_sliver_redirects_dying_slivers_to_library_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pulmonic_sliver());
    let sliver = g.add_card_to_battlefield(0, catalog::galerider_sliver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(sliver).unwrap().damage = 9;
    g.battlefield_find_mut(bear).unwrap().damage = 9;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(
        g.players[0].library.first().map(|c| c.definition.name),
        Some("Galerider Sliver"),
        "Sliver on top of library"
    );
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == sliver));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "non-Sliver dies normally");
}

/// CR 700.4 — a death redirected to the library (Pulmonic Sliver) never
/// happened: "whenever a creature dies" watchers must not fire for it.
#[test]
fn library_redirected_death_does_not_fire_dies_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pulmonic_sliver());
    g.add_card_to_battlefield(0, catalog::blood_artist());
    let sliver = g.add_card_to_battlefield(0, catalog::galerider_sliver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (p0, p1) = (g.players[0].life, g.players[1].life);
    g.battlefield_find_mut(sliver).unwrap().damage = 9;
    g.battlefield_find_mut(bear).unwrap().damage = 9;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 1, "Blood Artist fires only for the bear");
    assert_eq!(g.players[1].life, p1 - 1);
}

/// Twilight Prophet with the city's blessing drains each opponent for the
/// revealed card's mana value at upkeep (and puts it in hand).
#[test]
fn twilight_prophet_drains_with_citys_blessing() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::twilight_prophet());
    g.players[0].city_blessing = true;
    g.add_card_to_library(0, catalog::serra_angel()); // bottom; library empty before
    let hand_before = g.players[0].hand.len();
    let (p0, p1) = (g.players[0].life, g.players[1].life);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 5, "opponent loses Serra Angel's MV");
    assert_eq!(g.players[0].life, p0 + 5, "controller gains that much");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "revealed card to hand");
}

/// Without the city's blessing the drain trigger doesn't fire.
#[test]
fn twilight_prophet_inert_without_blessing() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::twilight_prophet());
    g.add_card_to_library(0, catalog::serra_angel());
    let p1 = g.players[1].life;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1, "no drain without the blessing");
}

/// Goblin Welder swaps a battlefield artifact with the highest-MV artifact in
/// its controller's graveyard.
#[test]
fn goblin_welder_swaps_artifact_with_graveyard() {
    let mut g = two_player_game();
    let welder = g.add_card_to_battlefield(0, catalog::goblin_welder());
    g.clear_sickness(welder);
    let small = g.add_card_to_battlefield(1, catalog::sol_ring());
    let big = g.add_card_to_graveyard(1, catalog::mind_stone());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: welder,
        ability_index: 0,
        target: Some(Target::Permanent(small)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("weld");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == small), "artifact sacrificed");
    let ret = g.battlefield_find(big).expect("graveyard artifact returned");
    assert_eq!(ret.controller, 1, "returned under its owner's control");
}

/// Gilt-Leaf Archdruid draws on Druid casts and steals a player's lands for
/// tapping seven Druids.
#[test]
fn gilt_leaf_archdruid_draws_and_steals_lands() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::gilt_leaf_archdruid());
    g.clear_sickness(druid);
    // Cast a Druid spell → draw.
    g.add_card_to_library(0, catalog::island());
    let hand_druid = g.add_card_to_hand(0, catalog::gilt_leaf_archdruid());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: hand_druid,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the second Archdruid");
    drain_stack(&mut g);
    // -1 for the cast card, +1 for the trigger draw.
    assert_eq!(g.players[0].hand.len(), hand_before, "cast-a-Druid draw fired");
    // Two Archdruids + five Elves = seven Druids; steal the lands.
    for _ in 0..5 {
        let d = g.add_card_to_battlefield(0, catalog::llanowar_elves());
        g.clear_sickness(d);
    }
    let land = g.add_card_to_battlefield(1, catalog::island());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("tap seven Druids");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(land).unwrap().controller, 0, "land stolen");
    assert!(g.battlefield_find(druid).unwrap().tapped, "Druids tapped for the cost");
}
