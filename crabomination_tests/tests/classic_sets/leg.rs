//! Legends (LEG) — the CR 702.22 "bands with other" cycle
//! (`catalog::sets::leg`).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn bands(g: &GameState, id: crabomination::card::CardId) -> bool {
    g.computed_permanent(id)
        .map(|c| c.keywords.iter().any(|k| matches!(k, Keyword::BandsWithOther(_))))
        .unwrap_or(false)
}

/// Each band land grants only its own colour's legends.
#[test]
fn band_lands_grant_their_own_color_only() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::cathedral_of_serra());
    let green = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    assert!(!bands(&g, green), "white land, green legend");
    g.add_card_to_battlefield(0, catalog::adventurers_guildhouse());
    assert!(bands(&g, green));
}

/// The other three band lands round out the cycle and tap for {C}.
#[test]
fn the_rest_of_the_band_land_cycle_taps_for_colorless() {
    let mut g = main_phase();
    for def in
        [catalog::mountain_stronghold(), catalog::seafarers_quay(), catalog::unholy_citadel()]
    {
        let land = g.add_card_to_battlefield(0, def);
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("tap for mana");
    }
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3);
}

/// Master of the Hunt's Wolves band with each other.
#[test]
fn master_of_the_hunt_mints_banding_wolves() {
    let mut g = main_phase();
    let master = g.add_card_to_battlefield(0, catalog::master_of_the_hunt());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: master,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("mint");
    drain_stack(&mut g);
    let wolf = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Wolves of the Hunt")
        .map(|c| c.id)
        .expect("a Wolf");
    assert!(bands(&g, wolf));
}

/// Shelkin Brownie strips the grant for the turn.
#[test]
fn shelkin_brownie_strips_bands_with_other() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::adventurers_guildhouse());
    let brownie = g.add_card_to_battlefield(0, catalog::shelkin_brownie());
    g.clear_sickness(brownie);
    let legend = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    assert!(bands(&g, legend));
    g.perform_action(GameAction::ActivateAbility {
        card_id: brownie,
        ability_index: 0,
        target: Some(Target::Permanent(legend)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(!bands(&g, legend));
}

/// Tolaria's band-hosing tap only works during an upkeep step.
#[test]
fn tolaria_hoses_bands_only_at_upkeep() {
    let mut g = main_phase();
    let tolaria = g.add_card_to_battlefield(0, catalog::tolaria());
    let legend = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tolaria,
            ability_index: 1,
            target: Some(Target::Permanent(legend)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "not an upkeep step"
    );
}
