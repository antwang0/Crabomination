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

fn declare(g: &mut GameState, attackers: &[CardId]) -> Result<(), GameError> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers
            .iter()
            .map(|&attacker| crabomination::game::types::Attack {
                attacker,
                target: crabomination::game::types::AttackTarget::Player(1),
            })
            .collect(),
    ))
    .map(|_| ())
}

/// CR 508.1g — Flooded Woodlands: one land sacrificed per attacking green
/// creature, paid as attackers are declared; nongreen attackers pay nothing,
/// and a short pool rejects the declaration.
#[test]
fn cr_508_1g_flooded_woodlands_charges_a_land_per_green_attacker() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::flooded_woodlands());
    let bears: Vec<CardId> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    let knight = g.add_card_to_battlefield(0, catalog::northern_paladin());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    for id in bears.iter().chain([&knight]) {
        g.clear_sickness(*id);
    }
    assert!(declare(&mut g.clone(), &bears).is_err(), "two green attackers, one land");
    declare(&mut g, &[bears[0], knight]).expect("one green attacker, one land");
    assert!(g.battlefield_find(land).is_none(), "the Forest paid for the Bears");
}

fn cast_x(g: &mut GameState, id: CardId, x: u32) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(x) })
        .expect("cast");
    drain_stack(g);
}

/// The Celestial Toymaker counts each spell that grouped cards into piles
/// this turn; at the end step every opponent loses 2 per spell.
#[test]
fn celestial_toymaker_drains_per_pile_spell_at_each_end_step() {
    let mut g = main_phase(3);
    stock_libraries(&mut g, 12);
    g.add_card_to_battlefield(0, catalog::the_celestial_toymaker());
    let epiphany = g.add_card_to_hand(0, catalog::epiphany_at_the_drownyard());
    let truth = g.add_card_to_hand(0, catalog::truth_or_tale());
    flood(&mut g, 0);
    cast_x(&mut g, epiphany, 1);
    cast_x(&mut g, truth, 0);
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, lives[0], "opponents only");
    assert_eq!(g.players[1].life, lives[1] - 4, "two pile spells: 2 x 2");
    assert_eq!(g.players[2].life, lives[2] - 4);
}

/// Truth or Tale: one card of the five reaches the hand, the other four go
/// to the bottom — none stays on top (ruling 2006-09-25).
#[test]
fn truth_or_tale_keeps_one_and_bottoms_four() {
    let mut g = main_phase(2);
    stock_libraries(&mut g, 8);
    let top5: Vec<CardId> = g.players[0].library.iter().take(5).map(|c| c.id).collect();
    let truth = g.add_card_to_hand(0, catalog::truth_or_tale());
    flood(&mut g, 0);
    cast_x(&mut g, truth, 0);
    let in_hand = top5.iter().filter(|id| g.players[0].hand.iter().any(|c| c.id == **id)).count();
    assert_eq!(in_hand, 1);
    let lib: Vec<CardId> = g.players[0].library.iter().map(|c| c.id).collect();
    assert_eq!(lib.len(), 7);
    assert!(lib[..3].iter().all(|id| !top5.contains(id)), "the untouched three are now on top");
}

/// Phyrexian Portal: ten or more cards — one pile exiled, one card tutored,
/// the rest shuffled back; nine cards — nothing happens.
#[test]
fn phyrexian_portal_needs_ten_cards() {
    let mut g = main_phase(2);
    let portal = g.add_card_to_battlefield(0, catalog::phyrexian_portal());
    for _ in 0..9 {
        g.add_card_to_library(0, catalog::island());
    }
    let activate = |g: &mut GameState| {
        flood(g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: portal,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("activate");
        drain_stack(g);
    };
    let hand = g.players[0].hand.len();
    activate(&mut g);
    assert_eq!((g.players[0].hand.len(), g.players[0].library.len()), (hand, 9), "nine: no split");
    g.add_card_to_library(0, catalog::island());
    activate(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one card tutored");
    let exiled = g.exile.len();
    assert_eq!(g.players[0].library.len() + exiled, 9, "one pile exiled, the rest shuffled back");
}

/// Bug fix (pod seed 9273): Death or Glory's "pile of an opponent's choice"
/// is ONE opponent — its `EachOpponent` chooser tripped `resolve_player`'s
/// fan-out assert at three seats (CR 800 multiplayer wording).
#[test]
fn death_or_glory_one_opponent_chooses_at_three_seats() {
    let mut g = main_phase(3);
    let bears: Vec<CardId> = (0..3).map(|_| g.add_card_to_graveyard(0, catalog::grizzly_bears())).collect();
    let spell = g.add_card_to_hand(0, catalog::death_or_glory());
    flood(&mut g, 0);
    cast_as(&mut g, 0, spell, None).expect("cast");
    let back = bears.iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    let exiled = bears.iter().filter(|id| g.exile.iter().any(|c| c.id == **id)).count();
    assert_eq!(back + exiled, 3, "every creature card went to one pile or the other");
}
