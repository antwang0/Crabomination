//! Functionality tests for `catalog::sets::decks::recent72`.

use crabomination::card::{CreatureType, Keyword, LandType};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::*;

#[test]
fn yavimaya_enchantress_grows_with_enchantments() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::yavimaya_enchantress());
    assert_eq!(g.computed_permanent(id).unwrap().power, 2, "no enchantments → 2/2");
    // An enchantment either player controls grows it (counts all in play).
    g.add_card_to_battlefield(0, catalog::wild_growth());
    g.add_card_to_battlefield(1, catalog::wild_growth());
    let p = g.computed_permanent(id).unwrap();
    assert_eq!((p.power, p.toughness), (4, 4), "two enchantments in play → 4/4");
}

#[test]
fn zombie_master_grants_swampwalk_to_other_zombies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zombie_master());
    let ally = g.add_card_to_battlefield(0, catalog::scathe_zombies());
    assert!(
        g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Landwalk(LandType::Swamp)),
        "other Zombie gains swampwalk from the Master",
    );
    // The Master itself does not gain swampwalk ("other").
    let master = g.battlefield.iter().find(|c| c.definition.name == "Zombie Master").unwrap().id;
    assert!(
        !g.computed_permanent(master).unwrap().keywords.contains(&Keyword::Landwalk(LandType::Swamp)),
        "the Master is excluded (other Zombies only)",
    );
}

#[test]
fn zombie_master_grants_regen_ability_to_other_zombies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zombie_master());
    let ally = g.add_card_to_battlefield(0, catalog::scathe_zombies());
    assert!(!g.granted_abilities_for(ally).is_empty(), "the granted {{B}}: Regenerate ability is present");
}

#[test]
fn cudgel_troll_regenerates() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cudgel_troll());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate regen");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().regeneration_shields, 1, "shield stamped");
    g.battlefield_find_mut(id).unwrap().damage = 3;
    g.check_state_based_actions();
    assert!(g.battlefield_find(id).is_some(), "regen shield saved the Troll from lethal");
}

#[test]
fn radjan_spirit_strips_flying() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let spirit = g.add_card_to_battlefield(0, catalog::radjan_spirit());
    g.clear_sickness(spirit);
    let flyer = g.add_card_to_battlefield(1, catalog::air_elemental());
    assert!(g.computed_permanent(flyer).unwrap().keywords.contains(&Keyword::Flying));
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spirit, ability_index: 0, target: Some(Target::Permanent(flyer)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(flyer).unwrap().keywords.contains(&Keyword::Flying),
        "target lost flying this turn");
}

#[test]
fn deadly_insect_has_shroud() {
    let d = catalog::deadly_insect();
    assert_eq!((d.power, d.toughness), (6, 1));
    assert!(d.keywords.contains(&Keyword::Shroud));
}

#[test]
fn retro_vanilla_and_keyword_stats() {
    assert!(catalog::longbow_archer().keywords.contains(&Keyword::FirstStrike));
    assert!(catalog::longbow_archer().keywords.contains(&Keyword::Reach));
    assert!(catalog::talruum_minotaur().keywords.contains(&Keyword::Haste));
    assert_eq!((catalog::giant_octopus().power, catalog::giant_octopus().toughness), (3, 3));
    assert_eq!((catalog::balduvian_bears().power, catalog::balduvian_bears().toughness), (2, 2));
    assert!(catalog::norwood_ranger().subtypes.creature_types.contains(&CreatureType::Scout));
}
