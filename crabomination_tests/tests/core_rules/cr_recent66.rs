//! CR conformance for this run's engine work:
//! - CR 102 — players, opponents, teams.
//! - CR 502 — the untap step (turn-based untap, and a global "don't untap").
//! - CR 211 / 212 / 902 — Vanguard: hand and life modifiers, and abilities
//!   that function from the command zone.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

// ── CR 102 — Players ────────────────────────────────────────────────────────

/// CR 102.2 — in a two-player game your opponent is the other player.
#[test]
fn cr_102_2_the_other_player_is_your_opponent() {
    let g = two_player_game();
    assert_eq!(g.opponents_of(0), vec![1]);
    assert_eq!(g.opponents_of(1), vec![0]);
}

/// CR 102.3 — teammates aren't opponents; everyone off your team is.
#[test]
fn cr_102_3_teammates_are_not_opponents() {
    let mut g = multi_player_game(4);
    g.teams = vec![
        crabomination::team::Team { id: crabomination::team::TeamId(0), members: vec![0, 2], shared_life: None },
        crabomination::team::Team { id: crabomination::team::TeamId(1), members: vec![1, 3], shared_life: None },
    ];
    assert!(g.same_team(0, 2));
    assert_eq!(g.opponents_of(0), vec![1, 3]);
}

/// CR 102.4 — with no teams, "your team" is just you.
#[test]
fn cr_102_4_your_team_is_you_without_teams() {
    let g = two_player_game();
    assert!(g.same_team(0, 0));
    assert!(!g.same_team(0, 1));
}

// ── CR 502 — Untap Step ─────────────────────────────────────────────────────

/// CR 502.2 — the active player untaps their permanents, not their opponent's.
#[test]
fn cr_502_2_only_the_active_player_untaps() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    g.do_untap();
    assert!(!g.battlefield_find(mine).unwrap().tapped);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// CR 502.4 — "permanents don't untap during their controllers' untap steps"
/// stops every seat, while summoning sickness still wears off.
#[test]
fn cr_502_4_global_dont_untap_stops_every_seat() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::mist_of_stagnation());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert!(!g.battlefield_find(bear).unwrap().summoning_sick, "sickness still clears");
}

// ── CR 211 / 212 / 902 — Vanguard ───────────────────────────────────────────

/// CR 211.1 — the avatar's hand modifier moves the maximum hand size.
#[test]
fn cr_211_1_vanguard_hand_modifier_applies_at_seating() {
    let mut g = main_phase();
    g.seat_vanguard(0, catalog::ashling_the_pilgrim_avatar());
    assert_eq!(g.players[0].max_hand_size, Some(6), "hand -1");
}

/// CR 212.1 — and its life modifier moves the starting life total.
#[test]
fn cr_212_1_vanguard_life_modifier_applies_at_seating() {
    let mut g = main_phase();
    g.seat_vanguard(0, catalog::ashling_the_pilgrim_avatar());
    assert_eq!(g.players[0].life, 26, "life +6");
    assert_eq!(g.players[0].starting_life, 26);
}

/// CR 902.5 — a Vanguard's activated ability functions from the command zone.
#[test]
fn cr_902_5_vanguard_activates_from_the_command_zone() {
    let mut g = main_phase();
    let avatar = g.seat_vanguard(0, catalog::ashling_the_pilgrim_avatar());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, avatar, None);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);
}

/// CR 902.5 — and so does its triggered ability.
#[test]
fn cr_902_5_vanguard_triggers_from_the_command_zone() {
    let mut g = main_phase();
    g.seat_vanguard(0, catalog::serra_angel_avatar());
    let before = g.players[0].life;
    let spell = g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 2);
}

/// Chronatog Avatar lifts the hand-size cap outright and only fires once a turn.
#[test]
fn cr_902_5_chronatog_avatar_lifts_the_cap_and_limits_itself() {
    let mut g = main_phase();
    let avatar = g.seat_vanguard(0, catalog::chronatog_avatar());
    assert_eq!(g.players[0].max_hand_size, None);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    activate(&mut g, 0, avatar, None);
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].skip_turns, 1);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: avatar,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "only once each turn"
    );
}

/// The server view carries a Vanguard's command-zone abilities so the client
/// activates the avatar where it lives instead of trying to cast it.
#[test]
fn view_projects_vanguard_command_zone_abilities() {
    let mut g = main_phase();
    g.seat_vanguard(0, catalog::ashling_the_pilgrim_avatar());
    g.seat_commanders(0, vec![catalog::grizzly_bears()]);
    let view = crabomination::server::view::project(&g, 0);
    let mut seen = view.players[0].command.iter().map(|c| match c {
        crabomination::net::HandCardView::Known(k) => (k.name.clone(), k.zone_abilities.len()),
        crabomination::net::HandCardView::Hidden { .. } => (String::new(), 0),
    });
    assert_eq!(seen.next(), Some(("Ashling the Pilgrim Avatar".into(), 1)));
    assert_eq!(seen.next(), Some(("Grizzly Bears".into(), 0)), "only avatars get zone abilities");
}
