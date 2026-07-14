//! Functionality tests for `catalog::sets::decks::recent190` (WOE/OTJ gaps).

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::two_player_game;
use crate::game::*;
use crate::mana::Color;

/// Rowdy Research costs {1} less per attacker and draws three.
#[test]
fn rowdy_research_affinity_and_draw() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    // Two creatures that attacked this turn → {6}{U} becomes {4}{U}.
    for _ in 0..2 {
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(a).unwrap().attacked_this_turn = true;
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::rowdy_research());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{4}{U} covers the discounted cost");
    drain_stack(&mut g);
    // -1 for the spell leaving hand, +3 drawn.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3, "drew three");
}

/// Brave the Wilds bargained animates a land and tutors a basic to hand.
#[test]
fn brave_the_wilds_bargained_animates_land() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let fodder = g.add_card_to_battlefield(0, catalog::howling_mine()); // artifact to bargain
    let basic = g.add_card_to_library(0, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::brave_the_wilds());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    g.perform_action(GameAction::CastSpellBargain {
        card_id: spell,
        sacrifice: Some(fodder),
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Brave the Wilds bargained");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "land animated to 3/3");
    assert!(cp.card_types.contains(&crate::card::CardType::Land), "still a land");
    assert!(g.players[0].hand.iter().any(|c| c.id == basic), "tutored a basic to hand");
}

/// Unbargained, Brave the Wilds only tutors — the land isn't animated.
#[test]
fn brave_the_wilds_unbargained_only_tutors() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let basic = g.add_card_to_library(0, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::brave_the_wilds());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Brave the Wilds");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(land).unwrap().card_types.contains(&crate::card::CardType::Creature), "not animated");
    assert!(g.players[0].hand.iter().any(|c| c.id == basic), "still tutored a basic");
}

/// Redrock Sentinel sacrifices a land to draw and make a Treasure.
#[test]
fn redrock_sentinel_sacs_land_for_value() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let sentinel = g.add_card_to_battlefield(0, catalog::redrock_sentinel());
    g.clear_sickness(sentinel);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: sentinel,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Redrock Sentinel");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "sacrificed a land");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
        "made a Treasure",
    );
}
