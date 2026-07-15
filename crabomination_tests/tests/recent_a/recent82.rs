//! Functionality tests for `catalog::sets::decks::recent82`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;

#[test]
fn alloy_myr_taps_for_any_color() {
    let mut g = two_player_game();
    let myr = g.add_card_to_battlefield(0, catalog::alloy_myr());
    g.clear_sickness(myr);
    g.perform_action(GameAction::ActivateAbility {
        card_id: myr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap Alloy Myr");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

#[test]
fn couriers_capsule_sacrifices_to_draw_two() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::couriers_capsule());
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cap, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Courier's Capsule");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
    assert!(!g.battlefield.iter().any(|c| c.id == cap), "capsule sacrificed");
}

#[test]
fn ballista_squad_pings_for_x() {
    let mut g = two_player_game();
    let bs = g.add_card_to_battlefield(0, catalog::ballista_squad());
    g.clear_sickness(bs);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bs, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: Some(2),
    }).expect("activate Ballista Squad for X=2");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "2 damage killed the 2/2");
}

#[test]
fn gelectrode_untaps_on_instant_or_sorcery() {
    let mut g = two_player_game();
    let gel = g.add_card_to_battlefield(0, catalog::gelectrode());
    g.clear_sickness(gel);
    // Ping to tap it.
    g.perform_action(GameAction::ActivateAbility {
        card_id: gel, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gel).unwrap().tapped, "tapped after activating");
    // Cast an instant → untaps Gelectrode.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    assert!(!g.battlefield_find(gel).unwrap().tapped, "untapped by the I/S cast");
}

#[test]
fn rally_the_peasants_pumps_team_and_has_flashback() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rally_the_peasants());
    assert!(g.find_card_anywhere(id).unwrap().definition.keywords.iter()
        .any(|k| matches!(k, Keyword::Flashback(_))), "has flashback");
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 → 4 power");
}

#[test]
fn tempered_steel_pumps_only_artifact_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tempered_steel());
    let myr = g.add_card_to_battlefield(0, catalog::alloy_myr()); // artifact creature 2/2
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-artifact
    assert_eq!(g.computed_permanent(myr).unwrap().power, 4, "artifact creature gets +2/+2");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "non-artifact unaffected");
}

#[test]
fn radiant_destiny_anthems_the_chosen_type() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let rd = g.add_card_to_battlefield(0, catalog::radiant_destiny());
    g.battlefield_find_mut(rd).unwrap().chosen_creature_type = Some(CreatureType::Bear);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "chosen-type Bear gets +1/+1");
}

#[test]
fn fires_of_yavimaya_grants_haste_and_sacs_to_pump() {
    let mut g = two_player_game();
    let fires = g.add_card_to_battlefield(0, catalog::fires_of_yavimaya());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
        "creatures you control have haste");
    g.perform_action(GameAction::ActivateAbility {
        card_id: fires, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("sac Fires for +2/+2");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fires), "Fires sacrificed");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+2 → 4 power");
}
