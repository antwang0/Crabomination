//! Multiplayer (3+ player) game tests. Covers turn rotation, priority
//! cycling, elimination, mulligan chaining, and format-specific starting
//! life across N players. Two-player coverage lives in `tests/game.rs`.

use super::*;
use super::{game_with_format, multi_player_game};
use crate::card::SelectionRequirement;
use crate::catalog;
use crate::decision::DecisionAnswer;
use crate::effect::PlayerRef;
use crate::format::Format;
use crate::game::effects::EffectContext;
use crate::team::{TeamError, TeamId};

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
        g.do_cleanup();
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

    g.do_cleanup();
    assert_eq!(g.active_player_idx, 1, "0 → 1");
    g.do_cleanup();
    assert_eq!(g.active_player_idx, 3, "1 → 3 (skip eliminated 2)");
    g.do_cleanup();
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
            crate::decision::Decision::Mulligan { player, .. } => {
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
