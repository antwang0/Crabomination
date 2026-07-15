//! Craft (CR 702.169) — LCI transforming artifacts. Exercises the
//! `craft_exile_cost` additional cost (exile N other objects) paired with
//! `Effect::ExileSelfReturnTransformed`, plus the sorcery-speed restriction.

use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

/// Pay {4}{B} + exile a battlefield creature to craft Tithing Blade into its
/// transformed Consuming Sepulcher back face.
#[test]
fn craft_exiles_a_battlefield_creature_and_returns_transformed() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::tithing_blade());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::ActivateAbility {
        card_id: blade,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate craft");
    drain_stack(&mut g);

    // Fodder creature was exiled as the craft cost.
    assert!(g.exile.iter().any(|c| c.id == fodder), "fodder exiled");
    // The artifact returned transformed: same id, now the back face.
    let card = g.battlefield_find(blade).expect("transformed permanent on bf");
    assert!(card.transformed, "marked transformed");
    assert_eq!(card.definition.name, "Consuming Sepulcher", "back face active");
}

/// Craft prefers exiling a graveyard card over a battlefield permanent, so the
/// board piece stays put.
#[test]
fn craft_exiles_graveyard_card_before_battlefield() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::tithing_blade());
    let board = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::ActivateAbility {
        card_id: blade,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate craft");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == gy), "graveyard card exiled");
    assert!(g.battlefield_find(board).is_some(), "board creature untouched");
}

/// Craft is sorcery-speed: it can't be activated on an opponent's turn.
#[test]
fn craft_rejected_at_instant_speed() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::tithing_blade());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 1; // opponent's turn
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);

    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: blade,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(err.is_err(), "craft barred at instant speed");
}

/// Crafting Inverted Iceberg returns the 6/6 Iceberg Titan artifact creature.
#[test]
fn craft_returns_artifact_creature_back_face() {
    let mut g = two_player_game();
    let ice = g.add_card_to_battlefield(0, catalog::inverted_iceberg());
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // an artifact to exile
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::ActivateAbility {
        card_id: ice,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate craft");
    drain_stack(&mut g);

    let titan = g.battlefield_find(ice).expect("titan on bf");
    assert_eq!(titan.definition.name, "Iceberg Titan");
    assert_eq!((titan.power(), titan.toughness()), (6, 6));
    assert!(titan.definition.is_creature());
}

/// Crafting Clay-Fired Bricks into Cosmium Kiln applies its +1/+1 anthem.
#[test]
fn craft_back_face_anthem_applies() {
    let mut g = two_player_game();
    let bricks = g.add_card_to_battlefield(0, catalog::clay_fired_bricks());
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact to exile
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::ActivateAbility {
        card_id: bricks,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate craft");
    drain_stack(&mut g);

    assert_eq!(g.battlefield_find(bricks).unwrap().definition.name, "Cosmium Kiln");
    // The 2/2 Grizzly Bears is buffed to 3/3 by the anthem.
    let buffed = g.computed_permanent(bear).unwrap();
    assert_eq!((buffed.power, buffed.toughness), (3, 3));
}

/// Craft is rejected when there aren't enough other objects to exile.
#[test]
fn craft_rejected_without_enough_fodder() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::visage_of_dread()); // craft with TWO creatures
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // only one available
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);

    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: blade,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(err.is_err(), "craft-with-two rejected with one creature");
}
