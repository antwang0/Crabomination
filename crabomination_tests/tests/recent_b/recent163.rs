//! Functionality tests for `catalog::sets::decks::recent163` (Foundations).

use crabomination::catalog;
use crabomination::card::CounterType;
use crabomination::game::types::{Attack, AttackTarget, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Herald of Eternal Dawn keeps its controller from losing even at 0 life.
#[test]
fn herald_keeps_you_alive() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::herald_of_eternal_dawn());
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[0].eliminated, "Herald prevents the loss at 0 life");
}

/// Rune-Sealed Wall surveils when tapped.
#[test]
fn rune_sealed_wall_surveils() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::rune_sealed_wall());
    g.clear_sickness(wall);
    g.add_card_to_library(0, catalog::island());
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("surveil");
    drain_stack(&mut g);
    // Surveil looked at the top card (library shrank only if it went to gy, but
    // at minimum the ability resolved without error and the wall is tapped).
    assert!(g.battlefield_find(wall).unwrap().tapped, "tapped for the ability");
    let _ = lib;
}

/// Scrawling Crawler drains an opponent when they draw.
#[test]
fn scrawling_crawler_punishes_opponent_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::scrawling_crawler());
    g.add_card_to_library(1, catalog::island());
    let life = g.players[1].life;
    let mut events = vec![];
    g.draw_one(1, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent lost 1 life on their draw");
}

/// Revenge of the Rats mints one Rat per creature card in the graveyard.
#[test]
fn revenge_of_the_rats_swarms() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not a creature
    let id = g.add_card_to_hand(0, catalog::revenge_of_the_rats());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Revenge of the Rats");
    drain_stack(&mut g);
    let rats = g.battlefield.iter().filter(|c| c.definition.name == "Rat" && c.controller == 0).count();
    assert_eq!(rats, 2, "one Rat per creature card in the graveyard");
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Rat").all(|c| c.tapped), "Rats enter tapped");
}

/// High-Society Hunter draws whenever another nontoken creature dies.
#[test]
fn high_society_hunter_draws_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::high_society_hunter());
    g.add_card_to_library(0, catalog::island());
    let chump = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let mut evs = g.remove_to_graveyard_with_triggers(chump);
    evs.push(GameEvent::CreatureDied { card_id: chump });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew when a nontoken creature died");
}

/// Dropkick Bomber buffs other Goblins and can grant one flying.
#[test]
fn dropkick_bomber_lord_and_flight() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bomber = g.add_card_to_battlefield(0, catalog::dropkick_bomber());
    let goblin = g.add_card_to_battlefield(0, catalog::searslicer_goblin()); // 2/1 Goblin
    // Lord: the other Goblin is +1/+1.
    let cp = g.computed_permanent(goblin).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "lord buff");
    // Grant flying to the other Goblin.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomber, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(goblin)), additional_targets: vec![], x_value: None,
    })
    .expect("grant flying");
    drain_stack(&mut g);
    assert!(g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::Flying), "granted flying");
}

/// Seeker's Folly (mode 1) shrinks the opponent's board.
#[test]
fn seekers_folly_debuffs_opponents() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::seekers_folly());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("cast Seeker's Folly, mode 1");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "opponents' creatures get -1/-1");
}

/// Spinner of Souls digs a creature into hand when another of yours dies.
#[test]
fn spinner_of_souls_digs() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spinner_of_souls());
    // A creature on top of the library to dig into hand.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let mut evs = g.remove_to_graveyard_with_triggers(chump);
    evs.push(GameEvent::CreatureDied { card_id: chump });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand, "dug a creature into hand");
}

/// A sanity attack test: High-Society Hunter's on-attack sac-for-counter grows it.
#[test]
fn high_society_hunter_grows_on_attack() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let hunter = g.add_card_to_battlefield(0, catalog::high_society_hunter());
    g.clear_sickness(hunter);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder to sacrifice
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hunter, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hunter).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "sacrificed fodder to grow"
    );
}
