//! Functionality tests for `catalog::sets::decks::recent130` (WOE wave 3).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Scream Puff makes a Food when it connects.
#[test]
fn scream_puff_food_on_combat_damage() {
    let mut g = two_player_game();
    let puff = g.add_card_to_battlefield(0, catalog::scream_puff());
    g.clear_sickness(puff);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: puff,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    // Pass through combat damage.
    while g.step == TurnStep::DeclareAttackers || g.step == TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
        "Food created on combat damage",
    );
}

/// Beanstalk Wurm's Plant Beans grants an extra land drop.
#[test]
fn plant_beans_grants_extra_land() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let card = g.add_card_to_hand(0, catalog::beanstalk_wurm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].extra_land_plays;
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Plant Beans");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_land_plays, before + 1, "one extra land play");
}

/// Return from the Wilds — choosing the Human + Food modes makes both.
#[test]
fn return_from_the_wilds_choose_two() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let spell = g.add_card_to_hand(0, catalog::return_from_the_wilds());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Pick modes 1 (Human) and 2 (Food).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1, 2])]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Return from the Wilds");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human"), "made a Human");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "made a Food");
}

/// Stockpiling Celebrant returns a permanent and scrys.
#[test]
fn stockpiling_celebrant_bounce_and_scry() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let clue = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cel = g.add_card_to_battlefield(0, catalog::stockpiling_celebrant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_self_etb_triggers(cel, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == clue), "returned the other creature to hand");
}

/// Elusive Otter's Grove's Bounty distributes X +1/+1 counters.
#[test]
fn groves_bounty_distributes_counters() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::elusive_otter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DamageDivision(vec![1, 1])]));
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast Grove's Bounty for X=2");
    drain_stack(&mut g);
    let total: u32 = [a, b]
        .iter()
        .map(|&id| g.battlefield_find(id).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0))
        .sum();
    assert_eq!(total, 2, "two +1/+1 counters distributed");
}

/// Elusive Otter can't be blocked by lower-power creatures.
#[test]
fn elusive_otter_evasion() {
    let mut g = two_player_game();
    let otter = g.add_card_to_battlefield(0, catalog::elusive_otter());
    let cp = g.computed_permanent(otter).unwrap();
    assert!(
        cp.keywords.contains(&Keyword::Prowess)
            && cp.keywords.contains(&Keyword::CantBeBlockedByPowerLess),
        "prowess + can't-be-blocked-by-lesser evasion",
    );
}
