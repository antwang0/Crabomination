//! Functionality tests for the `catalog::sets::decks::recent16` batch.

use crate::card::{CardType, CreatureType};
use crate::catalog;
use crate::game::*;

/// Throne of the God-Pharaoh drains each opponent for the number of tapped
/// creatures you control at your end step.
#[test]
fn throne_drains_by_tapped_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::throne_of_the_god_pharaoh());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // untapped — not counted
    g.battlefield.iter_mut().find(|c| c.id == a).unwrap().tapped = true;
    let life = g.players[1].life;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "one tapped creature → opponent loses 1");
}

/// Su-Chi adds four colorless mana when it dies.
#[test]
fn su_chi_adds_four_mana_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::su_chi());
    assert_eq!(g.players[0].mana_pool.total(), 0);
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 4, "four colorless on death");
}

/// Icon of Ancestry buffs creatures of the chosen type.
#[test]
fn icon_of_ancestry_buffs_chosen_type() {
    let mut g = two_player_game();
    let icon = g.add_card_to_battlefield(0, catalog::icon_of_ancestry());
    g.battlefield_find_mut(icon).unwrap().chosen_creature_type = Some(CreatureType::Elf);
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // 1/1 Elf
    let cp = g.computed_permanent(elf).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "+1/+1 for the chosen type");
}

/// Aeolipile sacrifices itself to deal 2 damage to any target.
#[test]
fn aeolipile_pings_for_two() {
    let mut g = two_player_game();
    let pile = g.add_card_to_battlefield(0, catalog::aeolipile());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pile, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("activate Aeolipile");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.battlefield_find(pile).is_none(), "sacrificed as a cost");
}

/// Phyrexian Vault sacrifices a creature to draw a card.
#[test]
fn phyrexian_vault_sacs_for_a_card() {
    let mut g = two_player_game();
    let vault = g.add_card_to_battlefield(0, catalog::phyrexian_vault());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: vault, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Vault");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Vanquisher's Banner buffs creatures of the chosen type.
#[test]
fn vanquishers_banner_buffs_chosen_type() {
    let mut g = two_player_game();
    let banner = g.add_card_to_battlefield(0, catalog::vanquishers_banner());
    g.battlefield_find_mut(banner).unwrap().chosen_creature_type = Some(CreatureType::Elf);
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    assert_eq!(g.computed_permanent(elf).unwrap().power, 2, "+1/+1 for the chosen type");
}

/// Secluded Courtyard is a chosen-type mana land with two mana abilities.
#[test]
fn secluded_courtyard_is_a_chosen_type_land() {
    let d = catalog::secluded_courtyard();
    assert!(d.card_types.contains(&CardType::Land));
    assert_eq!(d.activated_abilities.len(), 2, "colorless + chosen-type-restricted mana");
    // ETB chooses a creature type.
    assert!(d.triggered_abilities.iter().any(|t| matches!(
        t.effect,
        crate::effect::Effect::NameCreatureType { .. }
    )));
}
