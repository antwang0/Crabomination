//! Functionality tests for `catalog::sets::decks::recent168` — the four DFT
//! gap-card primitives: SacrificedWasVehicle, SelfIsCreatureIf, SetSaddled /
//! AnimateAsCreature, and SelfCrewsSaddlesWithToughness.

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;

/// Hellish Sideswipe draws only when the sacrificed fodder was a Vehicle.
#[test]
fn hellish_sideswipe_vehicle_fodder_draws() {
    let mut g = two_player_game();
    let vehicle = g.add_card_to_battlefield(0, catalog::midnight_mangler());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::hellish_sideswipe());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.pending_cast_sacrifices = Some(vec![vehicle]);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with Vehicle fodder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target destroyed");
    // -1 spell cast +1 draw = net same, but the draw fired: hand size unchanged
    // (spell left hand, one card drawn).
    assert_eq!(g.players[0].hand.len(), hand_before, "Vehicle fodder drew a card");
}

/// Non-Vehicle fodder destroys but does not draw.
#[test]
fn hellish_sideswipe_creature_fodder_no_draw() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::hellish_sideswipe());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.pending_cast_sacrifices = Some(vec![fodder]);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with creature fodder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target destroyed");
    // Spell left hand, no draw → hand shrank by one.
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "creature fodder: no draw");
}

/// Midnight Mangler is an artifact creature only during turns that aren't its
/// controller's.
#[test]
fn midnight_mangler_creature_off_turn() {
    let mut g = two_player_game();
    let mangler = g.add_card_to_battlefield(0, catalog::midnight_mangler());
    g.active_player_idx = 0;
    assert!(
        !g.computed_permanent(mangler).unwrap().card_types.contains(&CardType::Creature),
        "not a creature on your own turn"
    );
    g.active_player_idx = 1;
    assert!(
        g.computed_permanent(mangler).unwrap().card_types.contains(&CardType::Creature),
        "an artifact creature during other players' turns"
    );
}

/// Guidelight Matrix's first ability saddles a target Mount.
#[test]
fn guidelight_matrix_saddles_mount() {
    let mut g = two_player_game();
    let matrix = g.add_card_to_battlefield(0, catalog::guidelight_matrix());
    let mount = g.add_card_to_battlefield(0, catalog::bridled_bighorn());
    g.clear_sickness(matrix);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert!(!g.battlefield_find(mount).unwrap().saddled, "starts unsaddled");
    g.perform_action(GameAction::ActivateAbility {
        card_id: matrix,
        ability_index: 0,
        target: Some(Target::Permanent(mount)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("saddle activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mount).unwrap().saddled, "Mount is now saddled");
}

/// Guidelight Matrix's second ability animates a target Vehicle you control.
#[test]
fn guidelight_matrix_animates_vehicle() {
    let mut g = two_player_game();
    let matrix = g.add_card_to_battlefield(0, catalog::guidelight_matrix());
    let vehicle = g.add_card_to_battlefield(0, catalog::boommobile());
    g.clear_sickness(matrix);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert!(
        !g.computed_permanent(vehicle).unwrap().card_types.contains(&CardType::Creature),
        "Vehicle isn't a creature by default"
    );
    g.perform_action(GameAction::ActivateAbility {
        card_id: matrix,
        ability_index: 1,
        target: Some(Target::Permanent(vehicle)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("animate activation");
    drain_stack(&mut g);
    let cp = g.computed_permanent(vehicle).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "Vehicle became a creature");
    assert_eq!((cp.power, cp.toughness), (5, 5), "keeps its printed 5/5");
}

/// Interface Ace crews a Crew 2 Vehicle with its toughness (4), which a plain
/// 0-power creature cannot. Its untap trigger then untaps it that turn.
#[test]
fn interface_ace_crews_with_toughness_then_untaps() {
    let mut g = two_player_game();
    let vehicle = g.add_card_to_battlefield(0, catalog::boommobile());
    let ace = g.add_card_to_battlefield(0, catalog::interface_ace());
    let weakling = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.clear_sickness(ace);
    g.clear_sickness(weakling);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // A 0/2 can't crew (power 0 < 2).
    assert!(
        g.perform_action(GameAction::Crew { vehicle, crew_creatures: vec![weakling] }).is_err(),
        "0-power crewer rejected"
    );
    // Interface Ace crews via toughness 4 ≥ 2.
    g.perform_action(GameAction::Crew { vehicle, crew_creatures: vec![ace] })
        .expect("toughness crews the vehicle");
    drain_stack(&mut g);
    assert!(
        !g.battlefield_find(ace).unwrap().tapped,
        "becomes-tapped trigger untapped Interface Ace on your turn"
    );
}
