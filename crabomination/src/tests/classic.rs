//! Functionality tests for the classic core-set bodies added to the `lea`
//! set (`catalog::{gray_ogre, royal_assassin, essence_scatter, …}`). Each
//! card is cast (or set up) and its marquee behavior asserted.

use crate::card::{CardType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
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
