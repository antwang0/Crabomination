//! Functionality tests for `catalog::sets::decks::recent189` (OTJ gaps).

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::two_player_game;
use crate::game::*;
use crate::mana::Color;

/// Rodeo Pyromancers adds {R}{R} on your first spell each turn.
#[test]
fn rodeo_pyromancers_rituals_first_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rodeo_pyromancers());
    let spell = g.add_card_to_hand(0, catalog::ponder()); // {U}
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast first spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "first-spell ritual added RR");
}

/// Scalestorm Summoner mints a Dinosaur on attack only with a 4-power creature.
#[test]
fn scalestorm_summoner_ferocious_token() {
    let dinos = |ferocious: bool| -> usize {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let summoner = g.add_card_to_battlefield(0, catalog::scalestorm_summoner());
        g.clear_sickness(summoner);
        if ferocious {
            let big = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.battlefield_find_mut(big).unwrap().power_bonus = 2; // 4/4
        }
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: summoner,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        g.battlefield.iter().filter(|c| c.definition.name == "Dinosaur").count()
    };
    assert_eq!(dinos(false), 0, "no 4-power creature → no token");
    assert_eq!(dinos(true), 1, "ferocious → a Dinosaur token");
}

/// Marauding Sphinx surveils when you commit a crime, once each turn.
#[test]
fn marauding_sphinx_crime_surveil_once() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::marauding_sphinx());
    g.players[0].library.clear();
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![top],
    }]));
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveiled the top card away");
}

/// Raucous Entertainer counters only creatures that entered this turn.
#[test]
fn raucous_entertainer_counters_fresh_creatures() {
    let mut g = two_player_game();
    let old = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(old).unwrap().entered_turn = Some(0); // an earlier turn
    let ent = g.add_card_to_battlefield(0, catalog::raucous_entertainer());
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fresh).unwrap().entered_turn = Some(g.turn_number);
    g.clear_sickness(ent);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ent,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Raucous Entertainer");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(fresh).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne),
        1,
        "fresh creature got a counter",
    );
    assert_eq!(
        g.battlefield_find(old).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne),
        0,
        "older creature untouched",
    );
}

/// Ruthless Lawbringer's ETB sacrifices a creature to destroy a nonland permanent.
#[test]
fn ruthless_lawbringer_sacrifice_removal() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Accept the optional sacrifice; the reflexive destroy auto-targets the
    // opponent's permanent.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.move_card_to_battlefield_for_test(0, catalog::ruthless_lawbringer());
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the fodder creature");
    assert!(g.battlefield_find(victim).is_none(), "destroyed the opponent's permanent");
}
