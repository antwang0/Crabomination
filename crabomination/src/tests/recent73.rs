//! Functionality tests for `catalog::sets::decks::recent73`.

use crate::card::{CounterType, CreatureType, Keyword, LandType};
use crate::catalog;
use crate::game::two_player_game;
use crate::game::types::Target;
use crate::game::*;

#[test]
fn bog_rats_cant_be_blocked_by_walls() {
    let mut g = two_player_game();
    let rats = g.add_card_to_battlefield(0, catalog::bog_rats());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_ice());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(rats);
    assert!(!g.blocker_can_block_attacker(wall, rats), "a Wall can't block Bog Rats");
    assert!(g.blocker_can_block_attacker(bear, rats), "a non-Wall can block it");
}

#[test]
fn serrated_arrows_enters_with_three_and_shoots() {
    let mut g = two_player_game();
    // The printed `enters_with_counters` spec applies on the real ETB path; the
    // test helper bypasses it, so seed the three arrowheads directly.
    assert_eq!(catalog::serrated_arrows().enters_with_counters.unwrap().0, CounterType::Charge);
    let arrows = g.add_card_to_battlefield(0, catalog::serrated_arrows());
    g.battlefield_find_mut(arrows).unwrap().add_counters(CounterType::Charge, 3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: arrows, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("shoot");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(arrows).unwrap().counter_count(CounterType::Charge), 2, "spent one");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1,
        "-1/-1 counter placed");
}

#[test]
fn ghitu_slinger_pings_on_etb() {
    let mut g = two_player_game();
    let foe = g.players[1].life;
    let slinger = g.add_card_to_hand(0, catalog::ghitu_slinger());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: slinger, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 2, "ETB deals 2");
    assert!(catalog::ghitu_slinger().keywords.iter().any(|k| matches!(k, Keyword::Echo(_))));
}

#[test]
fn skittering_skirge_sacrifices_on_creature_cast() {
    let mut g = two_player_game();
    let skirge = g.add_card_to_battlefield(0, catalog::skittering_skirge());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature spell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(skirge).is_none(), "Skirge sacrificed when a creature spell was cast");
}

#[test]
fn viashino_sandstalker_returns_at_end_step() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::viashino_sandstalker());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(v).is_none(), "returned to hand at end step");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Viashino Sandstalker"));
}

#[test]
fn recent73_static_stats() {
    assert!(catalog::shanodin_dryads().keywords.contains(&Keyword::Landwalk(LandType::Forest)));
    assert!(catalog::mesa_falcon().keywords.contains(&Keyword::Flying));
    assert_eq!((catalog::highland_giant().power, catalog::highland_giant().toughness), (3, 4));
    assert!(catalog::ghitu_slinger().subtypes.creature_types.contains(&CreatureType::Nomad));
    assert!(catalog::cackling_fiend().subtypes.creature_types.contains(&CreatureType::Zombie));
}
