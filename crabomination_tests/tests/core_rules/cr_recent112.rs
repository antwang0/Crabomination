//! CR 508.1d — the attack declaration obeys the most requirements it can.
//! Where one creature's requirements name different defenders (a goad, a
//! chosen player, a lure), any defender meeting the most of them is legal.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, TurnStep};
use crabomination::game::*;

fn declare_step(seats: usize) -> GameState {
    let mut g = multi_player_game(seats);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g
}

/// CR 508.1d / 701.15b — Raving Dead bound to attack seat 1 and goaded by
/// seat 1: attacking seat 1 obeys "attack" and "attack that player",
/// attacking seat 2 obeys "attack" and "attack a player other than the
/// goader". Two each, so either is legal (each requirement used to reject
/// the other's defender, leaving no legal attack at all).
#[test]
fn cr_508_1d_a_goad_and_a_chosen_player_tie_and_either_is_legal() {
    let mut g = declare_step(3);
    let dead = g.add_card_to_battlefield(0, catalog::raving_dead());
    g.clear_sickness(dead);
    {
        let c = g.battlefield_find_mut(dead).unwrap();
        c.chosen_player = Some(1);
        c.goaded_by.push(1);
    }
    for q in [1, 2] {
        let mut probe = g.clone();
        probe
            .declare_attackers(vec![Attack { attacker: dead, target: AttackTarget::Player(q) }])
            .unwrap_or_else(|e| panic!("attacking seat {q} obeys two requirements: {e:?}"));
    }
    assert!(g.declare_attackers(vec![]).is_err(), "it still has to attack");
}

/// CR 508.1d / 701.15b — a goaded creature lured to the goader's planeswalker
/// (Gideon Jura's +2): the walker obeys "attack" and the lure, another
/// opponent "attack" and the goad. A tie, so either; the goader's face obeys
/// only "attack" and is not.
#[test]
fn cr_508_1d_a_goad_and_a_lure_tie() {
    let mut g = declare_step(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().goaded_by.push(1);
    let gideon = g.add_card_to_battlefield(1, catalog::gideon_jura());
    g.players[0].attack_lure = Some((gideon, 0));
    g.turn_number = 2;
    let ok = |g: &GameState, target| {
        g.clone().declare_attackers(vec![Attack { attacker: bear, target }]).is_ok()
    };
    assert!(ok(&g, AttackTarget::Planeswalker(gideon)), "the lure");
    assert!(ok(&g, AttackTarget::Player(2)), "the goad");
    assert!(!ok(&g, AttackTarget::Player(1)), "neither the lure nor the goad");
}
