//! Functionality tests for `catalog::sets::decks::recent74`.

use crabomination::card::{CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

#[test]
fn blood_pet_sacrifices_for_black_mana() {
    let mut g = two_player_game();
    let pet = g.add_card_to_battlefield(0, catalog::blood_pet());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pet, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac for mana");
    assert!(g.battlefield_find(pet).is_none(), "Blood Pet sacrificed");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "one black added");
}

#[test]
fn foul_imp_costs_two_life_on_etb() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let imp = g.add_card_to_hand(0, catalog::foul_imp());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: imp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "lost 2 life on ETB");
}

#[test]
fn skyshroud_vampire_discards_to_pump() {
    let mut g = two_player_game();
    let vamp = g.add_card_to_battlefield(0, catalog::skyshroud_vampire());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a creature card to discard
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vamp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("discard-pump");
    drain_stack(&mut g);
    let p = g.computed_permanent(vamp).unwrap();
    assert_eq!((p.power, p.toughness), (5, 5), "3/3 → 5/5");
}

#[test]
fn kris_mage_pings_with_discard() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::kris_mage());
    g.clear_sickness(mage);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    let foe = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 1, "1 damage dealt");
}

#[test]
fn sabertooth_nishoba_has_dual_protection() {
    let d = catalog::sabertooth_nishoba();
    assert!(d.keywords.contains(&Keyword::Protection(Color::Blue)));
    assert!(d.keywords.contains(&Keyword::Protection(Color::Red)));
    assert!(d.keywords.contains(&Keyword::Trample));
}

#[test]
fn recent74_misc_stats() {
    assert_eq!((catalog::water_elemental().power, catalog::water_elemental().toughness), (5, 4));
    assert!(catalog::wall_of_water().keywords.contains(&Keyword::Defender));
    assert!(catalog::spitting_drake().keywords.contains(&Keyword::Flying));
    assert!(catalog::feral_shadow().subtypes.creature_types.contains(&CreatureType::Nightstalker));
    assert_eq!((catalog::rowan_treefolk().power, catalog::rowan_treefolk().toughness), (3, 4));
}
