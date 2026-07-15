//! Functionality tests for `catalog::sets::decks::recent65` — black aggro.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.compute_battlefield();
    let c = c.iter().find(|c| c.id == id).unwrap();
    (c.power, c.toughness)
}

#[test]
fn ruthless_cullblade_swings_on_low_life() {
    let mut g = two_player_game();
    let cb = g.add_card_to_battlefield(0, catalog::ruthless_cullblade());
    assert_eq!(pt(&g, cb), (2, 1), "base while opponent is above 10");
    g.players[1].life = 10;
    assert_eq!(pt(&g, cb), (4, 2), "+2/+1 while opponent at 10 or less");
}

#[test]
fn guul_draz_vampire_gains_intimidate_on_low_life() {
    let mut g = two_player_game();
    let gd = g.add_card_to_battlefield(0, catalog::guul_draz_vampire());
    assert!(!g.computed_permanent(gd).unwrap().keywords.contains(&Keyword::Intimidate));
    g.players[1].life = 8;
    assert_eq!(pt(&g, gd), (3, 2));
    assert!(g.computed_permanent(gd).unwrap().keywords.contains(&Keyword::Intimidate));
}

#[test]
fn bloodrite_invoker_drains_three() {
    let mut g = two_player_game();
    let bi = g.add_card_to_battlefield(0, catalog::bloodrite_invoker());
    g.clear_sickness(bi);
    g.players[0].mana_pool.add_colorless(8);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let (my, opp) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bi, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 3);
    assert_eq!(g.players[0].life, my + 3);
}

#[test]
fn nip_gwyllion_has_lifelink() {
    assert!(catalog::nip_gwyllion().keywords.contains(&Keyword::Lifelink));
}

#[test]
fn barony_vampire_is_a_three_two_vampire() {
    let d = catalog::barony_vampire();
    assert_eq!((d.power, d.toughness), (3, 2));
    assert!(d.subtypes.creature_types.contains(&crabomination::card::CreatureType::Vampire));
}

#[test]
fn nested_shambler_leaves_squirrels_equal_to_power() {
    let mut g = two_player_game();
    let ns = g.add_card_to_battlefield(0, catalog::nested_shambler());
    // Pump to power 3 → 3 Squirrels on death.
    g.battlefield_find_mut(ns).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.remove_to_graveyard_with_triggers(ns);
    drain_stack(&mut g);
    let sq: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").collect();
    assert_eq!(sq.len(), 3, "X = its (pumped) power");
    assert!(sq.iter().all(|c| c.tapped), "Squirrels enter tapped");
}

#[test]
fn duty_bound_dead_regenerates() {
    let mut g = two_player_game();
    let d = g.add_card_to_battlefield(0, catalog::duty_bound_dead());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: d, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("regen shield");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(d).unwrap().regeneration_shields, 1, "shield stamped");
    // Lethal damage is soaked by the regeneration shield.
    g.battlefield_find_mut(d).unwrap().damage = 5;
    g.check_state_based_actions();
    assert!(g.battlefield_find(d).is_some(), "regen shield saved it");
}
