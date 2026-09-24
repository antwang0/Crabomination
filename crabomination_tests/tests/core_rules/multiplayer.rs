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

/// The client view says *who* in a pod: a goaded creature carries its
/// goader seats (CR 701.38b — the seats it must avoid attacking if able) and
/// a declared attacker carries its target and defending player (CR 508.1b),
/// so a four-player HUD can name them rather than show bare flags.
#[test]
fn view_projects_goaders_and_attack_targets_per_seat() {
    let mut g = multi_player_game(4);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().goaded_by = vec![1, 3];
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(2),
    }]))
    .expect("the goaded bear may attack the non-goader");

    for viewer in 0..4 {
        let view = crabomination::server::view::project(&g, viewer);
        let b = view.battlefield.iter().find(|p| p.id == bear).unwrap();
        assert!(b.goaded);
        assert_eq!(b.goaded_by, vec![1, 3], "viewer {viewer} sees both goaders");
        assert!(b.attacking);
        assert_eq!(b.attack_target, Some(AttackTarget::Player(2)));
        assert_eq!(b.defending_player, Some(2));
        let i = view.battlefield.iter().find(|p| p.id == idle).unwrap();
        assert!(i.goaded_by.is_empty());
        assert_eq!(i.attack_target, None);
        assert_eq!(i.defending_player, None);
        assert_eq!(i.protected_by, None, "a non-battle has no protector");
    }
    // CR 310.8 — a battle carries its protector, the seat an attack on it
    // is aimed at (CR 508.4), so the client can colour a planned attack.
    let battle = g.add_card_to_battlefield(0, catalog::invasion_of_zendikar());
    g.battlefield_find_mut(battle).unwrap().protected_by = Some(3);
    let view = crabomination::server::view::project(&g, 0);
    assert_eq!(view.battlefield.iter().find(|p| p.id == battle).unwrap().protected_by, Some(3));
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

/// CR 101.4 / 800.4d — the same APNAP guarantee with an eliminated seat in
/// the middle of the cycle. Active=0, seat 2 dead, so the alive cycle is
/// 0 → 1 → 3 and the live triggers keep that order.
///
/// Seat 2's trigger does not reach the stack at all: CR 800.4d, "if a
/// triggered ability that would be controlled by a player who has left the
/// game would be put onto the stack, it isn't put on the stack." Its
/// permanent is still there because this fixture sets `eliminated` directly
/// rather than running CR 800.4a's departure, which would have removed it —
/// that is what makes this a test of the *dispatch* filter rather than of
/// the board.
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
    let seat2 = g.add_card_to_battlefield(2, pinger("Pinger-2"));
    let seat3 = g.add_card_to_battlefield(3, pinger("Pinger-3"));
    g.players[2].eliminated = true;

    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 1 }]);
    assert_eq!(g.stack.len(), 3, "CR 800.4d — the dead seat's trigger is not put on the stack");

    let sources: Vec<CardId> = g
        .stack
        .iter()
        .map(|item| match item {
            StackItem::Trigger { source, .. } => *source,
            other => panic!("expected only Trigger stack items, got {other:?}"),
        })
        .collect();
    // APNAP rank for active=0 with seat 2 out of the game: 0 → 0, 1 → 1,
    // 3 → 2. Seat 2 has no rank because it has no trigger.
    assert_eq!(sources, vec![seat0, seat1, seat3]);
    assert!(
        g.battlefield_find(seat2).is_some(),
        "the permanent is untouched — CR 800.4d filters the trigger, not the board",
    );
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

/// A 2/2 whose whole text is "whenever this creature deals combat damage to a
/// player, draw a card" — a witness for damage actually being assigned.
fn damage_witness() -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, TriggeredAbility};
    use crabomination::effect::{EventKind, EventScope, EventSpec};
    CardDefinition {
        name: "Test Damage Witness",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: crabomination::effect::shortcut::draw(1),
        }],
        ..Default::default()
    }
}

/// CR 800.4e — "If combat damage would be assigned to a player who has left
/// the game, that damage isn't assigned."
///
/// The reachable shape is first strike (CR 702.7b): the first-strike sub-step
/// kills the defending seat, state-based actions take them out of the game
/// between the sub-steps, and the ordinary attacker is left pointing at a seat
/// that is gone. 800.4a's combat sweep is what makes 800.4e true here — the
/// witness never draws, so nothing was assigned.
#[test]
fn cr_800_4e_no_combat_damage_is_assigned_to_a_seat_that_left_mid_combat() {
    let mut g = multi_player_game(3);
    let fast = g.add_card_to_battlefield(0, catalog::white_knight());
    let slow = g.add_card_to_battlefield(0, damage_witness());
    for id in [fast, slow] {
        g.clear_sickness(id);
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    // Exactly lethal to the first strike, so the second sub-step is the one
    // under test rather than a seat that was already gone.
    g.players[1].life = 2;
    let hand_before = g.players[0].hand.len();

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: fast, target: AttackTarget::Player(1) },
        Attack { attacker: slow, target: AttackTarget::Player(1) },
    ]))
    .expect("a first-striker and an ordinary attacker at the same seat");
    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().expect("the first-strike sub-step");
    drain_stack(&mut g);
    assert!(!g.players[1].is_alive(), "the first strike took seat 1 out between the sub-steps");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("the regular sub-step");
    drain_stack(&mut g);

    assert!(!g.players[1].is_alive(), "the first strike took seat 1 out");
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "the ordinary attacker assigned nothing to a seat that has left (CR 800.4e)",
    );
    assert!(
        !g.attacking.iter().any(|a| a.attacker == slow),
        "and it is no longer in combat at all",
    );
}

/// CR 800.4k — "If a player who has left the game would begin a turn, that
/// turn doesn't begin." `next_alive_seat` is the whole implementation and it
/// had no test naming the rule; a turn handed to a departed seat is a table
/// that passes priority to nobody.
#[test]
fn cr_800_4k_a_departed_seats_turn_does_not_begin() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive(), "seat 1 is out before its turn would begin");

    for _ in 0..40 {
        if g.active_player_idx != 0 {
            break;
        }
        let _ = g.advance_step(Vec::new());
    }
    assert_eq!(g.active_player_idx, 2, "seat 1's turn does not begin — seat 2 takes it");
}

/// CR 800.4a, closing sentence — "If the player who left the game had
/// priority at the time they left, priority passes to the next player in turn
/// order who's still in the game." Priority gates the table exactly as a
/// pending decision does: every other seat's actions wait on the seat that
/// holds it, so leaving it on a player who is no longer in the game is the
/// same wedge the dropped-ask fix above closed.
#[test]
fn cr_800_4a_priority_passes_off_the_player_who_left() {
    let mut g = multi_player_game(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 2;

    g.players[2].life = 0;
    g.check_state_based_actions();

    assert!(!g.players[2].is_alive());
    assert_eq!(g.player_with_priority(), 0, "seat 2 → the next seat still in the game");
}

/// CR 800.4j — "If a player leaves the game during their turn, that turn
/// continues to its completion without an active player. If the active player
/// would receive priority, instead the next player in turn order receives
/// priority." The turn stays seat 0's; the priority does not.
#[test]
fn cr_800_4j_a_departed_active_player_never_receives_priority() {
    let mut g = multi_player_game(3);
    g.step = TurnStep::PreCombatMain;
    assert_eq!(g.active_player_idx, 0);

    g.players[0].life = 0;
    g.check_state_based_actions();

    assert_eq!(g.active_player_idx, 0, "the turn is still seat 0's and runs to completion");
    assert_eq!(g.player_with_priority(), 1, "priority is a live seat's");

    // Every later grant of priority to "the active player" — after each
    // resolution, and on entering each step — has to land on a seat that can
    // act, not on the one whose turn it nominally still is.
    g.give_priority_to_active();
    assert_eq!(g.player_with_priority(), 1);
}

/// CR 800.4d — "If a triggered ability that would be controlled by a player
/// who has left the game would be put onto the stack, it isn't put on the
/// stack." CR 800.4a removes what is already *on* the stack; a delayed
/// triggered ability is a registration that fires later, so it needs this
/// rule instead.
///
/// Puffer Extract's "destroy it at the beginning of the next end step" is
/// registered by seat 0 against a creature seat 0 controls but seat 2 owns.
/// Seat 0 leaves: the creature reverts to seat 2 (CR 800.4a) and survives the
/// end step. Seat 1's identical registration is the control — the end step
/// did fire.
#[test]
fn cr_800_4d_a_departed_players_delayed_trigger_is_not_put_on_the_stack() {
    use crabomination::game::types::Target;
    let mut g = multi_player_game(3);
    let arm = |g: &mut GameState, seat: usize, victim: crabomination::card::CardId| {
        let extract = g.add_card_to_battlefield(seat, catalog::puffer_extract());
        g.players[seat].mana_pool.add_colorless(2);
        g.priority.player_with_priority = seat;
        g.perform_action(GameAction::ActivateAbility {
            card_id: extract,
            ability_index: 0,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(1),
        })
        .expect("activate Puffer Extract");
        while !g.stack.is_empty() {
            g.resolve_top_of_stack().expect("resolve");
        }
    };

    // Seat 0 controls a creature seat 2 owns, and points its own Extract at it.
    let stolen = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.battlefield_find_mut(stolen).unwrap().controller = 0;
    arm(&mut g, 0, stolen);
    // Seat 1 arms the same ability against its own creature.
    let control = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    arm(&mut g, 1, control);
    assert_eq!(g.delayed_triggers.len(), 2, "both destructions are registered");

    g.players[0].life = 0;
    g.check_state_based_actions();
    assert_eq!(
        g.battlefield_find(stolen).unwrap().controller,
        2,
        "control reverts to the owner (CR 800.4a)",
    );
    assert!(
        !g.delayed_triggers.iter().any(|dt| dt.controller == 0),
        "the departed player's registration is gone, not left firing for ever",
    );

    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve");
    }

    assert!(
        g.battlefield_find(stolen).is_some(),
        "seat 0 left, so its delayed destroy never went on the stack",
    );
    assert!(g.battlefield_find(control).is_none(), "seat 1's did — the end step fired");
}

/// CR 800.4m — "When a player leaves the game, any continuous effects with
/// durations that last until that player's next turn or until a specific point
/// in that turn will last until that turn would have begun. They neither
/// expire immediately nor last indefinitely."
///
/// Mouth of the Storm gives every creature its controller's opponents control
/// -3/-0 until that controller's next turn. Seat 2 resolves it and then leaves:
/// the rotation never reaches seat 2 again, so without this rule seat 0's
/// creatures stay shrunk for the rest of the game.
#[test]
fn cr_800_4m_a_departed_players_until_your_next_turn_ends_when_that_turn_would_have() {
    let mut g = multi_player_game(3);
    for seat in 0..3 {
        for _ in 0..40 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let power = |g: &GameState| g.computed_permanent(bear).unwrap().power;

    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    g.move_card_to_battlefield_for_test(2, catalog::mouth_of_the_storm());
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve the ETB");
    }
    assert_eq!(power(&g), -1, "-3/-0 while it lasts");

    g.players[2].life = 0;
    g.check_state_based_actions();
    assert_eq!(power(&g), -1, "leaving does not end it early");

    // Seat 0's turn, then seat 1's: seat 2's next turn has not come round yet.
    let to_turn_of = |g: &mut GameState, seat: usize| {
        for _ in 0..400 {
            if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
                return;
            }
            let _ = g.advance_step(Vec::new());
        }
        panic!("never reached seat {seat}'s main phase");
    };
    to_turn_of(&mut g, 0);
    assert_eq!(power(&g), -1, "seat 0's turn is not seat 2's");
    to_turn_of(&mut g, 1);
    assert_eq!(power(&g), -1, "nor is seat 1's");

    // The next boundary skips seat 2 — the turn that would have begun.
    to_turn_of(&mut g, 0);
    assert_eq!(power(&g), 2, "and it does not last indefinitely either");
}

/// CR 800.4a — "any effects which give that player control of any objects or
/// players end" covers control of *players* (CR 723, Mindslaver), not only of
/// objects; CR 800.4b covers the other direction, "if a player would be
/// controlled by a player who has left the game, they aren't". A live seat
/// still pointed at a departed controller routes its whole turn through
/// `acting_seat_for` to a seat that cannot act.
#[test]
fn cr_800_4a_player_control_ends_when_either_side_leaves() {
    let mut g = multi_player_game(3);
    // Seat 0 takes control of seat 1's next turn, and of seat 2's after that.
    g.pending_player_control.push((1, 0));
    g.pending_player_control.push((2, 0));
    g.apply_pending_player_control(1);
    assert_eq!(g.acting_seat_for(1), 0, "seat 0 is playing seat 1's turn");

    g.players[0].life = 0;
    g.check_state_based_actions();

    assert_eq!(g.acting_seat_for(1), 1, "seat 1 acts for itself again (CR 800.4a)");
    assert!(
        g.pending_player_control.is_empty(),
        "and the grant queued against seat 2 never lands (CR 800.4b)",
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
fn commander_deck_validator_rejects_a_card_that_cannot_be_a_commander() {
    use crabomination::format::{validate_commander_deck, CommanderDeckError, Deck};

    // CR 903.3 — Lightning Bolt is an Instant: none of the kinds of card a
    // deck may designate as its commander.
    let deck = Deck {
        commanders: vec![catalog::lightning_bolt()],
        main: vec![],
        sideboard: vec![],
    };
    let err = validate_commander_deck(&deck).unwrap_err();
    let (_generic, cmd) = err;
    assert!(
        cmd.iter().any(|e| matches!(e, CommanderDeckError::IllegalCommander { .. })),
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

/// CR 702.124 — two commanders need a pairing ability: both with
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
    assert!(cmd.iter().any(|e| matches!(e, CommanderDeckError::IllegalCommander { .. })));
}

/// CR 702.124i — "Partner—[text]" pairs on label equality and nothing else:
/// two Friends forever commanders lead one deck, a Friends forever commander
/// and a plain Partner one do not, and neither does a different label.
#[test]
fn cr_702_124i_partner_label_pairs_only_with_the_same_label() {
    use crabomination::card::{CardDefinition, Keyword};
    use crabomination::format::commanders_may_pair;
    let elmar = catalog::elmar_ulvenwald_informant();
    let sophina = catalog::sophina_spearsage_deserter();
    assert!(
        commanders_may_pair(&elmar, &sophina),
        "two Friends forever commanders are a legal pair",
    );
    assert!(
        !commanders_may_pair(&elmar, &catalog::akiri_line_slinger()),
        "CR 702.124f — a labelled partner does not combine with plain Partner",
    );
    let survivor = CardDefinition {
        keywords: vec![Keyword::PartnerLabel("Survivors".into())],
        ..catalog::sophina_spearsage_deserter()
    };
    assert!(
        !commanders_may_pair(&elmar, &survivor),
        "a different label is a different ability",
    );
}

/// CR 702.124b/d — both commanders start in the command zone, and "when
/// casting a commander with partner, ignore how many times your *other*
/// commander has been cast": the CR 903.8 tax is per commander, not per seat.
#[test]
fn cr_702_124d_each_commander_carries_its_own_tax() {
    let mut g = game_with_format(Format::Commander, 2);
    let ids = g.seat_commanders(
        0,
        vec![catalog::akiri_line_slinger(), catalog::ravos_soultender()],
    );
    assert_eq!(ids.len(), 2);
    // CR 702.124b — both begin in the command zone.
    assert_eq!(g.players[0].command.len(), 2);
    assert!(ids.iter().all(|id| g.players[0].commanders.contains(id)));

    let (akiri, ravos) = (ids[0], ids[1]);
    // Two prior casts of one of them: its tax is {4}, the other's is still {0}.
    g.commander_cast_count.insert(akiri, 2);
    assert_eq!(g.commander_cast_count.get(&ravos).copied(), None);

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Ravos is {3}{W}{B}; the untaxed cast needs exactly that and no more.
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: ravos,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("Ravos pays his own printed cost — Akiri's two casts are not his");
    assert_eq!(g.commander_cast_count.get(&ravos).copied(), Some(1));
    assert_eq!(g.commander_cast_count.get(&akiri).copied(), Some(2), "untouched");
    assert_eq!(g.players[0].mana_pool.total(), 0, "and not a pip more");
}

/// CR 702.124d — "when determining whether a player has been dealt 21 or more
/// combat damage by the same commander, consider damage from each of your two
/// commanders separately." 20 from each is 40 and is not a loss.
#[test]
fn cr_702_124d_commander_damage_is_tracked_per_commander() {
    let mut g = game_with_format(Format::Commander, 2);
    let ids = g.seat_commanders(
        0,
        vec![catalog::akiri_line_slinger(), catalog::ravos_soultender()],
    );
    let (akiri, ravos) = (ids[0], ids[1]);
    for _ in 0..20 {
        g.record_commander_damage(1, akiri, 1);
        g.record_commander_damage(1, ravos, 1);
    }
    assert_eq!(g.commander_damage.get(&(1, akiri)).copied(), Some(20));
    assert_eq!(g.commander_damage.get(&(1, ravos)).copied(), Some(20));
    g.check_state_based_actions();
    assert!(g.players[1].is_alive(), "40 damage from two commanders is not 21 from one");

    // One more from either one crosses that one's own 21 (CR 903.10a).
    g.record_commander_damage(1, akiri, 1);
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive(), "21 from a single commander loses the game");
}

/// CR 702.124m — "Doctor's companion" pairs only with "a legendary Time Lord
/// Doctor creature card that has no other creature types". The last clause is
/// the whole rule: a Time Lord Scientist, a Human Doctor and a Doctor with a
/// third type are each one type away and each illegal.
#[test]
fn cr_702_124m_doctors_companion_needs_a_pure_time_lord_doctor() {
    use crabomination::card::{CardDefinition, CreatureType, Subtypes};
    use crabomination::format::commanders_may_pair;
    let graham = catalog::graham_obrien();
    let doctor = catalog::the_third_doctor();
    assert!(commanders_may_pair(&graham, &doctor), "the printed pair");
    assert!(commanders_may_pair(&doctor, &graham), "and in either order");

    let with_a_third_type = CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::TimeLord,
                CreatureType::Doctor,
                CreatureType::Scientist,
            ],
            ..Default::default()
        },
        ..catalog::the_third_doctor()
    };
    assert!(
        !commanders_may_pair(&graham, &with_a_third_type),
        "\"no other creature types\" — Romana II is a Time Lord Scientist",
    );
    let human_doctor = CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Doctor],
            ..Default::default()
        },
        ..catalog::the_third_doctor()
    };
    assert!(
        !commanders_may_pair(&graham, &human_doctor),
        "a Doctor that isn't a Time Lord isn't the Doctor",
    );
    // CR 702.124f — the partner abilities don't combine with each other.
    assert!(
        !commanders_may_pair(&graham, &catalog::akiri_line_slinger()),
        "Doctor's companion is not plain Partner",
    );
    assert!(
        !commanders_may_pair(&graham, &catalog::elmar_ulvenwald_informant()),
        "nor a labelled partner",
    );
    // Two companions are not a pair either: neither is a Doctor.
    assert!(!commanders_may_pair(&graham, &graham.clone()));
}

/// A 100-card list led by the pair validates, and the combined identity is
/// both commanders' (CR 702.124c): Graham is mono-green, the Third Doctor is
/// GU, so a blue card in the 99 is legal.
#[test]
fn cr_702_124c_doctor_pair_shares_a_combined_identity() {
    use crabomination::format::{validate_commander_deck, Deck};
    let mut main: Vec<crabomination::card::CardDefinition> = vec![catalog::island()];
    while main.len() < 97 {
        main.push(catalog::forest());
    }
    main.push(catalog::brainstorm());
    let deck = Deck {
        commanders: vec![catalog::graham_obrien(), catalog::the_third_doctor()],
        main,
        ..Default::default()
    };
    assert!(
        validate_commander_deck(&deck).is_ok(),
        "a blue card is inside the pair's combined GU identity: {:?}",
        validate_commander_deck(&deck),
    );
}

/// The two cards themselves: the Third Doctor grows with the noncreature
/// tokens his own ETB makes, and Graham's Paradox trigger reads the cast's
/// zone rather than the spell.
#[test]
fn the_doctor_pair_play_their_printed_abilities() {
    let pt = |g: &GameState, id| {
        let v = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
        (v.power, v.toughness)
    };
    let mut g = two_player_game();
    let doc = g.add_card_to_battlefield(0, catalog::the_third_doctor());
    assert_eq!(pt(&g, doc), (2, 2), "printed 2/2 with no tokens out");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, doc), (2, 2), "a nontoken creature is not a noncreature token");
    g.add_token_to_battlefield(0, &crabomination::card::TokenDefinition {
        name: "Bear".into(),
        power: 2,
        toughness: 2,
        card_types: vec![crabomination::card::CardType::Creature],
        ..Default::default()
    });
    assert_eq!(pt(&g, doc), (2, 2), "a creature token is not a noncreature token");
    g.add_token_to_battlefield(0, &crabomination_base::tokens::clue_token());
    assert_eq!(pt(&g, doc), (3, 3), "a Clue token is one");
    g.add_token_to_battlefield(1, &crabomination_base::tokens::treasure_token());
    assert_eq!(pt(&g, doc), (3, 3), "…that *you* control");

    // Graham's Paradox trigger reads the cast's *zone*, not the spell. The
    // command zone is the Commander-native "anywhere other than your hand",
    // so his own co-commander's cast is a Paradox; a cast from hand is not.
    let foods = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Food").count()
    };
    let mut g = game_with_format(Format::Commander, 2);
    g.add_card_to_battlefield(0, catalog::graham_obrien());
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from hand");
    drain_stack(&mut g);
    assert_eq!(foods(&g), 0, "a cast from hand is not a Paradox");

    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast from the command zone");
    drain_stack(&mut g);
    assert_eq!(foods(&g), 1, "the command zone is not your hand");
}

/// The two cards themselves: Elmar's second-spell trigger untaps and
/// investigates, Sophina's attack trigger counts the *nontoken* attackers.
#[test]
fn friends_forever_pair_play_their_printed_triggers() {
    use crabomination::mana::Color;
    let clues = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue").count()
    };

    // Elmar: the first spell does nothing, the second untaps and investigates.
    let mut g = two_player_game();
    let elmar = g.move_card_to_battlefield_for_test(0, catalog::elmar_ulvenwald_informant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    for _ in 0..2 {
        let spell = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
    }
    assert_eq!(clues(&g), 1, "one Clue, off the second spell only");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "…and the untap happened");
    assert!(g.battlefield_find(elmar).is_some());

    // Sophina: two nontoken attackers and a token one make two Clues.
    let mut g = two_player_game();
    let sophina = g.move_card_to_battlefield_for_test(0, catalog::sophina_spearsage_deserter());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [sophina, ally] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: sophina, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(clues(&g), 2, "one Clue per nontoken attacking creature");
}

/// CR 113.6b — the *static* half of eminence. The Ur-Dragon's cost reduction
/// functions from the command zone, so a Dragon spell in hand is {1} cheaper
/// before he has ever been cast; and "other" means he does not discount
/// himself.
#[test]
fn cr_113_6b_eminence_static_functions_from_the_command_zone() {
    use crabomination::mana::Color;
    // Ryusei, the Falling Star is {5}{R}: six mana, or five with the discount.
    let cast_ryusei_with = |mana: u32, seat_ur: bool| {
        let mut g = two_player_game();
        if seat_ur {
            g.seat_commanders(0, vec![catalog::the_ur_dragon()]);
        }
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let ryusei = g.add_card_to_hand(0, catalog::ryusei_the_falling_star());
        g.players[0].mana_pool.add(Color::Red, mana);
        g.perform_action(GameAction::CastSpell {
            card_id: ryusei,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok()
    };
    assert!(!cast_ryusei_with(5, false), "{{5}}{{R}} is six mana with no Ur-Dragon");
    assert!(cast_ryusei_with(5, true), "…and five with him in the command zone");
}

/// Says yes to a "you may" and defers every other ask to the `AutoDecider` —
/// what a scripted queue can't do, since it answers in ask order and the
/// engine's ask order is not the test's business.
struct SayYesToOptional;
impl crabomination::decision::Decider for SayYesToOptional {
    fn decide(
        &mut self,
        decision: &crabomination::decision::Decision,
    ) -> DecisionAnswer {
        match decision {
            crabomination::decision::Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(true),
            other => crabomination::decision::AutoDecider.decide(other),
        }
    }
}

/// The Ur-Dragon's battlefield half: "whenever one or more Dragons you control
/// attack, draw that many cards, then you may put a permanent card from your
/// hand onto the battlefield" — one trigger for the batch (CR 603.2c), and the
/// count is the attacking Dragons, not every Dragon.
#[test]
fn the_ur_dragon_draws_one_card_per_attacking_dragon() {
    let mut g = two_player_game();
    let ur = g.move_card_to_battlefield_for_test(0, catalog::the_ur_dragon());
    let ryusei = g.move_card_to_battlefield_for_test(0, catalog::ryusei_the_falling_star());
    // A third Dragon that stays home, so "that many" can't mean "all of them".
    g.move_card_to_battlefield_for_test(0, catalog::keiga_the_tide_star());
    for id in [ur, ryusei] {
        g.clear_sickness(id);
    }
    g.players[0].library.clear();
    for _ in 0..5 {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    // The "you may put a permanent" is the controller's ask, and the
    // `AutoDecider` declines every `OptionalTrigger`. A one-question decider
    // says yes to that and leaves every other ask (which permanent) alone —
    // a scripted queue would answer them in the wrong order.
    g.decider = Box::new(SayYesToOptional);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: ur, target: AttackTarget::Player(1) },
        Attack { attacker: ryusei, target: AttackTarget::Player(1) },
    ]))
    .expect("two Dragons attack");
    drain_stack(&mut g);
        assert_eq!(
        g.players[0].hand.len(),
        hand + 1,
        "two cards drawn for the two attacking Dragons, one of them put into play",
    );
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0).count(),
        4,
        "three Dragons plus the free permanent",
    );
}

/// CR 113.6b — "other Dragon spells": casting The Ur-Dragon himself out of the
/// command zone gets no discount from his own eminence static.
#[test]
fn the_ur_dragon_does_not_discount_himself() {
    use crabomination::mana::Color;
    let cast_with = |mana: u32| {
        let mut g = two_player_game();
        let ur = g.seat_commanders(0, vec![catalog::the_ur_dragon()])[0];
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 1);
        }
        g.players[0].mana_pool.add(Color::Red, mana);
        g.perform_action(GameAction::CastFromCommandZone {
            card_id: ur,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
            alternative: false,
            pitch_card: None,
        })
        .is_ok()
    };
    assert!(!cast_with(3), "{{4}}{{W}}{{U}}{{B}}{{R}}{{G}} is not discounted to eight");
    assert!(cast_with(4), "…and nine mana pays it");
}

// ── CR 207.2c — join forces ───────────────────────────────────────────────

/// Answers every `ChooseAmount` with `n`, everything else through the
/// `AutoDecider` — the join-forces ask is a number, and the `AutoDecider`
/// answers 0 to all of them.
struct PayAmount(u32);
impl crabomination::decision::Decider for PayAmount {
    fn decide(
        &mut self,
        decision: &crabomination::decision::Decision,
    ) -> DecisionAnswer {
        match decision {
            crabomination::decision::Decision::ChooseAmount { max, .. } => {
                DecisionAnswer::Amount(self.0.min(*max))
            }
            other => crabomination::decision::AutoDecider.decide(other),
        }
    }
}

/// CR 207.2c — "Join forces — Starting with you, each player may pay any
/// amount of mana. Each player draws X cards, where X is the *total* amount
/// paid this way." Every seat contributes, the mana actually leaves their
/// pools, and the body runs once off the sum.
#[test]
fn cr_207_2c_join_forces_sums_every_seat_and_draws_that_many() {
    use crabomination::mana::Color;
    let mut g = multi_player_game(3);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let aglow = g.add_card_to_hand(0, catalog::minds_aglow());
    for seat in 0..3 {
        g.players[seat].mana_pool.add(Color::Blue, 3);
        g.players[seat].library.clear();
        for _ in 0..10 {
            let id = g.next_id();
            g.players[seat].add_to_library_top(id, catalog::grizzly_bears());
        }
    }
    let hands: Vec<usize> = g.players.iter().map(|p| p.hand.len()).collect();

    // Each seat pledges 2; one {U} of seat 0's three pays for the spell.
    g.decider = Box::new(PayAmount(2));
    g.perform_action(GameAction::CastSpell {
        card_id: aglow,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Minds Aglow for {U}");
    drain_stack(&mut g);

    // Seat 0 paid the {U} and the spell left hand, so its net is 6 - 1.
    assert_eq!(g.players[0].hand.len(), hands[0] - 1 + 6, "the caster draws six too");
    for (seat, before) in hands.iter().enumerate().skip(1) {
        assert_eq!(g.players[seat].hand.len(), before + 6, "2 + 2 + 2 = X");
    }
    // Every pledge actually left a pool: seat 0 spent one on the spell and
    // pledged its last two; the others pledged two of three.
    assert_eq!(g.players[0].mana_pool.total(), 0);
    for seat in 1..3 {
        assert_eq!(g.players[seat].mana_pool.total(), 1, "two of the three were paid");
    }
}

/// CR 207.2c / 101.4 — Collective Voyage: every seat searches its *own*
/// library for X basics and puts them onto the battlefield under its own
/// control. A bare `SearchUpToN { who: EachPlayer }` searched one library
/// (seat 0's) and handed those lands to the caster; a debug pod caught it.
#[test]
fn cr_207_2c_collective_voyage_ramps_every_seat_under_its_own_control() {
    use crabomination::mana::Color;
    let mut g = multi_player_game(3);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let voyage = g.add_card_to_hand(0, catalog::collective_voyage());
    for seat in 0..3 {
        g.players[seat].mana_pool.add(Color::Green, 2);
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g.decider = Box::new(PayAmount(1));
    g.perform_action(GameAction::CastSpell {
        card_id: voyage,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Collective Voyage");
    drain_stack(&mut g);
    for seat in 0..3 {
        let lands = g
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.owner == seat && c.definition.name == "Forest")
            .count();
        assert_eq!(lands, 3, "seat {seat} fetched its own three (1 + 1 + 1 = X)");
    }
}

/// CR 207.2c — the `AutoDecider` answers 0 to a `ChooseAmount`, so a bot pod
/// resolves join forces as a no-op rather than stalling on the ask. The point
/// of the test is that it *resolves*: nothing is left pending.
#[test]
fn join_forces_resolves_with_the_auto_decider_declining_every_seat() {
    use crabomination::mana::Color;
    let mut g = multi_player_game(4);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let aglow = g.add_card_to_hand(0, catalog::minds_aglow());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hands: Vec<usize> = g.players.iter().map(|p| p.hand.len()).collect();

    g.perform_action(GameAction::CastSpell {
        card_id: aglow,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Minds Aglow");
    drain_stack(&mut g);
    assert!(g.pending_decision.is_none(), "no seat is left holding an ask");
    assert!(g.stack.is_empty(), "the spell resolved");
    assert_eq!(g.players[0].hand.len(), hands[0] - 1, "X was 0, so nobody drew");
    for (seat, before) in hands.iter().enumerate().skip(1) {
        assert_eq!(g.players[seat].hand.len(), *before);
    }
}

// ── CR 702.49d — commander ninjutsu ───────────────────────────────────────

/// A seat-0 attacker the defender left unblocked, with the game parked in the
/// declare-blockers step — the ninjutsu window.
fn unblocked_attack(g: &mut GameState) -> crabomination::card::CardId {
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    bear
}

/// CR 702.49d — commander ninjutsu reveals the card "from your hand or from
/// the command zone". Yuriko comes off the command zone tapped and attacking,
/// and CR 903.8's tax does not apply: the tax is on *casting*, and this puts
/// her onto the battlefield without casting her.
#[test]
fn cr_702_49d_commander_ninjutsu_comes_off_the_command_zone() {
    let mut g = two_player_game();
    let yuriko = g.seat_commanders(0, vec![catalog::yuriko_the_tigers_shadow()])[0];
    let bear = unblocked_attack(&mut g);
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::swamp());

    g.perform_action(GameAction::Ninjutsu { ninja: yuriko, returning: bear })
        .expect("commander ninjutsu off the command zone");
    assert!(g.players[0].command.iter().all(|c| c.id != yuriko), "she left the command zone");
    let on_board = g.battlefield.iter().find(|c| c.id == yuriko).expect("Yuriko is in play");
    assert!(on_board.tapped, "CR 702.49d — she enters tapped");
    assert!(
        g.attacking.iter().any(|a| a.attacker == yuriko),
        "…and attacking the same defender",
    );
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the attacker went back to hand");
    assert_eq!(
        g.commander_cast_count.get(&yuriko).copied(),
        None,
        "CR 903.8 taxes a cast; ninjutsu is not one",
    );
}

/// CR 702.49a vs 702.49d — plain ninjutsu functions only from the hand, so a
/// commander carrying the ordinary keyword can't be revealed from the command
/// zone. The variant keyword is the whole difference.
#[test]
fn cr_702_49a_plain_ninjutsu_does_not_reach_the_command_zone() {
    use crabomination::card::{CardDefinition, CardType, Keyword, Supertype};
    let sneak = CardDefinition {
        name: "Plain Ninjutsu Commander",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ninjutsu(crabomination::mana::ManaCost::default())],
        ..Default::default()
    };
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![sneak])[0];
    let bear = unblocked_attack(&mut g);
    assert!(
        g.perform_action(GameAction::Ninjutsu { ninja: cmd, returning: bear }).is_err(),
        "the ordinary keyword functions only in the hand",
    );
    assert!(g.players[0].command.iter().any(|c| c.id == cmd), "and nothing moved");
}

/// Yuriko's payoff: a Ninja connecting reveals the top card, puts it in hand,
/// and drains *each* opponent for its mana value — the multiplayer half, so
/// the pod reads it once per seat.
#[test]
fn yuriko_drains_each_opponent_for_the_revealed_mana_value() {
    let mut g = multi_player_game(3);
    let yuriko = g.move_card_to_battlefield_for_test(0, catalog::yuriko_the_tigers_shadow());
    g.clear_sickness(yuriko);
    // Grizzly Bears is {1}{G} — mana value 2.
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::grizzly_bears());
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: yuriko,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack seat 1");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == top), "the revealed card went to hand");
    assert_eq!(g.players[0].life, lives[0], "the controller loses nothing");
    assert_eq!(g.players[1].life, lives[1] - 1 - 2, "one combat damage plus the drain");
    assert_eq!(g.players[2].life, lives[2] - 2, "the untouched opponent drains too");
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

/// CR 113.6b — Edgar Markov's eminence trigger mints a Vampire while he sits
/// in the command zone, and only for a Vampire spell.
#[test]
fn edgar_markov_eminence_mints_a_vampire_from_the_command_zone() {
    use crabomination::card::CreatureType;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.seat_commanders(0, vec![catalog::edgar_markov()]);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    // A non-Vampire spell is not the trigger's business.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the bears");
    drain_stack(&mut g);
    let vampires = |g: &GameState| {
        g.battlefield
            .iter()
            .filter(|c| {
                c.controller == 0
                    && c.definition.subtypes.creature_types.contains(&CreatureType::Vampire)
            })
            .count()
    };
    assert_eq!(vampires(&g), 0, "a Bear is not a Vampire spell");

    let duelist = g.add_card_to_hand(0, catalog::dusk_legion_duelist());
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: duelist,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Vampire");
    drain_stack(&mut g);
    assert_eq!(
        vampires(&g),
        2,
        "the Vampire spell itself plus the eminence token",
    );
    assert!(
        g.battlefield.iter().any(|c| c.is_token && c.definition.power == 1),
        "the token is the 1/1",
    );
}

/// CR 113.6b — Oloro's third ability names the command zone alone, so the
/// upkeep gains 2 from the command zone and 2 (not 4) from the battlefield.
#[test]
fn oloro_gains_two_from_either_zone_but_never_both() {
    let mut g = two_player_game();
    let cmd = g.seat_commanders(0, vec![catalog::oloro_ageless_ascetic()])[0];
    g.active_player_idx = 0;

    let before = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life - before, 2, "the command-zone-only trigger");

    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let mut card = g.players[0].command.remove(pos);
    card.controller = 0;
    g.battlefield.push(card);
    let before = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].life - before,
        2,
        "on the battlefield only the printed upkeep ability fires",
    );
}

/// CR 113.6b — Arahbo's eminence pump reaches a Cat from the command zone.
/// "Another target Cat" excludes Arahbo himself, which matters only once he
/// is on the battlefield; from the command zone he is no legal target anyway.
#[test]
fn arahbo_eminence_pumps_a_cat_from_the_command_zone() {
    let mut g = two_player_game();
    g.seat_commanders(0, vec![catalog::arahbo_roar_of_the_world()]);
    g.active_player_idx = 0;
    let cat = g.add_card_to_battlefield(0, catalog::malamet_brawler());

    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(cat).expect("the Cat is on the battlefield");
    assert_eq!((cp.power, cp.toughness), (5, 5), "a 2/2 Cat gets +3/+3");
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

/// Put `cmd` from its owner's command zone onto the battlefield, the way the
/// 903.9a test does — the seating is what the tally is keyed on, not the zone.
fn command_zone_to_battlefield(g: &mut GameState, seat: usize, cmd: crabomination::card::CardId) {
    let pos = g.players[seat].command.iter().position(|c| c.id == cmd).expect("in command zone");
    let card = g.players[seat].command.remove(pos);
    g.battlefield.push(card);
    g.clear_sickness(cmd);
}

/// One seeded swing of `attacker` into `victim`, damage resolved.
fn swing(g: &mut GameState, attacker: crabomination::card::CardId, victim: usize) {
    g.priority.player_with_priority = g.active_player_idx;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(victim),
    }]))
    .expect("declare the attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(g);
    // CR 511.3 — leave the end of combat step so the attacker is removed from
    // combat before the next swing.
    g.step = TurnStep::EndCombat;
    let _ = g.advance_step(Vec::new());
    drain_stack(g);
}

/// CR 903.10a — the 21 is "by the same commander over the course of the
/// game", so the tally is a property of the *card* and survives it leaving
/// the battlefield. A commander that dies, goes to the command zone under
/// 903.9a and comes back keeps every point it has dealt: the key is
/// `(victim, CardId)` and a command-zone round trip does not re-issue the id.
#[test]
fn cr_903_10a_commander_damage_survives_a_zone_change() {
    let mut g = multi_player_game(3);
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    g.active_player_idx = 0;
    command_zone_to_battlefield(&mut g, 0, cmd);
    swing(&mut g, cmd, 1);
    assert_eq!(g.commander_damage.get(&(1, cmd)).copied(), Some(1), "one swing, one point");

    // Round trip: battlefield → graveyard → command zone (CR 903.9a's SBA).
    let mut evs = vec![];
    g.destroy_permanent(cmd, false, &mut evs);
    g.check_state_based_actions();
    assert!(g.players[0].command.iter().any(|c| c.id == cmd), "back in the command zone");
    command_zone_to_battlefield(&mut g, 0, cmd);
    swing(&mut g, cmd, 1);

    assert_eq!(
        g.commander_damage.get(&(1, cmd)).copied(),
        Some(2),
        "the tally is cumulative across the zone change, not restarted",
    );
}

/// CR 903.10a — and it survives a *control* change. "The same commander" is
/// the card its owner designated (CR 903.3, which control does not move), so
/// a stolen commander's combat damage still lands on the same
/// `(victim, commander)` row — the thief does not get a fresh 21.
#[test]
fn cr_903_10a_commander_damage_survives_a_control_change() {
    let mut g = multi_player_game(3);
    let cmd = g.seat_commanders(0, vec![test_commander()])[0];
    g.active_player_idx = 0;
    command_zone_to_battlefield(&mut g, 0, cmd);
    swing(&mut g, cmd, 1);
    assert_eq!(g.commander_damage.get(&(1, cmd)).copied(), Some(1));

    // Seat 2 takes it and swings at the same victim on their own turn.
    g.battlefield.iter_mut().find(|c| c.id == cmd).expect("on the battlefield").controller = 2;
    assert!(g.is_commander(cmd), "designation follows the card, not control (CR 903.3)");
    g.active_player_idx = 2;
    g.clear_sickness(cmd);
    g.battlefield.iter_mut().find(|c| c.id == cmd).unwrap().tapped = false;
    swing(&mut g, cmd, 1);

    assert_eq!(
        g.commander_damage.get(&(1, cmd)).copied(),
        Some(2),
        "the same commander, so the same row — stealing it does not reset the clock",
    );
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

/// CR 702.116a + 800.4a — a seat that has left the game is no longer an
/// opponent, so myriad mints no copy attacking it (it used to walk every seat
/// index, eliminated or not).
#[test]
fn cr_702_116a_myriad_skips_a_seat_that_left_the_game() {
    use crabomination::card::{CardDefinition, CardType};
    let mut g = multi_player_game(4);
    let m = g.add_card_to_battlefield(0, CardDefinition {
        name: "Myriad Marauder",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        triggered_abilities: vec![crabomination::effect::shortcut::myriad()],
        ..Default::default()
    });
    g.clear_sickness(m);
    g.concede(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: m, target: AttackTarget::Player(1),
    }])).expect("attack seat 1");
    drain_stack(&mut g);
    let copies: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Myriad Marauder").map(|c| c.id).collect();
    assert_eq!(copies.len(), 1, "one copy, for seat 2 only");
    assert!(g.attacking().iter().any(|a| a.attacker == copies[0] && a.target == AttackTarget::Player(2)));
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
/// planeswalker commander as `IllegalCommander`.
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
        cmd.iter().any(|e| matches!(e, CommanderDeckError::IllegalCommander { .. })),
        "Sol Ring prints no such line: {cmd:?}",
    );
}

// ── CR 903.3 — Vehicle and Spacecraft commanders ──────────────────────────

/// CR 903.3 — "That card must be either (a) a creature card, (b) a Vehicle
/// card, or (c) a Spacecraft card with one or more power/toughness boxes."
/// (b) and (c) were rejected as "not a legendary creature"; (c)'s qualifier is
/// the whole distinction between The Seriema (5/5 at its 7+ band) and The
/// Eternity Elevator (a station card whose bands only add mana).
#[test]
fn cr_903_3_a_vehicle_or_a_spacecraft_with_a_pt_box_can_be_a_commander() {
    use crabomination::format::is_legal_commander;

    for (def, want, why) in [
        (catalog::shorikai_genesis_engine(), true, "legendary Vehicle"),
        (catalog::parhelion_ii(), true, "legendary Vehicle"),
        (catalog::the_seriema(), true, "legendary Spacecraft with a 5/5 box"),
        (catalog::the_eternity_elevator(), false, "legendary Spacecraft, no P/T box"),
        (catalog::sol_ring(), false, "not even legendary"),
        (catalog::sigarda_host_of_herons(), true, "the ordinary case"),
    ] {
        assert_eq!(is_legal_commander(&def), want, "{} — {why}", def.name);
    }
}

/// CR 903.3(b) end to end: a legendary Vehicle leads a legal 100-card deck and
/// starts in the command zone like any other commander. Shorikai, Genesis
/// Engine is {2}{W}{U}, so the 99 is on a UW identity.
#[test]
fn cr_903_3b_a_vehicle_leads_a_legal_deck_and_is_seated() {
    use crabomination::format::{Deck, validate_commander_deck};

    let deck = Deck {
        main: std::iter::repeat_with(catalog::island)
            .take(50)
            .chain(std::iter::repeat_with(catalog::plains).take(49))
            .collect(),
        commanders: vec![catalog::shorikai_genesis_engine()],
        ..Default::default()
    };
    validate_commander_deck(&deck).expect("a legendary Vehicle is a legal commander");

    let mut g = game_with_format(Format::Commander, 4);
    let cmd = g.seat_commanders(0, vec![catalog::shorikai_genesis_engine()])[0];
    assert_eq!(g.players[0].command.len(), 1);
    assert!(g.is_commander(cmd), "and the designation is on the card, not on its types");
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

// ── CR 118.9 / 903 — "if you control a commander, cast it for free" ────────

/// The Commander free-spell cycle (Fierce Guardianship, Deflecting Swat,
/// Deadly Rollick, Flawless Maneuver) prints "If you control a commander, you
/// may cast this spell without paying its mana cost". All four shipped without
/// it — three of the four doc comments even said so — so the defining line of
/// the cycle did nothing.
///
/// It is an alternative cost of nothing gated on `ControlsOwnCommander`, and
/// the gate is the whole point: with no commander on the battlefield the free
/// cast must be refused, which is also why adding it cannot move a duel.
#[test]
fn cr_118_9_the_free_spell_cycle_needs_a_commander_on_the_battlefield() {
    use crabomination::game::types::Target;

    let cycle: [fn() -> crabomination::card::CardDefinition; 5] = [
        catalog::fierce_guardianship,
        catalog::deflecting_swat,
        catalog::deadly_rollick,
        catalog::flawless_maneuver,
        catalog::obscuring_haze,
    ];
    for make in cycle {
        let def = make();
        let name = def.name;
        let alt = def
            .alternative_cost
            .as_ref()
            .unwrap_or_else(|| panic!("{name} prints an alternative cost"));
        assert!(alt.mana_cost.symbols.is_empty(), "{name}: the alt cost is nothing");
        assert!(alt.condition.is_some(), "{name}: and it is gated");
    }

    // Deadly Rollick is the one of the four whose target is an ordinary
    // creature, so it is the one that can be cast end to end here.
    let mut g = game_with_format(Format::Commander, 4);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rollick = g.add_card_to_hand(0, catalog::deadly_rollick());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let free_cast = |target| GameAction::CastSpellAlternative {
        card_id: rollick,
        pitch_card: None,
        target: Some(target),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };

    // No commander on the battlefield — the seat has one seated in the command
    // zone, which is not the same thing (CR 903.3d).
    let cmd = g.seat_commanders(0, vec![catalog::sigarda_host_of_herons()])[0];
    assert_eq!(g.players[0].mana_pool.total(), 0);
    assert!(
        g.perform_action(free_cast(Target::Permanent(victim))).is_err(),
        "a commander in the command zone is not a commander you control",
    );

    // Now it is on the battlefield and the free cast is legal.
    g.players[0].command.clear();
    let on_bf = g.add_card_to_battlefield(0, catalog::sigarda_host_of_herons());
    g.players[0].commanders = vec![on_bf];
    let _ = cmd;
    g.perform_action(free_cast(Target::Permanent(victim))).expect("free with a commander out");
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve");
    }
    assert!(g.battlefield_find(victim).is_none(), "the Bears are exiled");
    assert_eq!(g.players[0].mana_pool.total(), 0, "and nothing was paid");
}

/// Obscuring Haze, the cycle's green member and the last to ship: free with a
/// commander out, and the damage it prevents is **all** damage from the
/// opponents' creatures, not the combat half. A pod's pinger is what it is
/// cast against — here Judith, whose death trigger is the damage in question.
#[test]
fn obscuring_haze_stops_an_opponents_noncombat_damage_for_the_turn() {
    use crabomination::game::effects::EntityRef;
    use crabomination::game::types::Target;

    let mut g = game_with_format(Format::Commander, 4);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pinger = g.add_card_to_battlefield(1, catalog::judith_the_scourge_diva());
    let haze = g.add_card_to_hand(0, catalog::obscuring_haze());

    let free_cast = GameAction::CastSpellAlternative {
        card_id: haze,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    // CR 903.3d — a commander in the command zone is not one you control.
    g.seat_commanders(0, vec![catalog::sigarda_host_of_herons()]);
    assert!(g.perform_action(free_cast.clone()).is_err());

    g.players[0].command.clear();
    let on_bf = g.add_card_to_battlefield(0, catalog::sigarda_host_of_herons());
    g.players[0].commanders = vec![on_bf];
    g.perform_action(free_cast).expect("free with a commander out");
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve");
    }
    assert_eq!(g.players[0].mana_pool.total(), 0, "nothing was paid");

    // The ping the combat-only fog would have let through.
    let mut evs = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(mine), 1, Some(pinger), &mut evs);
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "CR 615 — all damage, not combat");
    let before = g.players[0].life;
    g.deal_damage_to_from(EntityRef::Player(0), 1, Some(pinger), &mut evs);
    assert_eq!(g.players[0].life, before, "and to the caster's face as well");
    // A third seat's creature is an opponent's too.
    let theirs = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    assert!(g.combat_damage_prevented_from(theirs), "every opponent, not just the one");
    // The caster's own is untouched.
    assert!(!g.combat_damage_prevented_from(mine));
    let _ = Target::Permanent(mine);
}

// ── CR 603.2c — "whenever one or more … deal combat damage to a player" ────

/// A 0/1 whose whole text is the batched wording: "Whenever one or more
/// creatures you control deal combat damage to a player, draw a card."
fn batch_watcher() -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, TriggeredAbility};
    use crabomination::effect::{EventKind, EventScope, EventSpec};
    CardDefinition {
        name: "Test Batch Watcher",
        card_types: vec![CardType::Creature],
        power: 0,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .once_per_batch(),
            effect: crabomination::effect::shortcut::draw(1),
        }],
        ..Default::default()
    }
}

/// CR 603.2c + CR 510.4 — all combat damage in a sub-step is dealt at once,
/// but **each damaged player is its own event**: three attackers spread over
/// two defending seats fire the ability twice, not once (the whole step
/// collapsed to one) and not three times (one per dealer).
///
/// The per-attacker walk is the engine's, not the rules': the dedupe set that
/// collapses it was keyed by `(listener, ability)` alone, so a pod's alpha
/// strike across two seats drew one card.
#[test]
fn cr_603_2c_a_batched_combat_damage_trigger_fires_once_per_damaged_seat() {
    let mut g = multi_player_game(3);
    let watcher = g.add_card_to_battlefield(0, batch_watcher());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [watcher, a, b, c] {
        g.clear_sickness(id);
    }
    for _ in 0..8 {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    }
    let hand_before = g.players[0].hand.len();

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
        Attack { attacker: c, target: AttackTarget::Player(2) },
    ]))
    .expect("seat 0 attacks two seats");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);

    assert_eq!(
        g.players[0].hand.len() - hand_before,
        2,
        "one fire per damaged seat: two seats took damage, three creatures dealt it",
    );
}

/// The duel control, and the reason the two-player golden traces cannot move:
/// with one defending player there is one subject, so the key gains a field
/// whose value never varies and the collapse is the same collapse.
#[test]
fn cr_603_2c_a_batched_trigger_still_fires_once_in_a_duel() {
    let mut g = two_player_game();
    let watcher = g.add_card_to_battlefield(0, batch_watcher());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [watcher, a, b] {
        g.clear_sickness(id);
    }
    for _ in 0..8 {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    }
    let hand_before = g.players[0].hand.len();

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len() - hand_before, 1, "two dealers, one event, one draw");
}

/// A 1/1 Pirate with no text of its own — the batch's other members.
fn plain_pirate(name: &'static str) -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, CreatureType, Subtypes};
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pirate], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// Malcolm, Keen-Eyed Navigator — "Whenever one or more Pirates you control
/// deal damage to your opponents, you create a Treasure token **for each
/// opponent dealt damage**." The count is the number of damaged *opponents*,
/// which is exactly what CR 603.2c's per-subject batch gives. Before the batch
/// flag it was one Treasure per Pirate — wrong at two seats as well as in a
/// pod, and one of the seven `each_opponent` residuals.
#[test]
fn cr_603_2c_malcolm_makes_one_treasure_per_damaged_opponent() {
    let treasures = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count()
    };

    let mut g = multi_player_game(3);
    let malcolm = g.add_card_to_battlefield(0, catalog::malcolm_keen_eyed_navigator());
    let mate = g.add_card_to_battlefield(0, plain_pirate("Test Pirate A"));
    let landlubber = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [malcolm, mate, landlubber] {
        g.clear_sickness(id);
    }
    assert_eq!(treasures(&g), 0);

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: malcolm, target: AttackTarget::Player(1) },
        Attack { attacker: mate, target: AttackTarget::Player(1) },
        Attack { attacker: landlubber, target: AttackTarget::Player(2) },
    ]))
    .expect("two Pirates into seat 1, a Bear into seat 2");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);

    assert_eq!(
        treasures(&g),
        1,
        "two Pirates hit one opponent — one Treasure; the Bear's seat is not an \
         opponent *dealt damage by a Pirate* and adds none",
    );
}

/// The other half of the same count: a second damaged opponent is a second
/// event (CR 603.2c), so the same two Pirates split across two seats make two
/// Treasures. This is the assert the duel cannot make.
#[test]
fn cr_603_2c_malcolm_counts_each_damaged_opponent_separately() {
    let treasures = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count()
    };

    let mut g = multi_player_game(3);
    let malcolm = g.add_card_to_battlefield(0, catalog::malcolm_keen_eyed_navigator());
    let mate = g.add_card_to_battlefield(0, plain_pirate("Test Pirate A"));
    for id in [malcolm, mate] {
        g.clear_sickness(id);
    }

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: malcolm, target: AttackTarget::Player(1) },
        Attack { attacker: mate, target: AttackTarget::Player(2) },
    ]))
    .expect("one Pirate at each opponent");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);

    assert_eq!(treasures(&g), 2, "two opponents dealt damage — two Treasures");
}

// ── CR 800.4f/g — an ask whose seat has left the game ──────────────────────

/// CR 800.4g — "If an object requires a player who has left the game to make
/// a choice other than whether to pay a cost, the controller of the object
/// chooses another player to make that choice. If the original choice was to
/// be made by an opponent of the controller of the object, that player
/// chooses another opponent if possible."
///
/// Tribute (CR 702.104) asks *an opponent*, and the arm took the first one
/// `opponents_of` named — which is a seat index, alive or not. Four-seat pods
/// over the target decks posed a tribute question to a seat that had left in
/// 4.4 % of games, and the bot for that dead seat answered it.
#[test]
fn cr_800_4g_a_departed_seats_tribute_choice_moves_to_a_live_opponent() {
    let mut g = multi_player_game(4);
    // Only a `wants_ui` seat suspends, which is what makes the chooser
    // observable: whoever the ask lands on is who the pending decision names.
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive(), "seat 1 — the first opponent of seat 0 — has left");

    let demolok = g.add_card_to_battlefield(0, catalog::nessian_demolok());
    g.fire_self_etb_triggers(demolok, 0);
    while g.pending_decision.is_none() && !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve the tribute trigger");
    }
    let asked = g.pending_decision.as_ref().expect("the tribute ask is posed").acting_player();
    assert_ne!(asked, 1, "not the seat that left");
    assert_ne!(asked, 0, "and not the controller: 800.4g wants another opponent");
    assert!(g.players[asked].is_alive(), "seat {asked} is still in the game");
}

/// The control for the test above: with every seat alive the tribute ask goes
/// where it always did, so the re-seating is 800.4g and not a policy change.
#[test]
fn cr_702_104_tribute_still_asks_the_first_opponent_while_everyone_is_alive() {
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    let demolok = g.add_card_to_battlefield(0, catalog::nessian_demolok());
    g.fire_self_etb_triggers(demolok, 0);
    while g.pending_decision.is_none() && !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve the tribute trigger");
    }
    assert_eq!(
        g.pending_decision.as_ref().expect("the tribute ask is posed").acting_player(),
        1,
    );
}

/// CR 800.4f — "If an object requires a player who has left the game to pay a
/// cost or choose whether to pay a cost, that cost is not paid."
///
/// The rhystic shape (`Effect::UnlessPlayerPays`, Smothering Tithe's tax) with
/// the payer gone: nobody is asked, nothing of theirs is spent, and the
/// rider's unpaid half resolves. The distinction from 800.4g above is the
/// whole point — a cost is dropped where a choice is re-seated.
#[test]
fn cr_800_4f_a_departed_seats_cost_is_not_paid_and_the_rider_resolves() {
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    let tithe = g.add_card_to_battlefield(0, catalog::smothering_tithe());
    assert!(g.battlefield_find(tithe).is_some());
    // Enough mana that a live seat 1 could have paid the {2}.
    g.players[1].mana_pool.add_colorless(5);
    g.players[1].life = 0;
    g.check_state_based_actions();

    g.add_card_to_library(1, catalog::forest());
    let mut events = vec![];
    assert!(g.draw_one(1, &mut events), "the departed seat still draws a card");
    g.dispatch_triggers_for_events(&events);
    while g.pending_decision.is_none() && !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve the tithe trigger");
    }
    assert!(g.pending_decision.is_none(), "nobody is asked (CR 800.4f)");
    assert_eq!(g.players[1].mana_pool.total(), 5, "and nothing of theirs was spent");
}

/// CR 800.4a — "When a player leaves the game, all objects owned by that
/// player leave the game …" — however they came to leave.
///
/// The loss SBA did this for the seats *it* eliminated and skipped every
/// other one: an effect that ends a player's game (Phage the Untouchable's
/// combat trigger, an unpaid Pact, a win-the-game effect eliminating everyone
/// else) set `eliminated` and nothing more, and the sweep's first line is
/// `if eliminated { continue }`. Their permanents stayed on the battlefield
/// for the rest of the game, their stack items stayed on the stack, and an
/// ask addressed to them stayed pending. `Player::left_game` is what says the
/// pass is still owed.
#[test]
fn cr_800_4a_a_seat_an_effect_eliminates_still_leaves_the_game() {
    let mut g = multi_player_game(4);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let stolen = g.add_card_to_battlefield(2, catalog::llanowar_elves());
    g.battlefield_find_mut(stolen).unwrap().controller = 1;

    // What every `Effect::LoseGame`-shaped arm does, and all it did.
    g.players[1].eliminated = true;
    assert!(!g.players[1].left_game, "the leave pass has not run");
    g.check_state_based_actions();

    assert!(g.players[1].left_game, "the sweep owed it and ran it");
    assert!(g.battlefield_find(theirs).is_none(), "their own permanent left with them");
    assert_eq!(
        g.battlefield_find(stolen).unwrap().controller,
        2,
        "and a permanent they controlled but did not own reverts to its owner",
    );
    // Idempotent: a second sweep must not re-run the pass.
    g.check_state_based_actions();
    assert!(g.players[1].left_game);
}

/// CR 102.1 / 800.4a — a player who has left the game is not a player in it,
/// so they are not an opponent.
///
/// `opponents_of` handed back every other seat, alive or not, with a doc line
/// telling its forty-six callers to filter it themselves. None of them did,
/// and `Value::OpponentCount` read the length.
#[test]
fn cr_800_4a_a_seat_that_has_left_is_not_an_opponent() {
    let mut g = multi_player_game(4);
    assert_eq!(g.opponents_of(0), vec![1, 2, 3]);
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert_eq!(g.opponents_of(0), vec![2, 3], "seat 1 is out of the game");
    assert_eq!(g.opponents_of(2), vec![0, 3], "and out of everyone else's list too");
}

/// The same rule where it is visible on a card. Refurbished Familiar is
/// "each opponent discards a card; for each opponent who can't, you draw a
/// card" — the count and the loop are the same question asked twice, and one
/// of them counted the dead. At four seats with one out, two opponents
/// discard and the count said three, so the caster drew a card off a player
/// who was not in the game.
#[test]
fn cr_800_4a_for_each_opponent_does_not_count_the_departed() {
    let mut g = multi_player_game(4);
    for seat in 1..4 {
        g.add_card_to_hand(seat, catalog::forest());
    }
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(g.players[1].hand.is_empty(), "their hand left with them (CR 800.4a)");

    // A library to draw from, or the "you draw a card" half cannot happen
    // whatever the count says and the test stops discriminating.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let familiar = g.add_card_to_battlefield(0, catalog::refurbished_familiar());
    g.fire_self_etb_triggers(familiar, 0);
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve the ETB");
    }
    assert_eq!(g.players[2].hand.len(), 0, "the two live opponents discarded");
    assert_eq!(g.players[3].hand.len(), 0);
    assert_eq!(
        g.players[0].hand.len(),
        0,
        "both opponents could discard, so nothing is drawn — the departed seat \
         is not an opponent who 'can't'",
    );
}

/// CR 800.4c — "If an effect that gives a player still in the game control of
/// an object ends, there is no other effect giving control of that object to
/// another player in the game, and the player who controlled that object by
/// default has left the game, the object is exiled. This is not a state-based
/// action. It happens as soon as the control-changing effect ends."
///
/// Three deep, which is why CR 800.4a's own revert does not cover it: seat 2
/// owns the Bears, seat 1 takes them permanently, seat 0 takes them until end
/// of turn, seat 1 leaves. 800.4a only reverts permanents the departing seat
/// controls *at that moment*, and seat 1 controls nothing — so the Bears sit
/// under seat 0 until the Act of Treason ends, and then the seat they would
/// go back to is not in the game.
///
/// `change_control` refuses the move under CR 800.4b and returns `None`, so
/// without this rule seat 0 keeps a permanent for the rest of the game.
#[test]
fn cr_800_4c_control_reverting_to_a_departed_seat_exiles_the_object() {
    use crabomination::game::types::Target;
    let mut g = multi_player_game(3);
    let bears = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    // Seat 1's permanent steal: no reversion entry, which is the whole point.
    g.battlefield_find_mut(bears).unwrap().controller = 1;

    let treason = g.add_card_to_hand(0, catalog::act_of_treason());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: treason,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("cast Act of Treason");
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve");
    }
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0, "seat 0 has it for the turn");

    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive());
    assert_eq!(
        g.battlefield_find(bears).unwrap().controller,
        0,
        "CR 800.4a leaves it alone — seat 1 controls nothing to revert",
    );

    g.do_cleanup(&mut vec![]);
    assert!(
        g.battlefield_find(bears).is_none(),
        "CR 800.4c — the default controller is gone, so the object is exiled",
    );
    assert!(g.exile.iter().any(|c| c.id == bears), "and exiled is where it went");
}

/// The control for the rule above: with the default controller still in the
/// game the reversion is the ordinary one, so 800.4c changes nothing about
/// any board where nobody has left — which is every duel.
#[test]
fn cr_800_4c_an_ordinary_reversion_is_untouched() {
    use crabomination::game::types::Target;
    let mut g = multi_player_game(3);
    let bears = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().controller = 1;
    let treason = g.add_card_to_hand(0, catalog::act_of_treason());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: treason,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("cast Act of Treason");
    while !g.stack.is_empty() {
        g.resolve_top_of_stack().expect("resolve");
    }
    g.do_cleanup(&mut vec![]);
    assert_eq!(
        g.battlefield_find(bears).unwrap().controller,
        1,
        "back to the seat that held it by default",
    );
}

// ── CR 101.4 — a per-player loop whose body suspends ───────────────────────
//
// "If multiple players would make choices … at the same time, the active
// player makes any choices required, then each other player in turn order
// does the same." Every one of them, not the first: a body that suspends
// (a `wants_ui` seat's discard, sacrifice or exile pick) parks only its
// *own* remaining effect, so the loop around it used to run on and abandon
// the seats it had not reached. Invisible in a duel — "each opponent" is
// one iteration there — and it drops two thirds of a four-seat pod.

/// Resolve the top of the stack to completion, answering every ask the
/// installed decider would answer. `drain_stack` can't: it passes priority,
/// and a pending decision is exactly what blocks that.
pub(crate) fn resolve_answering(g: &mut GameState) {
    g.resolve_top_of_stack().expect("resolve the trigger");
    for _ in 0..64 {
        let Some(pending) = g.pending_decision.as_ref() else { break };
        let decision = pending.decision.clone();
        let answer = g.decider.decide(&decision);
        g.submit_decision(answer).expect("answer the ask");
    }
    assert!(g.pending_decision.is_none(), "every ask was answered");
}

/// Two cards into each opponent's hand, and the hand sizes before the loop.
fn stock_opponent_hands(g: &mut GameState, seats: std::ops::Range<usize>) -> Vec<usize> {
    for seat in seats {
        g.add_card_to_hand(seat, catalog::grizzly_bears());
        g.add_card_to_hand(seat, catalog::lightning_bolt());
    }
    g.players.iter().map(|p| p.hand.len()).collect()
}

/// CR 101.4 — `ForEachOpponent` over a suspending body reaches every
/// opponent. The body is bound to its opponent through `Triggerer`, which is
/// an `EffectContext` field the parked continuation cannot carry, so the
/// spliced tail names each remaining seat inside the effect instead.
#[test]
fn cr_101_4_for_each_opponent_reaches_every_opponent_when_the_body_suspends() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        // Only a `wants_ui` seat suspends, which is what makes the loop's
        // abandonment observable at all.
        p.wants_ui = true;
    }
    let before = stock_opponent_hands(&mut g, 1..4);
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::ForEachOpponent {
            body: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(1),
                random: false,
            }),
        })
        .build(),
    );
    resolve_answering(&mut g);
    for (seat, &had) in before.iter().enumerate().skip(1) {
        assert_eq!(
            g.players[seat].hand.len(),
            had - 1,
            "opponent {seat} discarded one — not just the first opponent",
        );
    }
}

/// CR 101.4 / 701.55 — every chooser makes their own villainous choice even
/// when the option they pick suspends. The tail re-enters the arm per
/// remaining seat, so each still chooses for themselves rather than
/// inheriting the first chooser's pick.
#[test]
fn cr_701_55_every_chooser_still_chooses_when_an_option_suspends() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    let before = stock_opponent_hands(&mut g, 1..4);
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::VillainousChoice {
            who: Selector::Player(PlayerRef::EachOpponent),
            option_a: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
                random: false,
            }),
            // Ten life against one card: the harm estimate picks the discard,
            // so every chooser takes the arm that suspends.
            option_b: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(10),
            }),
        })
        .build(),
    );
    resolve_answering(&mut g);
    for (seat, &had) in before.iter().enumerate().skip(1) {
        assert_eq!(g.players[seat].hand.len(), had - 1, "chooser {seat} discarded");
        assert_eq!(g.players[seat].life, 20, "and nobody was skipped into the life option");
    }
}

/// CR 101.4 — a punisher's "unless" option is a choice each opponent makes.
/// The sacrifice pick suspends for every one of them, and the arm is
/// re-entered per remaining chooser so the affordable option is re-derived
/// against the board as it stands then.
#[test]
fn cr_101_4_every_punisher_chooser_is_asked_when_the_option_suspends() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    for seat in 1..4 {
        // Two creatures apiece: sacrificing one is a choice, so it suspends.
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
    }
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::Punisher {
            chooser: Selector::Player(PlayerRef::EachOpponent),
            options: vec![Effect::Sacrifice {
                who: Selector::Player(PlayerRef::You),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            }],
            otherwise: Box::new(Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            }),
        })
        .build(),
    );
    resolve_answering(&mut g);
    for seat in 1..4 {
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == seat).count(),
            1,
            "chooser {seat} sacrificed one of their two",
        );
        assert_eq!(g.players[seat].life, 20, "the option was taken, so no punisher damage");
    }
}

/// CR 101.4 — a tempting offer's body runs once for the controller, once per
/// acceptor, then once more per acceptor. Those runs are a seat list like any
/// other loop's, and a suspend in one of them no longer eats the rest.
///
/// "Tempting offer" is an **ability word** (CR 207.2c): it has no individual
/// rules entry, so the shape is the printed text and the order is 101.4's.
#[test]
fn cr_101_4_every_tempting_offer_run_happens_when_the_body_suspends() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    let mut before = stock_opponent_hands(&mut g, 1..4);
    // Seat 0 discards four times: its own run plus one per acceptor.
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    before[0] = g.players[0].hand.len();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::TemptingOffer {
            body: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
                random: false,
            }),
        })
        .build(),
    );
    // `AutoDecider` declines every `OptionalTrigger`, which would leave one
    // run and nothing to splice; this says yes to the offer and leaves the
    // discard picks alone.
    g.decider = Box::new(SayYesToOptional);
    resolve_answering(&mut g);
    for (seat, &had) in before.iter().enumerate().skip(1) {
        assert_eq!(g.players[seat].hand.len(), had - 1, "acceptor {seat}'s own run");
    }
    assert_eq!(
        g.players[0].hand.len(),
        before[0] - 4,
        "the controller's run plus one more per acceptor",
    );
}

/// CR 101.4 — Strongarm Tactics: "Each player discards a card. Then each
/// player who didn't discard a creature card this way loses 4 life." Two
/// defects in one arm: the life check read the graveyard *before* a
/// suspended discard had moved the card, so a `wants_ui` seat was punished
/// whatever it pitched, and the seats after it were never asked at all.
#[test]
fn cr_101_4_strongarm_tactics_reaches_every_seat_and_reads_what_they_pitched() {
    use crabomination::effect::Effect;
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    for seat in 0..4 {
        // Homogeneous hands: `AutoDecider` discards the first card, so what
        // each seat pitches is fixed. Seats 0-1 pitch a creature, 2-3 don't.
        let card = |seat: usize| {
            if seat < 2 { catalog::grizzly_bears() } else { catalog::lightning_bolt() }
        };
        g.add_card_to_hand(seat, card(seat));
        g.add_card_to_hand(seat, card(seat));
    }
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::EachPlayerDiscardsElseLosesLife { life: 4 }).build(),
    );
    resolve_answering(&mut g);
    for seat in 0..4 {
        assert_eq!(g.players[seat].hand.len(), 1, "seat {seat} discarded");
    }
    assert_eq!(g.players[0].life, 20, "pitched a creature");
    assert_eq!(g.players[1].life, 20, "pitched a creature");
    assert_eq!(g.players[2].life, 16, "pitched a Bolt");
    assert_eq!(g.players[3].life, 16, "pitched a Bolt");
}

/// CR 101.4 — Possessed Portal's end-step tax is every player's. Both of its
/// asks suspend for a `wants_ui` seat, so the loop used to stop at the first.
#[test]
fn cr_101_4_possessed_portal_taxes_every_seat() {
    use crabomination::effect::Effect;
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    for seat in 0..4 {
        // Two permanents apiece, so the sacrifice is a choice and suspends.
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
    }
    let src = g.add_card_to_battlefield(0, catalog::possessed_portal());
    g.stack.push(TriggerPush::new(src, 0, Effect::EachPlayerSacrificesUnlessDiscards).build());
    resolve_answering(&mut g);
    // `AutoDecider` declines the discard, so every seat takes the sacrifice.
    for seat in 0..4 {
        let kept = g.battlefield.iter().filter(|c| c.controller == seat).count();
        assert_eq!(kept, if seat == 0 { 2 } else { 1 }, "seat {seat} paid the tax");
    }
}

/// CR 101.4 — Tariff asks every player, not just the ones before the first
/// `wants_ui` seat. The asks now all precede the payments, which also fixes
/// the resume: the parked `MayPay` came back under the stack item's context,
/// where `SacrificeSource` no longer named that seat's creature.
#[test]
fn cr_101_4_tariff_asks_every_seat_and_takes_their_biggest() {
    use crabomination::effect::Effect;
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
        // Enough to cover {1}{G}, so CR 118.6 doesn't skip the ask.
        p.mana_pool.add(crabomination::mana::Color::Green, 1);
        p.mana_pool.add_colorless(1);
    }
    for seat in 0..4 {
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.add_card_to_battlefield(seat, catalog::llanowar_elves());
    }
    let src = g.add_card_to_battlefield(0, catalog::tariff());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::EachPlayerSacrificesGreatestManaValueUnlessPays).build(),
    );
    resolve_answering(&mut g);
    // `AutoDecider` declines the payment, so every seat loses its Bears and
    // keeps the Elves.
    for seat in 0..4 {
        let names: Vec<&str> = g
            .battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| c.definition.name)
            .collect();
        assert!(!names.contains(&"Grizzly Bears"), "seat {seat} sacrificed its biggest");
        assert!(names.contains(&"Llanowar Elves"), "seat {seat} kept the smaller one");
    }
}

/// CR 101.4 — `Effect::Repeat` is the same shape without the seats: the
/// repetitions after a suspending body were abandoned. The tail carries the
/// remainder as a constant, so a body that changes what `count` reads
/// doesn't shorten the loop.
#[test]
fn cr_101_4_repeat_finishes_its_repetitions_when_the_body_suspends() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(4);
    g.players[0].wants_ui = true;
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let src = g.add_card_to_battlefield(0, catalog::lightning_bolt());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::Repeat {
            count: Value::Const(3),
            body: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::You),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            }),
        })
        .build(),
    );
    resolve_answering(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count(),
        1,
        "three of the four Bears went, not just the one whose pick suspended",
    );
}

// ── CR 700.2 — a modal run whose mode suspends ─────────────────────────────

/// Answers a `ChooseModes` ask with both modes and leaves every other ask to
/// the `AutoDecider` — a scripted queue would answer them in ask order, and
/// the engine's ask order is not the test's business.
struct ChooseBothModes;
impl crabomination::decision::Decider for ChooseBothModes {
    fn decide(
        &mut self,
        decision: &crabomination::decision::Decision,
    ) -> DecisionAnswer {
        match decision {
            crabomination::decision::Decision::ChooseModes { .. } => {
                DecisionAnswer::Modes(vec![0, 1])
            }
            other => crabomination::decision::AutoDecider.decide(other),
        }
    }
}

/// CR 700.2 — a modal spell's effect is every chosen mode. Escalate has two
/// loops in one arm:
/// one escalate cost per extra mode, then one run per mode. The printed cost
/// is a discard, which asks, so the *first* loop suspended and dropped the
/// costs after it **and** every mode. The second loop's tail also has to pin
/// each remaining mode's target slot inside the effect
/// (`Effect::BindTargetSlot`), because a parked continuation is resumed with
/// the spell's whole target list.
#[test]
fn cr_700_2_escalate_pays_every_cost_and_runs_every_mode_through_a_suspend() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(2);
    g.players[0].wants_ui = true;
    for _ in 0..5 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    let hand = g.players[0].hand.len();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let discard_one = Effect::Discard {
        who: Selector::You,
        amount: Value::Const(1),
        random: false,
    };
    g.stack.push(
        TriggerPush::new(src, 0, Effect::Escalate {
            modes: vec![
                discard_one.clone(),
                Effect::Destroy { what: Selector::Target(0) },
            ],
            cost: Box::new(discard_one),
        })
        .target(Some(crabomination::game::types::Target::Permanent(victim)))
        .build(),
    );
    g.decider = Box::new(ChooseBothModes);
    resolve_answering(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        hand - 2,
        "the escalate cost and mode 0 each took a card",
    );
    assert!(
        g.battlefield_find(victim).is_none(),
        "mode 1 ran too, and it found its own target slot",
    );
}

/// CR 608.2 — `ApplyToTargets` runs its inner effect once per target. An
/// inner that asks suspended and the targets after it were dropped; each
/// remaining one is pinned by its original slot, which the resumed context
/// still holds.
#[test]
fn cr_608_2_apply_to_targets_reaches_every_target_when_the_inner_suspends() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::types::Target;
    let mut g = multi_player_game(2);
    g.players[0].wants_ui = true;
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    let hand = g.players[0].hand.len();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature,
            // The discard is FIRST on purpose: it suspends, so the destroy
            // after it is what the continuation has to carry — with the
            // target still pinned, or it destroys the caster's slot-0 target
            // a second time instead of this one.
            effect: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Destroy { what: Selector::Target(0) },
            ])),
        })
        .target(Some(Target::Permanent(a)))
        .additional_targets(vec![Target::Permanent(b)])
        .build(),
    );
    resolve_answering(&mut g);
    assert!(g.battlefield_find(a).is_none(), "the first target");
    assert!(g.battlefield_find(b).is_none(), "and the one after the suspend");
    assert_eq!(g.players[0].hand.len(), hand - 2, "one discard per target");
}

/// CR 101.4 — the other half of a per-seat loop: whatever a seat's body
/// *parks* has to come back under that seat too. A parked continuation is
/// resumed with the stack item's context, so the rest of seat 1's body ran
/// as the caster — every seat's life loss landed on seat 0.
#[test]
fn cr_101_4_a_parked_tail_comes_back_under_the_seat_that_parked_it() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(3);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    let before = stock_opponent_hands(&mut g, 0..3);
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            // The discard suspends; the life loss after it is what the
            // continuation has to carry, with its seat still bound.
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ])),
        })
        .build(),
    );
    resolve_answering(&mut g);
    for (seat, &had) in before.iter().enumerate() {
        assert_eq!(g.players[seat].hand.len(), had - 1, "seat {seat} discarded");
        assert_eq!(g.players[seat].life, 19, "and seat {seat} paid its own life");
    }
}

/// CR 608.2 — `Effect::ForEach` binds each entity as the body's `Triggerer`,
/// which is an `EffectContext` field the continuation cannot carry. A body
/// that asks therefore dropped the entities after it *and* came back bound
/// to nothing. `Selector::ExactObjects` is what lets the tail name them.
#[test]
fn cr_608_2_for_each_reaches_every_entity_when_the_body_suspends() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(2);
    g.players[0].wants_ui = true;
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    let hand = g.players[0].hand.len();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let src = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            // The discard suspends; the destroy after it is what the
            // continuation has to carry, with its entity still bound.
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Destroy { what: Selector::TriggerSource },
            ])),
        })
        .build(),
    );
    resolve_answering(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0).count(),
        0,
        "all four creatures were reached, and each destroyed its own",
    );
    assert_eq!(g.players[0].hand.len(), hand - 4, "one discard per entity");
}

/// Votes the first ballot for the second choice and every later one for the
/// first, so the schedule's first run belongs to a seat that is NOT the
/// controller — which is where `PlayerRef::CurrentVoter`'s fallback would
/// otherwise cover for a missing binding.
struct FirstVoterDiffers(usize);
impl crabomination::decision::Decider for FirstVoterDiffers {
    fn decide(
        &mut self,
        decision: &crabomination::decision::Decision,
    ) -> DecisionAnswer {
        match decision {
            crabomination::decision::Decision::ChooseOption { .. } => {
                self.0 += 1;
                DecisionAnswer::Amount(u32::from(self.0 == 1))
            }
            other => crabomination::decision::AutoDecider.decide(other),
        }
    }
}

/// CR 101.4 / 701.38 — a council's dilemma runs one effect per vote cast, and
/// every one of them happens even when an earlier one asks. The voter is
/// pinned in `GameState.current_voter`, which no `EffectContext` carries, so
/// both the spliced tail and the suspending run's own remainder name their
/// voter inside the effect (`Effect::BindScratch`). Capital Punishment is the
/// printed shape: at four seats its "death" ballot asks three opponents to
/// sacrifice, and only the first was ever asked.
#[test]
fn cr_701_38_every_vote_resolves_when_an_earlier_votes_effect_suspends() {
    use crabomination::effect::{Effect, Selector, Value, VoteOption, VoteTally};
    let mut g = multi_player_game(4);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    let before = stock_opponent_hands(&mut g, 0..4);
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Both choices do the same thing, so the schedule is four votes long
    // however they split; the split only decides whose vote comes first.
    let body = || {
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::CurrentVoter),
                amount: Value::Const(1),
                random: false,
            },
            // Behind the ask, so it is the *parked* half: without the re-wrap
            // it comes back with `current_voter` restored and pays the
            // controller instead.
            Effect::GainLife {
                who: Selector::Player(PlayerRef::CurrentVoter),
                amount: Value::Const(5),
            },
        ])
    };
    g.stack.push(
        TriggerPush::new(src, 0, Effect::Vote {
            options: vec![VoteOption::new("aye", body()), VoteOption::new("nay", body())],
            tally: VoteTally::PerVote,
        })
        .build(),
    );
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    g.decider = Box::new(FirstVoterDiffers(0));
    resolve_answering(&mut g);
    for (seat, &had) in before.iter().enumerate() {
        assert_eq!(
            g.players[seat].hand.len(),
            had - 1,
            "seat {seat}'s own vote resolved — not just the first voter's",
        );
        assert_eq!(
            g.players[seat].life,
            life[seat] + 5,
            "and its parked half came back naming seat {seat}, not the controller",
        );
    }
}

/// The pair form of the same class: `SeparateIntoPiles` runs `chosen`, then
/// `other`, then drops the piles. A suspend inside `chosen` is `Ok(())`, so
/// `other` ran *before* `chosen` had finished and the piles were cleared out
/// from under `chosen`'s continuation — which then resolved
/// `Selector::SeparatedPile` to nothing.
#[test]
fn a_pile_splits_second_half_waits_for_the_first_to_finish() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = multi_player_game(2);
    g.players[0].wants_ui = true;
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let src = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::SeparateIntoPiles {
            what: Selector::ControlledBy {
                who: PlayerRef::Seat(0),
                filter: R::Creature.and(R::HasCreatureType(
                    crabomination::card::CreatureType::Bear,
                )),
            },
            splitter: PlayerRef::You,
            chooser: PlayerRef::You,
            // The discard suspends, so the destroy after it is what the
            // continuation carries — and it needs the piles still there.
            chosen: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::DestroyNoRegen { what: Selector::SeparatedPile { chosen: true } },
            ])),
            other: Box::new(Effect::DestroyNoRegen {
                what: Selector::SeparatedPile { chosen: false },
            }),
        })
        .build(),
    );
    resolve_answering(&mut g);
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
            .count(),
        0,
        "both piles were destroyed — the chosen one's destroy still found it",
    );
}


/// The ten-card "bond" cycle (Battlebond + Commander Legends): "This land
/// enters tapped unless you have **two or more opponents**." Its whole text is
/// a player count, so a duel and a pod are the two answers, and one table over
/// all ten is the test — they share one body (`sets::cmdr::crowd_land`) and
/// used to carry three.
///
/// CR 800.4a is the third row: an opponent who has left the game is not an
/// opponent, so a four-seat pod worn down to two players taps the next one.
#[test]
fn bond_lands_read_the_live_opponent_count() {
    type Land = fn() -> crabomination::card::CardDefinition;
    const CYCLE: [Land; 10] = [
        catalog::sea_of_clouds,
        catalog::bountiful_promenade,
        catalog::morphic_pool,
        catalog::spire_garden,
        catalog::luxury_suite,
        catalog::training_center,
        catalog::undergrowth_stadium,
        catalog::rejuvenating_springs,
        catalog::spectator_seating,
        catalog::vault_of_champions,
    ];
    let play = |g: &mut GameState, def: crabomination::card::CardDefinition| {
        let id = g.add_card_to_hand(0, def);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].lands_played_this_turn = 0;
        g.perform_action(GameAction::PlayLand(id)).expect("play land");
        id
    };
    for make in CYCLE {
        let name = make().name;
        assert_eq!(make().activated_abilities.len(), 2, "{name} taps for two colors");

        let mut duel = multi_player_game(2);
        let a = play(&mut duel, make());
        assert!(duel.battlefield_find(a).unwrap().tapped, "{name}: one opponent, tapped");

        let mut pod = multi_player_game(3);
        let b = play(&mut pod, make());
        assert!(!pod.battlefield_find(b).unwrap().tapped, "{name}: two opponents, untapped");

        // CR 800.4a — two of the four are out, so the count is one again.
        let mut worn = multi_player_game(4);
        for seat in [1, 2] {
            worn.players[seat].life = 0;
        }
        worn.check_state_based_actions();
        assert_eq!(worn.players.iter().filter(|p| p.is_alive()).count(), 2);
        let c = play(&mut worn, make());
        assert!(
            worn.battlefield_find(c).unwrap().tapped,
            "{name}: a seat that has left is not an opponent",
        );
    }
}

/// CR 701.38a + CR 800.4a — "starting with you, each player votes" counts the
/// players still **in the game**. A seat that has left is not a player, so it
/// casts no ballot.
///
/// The bug this pins: `Effect::Vote` walked `(controller + i) % n`, which is
/// every *seat index*, alive or not. CR 800.4g then re-seated the departed
/// seat's ask onto a live opponent, that opponent answered, and the answer was
/// tallied as the dead seat's vote — so a four-seat pod down to two still
/// decided a will-of-the-council four votes to nothing. Invisible in a duel,
/// where a seat leaving ends the game.
#[test]
fn cr_701_38a_a_departed_seat_casts_no_ballot() {
    use crabomination::effect::{Effect, Selector, Value, VoteOption, VoteTally};
    let mut g = multi_player_game(4);
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[2].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[2].is_alive(), "seat 2 has left the game");

    g.stack.push(
        TriggerPush::new(src, 0, Effect::Vote {
            options: vec![
                VoteOption::new("aye", Effect::Noop),
                VoteOption::new(
                    "nay",
                    Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                ),
            ],
            tally: VoteTally::PerVote,
        })
        .build(),
    );
    resolve_answering(&mut g);

    let voters: Vec<usize> = g.last_vote.iter().map(|(seat, _)| *seat).collect();
    assert_eq!(voters.len(), 3, "three players are left, so three votes: {voters:?}");
    assert!(!voters.contains(&2), "the departed seat voted: {voters:?}");
    // CR 701.38a's own half: the controller votes first, then turn order,
    // and turn order *skips* the seat that is gone.
    assert_eq!(voters, vec![0, 1, 3], "controller first, then the live seats in turn order");
}

/// CR 800.4a again, one card over: Grenzo's Rebuttal's "the player to their
/// left" is the next **player**, not the next seat index. With seat 1 gone,
/// seat 0 strips seat 2 — aiming at the empty board of a departed seat
/// destroyed nothing at all.
#[test]
fn cr_800_4a_the_player_to_your_left_skips_a_departed_seat() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::Effect;
    let mut g = multi_player_game(4);
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive());

    // One creature apiece on the two seats that are left besides the caster.
    let two = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let three = g.add_card_to_battlefield(3, catalog::grizzly_bears());
    g.stack.push(
        TriggerPush::new(src, 0, Effect::EachPlayerDestroysChosenFromLeftNeighbor {
            filters: vec![R::Creature],
        })
        .build(),
    );
    resolve_answering(&mut g);

    assert!(
        g.battlefield_find(two).is_none(),
        "seat 0's left-hand neighbour is seat 2 once seat 1 has left",
    );
    assert!(g.battlefield_find(three).is_none(), "and seat 2's is seat 3");
    assert!(
        g.battlefield_find(src).is_none(),
        "and seat 3's is the caster, so the Bears goes too",
    );
}

// ── CR 506.2 — "attacking YOU" is not "attacking" at more than two seats ────

/// Seat 1 swings two creatures at seat 2 and one at seat 0, with seat 0 also
/// holding a planeswalker under one attacker. Three predicates, three answers,
/// and at two seats all three would agree.
fn three_way_attack() -> (GameState, crabomination::card::CardId) {
    let mut g = multi_player_game(4);
    let pw = g.add_card_to_battlefield(0, catalog::jace_beleren());
    let mut swing = Vec::new();
    for target in [
        AttackTarget::Player(2),
        AttackTarget::Player(2),
        AttackTarget::Player(0),
        AttackTarget::Planeswalker(pw),
    ] {
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(a);
        swing.push(Attack { attacker: a, target });
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(swing).expect("declare across three defenders");
    (g, pw)
}

/// CR 506.2 — a creature attacking a planeswalker is attacking the
/// planeswalker, not its controller. `include_planeswalkers` is the printed
/// difference between Trouble in Pairs ("attacks **you** with two or more
/// creatures") and Mangara, the Diplomat ("attacking **you and/or
/// planeswalkers you control**"), and seat 0 is hit by exactly one of each.
#[test]
fn cr_506_2_attacked_defender_count_separates_the_player_from_their_planeswalkers() {
    use crabomination::card::Predicate;
    let (g, _pw) = three_way_attack();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let at = |defender: usize, at_least: u32, include_planeswalkers: bool| {
        g.evaluate_predicate(
            &Predicate::AttackedDefenderWithCountAtLeast {
                who: PlayerRef::Seat(1),
                defender: PlayerRef::Seat(defender),
                at_least,
                include_planeswalkers,
            },
            &ctx,
        )
    };

    // Seat 0: one attacker on the player, one on their planeswalker.
    assert!(at(0, 1, false), "one creature is attacking seat 0");
    assert!(!at(0, 2, false), "the planeswalker's attacker is not attacking seat 0");
    assert!(at(0, 2, true), "…but it counts once planeswalkers are included");
    assert!(!at(0, 3, true), "and there are only two either way");

    // Seat 2: two on the player, no planeswalker, so the flag changes nothing.
    assert!(at(2, 2, false), "two creatures are attacking seat 2");
    assert!(at(2, 2, true));
    assert!(!at(2, 3, false), "and not three");

    // Seat 3 was not attacked at all.
    assert!(!at(3, 1, false), "seat 3 is untouched");
    assert!(!at(3, 1, true));
}

/// ⚠ And the whole reason the predicate exists: the *undirected*
/// `AttackedWithCountAtLeast` counts seat 1's entire declaration and so reads
/// true for a defender who was never attacked. In a duel there is only one
/// defender and the two predicates cannot be told apart.
#[test]
fn cr_506_2_the_undirected_attacker_count_is_not_the_defender_side_one() {
    use crabomination::card::Predicate;
    let (g, _pw) = three_way_attack();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    assert!(
        g.evaluate_predicate(
            &Predicate::AttackedWithCountAtLeast { who: PlayerRef::Seat(1), at_least: 4 },
            &ctx,
        ),
        "seat 1 declared four attackers in total",
    );
    assert!(
        !g.evaluate_predicate(
            &Predicate::AttackedDefenderWithCountAtLeast {
                who: PlayerRef::Seat(1),
                defender: PlayerRef::Seat(3),
                at_least: 1,
                include_planeswalkers: true,
            },
            &ctx,
        ),
        "but none of them at seat 3 — the difference a duel cannot show",
    );
}

// ── "Choose an opponent" at 3+ seats (CR 601.2c's singular choice) ──────────

/// A printed "As this ~ enters, **choose an opponent**" names ONE seat. Eight
/// shipped cards spelled it `PlayerRef::EachOpponent` and resolved it through
/// the singular `resolve_player`, which is exact in a duel (the set holds one
/// seat) and in a pod answers with the *first* opponent by seat index — the
/// choice made by the table's seating rather than by the card's controller.
/// `PlayerRef::HostileOpponent` is the one ranked answer instead.
#[test]
fn cr_614_12a_choose_an_opponent_names_one_seat_and_not_by_seat_order() {
    let mut g = multi_player_game(4);
    // Seat 1 is the first opponent by index and the *least* worth choosing;
    // seat 3 is nearest to dying, which is what the ranked answer reads.
    g.players[1].life = 20;
    g.players[2].life = 15;
    g.players[3].life = 3;
    let rack = g.move_card_to_battlefield_for_test(0, catalog::the_rack());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(rack).and_then(|c| c.chosen_player),
        Some(3),
        "the lowest-life opponent, not seat 1"
    );
}

/// And the duel is unchanged: one opponent means one answer, which is why the
/// two-player traces do not move.
#[test]
fn cr_614_12a_choose_an_opponent_in_a_duel_is_the_lone_opponent() {
    let mut g = two_player_game();
    let rack = g.move_card_to_battlefield_for_test(0, catalog::the_rack());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rack).and_then(|c| c.chosen_player), Some(1));
}

/// The defect was silent in release and a `debug_assert!` in debug: a fan-out
/// ref resolved singularly drops every seat but the first. Pin that none of
/// the eight is a fan-out any more by entering each in a four-seat pod, which
/// trips the assertion if one regresses.
#[test]
fn cr_800_4_choose_an_opponent_cards_resolve_a_single_seat_in_a_pod() {
    type Chooser = (&'static str, fn() -> crabomination::card::CardDefinition);
    let choosers: [Chooser; 6] = [
        ("The Rack", catalog::the_rack),
        ("Cursed Rack", catalog::cursed_rack),
        ("Pallimud", catalog::pallimud),
        ("Haunting Apparition", catalog::haunting_apparition),
        ("Entropic Specter", catalog::entropic_specter),
        ("Skyshroud War Beast", catalog::skyshroud_war_beast),
    ];
    for (name, factory) in choosers {
        let mut g = multi_player_game(4);
        let id = g.move_card_to_battlefield_for_test(0, factory());
        drain_stack(&mut g);
        let chosen = g.battlefield_find(id).and_then(|c| c.chosen_player);
        assert!(
            matches!(chosen, Some(seat) if seat != 0),
            "{name} chose one opponent, got {chosen:?}"
        );
    }
}

/// CR 101.4 — "each opponent …" reaches EVERY opponent. Five shipped cards
/// spelled the clause `PlayerRef::EachOpponent` on an effect arm that
/// resolves `who` through the singular `resolve_player`, so in a pod only the
/// first opponent by seat index was touched. `Effect::EachPlayerDoes` is the
/// fan-out the arm could not do for itself, and it runs the body in APNAP
/// order with each seat as its own controller.
#[test]
fn cr_101_4_each_opponent_sacrifices_reaches_every_opponent() {
    let mut g = multi_player_game(4);
    let mut victims = Vec::new();
    for seat in 1..4 {
        victims.push(g.add_card_to_battlefield(seat, catalog::grizzly_bears()));
    }
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mandate = g.add_card_to_hand(0, catalog::silverquill_mandate());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    cast(&mut g, mandate);
    drain_stack(&mut g);
    for (i, v) in victims.iter().enumerate() {
        assert!(
            g.battlefield_find(*v).is_none(),
            "opponent {} sacrificed — not just the first by seat index",
            i + 1
        );
    }
    assert!(g.battlefield_find(mine).is_some(), "your own creature is untouched");
}

/// The discard half of the same class, and the one with a countable
/// observable: "each opponent discards two cards unless they discard a
/// nonland card" reached one opponent's hand in a pod.
#[test]
fn cr_101_4_each_opponent_discards_reaches_every_opponent() {
    let mut g = multi_player_game(4);
    for seat in 1..4 {
        for _ in 0..4 {
            g.add_card_to_hand(seat, catalog::grizzly_bears());
        }
    }
    let before: Vec<usize> = (0..4).map(|s| g.players[s].hand.len()).collect();
    g.move_card_to_battlefield_for_test(0, catalog::bandits_talent());
    drain_stack(&mut g);
    for (seat, was) in before.iter().enumerate().skip(1) {
        assert!(
            g.players[seat].hand.len() < *was,
            "opponent {seat} discarded — not just the first by seat index"
        );
    }
    assert_eq!(g.players[0].hand.len(), before[0], "you discard nothing");
    assert!(g.pending_decision.is_none(), "no seat is left mid-discard");
}

/// And the singular half: "AN opponent chooses a creature type" is one seat,
/// answered by the ranked helper rather than by seat order.
#[test]
fn cr_614_12a_an_opponent_chooses_names_one_seat() {
    let mut g = multi_player_game(4);
    g.players[1].life = 20;
    g.players[2].life = 20;
    g.players[3].life = 2;
    let id = g.move_card_to_battlefield_for_test(0, catalog::callous_oppressor());
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(id).and_then(|c| c.chosen_creature_type).is_some(),
        "CR 614.12a — the type is named as the Oppressor enters"
    );
}

// ── "Any player may …. If a player does, …" at N seats ────────────────────

/// CR 101.4a — every choice is made before any action is taken, so an
/// earlier acceptance does not close the offer. Desecration Demon's
/// 2024-11-08 ruling: "each opponent in turn order may choose to sacrifice a
/// creature, **even if an opponent already chose** to sacrifice a creature
/// that combat", and the Demon gets "a maximum of one +1/+1 counter each
/// combat, no matter how many creatures were sacrificed".
#[test]
fn cr_101_4a_every_opponent_is_offered_and_the_consequence_runs_once() {
    use crabomination::card::CounterType;
    use crabomination::decision::ScriptedDecider;
    use crabomination::game::types::TurnStep;
    let mut g = multi_player_game(4);
    let demon = g.move_card_to_battlefield_for_test(0, catalog::desecration_demon());
    let fodder: Vec<_> = (1..4)
        .map(|s| g.move_card_to_battlefield_for_test(s, catalog::grizzly_bears()))
        .collect();
    drain_stack(&mut g);
    g.decider = Box::new(ScriptedDecider::new(std::iter::repeat_n(DecisionAnswer::Bool(true), 3)));
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);

    for (i, id) in fodder.iter().enumerate() {
        assert!(
            g.battlefield_find(*id).is_none(),
            "opponent {} was offered too, not just the first", i + 1
        );
    }
    let d = g.battlefield_find(demon).expect("Demon survives");
    assert!(d.tapped, "tapped");
    assert_eq!(
        d.counter_count(CounterType::PlusOnePlusOne), 1,
        "one counter for three sacrifices, not three"
    );
}

/// The other half: nobody accepting still runs `otherwise` once, and no seat
/// is left mid-offer.
#[test]
fn cr_101_4a_all_declining_leaves_the_offer_closed() {
    use crabomination::card::CounterType;
    use crabomination::decision::ScriptedDecider;
    use crabomination::game::types::TurnStep;
    let mut g = multi_player_game(4);
    let demon = g.move_card_to_battlefield_for_test(0, catalog::desecration_demon());
    for s in 1..4 {
        g.move_card_to_battlefield_for_test(s, catalog::grizzly_bears());
    }
    drain_stack(&mut g);
    g.decider = Box::new(ScriptedDecider::new(std::iter::repeat_n(DecisionAnswer::Bool(false), 3)));
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);

    let d = g.battlefield_find(demon).expect("Demon survives");
    assert!(!d.tapped, "nobody paid, so nothing happened");
    assert_eq!(d.counter_count(CounterType::PlusOnePlusOne), 0);
    assert!(g.pending_decision.is_none(), "no seat is left mid-offer");
}

// ── Melee at N seats (CR 702.121) ─────────────────────────────────────────

/// CR 702.121a — melee counts each *opponent you attacked with a creature*
/// this combat, not each attacker. The 2016-08-23 ruling is explicit: "if you
/// attack one player with Wings of the Guard and another player with five
/// creatures, Wings of the Guard will get +2/+2." And CR 702.121b — "if a
/// creature has multiple instances of melee, each triggers separately", which
/// is what Adriana's second line manufactures on a creature that already
/// prints the keyword.
#[test]
fn cr_702_121_melee_counts_opponents_and_each_instance_triggers_separately() {
    let mut g = multi_player_game(4);
    let adriana = g.move_card_to_battlefield_for_test(0, catalog::adriana_captain_of_the_guard());
    let wings = g.move_card_to_battlefield_for_test(0, catalog::wings_of_the_guard());
    let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    for id in [adriana, wings, bear] {
        g.clear_sickness(id);
    }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;

    // Two opponents attacked, three attackers — the count is 2, not 3.
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: wings, target: AttackTarget::Player(1) },
        Attack { attacker: adriana, target: AttackTarget::Player(2) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("three attackers across two seats");
    drain_stack(&mut g);

    let pt = |id| {
        let c = g.compute_battlefield();
        let p = c.iter().find(|p| p.id == id).expect("on the battlefield");
        (p.power, p.toughness)
    };
    assert_eq!(pt(adriana), (6, 6), "4/4 + one melee instance x 2 opponents");
    assert_eq!(
        pt(wings), (5, 5),
        "1/1 + TWO instances (printed keyword and Adriana's grant) x 2 opponents",
    );
    assert_eq!(pt(bear), (4, 4), "2/2 + the granted instance x 2 opponents");
}

/// CR 506.2 — the cards that print "defending player controls" only differ
/// from "an opponent controls" at three seats or more, so this is where the
/// class is pinned. Cyclops Gladiator's attack fight may choose a creature
/// the seat it is attacking controls, and nothing a bystander controls.
#[test]
fn cr_506_2_an_attack_clause_reaches_only_the_seat_being_attacked() {
    use crabomination::decision::ScriptedDecider;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = multi_player_game(4);
    let cyclops = g.move_card_to_battlefield_for_test(0, catalog::cyclops_gladiator());
    let theirs = g.move_card_to_battlefield_for_test(1, catalog::grizzly_bears());
    let bystander = g.move_card_to_battlefield_for_test(2, catalog::grizzly_bears());
    drain_stack(&mut g);
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(4),
    ));
    g.clear_sickness(cyclops);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cyclops,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack seat 1");
    drain_stack(&mut g);

    // 4/4 fights a 2/2: the defending seat's bear dies, seat 2's does not.
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    assert!(g.battlefield_find(theirs).is_none(), "the defending seat's creature was fought");
    assert!(
        g.battlefield_find(bystander).is_some(),
        "CR 506.2 — seat 2 is an opponent but is not the defending player",
    );
}

/// CR 800.4a — a player who leaves the game is no longer a player, so a
/// "player with the most/least X" reference must not find them. The seat
/// stays in `players` with `eliminated` set, and four of the five extremum
/// `PlayerRef`s walked `0..players.len()` with no filter. `LowestLife` is the
/// one that bites every time: a departed seat is at or below zero life, so it
/// is *always* the lowest.
#[test]
fn cr_800_4a_an_extremum_player_ref_skips_a_seat_that_left_the_game() {
    use crabomination::effect::PlayerRef;
    use crabomination::game::effects::EffectContext;
    let mut g = multi_player_game(4);
    g.players[0].life = 20;
    g.players[1].life = 5;
    g.players[2].life = 30;
    g.players[3].life = 0;
    g.players[3].eliminated = true;

    // Give the departed seat what would make it win every extremum.
    for _ in 0..5 {
        g.add_card_to_hand(3, catalog::grizzly_bears());
    }
    g.move_card_to_battlefield_for_test(3, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(3, catalog::grizzly_bears());
    drain_stack(&mut g);

    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let ask = |r: PlayerRef| g.resolve_players(&r, &ctx).first().copied();
    assert_eq!(ask(PlayerRef::LowestLife), Some(1), "the dead seat is not the lowest");
    assert_eq!(ask(PlayerRef::HighestLife), Some(2), "and the highest is still seat 2");
    assert_ne!(
        ask(PlayerRef::MostCardsInHand),
        Some(3),
        "a departed seat's hand is not in the running either",
    );
    assert_ne!(ask(PlayerRef::MostCreatures), Some(3), "nor its board");
}

/// CR 701.38a — a vote proceeds "starting with you, **in turn order**".
/// `Effect::Vote` used the turn-order helper; the will-of-the-council half
/// built `controller + opponents_of(controller)`, which is seat-INDEX order
/// and the same list only when the controller is seat 0. The ballots are
/// logged in the order they are cast, so the order is observable.
#[test]
fn cr_701_38a_a_council_vote_runs_in_turn_order_from_the_controller() {
    use crabomination::decision::ScriptedDecider;
    let mut g = multi_player_game(4);
    // Seat 2 resolves the vote; turn order from there is 2, 3, 0, 1.
    for s in [0usize, 1, 3] {
        g.move_card_to_battlefield_for_test(s, catalog::grizzly_bears());
    }
    drain_stack(&mut g);
    g.decider = Box::new(ScriptedDecider::new([]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(2, None, 0, 0);
    g.resolve_effect(&catalog::councils_judgment().effect, &ctx).expect("vote resolves");

    // Each ballot's prompt names the seat it is cast on behalf of, so the
    // decider's `asked` log is the order in which the council voted.
    let order: Vec<usize> = match g.decider.kind() {
        crabomination::decision::DeciderKind::Scripted { asked, .. } => asked
            .iter()
            .filter_map(|d| match d {
                crabomination::decision::Decision::ChooseTarget { description, .. } => description
                    .rsplit_once("on P")
                    .and_then(|(_, tail)| tail.split('\'').next())
                    .and_then(|t| t.parse::<usize>().ok()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    assert_eq!(order, vec![2, 3, 0, 1], "CR 701.38a — turn order from the controller");
    assert!(
        g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "and a permanent with the most votes was exiled",
    );
}

/// The bot half of a council ballot, asserted rather than assumed. A
/// will-of-the-council ballot is a `Decision::ChooseTarget` whose candidate
/// list is built from the **controller's** perspective ("a nonland permanent
/// you don't control"), so an opponent votes from a list that can contain its
/// own permanents. `decide_choose_target` prefers a permanent the voter does
/// not control and only gives up its own when every candidate is its own —
/// which is the right answer for a ballot, an edict and free-floating removal
/// alike.
#[test]
fn a_council_ballot_never_votes_for_the_voters_own_permanent_when_another_is_legal() {
    use crabomination::decision::DecisionAnswer;
    use crabomination::game::types::Target;
    use crabomination::server::bot::{decide_choose_target, EvalWeights};
    let mut g = multi_player_game(4);
    let mine = g.move_card_to_battlefield_for_test(1, catalog::grizzly_bears());
    let theirs = g.move_card_to_battlefield_for_test(2, catalog::serra_angel());
    drain_stack(&mut g);

    // Seat 1 votes from a ballot holding its own Bears and seat 2's Angel.
    let legal = vec![Target::Permanent(mine), Target::Permanent(theirs)];
    let w = EvalWeights::default();
    let answer = decide_choose_target(&g, 1, &legal, &w);
    assert_eq!(
        answer,
        DecisionAnswer::Target(Target::Permanent(theirs)),
        "vote for the permanent you do not control",
    );

    // With only its own on the ballot it has to pick one, and does.
    let only_mine = vec![Target::Permanent(mine)];
    assert_eq!(
        decide_choose_target(&g, 1, &only_mine, &w),
        DecisionAnswer::Target(Target::Permanent(mine)),
        "and a ballot of only your own still returns a legal vote",
    );
}

/// CR 800.4a / 608.2b — a player who has left the game is an illegal target:
/// Okaun's Partner-with trigger aimed at a seat that then leaves is countered
/// on resolution. It used to resolve, re-seat the departed target's "may"
/// onto its controller (CR 800.4g) and search the departed seat's library.
#[test]
fn cr_608_2b_a_trigger_aimed_at_a_departed_player_is_countered() {
    use crabomination::decision::DecisionAnswer;
    let mut g = multi_player_game(3);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    g.add_card_to_library(1, catalog::zndrsplt_eye_of_wisdom());
    let okaun = g.add_card_to_battlefield(0, catalog::okaun_eye_of_chaos());
    g.fire_self_etb_triggers(okaun, 0);
    if g.pending_decision.is_some() {
        g.submit_decision(DecisionAnswer::Target(Target::Player(1))).expect("aim at seat 1");
    }
    // "Target player": aim it at seat 1 whatever the auto-target picked.
    match g.stack.last_mut() {
        Some(crabomination::game::types::StackItem::Trigger { target, .. }) => {
            *target = Some(Target::Player(1));
        }
        other => panic!("the trigger waits on the stack, got {other:?}"),
    }
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive());
    g.resolve_top_of_stack().expect("resolve");
    assert!(g.pending_decision.is_none(), "nobody is asked: the trigger was countered");
    assert!(g.stack.is_empty());
}

/// CR 800.4g — a "that player may" re-seated ONTO its controller (the
/// departed chooser was a teammate) is asked once. The replay used to fall into `MayDo`, which
/// doesn't read the answer log: the controller was asked a second time and
/// the first answer leaked (`CRAB_ANSWER_LOG=strict`, a 25-seat debug pod).
#[test]
fn cr_800_4g_a_may_reseated_onto_its_controller_is_asked_once() {
    use crabomination::decision::DecisionAnswer;
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = multi_player_game(3);
    for p in g.players.iter_mut() {
        p.wants_ui = true;
    }
    g.assign_teams(vec![vec![0, 1], vec![2]]).unwrap();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].eliminated = true;
    let life = g.players[0].life;
    g.stack.push(
        crabomination::game::types::TriggerPush::new(
            bear,
            0,
            Effect::MayDoBy {
                who: PlayerRef::Seat(1),
                description: "Gain 1?".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            },
        )
        .build(),
    );
    g.resolve_top_of_stack().expect("resolve");
    assert_eq!(g.pending_decision.as_ref().map(|p| p.acting_player()), Some(0), "re-seated on seat 0");
    g.submit_decision(DecisionAnswer::Bool(true)).expect("yes");
    assert!(g.pending_decision.is_none(), "asked once");
    assert_eq!(g.players[0].life, life + 1);
}

/// "Choose an opponent. That player returns a card from their graveyard to
/// their hand" — `Effect::AsPlayer` hands the body to the chosen seat, so the
/// return reads *their* graveyard, not the caster's (CR 109.5: "you" in an
/// effect run for another player means that player).
#[test]
fn as_player_runs_the_body_for_the_chosen_opponent() {
    use crabomination::card::{CardDefinition, CardType, SelectionRequirement as R};
    use crabomination::effect::{Effect, Value};
    use crabomination::mana::Color;
    let mut g = multi_player_game(3);
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Gift of Memory",
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseOpponentThen {
            then: Box::new(Effect::AsPlayer {
                who: PlayerRef::ChosenPlayerOfSource,
                body: Box::new(Effect::ReturnGraveyardCardsToHand { filter: R::Any, max: Value::Const(1) }),
            }),
        },
        ..Default::default()
    });
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "seat 1 got its own card back");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mine), "the caster's graveyard is untouched");
}
