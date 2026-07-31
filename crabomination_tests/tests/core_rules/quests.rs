//! Functionality tests for `catalog::sets::decks::quests` — the Zendikar
//! Quest cycle remainder.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

/// Quest for Pure Flame accrues on your damage to an opponent, and its sac
/// doubles your sources' damage for the turn.
#[test]
fn quest_for_pure_flame_accrues_and_doubles() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_pure_flame());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Bolt the opponent: 3 damage → one quest counter.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(quest).unwrap().counter_count(CounterType::Quest), 1);
    // Top up to four, sacrifice for the doubling, then a second bolt hits for 6.
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: quest, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac the quest");
    drain_stack(&mut g);
    assert!(g.battlefield_find(quest).is_none(), "quest sacrificed");
    let life = g.players[1].life;
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt 2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 6, "doubled bolt");
}

/// An opponent's damage doesn't accrue quest counters.
#[test]
fn quest_for_pure_flame_ignores_opponent_damage() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_pure_flame());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(quest).unwrap().counter_count(CounterType::Quest), 0);
}

/// Quest for Ula's Temple: creature on top at upkeep → counter; at 3+ counters
/// the end step deploys a sea monster from hand.
#[test]
fn quest_for_ulas_temple_accrues_and_deploys() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_ulas_temple());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(quest).unwrap().counter_count(CounterType::Quest), 1,
        "creature reveal accrues");
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 2);
    let kraken = g.add_card_to_hand(0, catalog::nadir_kraken());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![kraken]),
    ]));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(kraken).is_some(), "Kraken deployed from hand");
}

/// Quest for the Nihil Stone: opponent discards accrue; an empty-handed
/// opponent's upkeep with 2+ counters costs them 5 life.
#[test]
fn quest_for_the_nihil_stone_drains_empty_handed_opponent() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_nihil_stone());
    let junk = g.add_card_to_hand(1, catalog::island());
    let mut evs = Vec::new();
    g.discard_card(1, junk, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(quest).unwrap().counter_count(CounterType::Quest), 1,
        "opponent discard accrues");
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 1);
    let life = g.players[1].life;
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 5, "empty-handed opponent loses 5");
}
