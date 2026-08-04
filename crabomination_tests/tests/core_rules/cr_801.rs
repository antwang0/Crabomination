//! CR 801 — the limited range of influence option, and the CR 809 Emperor
//! variant and the CR 811 Alternating Teams variant built on top of it.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::{AttackOption, multi_player_game, *};
use crabomination::mana::Color;

/// A five-seat table with range 1, ranges already snapshotted.
fn table(range: u32) -> GameState {
    let mut g = multi_player_game(5);
    g.range_of_influence = Some(range);
    g.refresh_range_matrix();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn attacker(g: &mut GameState, seat: usize) -> CardId {
    let id = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
    g.clear_sickness(id);
    id
}

/// CR 801.2 / 801.2b — you are always in your own range; the table wraps.
#[test]
fn cr_801_2_range_one_covers_both_neighbours_and_wraps() {
    let g = table(1);
    for (observer, other, want) in [
        (0, 0, true),
        (0, 1, true),
        (0, 4, true),
        (0, 2, false),
        (0, 3, false),
        (2, 3, true),
        (2, 0, false),
    ] {
        assert_eq!(g.player_in_range_of(observer, other), want, "{observer} -> {other}");
    }
}

/// CR 801.3 — creatures can attack only opponents inside their controller's
/// range.
#[test]
fn cr_801_3_attacks_stop_at_the_range_edge() {
    let mut g = table(1);
    g.step = TurnStep::DeclareAttackers;
    let bear = attacker(&mut g, 0);
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(2),
        }]))
        .is_err(),
        "seat 2 is two seats away",
    );
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]))
        .is_ok(),
    );
    assert_eq!(g.attackable_players_for(0), vec![1, 4]);
}

/// CR 801.4 — out-of-range players and objects can't be targeted.
#[test]
fn cr_801_4_out_of_range_targets_are_illegal() {
    let g = table(1);
    assert!(g.check_target_legality(&Target::Player(1), 0).is_ok());
    assert!(g.check_target_legality(&Target::Player(2), 0).is_err());
}

#[test]
fn cr_801_4_out_of_range_permanents_are_illegal_targets() {
    let mut g = table(1);
    let near = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let far = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 5);
    }
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(far)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
    );
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(near)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok(),
    );
}

/// CR 801.6 — you can't activate abilities of objects outside your range.
#[test]
fn cr_801_6_cant_activate_out_of_range_abilities() {
    let mut g = table(1);
    let far = g.add_card_to_battlefield(2, catalog::millstone());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 5);
    }
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: far,
            ability_index: 0,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
    );
}

/// CR 801.10 — a sweeper only reaches creatures inside its controller's range.
#[test]
fn cr_801_10_sweepers_stop_at_the_range_edge() {
    let mut g = table(1);
    let near = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let far = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: wrath,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(near).is_none(), "in range");
    assert!(g.battlefield_find(far).is_some(), "out of range and untouched");
}

/// CR 801.2c — ranges are re-determined as each turn begins, so a player
/// leaving only shifts them on the next turn.
#[test]
fn cr_801_2c_ranges_settle_at_the_start_of_a_turn() {
    let mut g = table(1);
    g.players[1].eliminated = true;
    assert!(!g.player_in_range_of(0, 2), "still snapshotted from before seat 1 left");
    g.refresh_range_matrix();
    assert!(g.player_in_range_of(0, 2), "seat 2 slides into range once recomputed");
}

/// The unlimited default leaves every check wide open.
#[test]
fn cr_801_1_unlimited_range_is_the_default() {
    let g = multi_player_game(5);
    assert!(g.range_of_influence.is_none());
    assert!(g.player_in_range_of(0, 3));
}

// ── CR 809 — the Emperor variant ──────────────────────────────────────────

/// Two teams of three, seated together with the emperor in the middle.
fn emperor_table() -> GameState {
    let mut g = multi_player_game(6);
    g.set_emperor_variant(2, 3);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g
}

/// CR 809.2 / 809.3a/b — seating, per-seat ranges, and the deploy option.
#[test]
fn cr_809_3a_emperors_see_two_seats_and_generals_one() {
    let g = emperor_table();
    assert!(g.players[1].is_emperor && g.players[4].is_emperor);
    assert_eq!(g.players[1].range_of_influence, Some(2));
    assert_eq!(g.players[0].range_of_influence, Some(1));
    assert!(g.deploy_creatures);
    // The emperor at seat 1 reaches both generals and the far team's edge.
    assert!(g.player_in_range_of(1, 3));
    assert!(!g.player_in_range_of(1, 4));
    // A general only reaches its neighbours.
    assert!(g.player_in_range_of(0, 5));
    assert!(!g.player_in_range_of(0, 2));
}

/// CR 809.3c — an emperor can't attack at all at the start of the game: the
/// only opponents in range are two seats away.
#[test]
fn cr_809_3c_emperors_cant_attack_anyone() {
    let mut g = emperor_table();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    assert!(g.attackable_players_for(1).is_empty());
}

/// CR 809.3c — a general attacks only the opposing general beside them.
#[test]
fn cr_809_3c_generals_attack_only_their_neighbour() {
    let g = emperor_table();
    assert_eq!(g.attackable_players_for(0), vec![5]);
}

/// CR 809.5b — a team loses when its emperor loses.
#[test]
fn cr_809_5b_a_team_falls_with_its_emperor() {
    let mut g = emperor_table();
    g.players[4].life = 0;
    let _ = g.check_state_based_actions();
    for seat in 3..6 {
        assert!(g.players[seat].eliminated, "seat {seat} went out with its emperor");
    }
    for seat in 0..3 {
        assert!(!g.players[seat].eliminated, "the other team is untouched");
    }
}

// ── CR 811 — Alternating Teams ────────────────────────────────────────────

/// CR 811.2a/811.2c/811.3 — interleaved seating, range 2, no deploy option.
#[test]
fn cr_811_3_teams_are_interleaved_around_the_table() {
    let mut g = multi_player_game(6);
    g.set_alternating_teams(3, 2);
    for seat in 0..6 {
        let left = (seat + 1) % 6;
        assert!(!g.same_team(seat, left), "seat {seat} sits next to a teammate");
    }
    assert_eq!(g.range_of_influence, Some(2));
    assert!(!g.deploy_creatures);
}

/// CR 811.2b — attack-left composes with the range: the only legal defender
/// is the seat to your left, which is never a teammate.
#[test]
fn cr_811_2b_attack_left_hits_the_next_team_over() {
    let mut g = multi_player_game(6);
    g.set_alternating_teams(3, 2);
    g.attack_option = AttackOption::AttackLeft;
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert_eq!(g.attackable_players_for(0), vec![1]);
}
