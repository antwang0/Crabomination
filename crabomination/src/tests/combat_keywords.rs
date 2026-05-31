//! Functionality tests for the combat-keyword shortcuts
//! (`effect::shortcut::{bushido, frenzy, afflict}`) — CR 702.46 / 702.68
//! / 702.131. Each test builds a synthetic creature carrying the keyword
//! triggers and drives a combat to observe the rider.

use crate::card::{CardDefinition, CardType, Subtypes, TriggeredAbility};
use crate::catalog;
use crate::effect::shortcut;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::TurnStep;

/// A bare N/M creature carrying the given triggered abilities.
fn body(name: &'static str, p: i32, t: i32, trig: Vec<TriggeredAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: p,
        toughness: t,
        triggered_abilities: trig,
        ..Default::default()
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Pass priority until it is `player`'s `step` (used to reach an
/// opponent's combat so a defending creature can block).
fn advance_to_player_step(g: &mut GameState, player: usize, step: TurnStep) {
    while !(g.active_player_idx == player && g.step == step) {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── CR 702.46 Bushido ────────────────────────────────────────────────────────

#[test]
fn cr_702_46_bushido_pumps_when_attacking_and_blocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Samurai", 2, 2, shortcut::bushido(2)));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    // Becoming blocked pumps the Samurai to 4/4, so it survives the 2/2
    // and kills it.
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (4, 4), "bushido 2 pumps on becoming blocked");
}

#[test]
fn cr_702_46_bushido_pumps_when_blocking() {
    let mut g = two_player_game();
    // P1 attacks with a vanilla bear; P0's bushido creature blocks.
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(0, body("Samurai", 2, 2, shortcut::bushido(2)));
    g.clear_sickness(atk);
    // Hand priority to P1's combat so it can attack into P0.
    advance_to_player_step(&mut g, 1, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(0),
    }])).expect("p1 attacks");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)])).expect("block");
    drain_stack(&mut g);
    let s = g.battlefield_find(blocker).unwrap();
    assert_eq!((s.power(), s.toughness()), (4, 4), "bushido 2 pumps on blocking too");
}

// ── CR 702.68 Frenzy ─────────────────────────────────────────────────────────

#[test]
fn cr_702_68_frenzy_pumps_only_when_unblocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Berserker", 2, 2, vec![shortcut::frenzy(3)]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // No blockers declared → frenzy fires for +3/+0.
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (5, 2), "frenzy 3 pumps an unblocked attacker");
}

#[test]
fn cr_702_68_frenzy_silent_when_blocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Berserker", 2, 2, vec![shortcut::frenzy(3)]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 2), "frenzy does NOT fire when blocked");
}

// ── CR 702.131 Afflict ───────────────────────────────────────────────────────

#[test]
fn cr_702_131_afflict_drains_defender_on_becoming_blocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Raptor", 3, 3, vec![shortcut::afflict(2)]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let life_before = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "afflict 2 drains the defending player");
}
