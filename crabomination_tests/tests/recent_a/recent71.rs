//! Functionality tests for `catalog::sets::decks::recent71`.

use crabomination::card::{CreatureType, Keyword, LandType};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::*;

#[test]
fn nightmare_pt_tracks_swamps_you_control() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::nightmare());
    assert_eq!(g.computed_permanent(id).unwrap().power, 0, "no Swamps → 0/0");
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let p = g.computed_permanent(id).unwrap();
    assert_eq!((p.power, p.toughness), (2, 2), "two Swamps → 2/2");
    assert!(catalog::nightmare().keywords.contains(&Keyword::Flying));
}

#[test]
fn rukh_egg_mints_a_flying_bird_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rukh_egg());
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    let bird = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Bird")
        .expect("Rukh token minted");
    assert_eq!((bird.definition.power, bird.definition.toughness), (4, 4));
    assert!(bird.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn sabertooth_tiger_has_first_strike() {
    assert!(catalog::sabertooth_tiger().keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn segovian_leviathan_has_islandwalk() {
    assert!(catalog::segovian_leviathan().keywords.contains(&Keyword::Landwalk(LandType::Island)));
}

#[test]
fn vampire_bats_pumps_once_per_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vampire_bats());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("first activation");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, 2, "1/1 → 2/1");
    // Second activation the same turn is illegal (once per turn).
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "can't activate twice in one turn");
}

#[test]
fn wall_of_spears_is_defender_first_strike() {
    let d = catalog::wall_of_spears();
    assert!(d.keywords.contains(&Keyword::Defender) && d.keywords.contains(&Keyword::FirstStrike));
    assert!(d.card_types.contains(&crabomination::card::CardType::Artifact));
}

#[test]
fn rod_of_ruin_pings_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rod_of_ruin());
    let foe = g.players[1].life;
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 1, "1 damage to opponent");
}

#[test]
fn vanilla_bodies_have_expected_stats() {
    assert_eq!((catalog::ironroot_treefolk().power, catalog::ironroot_treefolk().toughness), (3, 5));
    assert_eq!((catalog::fire_elemental().power, catalog::fire_elemental().toughness), (5, 4));
    assert_eq!((catalog::dross_crocodile().power, catalog::dross_crocodile().toughness), (5, 1));
    assert_eq!((catalog::durkwood_boars().power, catalog::durkwood_boars().toughness), (5, 5));
    assert!(catalog::wall_of_ice().keywords.contains(&Keyword::Defender));
    assert!(catalog::dross_crocodile().subtypes.creature_types.contains(&CreatureType::Zombie));
}
