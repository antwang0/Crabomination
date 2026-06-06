//! Tests for the Murders at Karlov Manor (MKM) and Lost Caverns of Ixalan
//! (LCI) keyword actions: Suspect (701.60), Collect Evidence (701.59), and
//! Discover (701.57).

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::Target;
use crate::mana::Color;
use crate::TurnStep;

// ── Suspect (CR 701.60) ──────────────────────────────────────────────────────

/// A suspected creature gains menace and can't block (computed keywords).
#[test]
fn barbed_servitor_etb_suspects_itself() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::barbed_servitor());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Barbed Servitor");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().suspected, "ETB suspected itself");
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Menace), "suspected → menace");
    assert!(cp.keywords.contains(&Keyword::CantBlock), "suspected → can't block");
}

/// Repeat Offender suspects itself, then on a second activation (while
/// suspected) grows with a +1/+1 counter instead.
#[test]
fn repeat_offender_toggles_suspect_then_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::repeat_offender());
    drain_stack(&mut g);
    // First activation: not suspected → suspect it.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().suspected, "now suspected");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Second activation: suspected → +1/+1 counter.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activate again");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "suspected → grows instead",
    );
}

/// Reasonable Doubt suspects a creature alongside the counter clause.
#[test]
fn reasonable_doubt_suspects_target_creature() {
    let mut g = two_player_game();
    // The creature to suspect, on the battlefield.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // P1 casts a spell for the counter clause to target (P1 can't pay {2}).
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    let id = g.add_card_to_hand(0, catalog::reasonable_doubt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(spell)),
        additional_targets: vec![Target::Permanent(victim)], mode: None, x_value: None,
    }).expect("cast Reasonable Doubt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().suspected, "creature suspected");
}

// ── Collect Evidence (CR 701.59) ─────────────────────────────────────────────

/// Sample Collector collects evidence on attack and grows a creature.
#[test]
fn sample_collector_collects_evidence_and_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sample_collector());
    g.clear_sickness(id);
    // Graveyard fodder totaling MV ≥ 3 (two MV-2 bears = 4).
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Yes to "collect evidence 3"; the +1/+1 counter auto-targets the attacker.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.len() < 2, "evidence exiled from graveyard");
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "collected evidence → +1/+1 counter",
    );
}

/// Without enough evidence in the graveyard, the payoff does not fire.
#[test]
fn sample_collector_without_evidence_does_nothing() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sample_collector());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "no evidence → no counter",
    );
}

// ── Discover (CR 701.57) ─────────────────────────────────────────────────────

/// Geological Appraiser discovers 3 and the controller casts the hit for free.
#[test]
fn geological_appraiser_discovers_and_casts() {
    let mut g = two_player_game();
    // Top of library: a cheap creature the discover will hit.
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_hand(0, catalog::geological_appraiser());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Geological Appraiser");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "discovered creature cast for free");
}

/// Declining the free cast puts the discovered card into hand instead.
#[test]
fn discover_decline_puts_card_in_hand() {
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let id = g.add_card_to_hand(0, catalog::geological_appraiser());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Geological Appraiser");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears), "declined → went to hand");
}
