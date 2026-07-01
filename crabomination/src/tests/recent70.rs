//! Functionality tests for `catalog::sets::decks::recent70`.

use crate::card::{CreatureType, Keyword, LandType};
use crate::catalog;
use crate::game::two_player_game;
use crate::game::*;

#[test]
fn krosan_archer_discards_to_pump_toughness() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::krosan_archer());
    g.add_card_to_hand(0, catalog::island()); // card to discard
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().toughness, 5, "2/3 → 2/5");
    assert!(catalog::krosan_archer().keywords.contains(&Keyword::Reach));
}

#[test]
fn dwarven_grunt_has_mountainwalk() {
    assert!(catalog::dwarven_grunt().keywords.contains(&Keyword::Landwalk(LandType::Mountain)));
}

#[test]
fn vengeful_firebrand_haste_gated_on_warrior_in_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vengeful_firebrand());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Haste),
        "no Warrior in graveyard → no haste");
    g.add_card_to_graveyard(0, catalog::sabertooth_outrider()); // a Human Warrior card
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Haste),
        "Warrior card in graveyard → haste");
}

#[test]
fn anaba_shaman_pings_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::anaba_shaman());
    g.clear_sickness(id);
    let foe_life = g.players[1].life;
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "1 damage to the opponent");
}

#[test]
fn balduvian_barbarians_is_a_vanilla_3_2() {
    let c = catalog::balduvian_barbarians();
    assert_eq!((c.power, c.toughness), (3, 2));
    assert!(c.subtypes.creature_types.contains(&CreatureType::Barbarian));
}

#[test]
fn zephyr_falcon_flies_with_vigilance() {
    let d = catalog::zephyr_falcon();
    assert!(d.keywords.contains(&Keyword::Flying) && d.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn regal_unicorn_is_a_vanilla_2_3() {
    let c = catalog::regal_unicorn();
    assert_eq!((c.power, c.toughness), (2, 3));
    assert!(c.subtypes.creature_types.contains(&CreatureType::Unicorn));
}
