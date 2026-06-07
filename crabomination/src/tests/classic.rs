//! Functionality tests for the classic core-set bodies added to the `lea`
//! set (`catalog::{gray_ogre, royal_assassin, essence_scatter, …}`). Each
//! card is cast (or set up) and its marquee behavior asserted.

use crate::card::{CardType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::TurnStep;
use crate::mana::Color;

/// Give a player a heap of every color + colorless so any spell is payable.
fn flood_mana(g: &mut GameState, p: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[p].mana_pool.add(c, 12);
    }
    g.players[p].mana_pool.add_colorless(12);
}

/// Cast a creature from hand and return its battlefield id, asserting it
/// resolved onto the battlefield.
fn cast_creature(g: &mut GameState, def: crate::card::CardDefinition) -> CardId {
    let name = def.name;
    let id = g.add_card_to_hand(0, def);
    flood_mana(g, 0);
    cast(g, id);
    assert!(g.battlefield_find(id).is_some(), "{name} should resolve to the battlefield");
    id
}

type BodyCase = (fn() -> crate::card::CardDefinition, i32, i32, &'static [Keyword]);

#[test]
fn classic_vanilla_bodies_have_correct_stats() {
    // One cast each — confirms cost is payable and base P/T + keywords land.
    let cases: &[BodyCase] = &[
        (catalog::gray_ogre, 2, 2, &[]),
        (catalog::hurloon_minotaur, 2, 3, &[]),
        (catalog::spined_wurm, 5, 4, &[]),
        (catalog::trained_armodon, 3, 3, &[]),
        (catalog::pearled_unicorn, 2, 2, &[]),
        (catalog::obsianus_golem, 4, 6, &[]),
        (catalog::eager_cadet, 1, 1, &[]),
        (catalog::elite_vanguard, 2, 1, &[]),
        (catalog::devoted_hero, 1, 2, &[]),
        (catalog::giant_spider, 2, 4, &[Keyword::Reach]),
        (catalog::air_elemental, 4, 4, &[Keyword::Flying]),
        (catalog::scryb_sprites, 1, 1, &[Keyword::Flying]),
        (catalog::tundra_wolves, 1, 1, &[Keyword::FirstStrike]),
        (catalog::mesa_pegasus, 1, 1, &[Keyword::Flying, Keyword::Banding]),
        (catalog::wall_of_air, 1, 5, &[Keyword::Defender, Keyword::Flying]),
        (catalog::wall_of_swords, 3, 5, &[Keyword::Defender, Keyword::Flying]),
        (catalog::wall_of_wood, 0, 3, &[Keyword::Defender]),
        (catalog::wall_of_stone, 0, 8, &[Keyword::Defender]),
        (catalog::yotian_soldier, 1, 4, &[Keyword::Vigilance]),
        // batch 2
        (catalog::youthful_knight, 2, 1, &[Keyword::FirstStrike]),
        (catalog::standing_troops, 1, 4, &[Keyword::Vigilance]),
        (catalog::benalish_hero, 1, 1, &[Keyword::Banding]),
        (catalog::skyhunter_skirmisher, 1, 1, &[Keyword::Flying, Keyword::DoubleStrike]),
        (catalog::knight_errant, 2, 2, &[]),
        (catalog::snapping_drake, 3, 2, &[Keyword::Flying]),
        (catalog::phantom_warrior, 2, 2, &[Keyword::Unblockable]),
        (catalog::merfolk_of_the_pearl_trident, 1, 1, &[]),
        (catalog::vodalian_soldiers, 1, 2, &[]),
        (catalog::sea_eagle, 1, 1, &[Keyword::Flying]),
        (catalog::wind_spirit, 2, 3, &[Keyword::Flying]),
        (catalog::scathe_zombies, 2, 2, &[]),
        (catalog::walking_corpse, 2, 2, &[]),
        (catalog::bog_imp, 1, 1, &[Keyword::Flying]),
        (catalog::severed_legion, 2, 2, &[Keyword::Fear]),
        (catalog::mons_goblin_raiders, 1, 1, &[]),
        (catalog::raging_goblin, 1, 1, &[Keyword::Haste]),
        (catalog::goblin_piker, 2, 1, &[]),
        (catalog::goblin_chariot, 2, 2, &[Keyword::Haste]),
        (catalog::panther_warriors, 6, 1, &[]),
        (catalog::redwood_treefolk, 3, 6, &[]),
    ];
    for (factory, p, t, kws) in cases {
        let mut g = two_player_game();
        let def = factory();
        let name = def.name;
        let id = cast_creature(&mut g, def);
        let c = g.battlefield_find(id).unwrap();
        assert_eq!((c.power(), c.toughness()), (*p, *t), "{name} P/T");
        for kw in *kws {
            assert!(c.has_keyword(kw), "{name} should have {kw:?}");
        }
    }
}

#[test]
fn yotian_soldier_is_an_artifact_creature() {
    let mut g = two_player_game();
    let id = cast_creature(&mut g, catalog::yotian_soldier());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.definition.card_types.contains(&CardType::Artifact));
    assert!(c.definition.card_types.contains(&CardType::Creature));
}

#[test]
fn royal_assassin_destroys_a_tapped_creature() {
    let mut g = two_player_game();
    let assassin = g.add_card_to_battlefield(0, catalog::royal_assassin());
    g.clear_sickness(assassin);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: assassin, ability_index: 0,
        target: Some(Target::Permanent(victim)), x_value: None,
    }).expect("activate Royal Assassin on a tapped creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "tapped creature is destroyed");
}

#[test]
fn royal_assassin_cannot_target_an_untapped_creature() {
    let mut g = two_player_game();
    let assassin = g.add_card_to_battlefield(0, catalog::royal_assassin());
    g.clear_sickness(assassin);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // untapped
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: assassin, ability_index: 0,
        target: Some(Target::Permanent(victim)), x_value: None,
    });
    assert!(res.is_err(), "untapped creature is not a legal target");
    assert!(g.battlefield_find(victim).is_some(), "victim survives");
}

#[test]
fn wall_of_fire_pumps_its_power() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_fire());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None, x_value: None,
    }).expect("firebreathing");
    drain_stack(&mut g);
    let c = g.battlefield_find(wall).unwrap();
    assert_eq!((c.power(), c.toughness()), (1, 5), "{{R}}: +1/+0 makes the 0/5 a 1/5");
}

#[test]
fn goblin_balloon_brigade_gains_flying() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::goblin_balloon_brigade());
    assert!(!g.battlefield_find(gob).unwrap().has_keyword(&Keyword::Flying), "starts grounded");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gob, ability_index: 0, target: None, x_value: None,
    }).expect("gain flying");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gob).unwrap().has_keyword(&Keyword::Flying), "now flying");
}

#[test]
fn venerable_monk_gains_two_life_on_etb() {
    let mut g = two_player_game();
    let before = g.players[0].life;
    let _ = cast_creature(&mut g, catalog::venerable_monk());
    assert_eq!(g.players[0].life, before + 2, "ETB gains 2 life");
}

#[test]
fn looming_shade_pumps_plus_one_plus_one() {
    let mut g = two_player_game();
    let shade = g.add_card_to_battlefield(0, catalog::looming_shade());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shade, ability_index: 0, target: None, x_value: None,
    }).expect("shade pump");
    drain_stack(&mut g);
    let c = g.battlefield_find(shade).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "{{B}}: +1/+1 makes the 2/2 a 3/3");
}

#[test]
fn gorilla_chieftain_stamps_a_regeneration_shield() {
    let mut g = two_player_game();
    let ape = g.add_card_to_battlefield(0, catalog::gorilla_chieftain());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ape, ability_index: 0, target: None, x_value: None,
    }).expect("{1}{G}: Regenerate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ape).unwrap().regeneration_shields, 1,
        "regenerate stamps a shield");
}

#[test]
fn mountain_goat_is_unblockable_when_defender_has_a_mountain() {
    let mut g = two_player_game();
    let goat = g.add_card_to_battlefield(0, catalog::mountain_goat());
    g.clear_sickness(goat);
    g.add_card_to_battlefield(1, catalog::mountain()); // defender controls a Mountain
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goat, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // The block must be rejected — mountainwalk makes the Goat unblockable.
    let res = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, goat)]));
    assert!(res.is_err(), "mountainwalk: can't be blocked while defender has a Mountain");
}

#[test]
fn essence_scatter_counters_a_creature_spell() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    // P1 casts the bear; it goes on the stack.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    // P0 responds with Essence Scatter.
    let scatter = g.add_card_to_hand(0, catalog::essence_scatter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: scatter, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter the creature spell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the bear was countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "countered spell hits graveyard");
}

#[test]
fn essence_scatter_cannot_counter_a_noncreature_spell() {
    let mut g = two_player_game();
    // A noncreature spell (Lightning Bolt) on the stack is not a legal target.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    let scatter = g.add_card_to_hand(0, catalog::essence_scatter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    let res = g.perform_action(GameAction::CastSpell {
        card_id: scatter, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(res.is_err(), "Essence Scatter can't target a noncreature spell");
}

/// Smoke (LEA) — creatures don't untap during untap steps, but noncreature
/// permanents (lands) untap normally.
#[test]
fn smoke_prevents_creatures_from_untapping() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::smoke());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "Smoke keeps the creature tapped");
    assert!(!g.battlefield_find(land).unwrap().tapped, "Smoke leaves noncreature permanents alone");
}
