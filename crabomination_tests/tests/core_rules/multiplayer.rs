//! Multiplayer (3+ player) game tests. Covers turn rotation, priority
//! cycling, elimination, mulligan chaining, and format-specific starting
//! life across N players. Two-player coverage lives in `tests/game.rs`.

use crabomination::game::*;
use crabomination::game::{game_with_format, multi_player_game};
use crabomination::card::SelectionRequirement;
use crabomination::catalog;
use crabomination::decision::DecisionAnswer;
use crabomination::effect::PlayerRef;
use crabomination::format::Format;
use crabomination::game::effects::EffectContext;
use crabomination::team::{TeamError, TeamId};

// ── Setup ─────────────────────────────────────────────────────────────────

#[test]
fn four_player_ffa_starts_with_default_life() {
    let g = multi_player_game(4);
    assert_eq!(g.players.len(), 4);
    assert_eq!(g.active_player_idx, 0);
    assert_eq!(g.turn_number, 1);
    for p in &g.players {
        assert_eq!(p.life, 20);
        assert!(p.is_alive());
    }
}

#[test]
fn commander_format_applies_40_life_to_all_seats() {
    let g = game_with_format(Format::Commander, 4);
    for p in &g.players {
        assert_eq!(p.life, 40);
    }
    // Multiplayer (3+) doesn't skip the first draw.
    assert!(!g.skip_first_draw);
}

#[test]
fn two_headed_giant_format_applies_30_life() {
    let g = game_with_format(Format::TwoHeadedGiant, 4);
    for p in &g.players {
        assert_eq!(p.life, 30);
    }
}

// ── Turn rotation ─────────────────────────────────────────────────────────

#[test]
fn turn_rotates_through_all_four_seats() {
    let mut g = multi_player_game(4);
    let expected = [1usize, 2, 3, 0];
    for (i, want) in expected.iter().enumerate() {
        g.do_cleanup(&mut Vec::new());
        assert_eq!(
            g.active_player_idx, *want,
            "after cleanup #{i}: expected active seat {want}, got {}",
            g.active_player_idx
        );
    }
    assert_eq!(g.turn_number, 5, "4 rotations → turn 5");
}

#[test]
fn eliminated_player_is_skipped_in_turn_rotation() {
    let mut g = multi_player_game(4);
    // Eliminate seat 2 mid-table.
    g.players[2].eliminated = true;

    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 1, "0 → 1");
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 3, "1 → 3 (skip eliminated 2)");
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 0, "3 → 0");
}

// ── Priority cycling ──────────────────────────────────────────────────────

#[test]
fn priority_cycles_through_all_four_seats_before_step_advances() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    assert_eq!(g.player_with_priority(), 0);

    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.player_with_priority(), 1);
    assert_eq!(g.step, TurnStep::PreCombatMain, "step not advanced yet");

    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.player_with_priority(), 2);

    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.player_with_priority(), 3);
    assert_eq!(g.step, TurnStep::PreCombatMain, "still need one more pass");

    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.step, TurnStep::BeginCombat, "4 passes advance the step");
}

#[test]
fn eliminated_player_is_skipped_in_priority_rotation() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    g.players[1].eliminated = true; // 3 alive: seats 0, 2, 3.

    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.player_with_priority(), 2, "0 → 2 (skip 1)");

    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.player_with_priority(), 3);

    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.step, TurnStep::BeginCombat, "3 passes from 3 alive players advance");
}

// ── Mulligan chain ────────────────────────────────────────────────────────

#[test]
fn mulligan_chain_visits_every_seat_in_a_four_player_game() {
    let mut g = multi_player_game(4);
    // Each player needs a non-empty library to deal an opening hand from.
    for seat in 0..4 {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }

    g.start_mulligan_phase();

    for expected_seat in 0..4 {
        let pd = g
            .pending_decision
            .as_ref()
            .unwrap_or_else(|| panic!("expected mulligan decision for seat {expected_seat}"));
        match &pd.decision {
            crabomination::decision::Decision::Mulligan { player, .. } => {
                assert_eq!(
                    *player, expected_seat,
                    "mulligan should be on seat {expected_seat}",
                );
            }
            other => panic!("expected Mulligan decision, got {other:?}"),
        }
        g.submit_decision(DecisionAnswer::Keep).unwrap();
    }

    assert!(
        g.pending_decision.is_none(),
        "all 4 seats kept; mulligan phase should be complete",
    );
}

// ── Teams ─────────────────────────────────────────────────────────────────

#[test]
fn default_teams_are_singletons() {
    let g = multi_player_game(4);
    assert_eq!(g.teams.len(), 4, "one team per seat by default");
    for seat in 0..4 {
        assert_eq!(g.team_of(seat), TeamId(seat));
        assert!(g.teammates(seat).is_empty(), "singleton has no teammates");
        let opp: Vec<usize> = g.opponents_of(seat);
        let mut expected: Vec<usize> = (0..4).filter(|&s| s != seat).collect();
        let mut got = opp.clone();
        got.sort();
        expected.sort();
        assert_eq!(got, expected, "every other seat is an opponent");
        assert!(g.same_team(seat, seat));
        for other in 0..4 {
            if other != seat {
                assert!(!g.same_team(seat, other));
            }
        }
    }
}

#[test]
fn two_player_game_has_two_singleton_teams() {
    let g = two_player_game();
    assert_eq!(g.teams.len(), 2);
    assert_eq!(g.team_of(0), TeamId(0));
    assert_eq!(g.team_of(1), TeamId(1));
    assert!(!g.same_team(0, 1));
}

#[test]
fn assign_teams_partitions_2v2() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).unwrap();
    assert_eq!(g.teams.len(), 2);

    // Same-team partitioning.
    assert!(g.same_team(0, 2));
    assert!(g.same_team(1, 3));
    assert!(!g.same_team(0, 1));
    assert!(!g.same_team(2, 3));

    // Teammates exclude self.
    assert_eq!(g.teammates(0), vec![2]);
    assert_eq!(g.teammates(2), vec![0]);
    assert_eq!(g.teammates(1), vec![3]);

    // Opponents are the other team's full membership.
    let mut opp0 = g.opponents_of(0);
    opp0.sort();
    assert_eq!(opp0, vec![1, 3]);
    let mut opp3 = g.opponents_of(3);
    opp3.sort();
    assert_eq!(opp3, vec![0, 2]);
}

#[test]
fn assign_teams_rejects_duplicate_seat() {
    let mut g = multi_player_game(4);
    let err = g
        .assign_teams(vec![vec![0, 1, 2], vec![2, 3]])
        .unwrap_err();
    assert_eq!(err, TeamError::DuplicateSeat(2));
}

#[test]
fn assign_teams_rejects_missing_seat() {
    let mut g = multi_player_game(4);
    let err = g.assign_teams(vec![vec![0, 1], vec![2]]).unwrap_err();
    assert_eq!(err, TeamError::MissingSeat(3));
}

#[test]
fn assign_teams_rejects_out_of_range_seat() {
    let mut g = multi_player_game(4);
    let err = g
        .assign_teams(vec![vec![0, 1], vec![2, 3, 7]])
        .unwrap_err();
    assert_eq!(
        err,
        TeamError::UnknownSeat {
            seat: 7,
            num_players: 4
        }
    );
}

#[test]
fn assign_teams_rejects_empty_partition() {
    let mut g = multi_player_game(4);
    let err = g
        .assign_teams(vec![vec![0, 1, 2, 3], vec![]])
        .unwrap_err();
    assert_eq!(err, TeamError::EmptyTeam(1));
}

#[test]
fn empty_teams_falls_back_to_singleton_semantics() {
    // Simulates a snapshot from before the teams field existed: clear
    // the auto-populated singleton partition and confirm helpers still
    // produce sensible results.
    let mut g = multi_player_game(3);
    g.teams.clear();

    assert_eq!(g.team_of(0), TeamId(0));
    assert_eq!(g.team_of(2), TeamId(2));
    assert!(g.teammates(1).is_empty());

    let mut opp = g.opponents_of(0);
    opp.sort();
    assert_eq!(opp, vec![1, 2]);
}

// ── Team-aware opponent semantics (Phase C) ────────────────────────────────

#[test]
fn each_opponent_in_2v2_excludes_teammate() {
    // Teams: {0, 2} vs {1, 3}. EachOpponent for seat 0 must yield {1, 3},
    // not {1, 2, 3} — seat 2 is a teammate.
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).unwrap();
    let ctx = EffectContext::for_spell(0, None, 0, 0);

    let mut opps = g.resolve_players(&PlayerRef::EachOpponent, &ctx);
    opps.sort();
    assert_eq!(opps, vec![1, 3], "teammate seat 2 must not appear");
}

#[test]
fn each_opponent_in_2_player_unchanged() {
    // Baseline: 1v1 behavior must be identical to pre-Phase-C.
    let g = two_player_game();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let opps = g.resolve_players(&PlayerRef::EachOpponent, &ctx);
    assert_eq!(opps, vec![1]);
}

#[test]
fn each_player_fans_out_in_apnap_order() {
    // CR 101.4 / 121.2c — "each player" resolves active-player-first, then
    // in turn order. With seat 1 active in a 3-player game the order is
    // [1, 2, 0], not raw seat index [0, 1, 2].
    let mut g = multi_player_game(3);
    g.active_player_idx = 1;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    assert_eq!(g.resolve_players(&PlayerRef::EachPlayer, &ctx), vec![1, 2, 0]);
}

#[test]
fn each_player_apnap_order_unchanged_when_seat_zero_active() {
    let g = multi_player_game(3); // seat 0 active by default
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    assert_eq!(g.resolve_players(&PlayerRef::EachPlayer, &ctx), vec![0, 1, 2]);
}

#[test]
fn each_opponent_skips_eliminated_seats() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).unwrap();
    g.players[1].eliminated = true; // dead opposing-team player

    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let opps = g.resolve_players(&PlayerRef::EachOpponent, &ctx);
    assert_eq!(opps, vec![3], "dead opponent is filtered out");
}

#[test]
fn controlled_by_opponent_predicate_excludes_teammate_card() {
    // A creature owned & controlled by seat 2 (P0's teammate in a 2v2)
    // must NOT match `ControlledByOpponent` when the controller of the
    // checking effect is seat 0.
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).unwrap();
    let teammate_creature = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let opponent_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // From seat 0's perspective:
    assert!(
        !g.evaluate_requirement_static(
            &SelectionRequirement::ControlledByOpponent,
            &Target::Permanent(teammate_creature),
            0,
            None,
        ),
        "teammate's creature is not 'controlled by an opponent'",
    );
    assert!(
        g.evaluate_requirement_static(
            &SelectionRequirement::ControlledByOpponent,
            &Target::Permanent(opponent_creature),
            0,
            None,
        ),
        "other-team creature IS controlled by an opponent",
    );
}

#[test]
fn controlled_by_opponent_player_target_excludes_teammate() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).unwrap();

    // From seat 0:
    assert!(
        !g.evaluate_requirement_static(
            &SelectionRequirement::ControlledByOpponent,
            &Target::Player(2),
            0,
            None,
        ),
        "teammate seat 2 is not an opponent",
    );
    assert!(
        !g.evaluate_requirement_static(
            &SelectionRequirement::ControlledByOpponent,
            &Target::Player(0),
            0,
            None,
        ),
        "self is not an opponent",
    );
    for opp in [1, 3] {
        assert!(
            g.evaluate_requirement_static(
                &SelectionRequirement::ControlledByOpponent,
                &Target::Player(opp),
                0,
                None,
            ),
            "seat {opp} on opposing team IS an opponent",
        );
    }
}

#[test]
fn controlled_by_opponent_in_2_player_unchanged() {
    // Baseline: 1v1 predicate behavior unchanged.
    let mut g = two_player_game();
    let p0_creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    assert!(!g.evaluate_requirement_static(
        &SelectionRequirement::ControlledByOpponent,
        &Target::Permanent(p0_creature),
        0,
        None,
    ));
    assert!(g.evaluate_requirement_static(
        &SelectionRequirement::ControlledByOpponent,
        &Target::Permanent(p1_creature),
        0,
        None,
    ));
}

#[test]
fn auto_targeter_picks_opposing_team_player() {
    // The auto-target heuristic for "deal damage to a player" effects
    // (Lightning Bolt-shaped) should pick an opposing-team player, not a
    // teammate. Validates that the `(controller + 1) % n` fallback now
    // routes through `opponents_of`.
    let mut g = multi_player_game(4);
    // Teammates 0 + 1, opponents 2 + 3. With singletons this used to
    // auto-pick seat 1; now it must pick seat 2 (the first opponent).
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();
    let lightning_bolt = catalog::lightning_bolt();
    let pick = g.auto_target_for_effect(&lightning_bolt.effect, 0);
    match pick {
        Some(Target::Player(p)) => assert!(
            !g.same_team(0, p),
            "auto-target {p} must not be a teammate of controller 0",
        ),
        other => panic!("expected Target::Player from Lightning Bolt picker, got {other:?}"),
    }
}

#[test]
fn assign_teams_supports_three_way_ffa_regrouping() {
    // 4-player game can be re-partitioned into a 2-1-1 free-for-all
    // (one allied pair and two solo seats).
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2], vec![3]]).unwrap();

    assert_eq!(g.teams.len(), 3);
    assert!(g.same_team(0, 1));
    assert_eq!(g.teammates(2), Vec::<usize>::new());
    let mut opp1 = g.opponents_of(1);
    opp1.sort();
    assert_eq!(opp1, vec![2, 3]);
    let mut opp2 = g.opponents_of(2);
    opp2.sort();
    assert_eq!(opp2, vec![0, 1, 3]);
}

// ── Phase D — multiplayer combat ──────────────────────────────────────────

/// In a 3-player FFA, an attacker may target either of the two other
/// seats — and only those two. Self-targeting is still rejected.
#[test]
fn three_player_ffa_can_attack_either_opponent() {
    let mut g = multi_player_game(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;

    // Attack seat 1: legal.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("seat 0 may attack seat 1 in 3p FFA");
    assert!(g.attacking().iter().any(|a| matches!(a.target, AttackTarget::Player(1))));

    // Reset: untap the bear, clear combat, retry on seat 2.
    g.attacking.clear();
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.tapped = false;
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(2),
    }]))
    .expect("seat 0 may also attack seat 2");

    // Self-attack still rejected.
    g.attacking.clear();
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.tapped = false;
    }
    let err = g
        .perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(0),
        }]))
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAttackTarget(0)));
}

/// In a 2v2 team game, an attacker may not target a teammate. The
/// pre-Phase-D check only rejected `target == active_player_idx`; with
/// teams enabled, the active player's partner is equally off-limits.
#[test]
fn two_v_two_rejects_attack_on_teammate() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;

    // Seat 1 is on team A with seat 0 — attacking them is illegal.
    let err = g
        .perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]))
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAttackTarget(1)));

    // Seat 2 (team B) is legal.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(2),
    }]))
    .expect("seat 0 may attack opposing-team seat 2");
}

/// 2v2: attacking a planeswalker controlled by a *teammate* must be
/// rejected. Pre-Phase-D only the active player's own PWs were blocked;
/// a teammate's PWs were attackable.
#[test]
fn two_v_two_rejects_attack_on_teammate_planeswalker() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Teammate (seat 1) controls a planeswalker.
    let karn_teammate = g.add_card_to_battlefield(1, catalog::karn_scion_of_urza());
    // Opposing-team (seat 2) controls another.
    let karn_opp = g.add_card_to_battlefield(2, catalog::karn_scion_of_urza());

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;

    // Teammate's planeswalker — illegal.
    let err = g
        .perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Planeswalker(karn_teammate),
        }]))
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidPlaneswalkerAttackTarget(_)));

    // Opposing-team planeswalker — legal.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Planeswalker(karn_opp),
    }]))
    .expect("seat 0 may attack opposing-team Karn");
}

/// 2v2: when the active player attacks one opposing-team seat, the
/// *other* member of the defending team may block on their teammate's
/// behalf. Pre-Phase-D the blocker.controller had to equal the exact
/// defending seat; now any same-team member can block.
#[test]
fn two_v_two_teammate_can_block_for_partner() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // The teammate-blocker is controlled by seat 3, but attack targets
    // seat 2. Pre-fix this declaration was rejected with
    // BlockerWrongDefender.
    let blocker = g.add_card_to_battlefield(3, catalog::grizzly_bears());
    g.clear_sickness(blocker);

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(2),
    }]))
    .unwrap();

    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("seat 3 (teammate of attacked seat 2) may block");

    assert_eq!(
        g.attackers_blocked_by(blocker).first().copied(),
        Some(attacker),
        "blocker registered against the attacker",
    );
}

/// 2v2: a creature controlled by a player on the *attacking* team can
/// not block — even though, viewed only as "not the targeted seat,"
/// they used to be allowed. The team check rejects them.
#[test]
fn two_v_two_attacking_team_cannot_block_for_defender() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Seat 1 is on the *attacking* team; their creature can't block
    // for the defenders.
    let intruder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(intruder);

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(2),
    }]))
    .unwrap();

    g.step = TurnStep::DeclareBlockers;
    let err = g
        .perform_action(GameAction::DeclareBlockers(vec![(intruder, attacker)]))
        .unwrap_err();
    assert!(matches!(
        err,
        GameError::BlockerWrongDefender { blocker } if blocker == intruder
    ));
}

// ── Phase E — APNAP trigger ordering ──────────────────────────────────────

/// CR 603.3b: when a single event triggers several abilities at once,
/// the active player puts their triggers on the stack first (in any
/// order they choose), then each non-active player in turn order.
/// Because the stack is LIFO, the active player's triggers therefore
/// resolve LAST. Pre-Phase-E the unified dispatcher pushed triggers in
/// battlefield-iteration order regardless of who controlled them —
/// observable any time more than one player controls a triggering
/// permanent (4p FFA, 2HG, Commander; invisible in 1v1).
#[test]
fn apnap_orders_simultaneous_triggers_active_pushed_first() {
    use crabomination::card::{CardDefinition, CardId, CardType, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};

    // A "lifegain pinger" — minimal triggered ability that fires on
    // LifeGained / AnyPlayer. The body is irrelevant; we inspect the
    // stack order, not the resolution.
    let pinger = |name: &'static str| CardDefinition {
        name,
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::AnyPlayer),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    };

    let mut g = multi_player_game(4);
    // Make seat 1 the active player so the APNAP cycle (1, 2, 3, 0)
    // visibly differs from battlefield-insertion order (0, 1, 2, 3).
    g.active_player_idx = 1;

    let seat0 = g.add_card_to_battlefield(0, pinger("Pinger-0"));
    let seat1 = g.add_card_to_battlefield(1, pinger("Pinger-1"));
    let seat2 = g.add_card_to_battlefield(2, pinger("Pinger-2"));
    let seat3 = g.add_card_to_battlefield(3, pinger("Pinger-3"));

    // Synthesize a LifeGained event and dispatch. Which seat actually
    // gained life doesn't matter — AnyPlayer-scope triggers fire
    // regardless.
    let ev = GameEvent::LifeGained { player: 0, amount: 1 };
    g.dispatch_triggers_for_events(&[ev]);

    assert_eq!(g.stack.len(), 4, "all four pingers must trigger");
    // Stack[0] was pushed first (resolves last) → must be the active
    // player's. Stack[last] was pushed last (resolves first) → must
    // be the "furthest" seat in APNAP order from the active player.
    let sources: Vec<CardId> = g
        .stack
        .iter()
        .map(|item| match item {
            StackItem::Trigger { source, .. } => *source,
            other => panic!("expected only Trigger stack items, got {other:?}"),
        })
        .collect();
    assert_eq!(
        sources,
        vec![seat1, seat2, seat3, seat0],
        "push order must be APNAP from active=1 (1 → 2 → 3 → 0)",
    );
}

/// Same APNAP guarantee with an eliminated seat in the middle of the
/// cycle: the eliminated player is skipped. Active=0, seat 2 dead, so
/// the alive cycle is 0 → 1 → 3, and any trigger on seat 2 is filtered
/// out before reaching the stack (battlefield permanents controlled
/// by an eliminated player still exist physically, but they shouldn't
/// re-order the live ones).
#[test]
fn apnap_skips_eliminated_seat_in_cycle() {
    use crabomination::card::{CardDefinition, CardId, CardType, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};

    let pinger = |name: &'static str| CardDefinition {
        name,
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::AnyPlayer),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    };

    let mut g = multi_player_game(4);
    g.active_player_idx = 0;
    let seat0 = g.add_card_to_battlefield(0, pinger("Pinger-0"));
    let seat1 = g.add_card_to_battlefield(1, pinger("Pinger-1"));
    // Seat 2's permanent is still on the battlefield, but the player
    // is eliminated. With singleton-team semantics there's no special
    // filter that drops their triggers — they will still appear, but
    // sorted to the back of APNAP (rank == n_players) since the
    // alive cycle skips them.
    let seat2 = g.add_card_to_battlefield(2, pinger("Pinger-2"));
    let seat3 = g.add_card_to_battlefield(3, pinger("Pinger-3"));
    g.players[2].eliminated = true;

    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 1 }]);
    assert_eq!(g.stack.len(), 4);

    let sources: Vec<CardId> = g
        .stack
        .iter()
        .map(|item| match item {
            StackItem::Trigger { source, .. } => *source,
            other => panic!("expected only Trigger stack items, got {other:?}"),
        })
        .collect();
    // APNAP-rank for active=0 (seat 2 dead): seat 0 → 0, seat 1 → 1,
    // seat 3 → 2, seat 2 → n_players (fall-through). Push order
    // therefore: 0, 1, 3, 2.
    assert_eq!(sources, vec![seat0, seat1, seat3, seat2]);
}

// ── Phase G-lite — game ends on last team standing ────────────────────────

/// In 2v2, when one player on a team is eliminated, the team is still
/// alive (their teammate carries on). The game must NOT end. Pre-fix
/// the SBA loop terminated as soon as alive seats == 1.
#[test]
fn two_v_two_game_continues_after_one_teammate_eliminated() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();

    // Seats 1, 2, 3 lose; only seat 0 (team A) is alive of three —
    // but seat 0's teammate (seat 1) and seat 3 are both dead, so
    // ONLY team A survives. Game should end.
    //
    // Step 1: drop only seat 1's life. Team A still has seat 0 alive;
    // team B both alive. SBAs must NOT end the game.
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(g.players[1].eliminated, "seat 1 lost (life <= 0)");
    assert!(g.game_over.is_none(),
        "team A still has seat 0; team B still alive; game continues");
}

/// 2v2: when both members of one team are eliminated, the surviving
/// team wins. `winner` is reported as the surviving team's lowest-
/// numbered alive seat (its representative).
#[test]
fn two_v_two_game_ends_when_one_team_fully_eliminated() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();

    // Wipe out team B entirely.
    g.players[2].life = 0;
    g.players[3].life = 0;
    let events = g.check_state_based_actions();

    assert!(g.players[2].eliminated && g.players[3].eliminated);
    assert!(
        matches!(g.game_over, Some(Some(w)) if w == 0),
        "team A wins; representative is its lowest alive seat (0), got {:?}",
        g.game_over,
    );
    assert!(events.iter().any(|e| matches!(e, GameEvent::GameOver { winner: Some(0) })));
}

/// 2v2: if seat 0 dies but seat 1 (teammate) is still alive, team A
/// remains in. Then if team B is wiped, team A wins — and the
/// representative is now seat 1 (lowest alive on team A).
#[test]
fn two_v_two_winner_seat_skips_dead_team_members() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).unwrap();

    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(g.players[0].eliminated);
    assert!(g.game_over.is_none(), "team A still has seat 1");

    g.players[2].life = 0;
    g.players[3].life = 0;
    let events = g.check_state_based_actions();

    assert!(
        matches!(g.game_over, Some(Some(w)) if w == 1),
        "winner must be seat 1 (lowest alive on the surviving team)",
    );
    assert!(events.iter().any(|e| matches!(e, GameEvent::GameOver { winner: Some(1) })));
}

/// FFA baseline: with singleton teams, the team-count check collapses
/// to the original "one player left" semantics. 3p game with two
/// eliminations ends with the remaining seat as winner.
#[test]
fn three_player_ffa_ends_with_last_player_standing() {
    let mut g = multi_player_game(3);
    g.players[0].life = 0;
    g.players[2].life = 0;
    let events = g.check_state_based_actions();
    assert!(
        matches!(g.game_over, Some(Some(1))),
        "seat 1 is the last alive player in FFA — must win",
    );
    assert!(events.iter().any(|e| matches!(e, GameEvent::GameOver { winner: Some(1) })));
}

/// CR 509.1a — blocks are declared by the *defending* player. In a pod, a
/// seat nobody is attacking is not one of them; the gate used to answer
/// "anyone but the active player", which is only the same thing in a duel.
#[test]
fn cr_509_1a_only_the_attacked_seat_may_declare_blocks() {
    let mut g = multi_player_game(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("seat 0 attacks seat 1");

    assert!(!g.may_declare_blocks(0), "the attacking player never blocks");
    assert!(g.may_declare_blocks(1), "the attacked seat does");
    assert!(!g.may_declare_blocks(2), "a seat nobody attacked does not");
}

/// The same gate in a duel is unchanged — which is what keeps the two-player
/// golden traces byte-identical across the CR 509.1a tightening.
#[test]
fn cr_509_1a_duel_gate_is_unchanged() {
    let mut g = two_player_game();
    // Nothing attacking: the non-active seat may still be asked.
    assert!(!g.may_declare_blocks(0));
    assert!(g.may_declare_blocks(1));

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(!g.may_declare_blocks(0));
    assert!(g.may_declare_blocks(1));
}

/// CR 800.4a — when a player leaves the game, the cards/tokens they own
/// leave with them, and permanents they controlled but didn't own revert to
/// their owners' control.
#[test]
fn cr_800_4a_departed_players_objects_leave_and_control_reverts() {
    let mut g = multi_player_game(3);
    // Seat 0 owns a creature; seat 2 owns one but seat 0 has stolen it.
    let owned = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let stolen = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == stolen) {
        c.controller = 0; // seat 0 controls seat 2's creature
    }
    g.add_card_to_hand(0, catalog::lightning_bolt());

    g.players[0].life = 0; // seat 0 leaves the game
    g.check_state_based_actions();

    assert!(!g.battlefield.iter().any(|c| c.id == owned),
        "seat 0's owned creature leaves the game with them");
    let reverted = g.battlefield.iter().find(|c| c.id == stolen)
        .expect("seat 2's creature stays in play");
    assert_eq!(reverted.controller, 2, "control reverts to its owner (seat 2)");
    assert!(g.players[0].hand.is_empty(), "the departed player's hand leaves");
}

/// CR 800.4a — "all spells and abilities on the stack controlled by that
/// player cease to exist". They are dropped, not countered: nothing resolves
/// and no "whenever a spell is countered" trigger fires.
#[test]
fn cr_800_4a_departed_players_stack_items_cease_to_exist() {
    let mut g = multi_player_game(3);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(crabomination::game::GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the bolt");
    assert_eq!(g.stack.len(), 1);

    g.players[0].life = 0;
    g.check_state_based_actions();

    assert!(g.stack.is_empty(), "the departed player's spell is gone");
    assert_eq!(g.players[1].life, 20, "the bolt never resolved");
}

/// CR 800.4a, the ability half: a triggered ability the departed player
/// controls ceases to exist along with their spells. Kokusho's death trigger
/// is on the stack when its controller leaves, so nobody loses 5 life.
#[test]
fn cr_800_4a_departed_players_trigger_ceases_to_exist() {
    use crabomination::game::types::StackItem;
    let mut g = multi_player_game(3);
    let kokusho = g.add_card_to_battlefield(0, catalog::kokusho_the_evening_star());
    g.destroy_permanent(kokusho, false, &mut Vec::new());
    g.check_state_based_actions();
    assert!(
        g.stack.iter().any(|i| matches!(i, StackItem::Trigger { controller: 0, .. })),
        "the death trigger is on the stack under seat 0's control",
    );

    g.players[0].life = 0;
    g.check_state_based_actions();

    assert!(
        !g.stack.iter().any(|i| matches!(i, StackItem::Trigger { controller: 0, .. })),
        "the departed player's trigger is gone",
    );
    assert_eq!(g.players[1].life, 20, "and nobody lost 5 life to it");
}

/// CR 800.4a — the command zone is a zone like any other: a departed
/// Commander player's commander leaves the game with them rather than
/// sitting in the command zone for the rest of the game.
#[test]
fn cr_800_4a_departed_players_command_zone_leaves_too() {
    let mut g = game_with_format(Format::Commander, 4);
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    assert_eq!(g.players[0].command.len(), 1);

    g.players[0].life = 0;
    g.check_state_based_actions();

    assert!(g.players[0].command.is_empty(), "the command zone leaves with the player");
    assert!(g.find_card_anywhere(cmd).is_none(), "the commander is nowhere");
}

/// CR 800.4 — a decision the departed player was being asked to make is
/// dropped. A pending decision suppresses every *other* seat's actions until
/// it is answered, so leaving one addressed to a player who is no longer in
/// the game deadlocks the table — which is what a four-player bot pod hit as
/// a 15 % "no legal move" rate.
#[test]
fn cr_800_4_a_pending_decision_for_a_departed_player_is_dropped() {
    use crabomination::decision::Decision;
    use crabomination::game::types::{PendingDecision, ResumeContext};
    let mut g = multi_player_game(3);
    g.pending_decision = Some(Box::new(PendingDecision {
        decision: Decision::ChooseColor {
            source: crabomination::card::CardId(0),
            legal: crabomination::mana::Color::ALL.to_vec(),
        },
        resume: ResumeContext::Mulligan { player: 0, mulligans_taken: 0, next_player: None },
    }));
    // The resume context owns seat 0 by default; make the ask seat 0's.
    assert_eq!(g.pending_decision.as_ref().unwrap().acting_player(), 0);

    g.players[0].life = 0;
    g.check_state_based_actions();

    assert!(g.pending_decision.is_none(), "the departed player's ask is dropped");
}

/// CR 800.4a during combat: a departed attacker's creatures leave, and the
/// combat they were in does not hold references to them.
#[test]
fn cr_800_4a_leaving_mid_combat_clears_the_departed_attackers() {
    let mut g = multi_player_game(3);
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = crabomination::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(crabomination::game::GameAction::DeclareAttackers(vec![
        Attack { attacker: atk, target: AttackTarget::Player(1) },
    ]))
    .expect("declare an attack");
    assert!(!g.attacking.is_empty());

    g.players[0].life = 0;
    g.check_state_based_actions();

    assert!(!g.battlefield.iter().any(|c| c.id == atk), "the attacker left the game");
    assert!(
        !g.attacking.iter().any(|a| a.attacker == atk),
        "combat does not still name a creature that left",
    );
}

/// All seats eliminated simultaneously → draw (winner=None). Pre-existing
/// behavior preserved through the team-aware refactor.
#[test]
fn four_player_simultaneous_elimination_is_a_draw() {
    let mut g = multi_player_game(4);
    for p in &mut g.players {
        p.life = 0;
    }
    let events = g.check_state_based_actions();
    assert!(matches!(g.game_over, Some(None)));
    assert!(events.iter().any(|e| matches!(e, GameEvent::GameOver { winner: None })));
}

// ── Phase F — shared life pool (2HG) ──────────────────────────────────────

/// `apply_format(TwoHeadedGiant)` should partition seats into pairs
/// (0+1, 2+3) and seed each team's `shared_life` to 30. The
/// per-player `life` field is also set to 30, but only the shared
/// pool is consulted for game mechanics (Phase F).
#[test]
fn two_headed_giant_format_partitions_and_seeds_shared_pool() {
    let g = game_with_format(Format::TwoHeadedGiant, 4);

    // Teams partitioned as 0+1 vs 2+3 (default 2HG seating).
    assert_eq!(g.teams.len(), 2);
    assert_eq!(g.teams[0].members, vec![0, 1]);
    assert_eq!(g.teams[1].members, vec![2, 3]);

    // Both teams share 30 life.
    assert_eq!(g.teams[0].shared_life, Some(30));
    assert_eq!(g.teams[1].shared_life, Some(30));

    // effective_life collapses to the shared pool for every seat.
    for seat in 0..4 {
        assert_eq!(g.effective_life(seat), 30);
    }
}

/// FFA / non-2HG formats leave teams as singletons with `shared_life
/// == None` so `effective_life` reduces to `players[seat].life`.
#[test]
fn ffa_leaves_singleton_teams_with_no_shared_life() {
    let g = multi_player_game(4);
    assert_eq!(g.teams.len(), 4);
    for t in &g.teams {
        assert!(t.shared_life.is_none(), "singleton teams hold no shared pool");
    }
    // Effective life equals each player's life.
    for seat in 0..4 {
        assert_eq!(g.effective_life(seat), g.players[seat].life);
    }
}

/// 2HG: damage to one teammate drains the shared pool by the full
/// amount; their teammate sees the same drop because both consult
/// `effective_life` → shared pool.
#[test]
fn two_headed_giant_damage_to_one_teammate_drains_shared_pool() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);

    // Damage seat 0 for 7. Pre-Phase-F this would have only dropped
    // seat 0's individual life. Now it nudges the shared pool that
    // seat 1 also sees.
    g.adjust_life(0, -7);
    assert_eq!(g.teams[0].shared_life, Some(23));
    assert_eq!(g.effective_life(0), 23);
    assert_eq!(g.effective_life(1), 23, "teammate sees the same shared pool");
    // Team B's pool is untouched.
    assert_eq!(g.teams[1].shared_life, Some(30));
    assert_eq!(g.effective_life(2), 30);
}

/// 2HG: life gain by either teammate goes into the same shared pool.
/// Two separate `adjust_life(+x)` calls on different seats both
/// bump team A's pool — they don't compound on per-player.life.
#[test]
fn two_headed_giant_lifegain_by_either_teammate_pools() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);
    g.adjust_life(0, 3); // seat 0 gains 3
    g.adjust_life(1, 5); // teammate also gains 5
    assert_eq!(g.teams[0].shared_life, Some(38));
    assert_eq!(g.effective_life(0), 38);
    assert_eq!(g.effective_life(1), 38);
    // Team B untouched.
    assert_eq!(g.teams[1].shared_life, Some(30));

    // Per-seat `life_gained_this_turn` still tracks the receiving
    // seat (it's a "you" payoff bound to who took the action). Seat 0
    // received +3, seat 1 received +5. Seats on the other team
    // unchanged.
    assert_eq!(g.players[0].life_gained_this_turn, 3);
    assert_eq!(g.players[1].life_gained_this_turn, 5);
    assert_eq!(g.players[2].life_gained_this_turn, 0);
}

/// 2HG SBA: when the shared pool drops to ≤ 0, BOTH teammates are
/// eliminated (CR 810.8 + 704.5a). The surviving-team check then
/// ends the game with the opposing team as winner.
#[test]
fn two_headed_giant_zero_shared_life_eliminates_both_teammates() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);

    // Lethal damage to seat 0 takes the shared pool to 0. Seat 1
    // (the teammate) hasn't taken any damage personally, but their
    // effective_life is the now-zero pool, so they lose too.
    g.adjust_life(0, -30);
    assert_eq!(g.effective_life(0), 0);
    assert_eq!(g.effective_life(1), 0);

    let events = g.check_state_based_actions();
    assert!(g.players[0].eliminated);
    assert!(g.players[1].eliminated, "teammate eliminated by shared pool ≤ 0");
    assert!(!g.players[2].eliminated);
    assert!(!g.players[3].eliminated);

    // Team B wins; representative is seat 2 (lowest alive on the
    // surviving team).
    assert!(matches!(g.game_over, Some(Some(2))));
    assert!(events.iter().any(|e| matches!(e, GameEvent::GameOver { winner: Some(2) })));
}

// ── Phase H — zone-change replacement effects ─────────────────────────────

/// Baseline: with no replacement registered, a destroyed creature
/// lands in its owner's graveyard. Just confirms the test scaffolding
/// (`remove_from_battlefield_to_graveyard_raw` is the engine entry point
/// destroy / lethal-damage SBA / Effect::Destroy all funnel through).
#[test]
fn replacement_baseline_destroyed_creature_hits_graveyard() {
    use crabomination::card::Zone;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Sanity-check the resolver returns the intended zone with no
    // replacements registered.
    assert_eq!(
        g.resolve_zone_change(bear, Zone::Battlefield, Zone::Graveyard),
        Zone::Graveyard,
    );

    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().all(|c| c.id != bear));
}

/// A registered "would go to graveyard → exile instead" replacement
/// for a specific CardId reroutes the destroyed creature. Verifies
/// the wiring end-to-end through
/// `remove_from_battlefield_to_graveyard_raw` → resolver →
/// `place_card_at_resolved_zone`.
#[test]
fn replacement_redirects_graveyard_to_exile() {
    use crabomination::card::Zone;
    use crabomination::replacement::{
        ReplacementEffect, ReplacementId, ReplacementSource,
    };
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    g.register_replacement(ReplacementEffect {
        id: ReplacementId(0), // overwritten by register_replacement
        source: ReplacementSource::Card(bear),
        from: Some(Zone::Battlefield),
        to_zones: vec![Zone::Graveyard],
        redirect_to: Zone::Exile,
        optional: false,
    });

    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert!(
        g.exile.iter().any(|c| c.id == bear),
        "replacement should redirect the destroyed bear to exile",
    );
    assert!(
        g.players[0].graveyard.iter().all(|c| c.id != bear),
        "redirected bear must not also land in the graveyard",
    );
}

/// The replacement applies only to the card identified by its
/// `ReplacementSource::Card(_)` — other permanents leave normally.
#[test]
fn replacement_scoped_to_specific_card_id() {
    use crabomination::card::Zone;
    use crabomination::replacement::{
        ReplacementEffect, ReplacementId, ReplacementSource,
    };
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    g.register_replacement(ReplacementEffect {
        id: ReplacementId(0),
        source: ReplacementSource::Card(target),
        from: None,
        to_zones: vec![Zone::Graveyard],
        redirect_to: Zone::Exile,
        optional: false,
    });

    g.remove_from_battlefield_to_graveyard_raw(target);
    g.remove_from_battlefield_to_graveyard_raw(bystander);

    assert!(g.exile.iter().any(|c| c.id == target));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bystander));
}

/// CR 614.5 — a replacement effect applies at most once to a given
/// event. Without that guard, a "graveyard → exile" replacement
/// followed by an "exile → graveyard" replacement would loop. The
/// resolver tracks already-applied ids and refuses to re-fire any
/// single replacement; the second one's exile-source path fires
/// once, ending the walk at graveyard.
#[test]
fn replacement_does_not_apply_same_effect_twice() {
    use crabomination::card::Zone;
    use crabomination::replacement::{
        ReplacementEffect, ReplacementId, ReplacementSource,
    };
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Two replacements forming a loop: A: graveyard → exile, B: exile
    // → graveyard. With the applied-once guard, the destroyed bear
    // goes graveyard → exile (A) → graveyard (B) and stops.
    g.register_replacement(ReplacementEffect {
        id: ReplacementId(0),
        source: ReplacementSource::Card(bear),
        from: None,
        to_zones: vec![Zone::Graveyard],
        redirect_to: Zone::Exile,
        optional: false,
    });
    g.register_replacement(ReplacementEffect {
        id: ReplacementId(0),
        source: ReplacementSource::Card(bear),
        from: None,
        to_zones: vec![Zone::Exile],
        redirect_to: Zone::Graveyard,
        optional: false,
    });

    // Resolver alone — should terminate at Graveyard after walking
    // A then B.
    assert_eq!(
        g.resolve_zone_change(bear, Zone::Battlefield, Zone::Graveyard),
        Zone::Graveyard,
    );

    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().all(|c| c.id != bear));
}

/// The `from` filter on a replacement effect gates by origin zone. A
/// replacement keyed on `from: Some(Library)` should NOT fire when
/// the card is leaving the battlefield.
#[test]
fn replacement_from_filter_gates_origin_zone() {
    use crabomination::card::Zone;
    use crabomination::replacement::{
        ReplacementEffect, ReplacementId, ReplacementSource,
    };
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Replacement only fires for library origins; the battlefield
    // departure should not be intercepted.
    g.register_replacement(ReplacementEffect {
        id: ReplacementId(0),
        source: ReplacementSource::Card(bear),
        from: Some(Zone::Library),
        to_zones: vec![Zone::Graveyard],
        redirect_to: Zone::Exile,
        optional: false,
    });

    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().all(|c| c.id != bear));
}

/// `unregister_replacement` drops the entry; subsequent zone changes
/// behave as if it was never registered.
#[test]
fn replacement_unregister_drops_effect() {
    use crabomination::card::Zone;
    use crabomination::replacement::{
        ReplacementEffect, ReplacementId, ReplacementSource,
    };
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let rid = g.register_replacement(ReplacementEffect {
        id: ReplacementId(0),
        source: ReplacementSource::Card(bear),
        from: None,
        to_zones: vec![Zone::Graveyard],
        redirect_to: Zone::Exile,
        optional: false,
    });
    assert!(g.unregister_replacement(rid));
    // No replacement now active.
    assert_eq!(
        g.resolve_zone_change(bear, Zone::Battlefield, Zone::Graveyard),
        Zone::Graveyard,
    );

    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

// ── Phase I/J/L/M — Commander format end-to-end ───────────────────────────

/// Minimal legendary creature for Commander tests — free to cast,
/// no abilities, sorcery-speed (the default for a Creature card type).
/// Avoids depending on a real catalog commander with a 5-color cost
/// while still being a Legal commander (Legendary + Creature).
fn test_commander() -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, Supertype};
    CardDefinition {
        name: "Test Commander",
        cost: crabomination::mana::ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// `seat_commanders` installs the card into the seat's command zone,
/// records its CardId on `Player.commanders`, and registers the CR 903.9b
/// hand / library replacement. Graveyard and exile are NOT replaced — CR
/// 903.9a moves those as a state-based action after the card arrives
/// (see `cr_903_9a_commander_dies_then_returns_as_an_sba`). Verifies the
/// Phase I/J wiring as a whole.
#[test]
fn seat_commanders_sets_up_command_zone_and_replacement() {
    use crabomination::card::Zone;
    let mut g = two_player_game();
    let ids = g.seat_commanders(0, vec![test_commander()]);
    assert_eq!(ids.len(), 1);
    let cmd = ids[0];

    // Command zone populated.
    assert_eq!(g.players[0].command.len(), 1);
    assert_eq!(g.players[0].command[0].id, cmd);

    // CardId tracked on Player.commanders + reachable via is_commander.
    assert!(g.players[0].commanders.contains(&cmd));
    assert!(g.is_commander(cmd));
    // Non-commander cards should not be flagged.
    assert!(!g.is_commander(crabomination::card::CardId(99_999)));

    // CR 903.9b — hand and library are replaced by the command zone.
    for would_be in [Zone::Hand, Zone::Library] {
        assert_eq!(
            g.resolve_zone_change(cmd, Zone::Battlefield, would_be),
            Zone::Command,
            "{would_be:?} should redirect to Command",
        );
    }
    // CR 903.9a — graveyard and exile are not: the commander gets there.
    for zone in [Zone::Graveyard, Zone::Exile] {
        assert_eq!(g.resolve_zone_change(cmd, Zone::Battlefield, zone), zone);
    }
}

/// CR 903.9a — a destroyed commander goes to the graveyard, and the next
/// state-based-action check moves it to the command zone. (Until this was
/// fixed it was replaced straight into the command zone — the pre-2020
/// rule — so it never died.)
#[test]
fn destroyed_commander_returns_to_command_zone() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    // Manually move the commander out of the command zone onto the
    // battlefield (skipping the full cast flow — the wiring point we
    // care about is the leave-play replacement).
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let mut card = g.players[0].command.remove(pos);
    card.controller = 0;
    g.battlefield.push(card);

    g.remove_from_battlefield_to_graveyard_raw(cmd);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == cmd),
        "a destroyed commander really reaches the graveyard",
    );

    g.check_state_based_actions();
    assert!(
        g.players[0].command.iter().any(|c| c.id == cmd),
        "the SBA returns it to the command zone",
    );
    assert!(
        g.players[0].graveyard.iter().all(|c| c.id != cmd),
        "destroyed commander must NOT also be in the graveyard",
    );
}

/// Phase L — casting from the command zone pays the printed cost
/// the first time and `{2}` extra each subsequent time
/// (the commander tax). With a free test commander, first cast
/// costs nothing, second cast needs {2}.
#[test]
fn commander_cast_tax_accrues_per_recast() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    // First cast — free.
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("first cast from CZ should succeed with no mana required");
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(1));
    // Command zone empty post-cast (until SBA / leave-play bounces
    // the resolved permanent back).
    assert!(g.players[0].command.iter().all(|c| c.id != cmd));
    // A spell is on the stack.
    assert_eq!(g.stack.len(), 1);
}

/// The client view surfaces the running commander tax (CR 903.8) so the
/// HUD can show a per-commander chip: `commander_casts` carries the
/// commander's name and its command-zone cast count, for every viewer.
#[test]
fn view_projects_commander_cast_tally() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    // Before any cast: entry present with a zero count (chip hidden).
    let view = crabomination::server::view::project(&g, 1);
    assert_eq!(view.players[0].commander_casts, vec![("Test Commander".to_string(), 0)]);

    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .unwrap();
    crabomination::game::drain_stack(&mut g);

    // After one cast the tally is public to every seat — including while
    // the commander is on the battlefield (name resolves outside the CZ).
    for viewer in 0..2 {
        let view = crabomination::server::view::project(&g, viewer);
        assert_eq!(
            view.players[0].commander_casts,
            vec![("Test Commander".to_string(), 1)],
            "viewer {viewer} sees the cast tally",
        );
    }
}

/// Phase L — second cast pays the tax. Cast once, drain stack,
/// destroy the commander (which bounces it back via the J replacement),
/// then attempt the second cast: with no mana in pool the cast
/// should fail (the {2} tax is unpaid). Add 2 colorless mana → cast
/// succeeds, count bumps to 2.
#[test]
fn commander_cast_tax_blocks_unpaid_recast() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    // Cast 1 (free).
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .unwrap();
    crabomination::game::drain_stack(&mut g);
    // Commander should now be on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == cmd));

    // Destroy it — the CR 903.9a SBA brings it back to the command zone.
    g.remove_from_battlefield_to_graveyard_raw(cmd);
    g.check_state_based_actions();
    assert!(g.players[0].command.iter().any(|c| c.id == cmd));

    // Reset priority/step (drain_stack may have advanced).
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    // Cast 2 with empty mana pool — must fail; tax is {2}.
    let res = g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    });
    assert!(res.is_err(), "second cast with no mana should fail tax payment");
    // Tax wasn't paid → count stays at 1, commander returns to CZ.
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(1));
    assert!(
        g.players[0].command.iter().any(|c| c.id == cmd),
        "failed cast must put the commander back in the command zone",
    );

    // Pay the tax by stocking 2 colorless mana, then try again.
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("second cast pays {2} tax via 2 white mana");
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(2));
}

/// Phase M — 21 commander damage from a single commander eliminates
/// the victim, even if their life total is still positive.
#[test]
fn twenty_one_commander_damage_eliminates_victim() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];

    // Record 21 commander damage directly (mirrors what the combat
    // path does after combat-damage-to-player resolves). SBA must
    // then eliminate seat 1.
    g.record_commander_damage(1, cmd, 21);
    assert_eq!(
        g.commander_damage.get(&(1, cmd)).copied(),
        Some(21),
    );
    g.check_state_based_actions();
    assert!(
        g.players[1].eliminated,
        "seat 1 should lose to 21 commander damage even with life > 0",
    );
}

/// Phase M — accumulated damage below 21 does NOT eliminate, even
/// if it spans multiple smaller hits. Crossing the threshold via a
/// final hit triggers the SBA.
#[test]
fn commander_damage_under_21_does_not_eliminate() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];

    g.record_commander_damage(1, cmd, 10);
    g.record_commander_damage(1, cmd, 10);
    g.check_state_based_actions();
    assert!(!g.players[1].eliminated, "20 commander damage is not lethal");

    g.record_commander_damage(1, cmd, 1);
    g.check_state_based_actions();
    assert!(g.players[1].eliminated, "21st point of commander damage is lethal");
}

/// Phase M — damage from a *non-commander* source doesn't accumulate
/// into the commander-damage tally. `record_commander_damage` is the
/// gate; the combat / direct-damage paths only call it when
/// `is_commander(source)` is true. Verify both paths via the
/// effects/movement.rs direct-damage entry: a non-commander hit on
/// a player doesn't touch the table.
#[test]
fn non_commander_damage_does_not_count_toward_21() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No seat_commanders call — `bear` isn't a commander.
    assert!(!g.is_commander(bear));

    // Hypothetically deal 21 damage from this non-commander; since
    // it's not a commander, the table stays empty and the SBA
    // doesn't fire on it. (We exercise the gate directly rather
    // than through the full damage pipeline to keep the test focused.)
    if g.is_commander(bear) {
        g.record_commander_damage(1, bear, 21);
    }
    g.check_state_based_actions();
    assert!(!g.players[1].eliminated, "non-commander damage doesn't kill via the 21-rule");
    assert!(g.commander_damage.is_empty());
}

// ── Phase K — color identity & Commander deck validation ──────────────────

#[test]
fn color_identity_unions_cost_colors() {
    use crabomination::format::color_identity;
    use crabomination::mana::{Color, ColorSet};

    // Atraxa, Grand Unifier — {3}{G}{W}{U}{B}: all 5 colors? Actually
    // 4-color (no red). Verify each color in cost is in identity.
    let atraxa = catalog::atraxa_grand_unifier();
    let id = color_identity(&atraxa);
    assert!(id.contains(Color::Green));
    assert!(id.contains(Color::White));
    assert!(id.contains(Color::Blue));
    assert!(id.contains(Color::Black));
    assert!(!id.contains(Color::Red), "Atraxa has no red pip");
    assert_eq!(id.len(), 4);

    // CR 903.4 counts rules-text symbols too: Mox Pearl's `{0}` cost is
    // colorless but its "{T}: Add {W}" makes it white, and so illegal in a
    // deck whose commander is not. (This assertion read `ColorSet::empty()`
    // until the identity walk learned to read past the mana cost.)
    let mox = catalog::mox_pearl();
    assert_eq!(color_identity(&mox), ColorSet::single(Color::White));

    // A source that adds *any* color prints no colored symbol — Sol Ring
    // stays colorless and fits every commander.
    assert_eq!(color_identity(&catalog::sol_ring()), ColorSet::empty());
}

#[test]
fn commander_deck_validator_catches_off_color_card() {
    use crabomination::format::{validate_commander_deck, CommanderDeckError, Deck};

    // Mono-green commander → only green cards allowed.
    let llanowar = catalog::llanowar_elves();
    let main = vec![catalog::lightning_bolt()]; // red — off-color
    let deck = Deck {
        commanders: vec![llanowar],
        main,
        sideboard: vec![],
    };
    let err = validate_commander_deck(&deck).unwrap_err();
    let (_generic, cmd) = err;
    assert!(
        cmd.iter().any(|e| matches!(e, CommanderDeckError::OffColorIdentity { .. })),
        "Lightning Bolt under a green commander should be flagged off-color",
    );
}

#[test]
fn commander_deck_validator_requires_legendary_creature() {
    use crabomination::format::{validate_commander_deck, CommanderDeckError, Deck};

    // Lightning Bolt is an Instant — not a legendary creature.
    let deck = Deck {
        commanders: vec![catalog::lightning_bolt()],
        main: vec![],
        sideboard: vec![],
    };
    let err = validate_commander_deck(&deck).unwrap_err();
    let (_generic, cmd) = err;
    assert!(
        cmd.iter().any(|e| matches!(e, CommanderDeckError::NotLegendaryCreature { .. })),
    );
}

#[test]
fn commander_deck_validator_requires_a_commander() {
    use crabomination::format::{validate_commander_deck, CommanderDeckError, Deck};
    let deck = Deck::default();
    let err = validate_commander_deck(&deck).unwrap_err();
    let (_generic, cmd) = err;
    assert!(cmd.iter().any(|e| matches!(e, CommanderDeckError::MissingCommander)));
}

/// CR 903.3c / 702.124 — two commanders need a pairing ability: both with
/// Partner, "partner with" each other, or Choose a Background plus a
/// Background (which, alone among commanders, needn't be a creature).
#[test]
fn cr_702_124_commander_pair_needs_partner_or_background() {
    use crabomination::card::{CardDefinition, CardType, EnchantmentSubtype, Subtypes, Supertype};
    use crabomination::format::{commanders_may_pair, validate_commander_deck, CommanderDeckError, Deck};

    let pair_errors = |a: CardDefinition, b: CardDefinition| {
        let deck = Deck { commanders: vec![a, b], main: vec![], sideboard: vec![] };
        let (_generic, cmd) = validate_commander_deck(&deck).unwrap_err();
        cmd
    };
    // Akiri and Ravos both have Partner.
    assert!(commanders_may_pair(&catalog::akiri_line_slinger(), &catalog::ravos_soultender()));
    let cmd = pair_errors(catalog::akiri_line_slinger(), catalog::ravos_soultender());
    assert!(!cmd.iter().any(|e| matches!(e, CommanderDeckError::NotPartners { .. })));

    // Partner on one side only is not enough.
    let cmd = pair_errors(catalog::akiri_line_slinger(), test_commander());
    assert!(cmd.iter().any(|e| matches!(e, CommanderDeckError::NotPartners { .. })));

    // "Partner with Khorvath" doesn't pair with a plain Partner.
    assert!(!commanders_may_pair(&catalog::sylvia_brightspear(), &catalog::akiri_line_slinger()));

    // Gut chooses a Background; the Background is a legal commander even
    // though it isn't a creature.
    let background = CardDefinition {
        name: "Test Background",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Background],
            ..Default::default()
        },
        ..Default::default()
    };
    let cmd = pair_errors(catalog::gut_true_soul_zealot(), background.clone());
    assert!(cmd.is_empty(), "Gut + a Background is a legal pair: {cmd:?}");
    // …but a Background next to a Partner commander is neither pair.
    let cmd = pair_errors(catalog::akiri_line_slinger(), background);
    assert!(cmd.iter().any(|e| matches!(e, CommanderDeckError::NotPartners { .. })));
    assert!(cmd.iter().any(|e| matches!(e, CommanderDeckError::NotLegendaryCreature { .. })));
}

// ── CR 113.6b — abilities that function from the command zone (Eminence) ──

/// A commander whose upkeep trigger functions in the zone `zone` names.
fn upkeep_gain_commander(
    name: &'static str,
    zone: crabomination::effect::TriggerZone,
) -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, Supertype, TriggeredAbility};
    use crabomination::effect::{EventKind, EventScope, EventSpec, Value};
    CardDefinition {
        name,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                zone,
                ..EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
            },
            effect: crabomination::effect::Effect::GainLife {
                who: crabomination::card::Selector::Player(PlayerRef::You),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// CR 113.6b — "an ability that states which zones it functions in functions
/// only from those zones". Eminence (CR 207.2c) states the command zone *or*
/// the battlefield, so the trigger fires from the command zone; an ordinary
/// trigger on a commander sitting in the command zone does not.
#[test]
fn cr_113_6b_eminence_trigger_fires_from_the_command_zone() {
    use crabomination::effect::TriggerZone;
    for (zone, expected) in [(TriggerZone::Printed, 0), (TriggerZone::CommandZoneToo, 2)] {
        let mut g = two_player_game();
        g.seat_commanders(0, vec![upkeep_gain_commander("Eminence Test", zone)]);
        g.active_player_idx = 0;
        let before = g.players[0].life;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(
            g.players[0].life - before,
            expected,
            "{zone:?} from the command zone",
        );
    }
}

/// CR 113.6b — the exclusive spelling. Oloro, Ageless Ascetic's second upkeep
/// trigger reads "if Oloro is in the command zone", with no "or on the
/// battlefield", so it must *not* fire once the commander has been cast.
#[test]
fn cr_113_6b_command_zone_only_trigger_is_silent_on_the_battlefield() {
    use crabomination::effect::TriggerZone;
    let mut g = two_player_game();
    let cmd = g.seat_commanders(
        0,
        vec![upkeep_gain_commander("Command Zone Only", TriggerZone::CommandZoneOnly)],
    )[0];
    g.active_player_idx = 0;

    let before = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life - before, 2, "it fires from the command zone");

    // Move it to the battlefield — the same trigger is now silent.
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let mut card = g.players[0].command.remove(pos);
    card.controller = 0;
    g.battlefield.push(card);
    let before = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life - before, 0, "…and nowhere else");
}

/// CR 113.6b — the same for an *event* trigger rather than a step one:
/// Edgar Markov's "whenever you cast another Vampire spell" reaches the
/// dispatcher from the command zone, which is a different walk.
#[test]
fn cr_113_6b_eminence_event_trigger_fires_from_the_command_zone() {
    use crabomination::card::{CardDefinition, CardType, Supertype, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Value};
    let watcher = CardDefinition {
        name: "Eminence Watcher",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .in_command_zone(),
            effect: Effect::GainLife {
                who: crabomination::card::Selector::Player(PlayerRef::You),
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    g.seat_commanders(0, vec![watcher]);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let spell = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);

    let before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the bears");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life - before, 3, "the command-zone trigger saw the cast");
}

/// CR 601.2f + CR 903.8 — a commander cast for an *alternative* cost still
/// pays the commander tax: the tax is an additional cost, and additional
/// costs ride on whichever cost the spell is being cast for. Zurgo
/// Bellstriker's printed cost is {R} and its dash cost is {1}{R}, so the
/// mana left in the pool says which one was charged.
#[test]
fn cr_601_2f_command_zone_alt_cost_still_pays_the_tax() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![catalog::zurgo_bellstriker()])[0];
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    // Dash {1}{R} with no tax yet: two red pays {R} and the {1}.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: true,
        pitch_card: None,
    })
    .expect("dashing the commander out of the command zone is legal");
    assert_eq!(
        g.players[0].mana_pool.total(),
        0,
        "the dash cost {{1}}{{R}} was charged, not the printed {{R}}",
    );
    assert_eq!(
        g.commander_cast_count.get(&cmd).copied(),
        Some(1),
        "an alt-cost command-zone cast bumps the tax counter like any other",
    );
    drain_stack(&mut g);

    // Back to the command zone, then dash again: {1}{R} + {2} of tax.
    g.remove_from_battlefield_to_graveyard_raw(cmd);
    g.check_state_based_actions();
    assert!(g.players[0].command.iter().any(|c| c.id == cmd));
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    g.players[0].mana_pool.add(Color::Red, 3);
    let res = g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: true,
        pitch_card: None,
    });
    assert!(res.is_err(), "three mana is a mana short of {{1}}{{R}} plus the {{2}} tax");
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(1));
    assert!(
        g.players[0].command.iter().any(|c| c.id == cmd),
        "the failed alt cast puts the commander back in the command zone",
    );

    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: true,
        pitch_card: None,
    })
    .expect("four mana pays the dash cost plus the {2} tax");
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(2));
}

/// CR 903.9b — dash's end-step bounce would put the commander in its
/// owner's *hand*, which is exactly the event the commander replacement
/// covers: the owner may put it into the command zone instead. The two
/// rules meet on the one card, so the dashed commander comes home rather
/// than sitting in hand.
#[test]
fn cr_903_9b_dashed_commander_returns_to_the_command_zone() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![catalog::zurgo_bellstriker()])[0];
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: true,
        pitch_card: None,
    })
    .expect("dash from the command zone");
    drain_stack(&mut g);
    assert!(
        g.battlefield
            .iter()
            .find(|c| c.id == cmd)
            .is_some_and(|c| c.granted_keywords_eot.contains(&crabomination::card::Keyword::Haste)),
        "CR 702.110 — a dashed commander still gains haste",
    );

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == cmd), "the dash bounce happened");
    assert!(
        g.players[0].hand.iter().all(|c| c.id != cmd),
        "CR 903.9b replaces the move to hand",
    );
    assert!(
        g.players[0].command.iter().any(|c| c.id == cmd),
        "the dashed commander goes to the command zone instead",
    );
}

/// CR 903.8 — the heuristic bot casts its commander from the command zone.
/// It had no candidate for it at all, so bot seats in a Commander game
/// played their 99 without the commander.
#[test]
fn bot_casts_its_commander_from_the_command_zone() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![catalog::akiri_line_slinger()])[0];
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    // Postcombat: in the first main the bot's summon-sick hold (round 78)
    // rightly waits with a creature that can't attack this turn.
    g.step = TurnStep::PostCombatMain;
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(0, catalog::plains());
    let action = HeuristicBot::new().next_action(&g, 0).expect("the bot acts");
    assert!(
        matches!(action, GameAction::CastFromCommandZone { card_id, .. } if card_id == cmd),
        "expected the commander cast, got {action:?}",
    );
}

// ── Polish: cross-team triggers / optional commander redirect / 2HG mulligan ──

/// CR 810.8 — in 2HG, "whenever you gain life" fires for the
/// teammate too when the *other* teammate gains life. We can't
/// observe this directly with the catalog because the resolved
/// trigger pushes onto the stack and resolves; the cleanest proof
/// is via the dispatcher's candidate list. Use an inline pinger
/// with `LifeGained / YourControl` scope, register it on each
/// teammate, then synthesize a LifeGained event for one teammate
/// and confirm both pinger triggers stacked.
#[test]
fn two_headed_giant_lifegain_fires_partner_yourcontrol_trigger() {
    use crabomination::card::{CardDefinition, CardId, CardType, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};

    let pinger = |name: &'static str| CardDefinition {
        name,
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    };

    let mut g = game_with_format(Format::TwoHeadedGiant, 4);
    // Teams 0+1 vs 2+3 (default 2HG partitioning).
    let team_a_pinger = g.add_card_to_battlefield(0, pinger("Pinger-A"));
    let team_a_partner_pinger = g.add_card_to_battlefield(1, pinger("Pinger-A2"));
    let team_b_pinger = g.add_card_to_battlefield(2, pinger("Pinger-B"));

    // Seat 1 gains life. Pre-polish, only seat 1's own pinger
    // (`team_a_partner_pinger`) would fire. With the CR 810.8 widening
    // both team-A pingers should fire; team-B's should not.
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 1 }]);

    let sources: Vec<CardId> = g
        .stack
        .iter()
        .map(|item| match item {
            StackItem::Trigger { source, .. } => *source,
            other => panic!("expected only Trigger stack items, got {other:?}"),
        })
        .collect();

    assert!(
        sources.contains(&team_a_pinger),
        "seat 0's YourControl trigger should fire when teammate (seat 1) gains life",
    );
    assert!(
        sources.contains(&team_a_partner_pinger),
        "seat 1's own YourControl trigger should still fire",
    );
    assert!(
        !sources.contains(&team_b_pinger),
        "seat 2's YourControl trigger must NOT fire — they're an opponent",
    );
}

/// CR 903.9a — the move to the command zone is a "may." A scripted decider
/// answering Bool(false) to `Decision::CommanderRedirect` leaves the
/// commander in the graveyard, and it is asked once per arrival: the next
/// SBA check (where the script has run dry and would answer "yes") must not
/// ask again.
#[test]
fn commander_redirect_can_be_declined() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];

    // Script the next CommanderRedirect prompt to "no" — the
    // following destroy should land in the graveyard not the
    // command zone.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));

    // Put the commander on the battlefield (skipping the cast flow)
    // and destroy it.
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let card = g.players[0].command.remove(pos);
    g.battlefield.push(card);
    g.remove_from_battlefield_to_graveyard_raw(cmd);
    g.check_state_based_actions();
    g.check_state_based_actions();

    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == cmd),
        "with redirect declined, commander stays in the graveyard",
    );
    assert!(
        g.players[0].command.iter().all(|c| c.id != cmd),
        "redirected-out commander must NOT also land in the command zone",
    );
}

/// CR 903.9a — a commander that dies really is put into the graveyard: it
/// counts as a permanent put into a graveyard this turn (Gravestorm's tally,
/// which the pre-2020 replacement skipped because the card never got
/// there), "whenever a creature dies" sees it (Blood Artist drains), and it
/// still ends up in the command zone once the SBA runs.
#[test]
fn cr_903_9a_commander_dies_then_returns_as_an_sba() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    g.add_card_to_battlefield(0, catalog::blood_artist());
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let card = g.players[0].command.remove(pos);
    g.battlefield.push(card);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: blade,
        target: Some(Target::Permanent(cmd)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    g.check_state_based_actions();

    assert_eq!(g.permanents_to_graveyard_this_turn, 1, "it reached the graveyard");
    assert_eq!(g.players[1].life, 19, "Blood Artist saw the commander die");
    assert!(g.players[0].command.iter().any(|c| c.id == cmd));
    assert!(g.players[0].graveyard.iter().all(|c| c.id != cmd));
}

/// CR 903.10a / 704.6c — only *combat* damage from a commander counts
/// toward 21. Non-combat damage from it (a pinger commander's ability, a
/// fight) is ordinary damage.
#[test]
fn cr_903_10a_noncombat_commander_damage_does_not_count() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(1), 3, Some(cmd), &mut evs);
    assert_eq!(g.players[1].life, 17, "the damage is still dealt");
    assert!(g.commander_damage.is_empty(), "but it is not commander damage");
}

/// 2HG inherits the per-seat mulligan chain from Phase A — each
/// teammate decides independently (CR 103.5). This locks in the
/// inherited behavior so a future change to mulligan logic that
/// inadvertently couples teammates would be caught.
#[test]
fn two_headed_giant_mulligan_chain_is_per_seat() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);
    // Stock each player's library so opening hands draw.
    for seat in 0..4 {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g.start_mulligan_phase();

    // The mulligan decision should visit each seat in order, one at
    // a time — exactly the FFA behavior. Pre-2HG-mulligan polish
    // there's no team coupling.
    for expected_seat in 0..4 {
        let pd = g
            .pending_decision
            .as_ref()
            .unwrap_or_else(|| panic!("expected mulligan decision for seat {expected_seat}"));
        match &pd.decision {
            crabomination::decision::Decision::Mulligan { player, .. } => {
                assert_eq!(
                    *player, expected_seat,
                    "2HG mulligan must visit seat {expected_seat} independently",
                );
            }
            other => panic!("expected Mulligan, got {other:?}"),
        }
        g.submit_decision(crabomination::decision::DecisionAnswer::Keep).unwrap();
    }
    assert!(g.pending_decision.is_none(), "all four mulligans resolved");
}

// ── DefendingPlayer in combat-damage triggers (CR 509.2) ────────────────────

#[test]
fn abyssal_specter_only_defending_player_discards_in_ffa() {
    use crabomination::game::types::TurnStep;
    let mut g = multi_player_game(3);
    let spec = g.add_card_to_battlefield(0, catalog::abyssal_specter());
    g.clear_sickness(spec);
    // Give every opponent a card to lose so we can see who's hit.
    for seat in [1usize, 2] {
        g.add_card_to_hand(seat, catalog::grizzly_bears());
    }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: spec,
        target: AttackTarget::Player(2),
    }]))
    .expect("specter attacks seat 2");
    let (h1, h2) = (g.players[1].hand.len(), g.players[2].hand.len());
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1, "non-defending opponent keeps cards");
    assert_eq!(g.players[2].hand.len(), h2 - 1, "only the defending player discards");
}

// ── Myriad (CR 702.115) ────────────────────────────────────────────────────

#[test]
fn cr_702_115_myriad_copies_attack_each_other_opponent_then_exile() {
    use crabomination::card::{CardDefinition, CardType, CreatureType};
    use crabomination::effect::shortcut;
    let mut g = multi_player_game(3);
    let mythic = CardDefinition {
        name: "Myriad Marauder",
        card_types: vec![CardType::Creature],
        subtypes: crabomination::card::Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![shortcut::myriad()],
        ..Default::default()
    };
    let m = g.add_card_to_battlefield(0, mythic);
    g.clear_sickness(m);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: m, target: AttackTarget::Player(1),
    }])).expect("attack seat 1");
    drain_stack(&mut g);
    // One copy minted, tapped + attacking seat 2 (the other opponent).
    let copies: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Myriad Marauder").collect();
    assert_eq!(copies.len(), 1, "one copy per non-defending opponent");
    let copy = copies[0];
    assert!(copy.tapped, "copy is tapped");
    assert!(g.attacking().iter().any(|a| a.attacker == copy.id
        && a.target == AttackTarget::Player(2)), "copy attacks the other opponent");
    let copy_id = copy.id;
    // Step through to postcombat main; leaving EndCombat exiles the copy.
    let mut iters = 0;
    while g.step != TurnStep::PostCombatMain && iters < 50 {
        let _ = g.pass_priority();
        drain_stack(&mut g);
        iters += 1;
    }
    // The copy is exiled at end of combat; a token in exile then ceases to
    // exist as a state-based action (CR 111.7), so it's gone entirely.
    assert!(!g.battlefield.iter().any(|c| c.id == copy_id), "copy left the battlefield");
    assert!(!g.exile.iter().any(|c| c.id == copy_id), "exiled token ceased to exist");
}

// ── CR 810 — Two-Headed Giant ────────────────────────────────────────────────

/// CR 810.4/810.9 — a team shares one 30-life pool, and damage dealt to each
/// player individually lands on it twice.
#[test]
fn cr_810_9_damage_to_each_teammate_hits_the_shared_pool_twice() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).expect("2v2");
    assert_eq!(g.effective_life(0), 30);
    g.adjust_life(0, -4);
    g.adjust_life(1, -4);
    assert_eq!(g.effective_life(0), 22, "both hits landed on the one pool");
    assert_eq!(g.effective_life(1), 22, "teammates read the same total");
    assert_eq!(g.effective_life(2), 30, "the other team is untouched");
}

/// CR 810.8c — the shared pool hitting 0 eliminates the whole team at once.
#[test]
fn cr_810_8c_zero_shared_life_eliminates_both_teammates() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).expect("2v2");
    g.adjust_life(0, -30);
    g.check_state_based_actions();
    assert!(g.players[0].eliminated && g.players[1].eliminated, "the team lost together");
    assert!(!g.players[2].eliminated && !g.players[3].eliminated);
}

/// CR 810.5/810.8d — poison is a shared team resource with a fifteen-counter
/// threshold, so a teammate at 10 doesn't lose on their own.
#[test]
fn cr_810_8d_team_poison_is_shared_and_loses_at_fifteen() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);
    g.assign_teams(vec![vec![0, 1], vec![2, 3]]).expect("2v2");
    g.players[0].poison_counters = 10;
    g.check_state_based_actions();
    assert!(!g.players[0].eliminated, "ten is the solo threshold, not the team's");
    assert_eq!(g.effective_poison(1), 10, "the teammate reads the team total");

    g.players[1].poison_counters = 5;
    g.check_state_based_actions();
    assert!(g.players[0].eliminated && g.players[1].eliminated, "fifteen between them");
}

/// A solo-team (FFA) seat keeps the plain ten-counter threshold.
#[test]
fn cr_704_5c_solo_player_still_loses_at_ten_poison() {
    let mut g = game_with_format(Format::Commander, 4);
    g.players[0].poison_counters = 10;
    g.check_state_based_actions();
    assert!(g.players[0].eliminated);
    assert!(!g.players[1].eliminated);
}

/// The server view reports the *team's* life and poison to both members of a
/// shared pool, along with the threshold the HUD should render against.
#[test]
fn two_headed_giant_view_reports_the_shared_pool() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);
    g.adjust_life(0, -7);
    g.players[1].poison_counters = 4;
    for seat in [0, 1] {
        let v = crabomination::server::view::project(&g, seat);
        assert_eq!(v.players[seat].life, 23, "seat {seat} reads the team pool");
        assert_eq!(v.players[seat].poison_counters, 4, "team poison, not the seat's own");
        assert_eq!(v.players[seat].poison_loss_threshold, 15);
    }
    let v = crabomination::server::view::project(&g, 2);
    assert_eq!(v.players[2].life, 30, "the other team is untouched");
    assert_eq!(v.players[2].poison_counters, 0);
}

// ── CR 702.124c — "Partner with" ──────────────────────────────────────────

/// CR 702.124c — "Partner with [name]" prints a trigger, not just a
/// deck-construction pairing: "When this creature enters, target player may
/// put [name] into their hand from their library, then shuffle." The keyword
/// had been modelled as the pairing alone, so the ETB half did nothing.
/// Every seat is given a copy, so a mis-aimed pick would show up as the wrong
/// seat rather than as "nothing happened" — which is how the auto-targeter's
/// opponent-facing default for a "target player" slot was caught. A tutor
/// into the *target's own* hand is a gift, so the picker aims at the caster.
#[test]
fn cr_702_124c_partner_with_fetches_the_named_card_from_the_library() {
    use crabomination::game::types::Target;
    let mut g = multi_player_game(3);
    for seat in 0..3 {
        g.add_card_to_library(seat, catalog::khorvath_brightflame());
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    let sylvia = g.add_card_to_hand(0, catalog::sylvia_brightspear());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: sylvia,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Sylvia");
    crabomination::game::drain_stack(&mut g);

    let fetched: Vec<usize> = (0..3)
        .filter(|&s| {
            g.players[s].hand.iter().any(|c| c.definition.name == "Khorvath Brightflame")
        })
        .collect();
    assert_eq!(fetched, vec![0], "the fetch is a gift, so it aims at the caster");
    let seat = fetched[0];
    assert_eq!(
        g.players[seat].library.iter().filter(|c| c.definition.name == "Khorvath Brightflame").count(),
        0,
        "and it came out of that player's library",
    );
}

/// CR 702.124a/c — the pairing half is unchanged: the two may lead a deck
/// together, and a card that does not name them may not.
#[test]
fn cr_702_124a_partner_with_still_pairs_for_deck_construction() {
    use crabomination::format::commanders_may_pair;
    let sylvia = catalog::sylvia_brightspear();
    let khorvath = catalog::khorvath_brightflame();
    assert!(commanders_may_pair(&sylvia, &khorvath));
    assert!(commanders_may_pair(&khorvath, &sylvia));
    assert!(!commanders_may_pair(&sylvia, &catalog::llanowar_elves()));
}

// ── CR 903.3a — "can be your commander" on a non-creature ─────────────────

/// CR 903.3a — a legendary card that prints "[this] can be your commander"
/// may lead a deck even though it is not a legendary *creature*. Freyalise,
/// Llanowar's Fury is the implemented one; the validator used to reject every
/// planeswalker commander as `NotLegendaryCreature`.
#[test]
fn cr_903_3a_a_planeswalker_that_says_so_can_be_your_commander() {
    use crabomination::format::{CommanderDeckError, Deck, validate_commander_deck};

    let freyalise = catalog::freyalise_llanowars_fury();
    assert!(freyalise.can_be_commander);
    let deck = Deck {
        main: std::iter::repeat_with(catalog::forest).take(99).collect(),
        commanders: vec![freyalise],
        ..Default::default()
    };
    assert!(validate_commander_deck(&deck).is_ok(), "a planeswalker commander is legal");

    // A legendary non-creature *without* the line is still rejected.
    let deck = Deck {
        main: std::iter::repeat_with(catalog::forest).take(99).collect(),
        commanders: vec![catalog::sol_ring()],
        ..Default::default()
    };
    let (_, cmd) = validate_commander_deck(&deck).unwrap_err();
    assert!(
        cmd.iter().any(|e| matches!(e, CommanderDeckError::NotLegendaryCreature { .. })),
        "Sol Ring prints no such line: {cmd:?}",
    );
}

/// CR 903.8 — a planeswalker commander is cast from the command zone like any
/// other, tax included, and enters with its printed loyalty.
#[test]
fn cr_903_8_a_planeswalker_commander_casts_from_the_command_zone() {
    use crabomination::mana::Color;
    let mut g = game_with_format(Format::Commander, 4);
    let cmd = g.seat_commanders(0, vec![catalog::freyalise_llanowars_fury()])[0];
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(crabomination::game::GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast Freyalise from the command zone");
    crabomination::game::drain_stack(&mut g);
    let pw = g.battlefield_find(cmd).expect("Freyalise is on the battlefield");
    assert_eq!(
        pw.counter_count(crabomination::card::CounterType::Loyalty),
        3,
        "printed loyalty 3",
    );
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(1), "the tax counter moved");
}

// ── CR 903.4 — mana in the commander's color identity ─────────────────────

/// CR 903.4 — "one mana of any color in your commander's color identity".
/// Command Tower under a mono-green commander makes {G}, and the legal set it
/// offers the decider is that one color, not all five. Before
/// `ManaPayload::AnyColorInCommanderIdentity` these cards were plain
/// any-one-color, so a mono-green deck could tap Command Tower for {U}.
#[test]
fn cr_903_4_command_tower_is_limited_to_the_commander_identity() {
    use crabomination::mana::Color;
    let mut g = game_with_format(Format::Commander, 4);
    g.seat_commanders(0, vec![catalog::llanowar_elves()]); // mono-green identity
    assert_eq!(g.commander_identity_colors(0), vec![Color::Green]);

    let tower = g.add_card_to_battlefield(0, catalog::command_tower());
    g.clear_sickness(tower);
    g.priority.player_with_priority = 0;
    g.perform_action(crabomination::game::GameAction::ActivateAbility {
        card_id: tower,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap Command Tower");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "made {{G}}");
    for c in [Color::White, Color::Blue, Color::Black, Color::Red] {
        assert_eq!(g.players[0].mana_pool.amount(c), 0, "no off-identity mana");
    }
}

/// CR 903.4 — two commanders combine their identities (Partner), and a
/// commander's identity is read wherever the card currently is, not only from
/// the command zone.
#[test]
fn cr_903_4_identity_combines_commanders_and_survives_a_zone_change() {
    use crabomination::mana::Color;
    let mut g = game_with_format(Format::Commander, 4);
    let ids = g.seat_commanders(0, vec![catalog::llanowar_elves(), catalog::savannah_lions()]);
    assert_eq!(g.commander_identity_colors(0), vec![Color::White, Color::Green]);

    // Move one commander to the battlefield: the identity is unchanged.
    let pos = g.players[0].command.iter().position(|c| c.id == ids[0]).unwrap();
    let card = g.players[0].command.remove(pos);
    g.battlefield.push(card);
    assert_eq!(g.commander_identity_colors(0), vec![Color::White, Color::Green]);
}

/// Outside a Commander game nobody has a commander, so the identity set is
/// empty and these cards keep the engine's pre-Commander behaviour: any
/// color. This is what keeps two-player traces byte-identical.
#[test]
fn cr_903_4_no_commander_leaves_the_any_color_behaviour_alone() {
    use crabomination::mana::Color;
    let g = two_player_game();
    assert_eq!(g.commander_identity_colors(0), Color::ALL.to_vec());
}

/// CR 903 / Command Beacon — "{T}, Sacrifice this land: Put your commander
/// into your hand from the command zone." The commander leaves the command
/// zone for the hand, and the CR 903.9b replacement does *not* bounce it
/// straight back (which would make the ability do nothing).
#[test]
fn command_beacon_moves_the_commander_from_the_command_zone_to_hand() {
    let mut g = game_with_format(Format::Commander, 4);
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    let beacon = g.add_card_to_battlefield(0, catalog::command_beacon());
    g.clear_sickness(beacon);
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(crabomination::game::GameAction::ActivateAbility {
        card_id: beacon,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice Command Beacon");
    crabomination::game::drain_stack(&mut g);
    assert!(g.players[0].command.is_empty(), "command zone emptied");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(g.players[0].hand.iter().any(|c| c.id == cmd), "the commander is in hand");
    // It is still this seat's commander — designation is a property of the
    // card, not of the zone (CR 903.3).
    assert!(g.is_commander(cmd));
}
