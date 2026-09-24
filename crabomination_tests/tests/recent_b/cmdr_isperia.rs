//! Commander: the First Flight precon (SCD, Isperia, Supreme Judge,
//! `decks::cmdr_isperia`) and the primitive it needed.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn declare(g: &mut GameState, seat: usize, attacks: Vec<Attack>) -> Result<(), String> {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.would_accept(GameAction::DeclareAttackers(attacks.clone()))
        .then_some(())
        .ok_or_else(|| "rejected".to_string())
}

fn ready(g: &mut GameState, id: CardId) {
    g.clear_sickness(id);
    g.battlefield_find_mut(id).unwrap().tapped = false;
}

/// CR 508.1d — Gideon Jura's +2: during the target opponent's next turn each
/// creature they control that can attack must attack Gideon. A declaration
/// aimed elsewhere, or one leaving an able creature home, is illegal; other
/// players are unbound, and the lure ends with that turn.
#[test]
fn cr_508_1d_gideon_juras_lure_binds_one_opponents_next_turn() {
    let mut g = pod(3);
    let gideon = g.add_card_to_battlefield(0, catalog::gideon_jura());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: gideon,
        ability_index: 0,
        target: Some(Target::Player(1)),
        x_value: None,
    })
    .expect("+2");
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elves = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    for id in [bear, elves, other] {
        ready(&mut g, id);
    }
    g.turn_number += 1;
    let at = |id, t| Attack { attacker: id, target: t };
    let pw = AttackTarget::Planeswalker(gideon);
    assert!(declare(&mut g, 1, vec![at(bear, AttackTarget::Player(2)), at(elves, pw)]).is_err(), "aimed elsewhere");
    assert!(declare(&mut g, 1, vec![at(bear, pw)]).is_err(), "an able creature stayed home");
    assert!(declare(&mut g, 1, vec![]).is_err(), "nobody attacked");
    declare(&mut g, 1, vec![at(bear, pw), at(elves, pw)]).expect("both at Gideon");
    // The bot's own declaration obeys it.
    let picked = crabomination::server::bot::pick_attacks(&g, 1);
    assert_eq!(picked.len(), 2);
    assert!(picked.iter().all(|a| a.target == pw), "{picked:?}");
    // Seat 2 was never lured.
    declare(&mut g, 2, vec![]).expect("seat 2 may stay home");
    // The lure lasts one turn: it's gone once seat 1's turn has ended.
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    for _ in 0..6 {
        if g.active_player_idx != 1 {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_ne!(g.active_player_idx, 1);
    assert!(g.players[1].attack_lure.is_none());
}

/// Angler Turtle makes every creature an opponent controls attack each combat.
#[test]
fn angler_turtle_forces_opposing_attacks() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::angler_turtle());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    ready(&mut g, bear);
    assert!(declare(&mut g, 1, vec![]).is_err());
    declare(&mut g, 1, vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]).expect("attack");
}

/// Inspired Sphinx draws one card per opponent.
#[test]
fn inspired_sphinx_draws_per_opponent() {
    let mut g = pod(4);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    flood(&mut g, 0);
    let sphinx = g.add_card_to_hand(0, catalog::inspired_sphinx());
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sphinx,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 3);
}

/// Gravitational Shift: fliers +2/+0, the rest −2/−0.
#[test]
fn gravitational_shift_splits_by_flying() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gravitational_shift());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 0);
    assert_eq!(g.computed_permanent(angel).unwrap().power, 6);
}

fn swing(g: &mut GameState, seat: usize, attacks: Vec<Attack>) {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

/// Windreader Sphinx draws off *any* player's flier attacking, not only its
/// controller's.
#[test]
fn windreader_sphinx_draws_on_an_opponents_flier() {
    let mut g = pod(3);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.add_card_to_battlefield(0, catalog::windreader_sphinx());
    g.add_card_to_library(0, catalog::island());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    ready(&mut g, angel);
    let hand = g.players[0].hand.len();
    swing(&mut g, 1, vec![Attack { attacker: angel, target: AttackTarget::Player(2) }]);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Tide Skimmer draws only when two or more fliers attack together.
#[test]
fn tide_skimmer_counts_attacking_fliers() {
    let mut g = pod(2);
    let skimmer = g.add_card_to_battlefield(0, catalog::tide_skimmer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    for id in [skimmer, bear] {
        ready(&mut g, id);
    }
    let hand = g.players[0].hand.len();
    let at = |id| Attack { attacker: id, target: AttackTarget::Player(1) };
    swing(&mut g, 0, vec![at(skimmer), at(bear)]);
    assert_eq!(g.players[0].hand.len(), hand, "one flier");
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    for id in [skimmer, angel] {
        ready(&mut g, id);
    }
    g.attacking.clear();
    swing(&mut g, 0, vec![at(skimmer), at(angel)]);
    assert_eq!(g.players[0].hand.len(), hand + 1, "two fliers");
}
