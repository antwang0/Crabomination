//! Functionality tests for The Brothers' War (BRO) — the Prototype mechanic
//! (CR 702.160) and the prototype artifact creatures.

use crate::card::{Keyword, WardCost};
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

/// Helper: flood a seat with plenty of every color + colorless mana.
fn flood_mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Cast for the full {6} cost: a colorless 5/4 Construct.
#[test]
fn goring_warplow_full_cost_is_colorless_5_4() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goring_warplow());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast full");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).expect("on battlefield");
    assert_eq!((cp.power, cp.toughness), (5, 4));
    assert!(cp.colors.is_empty(), "full-cost prototype is colorless");
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
}

/// Cast for the prototype {1}{B} cost: a black 1/1 that keeps its abilities.
#[test]
fn goring_warplow_prototype_is_black_1_1_with_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goring_warplow());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1); // {1}{B}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).expect("on battlefield");
    assert_eq!((cp.power, cp.toughness), (1, 1), "prototype size");
    assert_eq!(cp.colors, vec![Color::Black], "prototype color follows its cost");
    assert!(cp.keywords.contains(&Keyword::Deathtouch), "keeps abilities");
    let r = g.battlefield_find(id).unwrap();
    assert!(r.cast_as_prototype);
    assert_eq!(r.definition.cost.cmc(), 2, "prototype mana value");
}

/// A prototype creature round-trips its smaller cost/color/size through a
/// name→factory snapshot (CR 702.160c copiable values).
#[test]
fn prototype_state_survives_snapshot_roundtrip() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blitz_automaton());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{R}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: GameState = serde_json::from_str(&json).expect("deserialize");
    let cp = g2.computed_permanent(id).expect("on battlefield after restore");
    assert_eq!((cp.power, cp.toughness), (3, 2));
    assert_eq!(cp.colors, vec![Color::Red]);
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Combat Thresher's ETB draws a card regardless of cast mode.
#[test]
fn combat_thresher_prototype_draws_and_double_strikes() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::combat_thresher());
    let lib_id = g.next_id();
    g.players[0].library.push(crate::card::CardInstance::new(
        lib_id, catalog::goring_warplow(), 0,
    ));
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    // The spell left hand (−1) but the ETB drew a card (+1) → net same.
    assert_eq!(g.players[0].hand.len(), before);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
}

/// Boulderbranch Golem gains life equal to its power on ETB — the prototype
/// face gains 3 (its 3/3), not the full 6.
#[test]
fn boulderbranch_golem_prototype_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::boulderbranch_golem());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{G}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gain life = prototype power 3");
}

/// Cradle Clearcutter taps for {G} equal to its power (prototype 1/3 → 1).
#[test]
fn cradle_clearcutter_taps_for_power_in_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cradle_clearcutter());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
        c.summoning_sick = false;
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    // Full-cost body is a 3/6, so it taps for 3 green.
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
}

/// Phyrexian Fleshgorger's Ward—Pay life equal to its power: targeting the
/// full-cost 7/5 with an opponent's removal costs 7 life or the spell is
/// countered by Ward.
#[test]
fn fleshgorger_ward_costs_life_equal_to_power() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::phyrexian_fleshgorger());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 5));
    assert!(cp.keywords.contains(&Keyword::Ward(WardCost::LifeSourcePower)));
    assert!(cp.keywords.contains(&Keyword::Menace));
    assert!(cp.keywords.contains(&Keyword::Lifelink));
    // P1 tries to Shock it: Ward triggers, P1 must pay 7 life.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    // Ward auto-paid 7 life; the creature survives the 3-damage bolt (7 tough).
    assert_eq!(g.players[1].life, p1_life - 7, "Ward—pay 7 life (its power)");
    assert!(g.battlefield_find(id).is_some(), "Fleshgorger survives the bolt");
}

/// Frogmyr Enforcer's Affinity for artifacts reduces the prototype cost by
/// {1} per artifact controlled.
#[test]
fn frogmyr_enforcer_affinity_reduces_prototype_cost() {
    let mut g = two_player_game();
    // Two artifacts in play → affinity {2}.
    g.add_card_to_battlefield(0, catalog::goring_warplow());
    g.add_card_to_battlefield(0, catalog::blitz_automaton());
    let id = g.add_card_to_hand(0, catalog::frogmyr_enforcer());
    // Prototype {3}{R} − {2} affinity = {1}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("affinity-discounted prototype");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert_eq!(cp.colors, vec![Color::Red]);
}

/// Skitterbeam Battalion's ETB makes two token copies of itself (prototype
/// 2/2 face → two 2/2 tokens).
#[test]
fn skitterbeam_battalion_prototype_mints_two_copies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skitterbeam_battalion());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3); // {3}{R}{R}
    g.perform_action(GameAction::CastPrototype {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast prototype");
    drain_stack(&mut g);
    let battalions = g.battlefield.iter()
        .filter(|c| c.definition.name == "Skitterbeam Battalion" && c.controller == 0)
        .count();
    assert_eq!(battalions, 3, "original + two token copies");
}

/// The affordance probe surfaces a payable prototype cast.
#[test]
fn prototype_affordance_surfaced() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rust_goliath());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3); // {3}{G}{G}
    let aff = g.compute_hand_affordances(0);
    assert!(aff.prototypable.contains(&id), "prototype cast offered when payable");
    // Full {10} cost isn't available, so the plain cast is not.
    assert!(!aff.castable.contains(&id));
}
