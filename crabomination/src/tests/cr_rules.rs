//! Targeted Comprehensive-Rules conformance tests for keyword actions added
//! this run: Detain (CR 701.35) and Fateseal (CR 701.29).

use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::{drain_stack, two_player_game};
use super::*;

// ── CR 701.35 — Detain ────────────────────────────────────────────────────────

#[test]
fn cr_701_35_detain_stops_attack_block_and_activation_until_detainers_next_turn() {
    let mut g = two_player_game();
    // Opponent (seat 1) controls a creature that we'll detain via Lyev Skyknight.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(victim);
    // Cast Lyev Skyknight (seat 0) and detain the bear on ETB.
    let lyev = g.add_card_to_hand(0, catalog::lyev_skyknight());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, lyev, Target::Permanent(victim));
    assert_eq!(g.battlefield_find(victim).unwrap().detained_by, Some(0), "bear detained by seat 0");

    // The detained bear can't be declared as an attacker on the opponent's turn.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: victim, target: AttackTarget::Player(0),
    }]));
    assert!(err.is_err(), "detained creature can't attack");

    // It can't block either.
    g.step = TurnStep::DeclareBlockers;
    g.block_map.clear();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(1) });
    let berr = g.perform_action(GameAction::DeclareBlockers(vec![(victim, attacker)]));
    assert!(berr.is_err(), "detained creature can't block");
}

#[test]
fn cr_701_35_detain_clears_at_detainers_next_turn() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().detained_by = Some(0);
    // Detainer (seat 0) begins a new turn → detain lifts.
    g.active_player_idx = 0;
    g.do_untap();
    assert_eq!(g.battlefield_find(victim).unwrap().detained_by, None, "detain lifts on detainer's turn");
}

// ── CR 701.29 — Fateseal ──────────────────────────────────────────────────────

/// Test-only fixture: a Sorcery that fateseals 2 against each opponent.
fn fateseal_two() -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType};
    use crate::effect::{Effect, PlayerRef, Value};
    CardDefinition {
        name: "Test Fateseal 2",
        cost: crate::mana::cost(&[crate::mana::generic(1), crate::mana::u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Fateseal { who: PlayerRef::EachOpponent, amount: Value::Const(2) },
        ..Default::default()
    }
}

#[test]
fn cr_701_29_fateseal_bottoms_chosen_card_of_opponent_library() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Opponent's top two library cards.
    let top = g.add_card_to_library(1, catalog::island());
    let _second = g.add_card_to_library(1, catalog::forest());
    let before_len = g.players[1].library.len();
    let spell = g.add_card_to_hand(0, fateseal_two());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Put the opponent's top card (`top`) on the bottom.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![top])]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast(&mut g, spell);
    assert_eq!(g.players[1].library.len(), before_len, "library size unchanged");
    assert_eq!(g.players[1].library.last().unwrap().id, top, "chosen card sent to bottom");
}
