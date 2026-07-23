//! CR conformance for rules exercised by this run's DGM gap wave:
//! CR 701.10e (double each kind of counter — near Vorel of the Hull Clade /
//! `Effect::DoubleAllCountersOn`), CR 702.102 (Fuse — near the Armed //
//! Dangerous split), and CR 509.1c (true Lure forces every able blocker — the
//! in-progress `AllMustBlock` enforcement, exercised through Dangerous).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameError};
use crabomination::mana::Color;

/// CR 701.10e — doubling a kind of counter gives as many more as already
/// present; Vorel doubles *every* kind at once (3 +1/+1 → 6, 1 charge → 2).
#[test]
fn cr_701_10e_double_each_kind_of_counter() {
    let mut g = two_player_game();
    let vorel = g.add_card_to_battlefield(0, catalog::vorel_of_the_hull_clade());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::Charge, 1);
    g.clear_sickness(vorel);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vorel, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Vorel");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 6, "3 → 6");
    assert_eq!(c.counter_count(CounterType::Charge), 2, "1 → 2");
}

/// CR 702.102 — Fuse lets you cast both halves as one spell, paying both
/// costs; each half resolves with its own target. Fused Armed // Dangerous
/// pumps your creature and Lures the opponent's.
#[test]
fn cr_702_102_fuse_casts_both_halves() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::armed_dangerous());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSplitFused {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    }).expect("cast fused Armed // Dangerous");
    drain_stack(&mut g);
    let m = g.computed_permanent(mine).unwrap();
    assert_eq!((m.power, m.toughness), (3, 3), "Armed pumped +1/+1");
    assert!(m.keywords.contains(&Keyword::DoubleStrike), "Armed granted double strike");
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::AllMustBlock),
        "Dangerous Lured the opponent's creature");
}

/// CR 509.1c — "all creatures able to block do so" forces every idle defender
/// onto the Lured attacker; declaring fewer blocks is illegal. Dangerous grants
/// the true Lure keyword.
#[test]
fn cr_509_1c_true_lure_forces_all_blockers() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let d1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    // Dangerous (right half) Lures the attacker.
    let spell = g.add_card_to_hand(0, catalog::armed_dangerous());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSplitRight {
        card_id: spell, target: Some(Target::Permanent(atk)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dangerous");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // Only one of the two able blockers assigned → illegal.
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(d1, atk)])).unwrap_err();
    assert!(matches!(err, GameError::MustBeBlockedIfAble(_)), "both defenders must block");
    // Both assigned → accepted.
    g.perform_action(GameAction::DeclareBlockers(vec![(d1, atk), (d2, atk)])).expect("both block");
}
