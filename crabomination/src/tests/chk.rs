//! Champions of Kamigawa — Spirit/Arcane bodies: Offering Patrons (tested in
//! `cr_rules`), Bushido, Soulshift, sacrifice/tap activated abilities.

use super::*;
use crate::card::Keyword;
use crate::catalog;
use crate::game::two_player_game;

/// Kami of Ancient Law's "Sacrifice this creature: Destroy target enchantment."
#[test]
fn kami_of_ancient_law_sacrifices_to_destroy_enchantment() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_ancient_law());
    let ench = g.add_card_to_battlefield(1, catalog::concordant_crossroads());
    g.clear_sickness(kami);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(ench)), x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == kami), "Kami sacrificed as a cost");
    assert!(!g.battlefield.iter().any(|c| c.id == ench), "enchantment destroyed");
}

/// Kabuto Moth's "{T}: target creature gets +1/+2 until end of turn."
#[test]
fn kabuto_moth_taps_to_pump() {
    let mut g = two_player_game();
    let moth = g.add_card_to_battlefield(0, catalog::kabuto_moth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(moth);
    g.perform_action(GameAction::ActivateAbility {
        card_id: moth, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("tap to pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+2 applied");
    assert!(g.battlefield_find(moth).unwrap().tapped, "Moth tapped for the ability");
}

/// Gibbering Kami's Soulshift 3 returns a Spirit with MV ≤ 3 on death.
#[test]
fn gibbering_kami_soulshift_returns_small_spirit() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::gibbering_kami());
    let spirit = g.add_card_to_graveyard(0, catalog::kami_of_ancient_law()); // Spirit, MV 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let events = g.remove_to_graveyard_with_triggers(kami);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spirit), "soulshift returned the Spirit");
}

/// Kitsune Blademaster carries First strike + Bushido 1.
#[test]
fn kitsune_blademaster_has_first_strike_and_bushido() {
    let d = catalog::kitsune_blademaster();
    assert!(d.keywords.contains(&Keyword::FirstStrike));
    assert!(d.keywords.contains(&Keyword::Bushido(1)));
}

/// Kodama of the North Tree is a 6/4 with Trample + Shroud.
#[test]
fn kodama_of_the_north_tree_trample_shroud() {
    let d = catalog::kodama_of_the_north_tree();
    assert_eq!((d.power, d.toughness), (6, 4));
    assert!(d.keywords.contains(&Keyword::Trample) && d.keywords.contains(&Keyword::Shroud));
}

/// Spiritcraft: casting an Arcane spell triggers Kami of Fire's Roar, making
/// a target creature unable to block.
#[test]
fn kami_of_fires_roar_spiritcraft_on_arcane_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kami_of_fires_roar());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists()); // {U} Arcane
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantBlock),
        "spiritcraft granted CantBlock to the opponent's creature");
}

/// Nezumi Cutthroat has Fear and can't block.
#[test]
fn nezumi_cutthroat_fear_cant_block() {
    let d = catalog::nezumi_cutthroat();
    assert!(d.keywords.contains(&Keyword::Fear) && d.keywords.contains(&Keyword::CantBlock));
}

/// Befoul destroys a land.
#[test]
fn befoul_destroys_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::befoul());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Befoul on a land");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == land), "land destroyed");
}

/// Rend Flesh destroys a non-Spirit creature but can't target a Spirit.
#[test]
fn rend_flesh_only_hits_non_spirits() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spirit = g.add_card_to_battlefield(1, catalog::river_kaijin()); // Spirit
    let id = g.add_card_to_hand(0, catalog::rend_flesh());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(spirit)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "can't target a Spirit");
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("destroy a non-Spirit");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "non-Spirit destroyed");
}

/// Yamabushi's Flame exiles a creature it would kill instead of it dying.
#[test]
fn yamabushis_flame_exiles_what_it_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::yamabushis_flame());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("burn the bear");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear is gone");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "exiled, not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear), "bear exiled");
}

/// Kami of Twisted Reflection sacrifices itself to bounce your own creature.
#[test]
fn kami_of_twisted_reflection_bounces_your_creature() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_twisted_reflection());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(kami);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("sac to bounce");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == kami), "Kami sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "your creature returned to hand");
}

/// Sokenzan Bruiser / Moss Kami carry their evasion / combat keywords.
#[test]
fn kamigawa_vanilla_keywords() {
    assert!(catalog::sokenzan_bruiser().keywords
        .contains(&Keyword::Landwalk(crate::card::LandType::Mountain)));
    assert!(catalog::moss_kami().keywords.contains(&Keyword::Trample));
    let numai = catalog::numai_outcast();
    assert!(numai.keywords.contains(&Keyword::Bushido(2)));
    assert_eq!(numai.activated_abilities.len(), 1, "regenerate ability present");
}
