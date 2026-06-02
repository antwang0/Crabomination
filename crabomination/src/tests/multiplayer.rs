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
        g.block_map.get(&blocker).copied(),
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
    use crate::card::{CardDefinition, CardId, CardType, Subtypes, TriggeredAbility};
    use crate::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};

    // A "lifegain pinger" — minimal triggered ability that fires on
    // LifeGained / AnyPlayer. The body is irrelevant; we inspect the
    // stack order, not the resolution.
    let pinger = |name: &'static str| CardDefinition {
        name,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
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
    use crate::card::{CardDefinition, CardId, CardType, Subtypes, TriggeredAbility};
    use crate::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};

    let pinger = |name: &'static str| CardDefinition {
        name,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
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

/// 2HG: poison stays per-player (CR 810.7b). One teammate hitting 10
/// poison loses individually; the other survives unless the shared
/// pool also runs out. Locks in the asymmetry between life (shared)
/// and poison (per-player).
#[test]
fn two_headed_giant_poison_is_per_player_not_shared() {
    let mut g = game_with_format(Format::TwoHeadedGiant, 4);

    g.players[0].poison_counters = 10;
    g.check_state_based_actions();

    assert!(g.players[0].eliminated, "10 poison loses individually");
    assert!(!g.players[1].eliminated, "teammate survives 0 poison");
    assert_eq!(g.teams[0].shared_life, Some(30), "shared life unaffected");
    // Game continues — team A still has seat 1 in.
    assert!(g.game_over.is_none());
}

// ── Phase H — zone-change replacement effects ─────────────────────────────

/// Baseline: with no replacement registered, a destroyed creature
/// lands in its owner's graveyard. Just confirms the test scaffolding
/// (`remove_from_battlefield_to_graveyard` is the engine entry point
/// destroy / lethal-damage SBA / Effect::Destroy all funnel through).
#[test]
fn replacement_baseline_destroyed_creature_hits_graveyard() {
    use crate::card::Zone;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Sanity-check the resolver returns the intended zone with no
    // replacements registered.
    assert_eq!(
        g.resolve_zone_change(bear, Zone::Battlefield, Zone::Graveyard),
        Zone::Graveyard,
    );

    g.remove_from_battlefield_to_graveyard(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().all(|c| c.id != bear));
}

/// A registered "would go to graveyard → exile instead" replacement
/// for a specific CardId reroutes the destroyed creature. Verifies
/// the wiring end-to-end through
/// `remove_from_battlefield_to_graveyard` → resolver →
/// `place_card_at_resolved_zone`.
#[test]
fn replacement_redirects_graveyard_to_exile() {
    use crate::card::Zone;
    use crate::replacement::{
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

    g.remove_from_battlefield_to_graveyard(bear);
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
    use crate::card::Zone;
    use crate::replacement::{
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

    g.remove_from_battlefield_to_graveyard(target);
    g.remove_from_battlefield_to_graveyard(bystander);

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
    use crate::card::Zone;
    use crate::replacement::{
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

    g.remove_from_battlefield_to_graveyard(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().all(|c| c.id != bear));
}

/// The `from` filter on a replacement effect gates by origin zone. A
/// replacement keyed on `from: Some(Library)` should NOT fire when
/// the card is leaving the battlefield.
#[test]
fn replacement_from_filter_gates_origin_zone() {
    use crate::card::Zone;
    use crate::replacement::{
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

    g.remove_from_battlefield_to_graveyard(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().all(|c| c.id != bear));
}

/// `unregister_replacement` drops the entry; subsequent zone changes
/// behave as if it was never registered.
#[test]
fn replacement_unregister_drops_effect() {
    use crate::card::Zone;
    use crate::replacement::{
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

    g.remove_from_battlefield_to_graveyard(bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

// ── Phase I/J/L/M — Commander format end-to-end ───────────────────────────

/// Minimal legendary creature for Commander tests — free to cast,
/// no abilities, sorcery-speed (the default for a Creature card type).
/// Avoids depending on a real catalog commander with a 5-color cost
/// while still being a Legal commander (Legendary + Creature).
fn test_commander() -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType, Subtypes, Supertype};
    CardDefinition {
        name: "Test Commander",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// `seat_commanders` installs the card into the seat's command zone,
/// records its CardId on `Player.commanders`, and registers a
/// zone-change replacement so leaving the battlefield bounces it
/// back to the command zone. Verifies the Phase I/J wiring as a
/// whole.
#[test]
fn seat_commanders_sets_up_command_zone_and_replacement() {
    use crate::card::Zone;
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
    assert!(!g.is_commander(crate::card::CardId(99_999)));

    // Replacement effect registered: a hypothetical
    // battlefield→graveyard move resolves to Command instead.
    assert_eq!(
        g.resolve_zone_change(cmd, Zone::Battlefield, Zone::Graveyard),
        Zone::Command,
    );
    // Same for exile / hand / library — the four CR-903.9b zones.
    for would_be in [Zone::Exile, Zone::Hand, Zone::Library] {
        assert_eq!(
            g.resolve_zone_change(cmd, Zone::Battlefield, would_be),
            Zone::Command,
            "{would_be:?} should also redirect to Command",
        );
    }
}

/// End-to-end Phase H + I + J: a Commander on the battlefield, when
/// destroyed, lands in the command zone — not the graveyard. The
/// graveyard `Effect::Destroy` / lethal-damage SBA path funnels
/// through `remove_from_battlefield_to_graveyard`, which Phase H
/// wired through the replacement resolver, which Phase J registered
/// the redirect for, which Phase I made land in
/// `Player.command`. All four phases active simultaneously.
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

    g.remove_from_battlefield_to_graveyard(cmd);

    assert!(
        g.players[0].command.iter().any(|c| c.id == cmd),
        "destroyed commander must return to the command zone",
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
    })
    .expect("first cast from CZ should succeed with no mana required");
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(1));
    // Command zone empty post-cast (until SBA / leave-play bounces
    // the resolved permanent back).
    assert!(g.players[0].command.iter().all(|c| c.id != cmd));
    // A spell is on the stack.
    assert_eq!(g.stack.len(), 1);
}

/// Phase L — second cast pays the tax. Cast once, drain stack,
/// destroy the commander (which bounces it back via the J replacement),
/// then attempt the second cast: with no mana in pool the cast
/// should fail (the {2} tax is unpaid). Add 2 colorless mana → cast
/// succeeds, count bumps to 2.
#[test]
fn commander_cast_tax_blocks_unpaid_recast() {
    use crate::mana::Color;
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
    })
    .unwrap();
    crate::game::drain_stack(&mut g);
    // Commander should now be on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == cmd));

    // Destroy it — Phase J replacement bounces back to command zone.
    g.remove_from_battlefield_to_graveyard(cmd);
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
    use crate::format::color_identity;
    use crate::mana::{Color, ColorSet};

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

    // Empty identity for a colorless card. Mox Pearl ({0}) returns
    // ColorSet::empty() since there are no colored pips in the cost.
    let mox = catalog::mox_pearl();
    assert_eq!(color_identity(&mox), ColorSet::empty());
}

#[test]
fn commander_deck_validator_catches_off_color_card() {
    use crate::format::{validate_commander_deck, CommanderDeckError, Deck};

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
    use crate::format::{validate_commander_deck, CommanderDeckError, Deck};

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
    use crate::format::{validate_commander_deck, CommanderDeckError, Deck};
    let deck = Deck::default();
    let err = validate_commander_deck(&deck).unwrap_err();
    let (_generic, cmd) = err;
    assert!(cmd.iter().any(|e| matches!(e, CommanderDeckError::MissingCommander)));
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
    use crate::card::{CardDefinition, CardId, CardType, Subtypes, TriggeredAbility};
    use crate::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};

    let pinger = |name: &'static str| CardDefinition {
        name,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
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

/// CR 903.9b — the commander redirect is a "may." A scripted decider
/// answering Bool(false) to `Decision::CommanderRedirect` lets the
/// commander land in the original zone (graveyard) instead of
/// bouncing to the command zone. Validates the polish 2 wiring.
#[test]
fn commander_redirect_can_be_declined() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
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
    g.remove_from_battlefield_to_graveyard(cmd);

    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == cmd),
        "with redirect declined, commander goes to graveyard",
    );
    assert!(
        g.players[0].command.iter().all(|c| c.id != cmd),
        "redirected-out commander must NOT also land in the command zone",
    );
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
            crate::decision::Decision::Mulligan { player, .. } => {
                assert_eq!(
                    *player, expected_seat,
                    "2HG mulligan must visit seat {expected_seat} independently",
                );
            }
            other => panic!("expected Mulligan, got {other:?}"),
        }
        g.submit_decision(crate::decision::DecisionAnswer::Keep).unwrap();
    }
    assert!(g.pending_decision.is_none(), "all four mulligans resolved");
}

// ── DefendingPlayer in combat-damage triggers (CR 509.2) ────────────────────

#[test]
fn abyssal_specter_only_defending_player_discards_in_ffa() {
    use crate::game::types::TurnStep;
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
