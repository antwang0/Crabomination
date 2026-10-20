//! Commander: the 2026-10-20 (`01FnAVVR`) closers (`decks::cmdr_01fn_*`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_granted(g: &mut GameState, id: CardId, target: Target) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: id,
        target: Some(target),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Seat 1 bolts seat 0 (who holds an Immortal Coil).
fn bolt_seat_zero(g: &mut GameState) {
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(g, 1);
    cast_as(g, 1, bolt, Some(Target::Player(0))).expect("bolt");
}

/// CR 615 / ruling 2008-10-01 — Immortal Coil prevents ALL the damage and
/// exiles a graveyard card per point, as many as there are.
#[test]
fn cr_615_immortal_coil_prevents_and_exiles_per_point() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::immortal_coil());
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let life = g.players[0].life;
    bolt_seat_zero(&mut g);
    assert_eq!(g.players[0].life, life, "prevented");
    assert_eq!(g.players[0].graveyard.len(), 2, "three of five exiled");
    assert!(g.players[0].is_alive());
}

/// CR 603.8 — with fewer cards than damage, the Coil still prevents it all,
/// then its state trigger sees the empty graveyard and its controller loses.
#[test]
fn cr_603_8_immortal_coil_empty_graveyard_loses() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::immortal_coil());
    g.add_card_to_graveyard(0, catalog::island());
    let life = g.players[0].life;
    bolt_seat_zero(&mut g);
    assert_eq!(g.players[0].life, life, "all 3 prevented with one card to exile");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(!g.players[0].is_alive(), "no cards in the graveyard: lose the game");
}

/// CR 121.2a — Forbidden Crypt turns a draw into returning a graveyard card,
/// and your cards bound for the graveyard are exiled (CR 614.6). An empty
/// graveyard at a draw loses the game (ruling 2004-10-04).
#[test]
fn cr_121_2a_forbidden_crypt_draws_from_the_graveyard() {
    let mut g = main_phase(3);
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::forbidden_crypt());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let library = g.players[0].library.len();
    let mut ev = Vec::new();
    assert!(g.draw_one(0, &mut ev));
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the Bear came back");
    assert_eq!(g.players[0].library.len(), library, "no card drawn");

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast_as(&mut g, 0, bolt, Some(Target::Player(1))).expect("bolt");
    assert!(g.players[0].graveyard.is_empty(), "the Bolt was exiled, not binned");
    assert!(g.exile.iter().any(|c| c.id == bolt));

    let mut ev = Vec::new();
    g.draw_one(0, &mut ev);
    g.check_state_based_actions();
    assert!(!g.players[0].is_alive(), "nothing to return: lose the game");
    assert!(g.players[1].is_alive() && g.players[2].is_alive(), "the pod plays on");
}

/// Colfenor's Plans: its seven exiled cards are playable while it stays, and
/// its controller — only — can't cast more than one spell each turn.
#[test]
fn colfenors_plans_plays_its_exile_one_spell_a_turn() {
    let mut g = main_phase(2);
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let plans = g.add_card_to_hand(0, catalog::colfenors_plans());
    flood(&mut g, 0);
    cast_as(&mut g, 0, plans, None).expect("cast the Plans");
    assert!(g.players[0].library.is_empty(), "seven exiled");
    let exiled: Vec<CardId> = g.exile.iter().filter(|c| c.face_down && c.may_play_until.is_some()).map(|c| c.id).collect();
    assert_eq!(exiled.len(), 7);
    let err = cast_granted(&mut g, exiled[0], Target::Player(1));
    assert!(err.is_err_and(|e| e.contains("SpellLimit")), "the Plans was this turn's spell");

    // A new turn's one spell may come from among the exiled cards.
    g.players[0].spells_cast_this_game_turn = 0;
    flood(&mut g, 0);
    let life = g.players[1].life;
    cast_granted(&mut g, exiled[0], Target::Player(1)).expect("cast from the Plans' exile");
    assert_eq!(g.players[1].life, life - 3);
    flood(&mut g, 0);
    assert!(cast_granted(&mut g, exiled[1], Target::Player(1)).is_err(), "one spell each turn");

    // The lock binds its controller, not the table.
    let other = g.add_card_to_hand(1, catalog::lightning_bolt());
    let second = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    cast_as(&mut g, 1, other, Some(Target::Player(0))).expect("opponent's first");
    cast_as(&mut g, 1, second, Some(Target::Player(0))).expect("opponent's second");
}
