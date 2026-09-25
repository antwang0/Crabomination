//! Commander: the Cavalry Charge precon (MOC, Sidar Jabari, `decks::cmdr_sidar`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn declare(g: &mut GameState, attacks: Vec<Attack>) {
    for a in &attacks {
        g.clear_sickness(a.attacker);
    }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

fn at(attacker: CardId, p: usize) -> Attack {
    Attack { attacker, target: AttackTarget::Player(p) }
}

fn run_combat_out(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn draw(g: &mut GameState, seat: usize, n: usize) {
    let mut evs = Vec::new();
    for _ in 0..n {
        g.draw_one(seat, &mut evs);
    }
}

/// CR 903.3 + Eminence (CR 207.2c): Sidar loots from the command zone when a
/// Knight attacks; in play, connecting returns a Knight card.
#[test]
fn sidar_loots_from_the_command_zone_and_recurs_knights() {
    let mut g = main_phase(3);
    library(&mut g, 0, 4);
    g.seat_commanders(0, vec![catalog::sidar_jabari_of_zhalfir()]);
    let knight = g.add_card_to_battlefield(0, catalog::worthy_knight());
    g.add_card_to_hand(0, catalog::plains());
    let (hand, yard) = (g.players[0].hand.len(), g.players[0].graveyard.len());
    declare(&mut g, vec![at(knight, 1)]);
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), yard + 1);

    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    let sidar = g.add_card_to_battlefield(0, catalog::sidar_jabari_of_zhalfir());
    let dead = g.add_card_to_graveyard(0, catalog::locthwain_lancer());
    declare(&mut g, vec![at(sidar, 1)]);
    run_combat_out(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "the Knight card came back");
}

/// Haakon: never from the hand; from the graveyard by its own permission, and
/// while it is in play other Knights come from the graveyard too.
#[test]
fn haakon_casts_only_from_the_graveyard_and_opens_it_for_knights() {
    let mut g = main_phase(2);
    let in_hand = g.add_card_to_hand(0, catalog::haakon_stromgald_scourge());
    assert!(cast_by(&mut g, 0, in_hand, &[]).is_err(), "not from the hand");
    let knight = g.add_card_to_graveyard(0, catalog::worthy_knight());
    assert!(cast_by(&mut g, 0, knight, &[]).is_err(), "no permission yet");
    let haakon = g.add_card_to_graveyard(0, catalog::haakon_stromgald_scourge());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: haakon,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Haakon from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(haakon).is_some());
    cast_by(&mut g, 0, knight, &[]).expect("a Knight from the graveyard");
    assert!(g.battlefield_find(knight).is_some());
}

/// Aryel taps as many Knights as the target's power (CR 602.5b "tap X").
#[test]
fn aryel_taps_knights_to_destroy() {
    let mut g = main_phase(2);
    let aryel = g.add_card_to_battlefield(0, catalog::aryel_knight_of_windgrace());
    let a = g.add_card_to_battlefield(0, catalog::worthy_knight());
    let b = g.add_card_to_battlefield(0, catalog::worthy_knight());
    for id in [aryel, a, b] {
        g.clear_sickness(id);
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::craw_wurm());
    assert!(activate(&mut g, aryel, 1, &[Target::Permanent(giant)]).is_err(), "six power needs six Knights");
    activate(&mut g, aryel, 1, &[Target::Permanent(bear)]).expect("destroy");
    assert!(g.battlefield_find(bear).is_none());
    let tapped = [a, b].iter().filter(|id| g.battlefield_find(**id).unwrap().tapped).count();
    assert_eq!(tapped, 2, "X = 2 Knights tapped");
}

/// "This turn" is one shared turn: a non-active player's draw tally resets at
/// every untap, not only at their own (Elenda and Azor counts it at each end
/// step).
#[test]
fn cards_drawn_this_turn_resets_for_every_player() {
    let mut g = main_phase(3);
    library(&mut g, 2, 6);
    draw(&mut g, 2, 2);
    assert_eq!(g.players[2].cards_drawn_this_turn, 2);
    g.active_player_idx = 1;
    g.do_untap();
    assert_eq!(g.players[2].cards_drawn_this_turn, 0, "seat 2's count is turn-scoped too");

    let elenda = g.add_card_to_battlefield(2, catalog::elenda_and_azor());
    let _ = elenda;
    draw(&mut g, 2, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 2, "Vampire Knight"), 3, "three cards drawn on another player's turn");
}

/// Knights' Charge drains each opponent 1 per attacking Knight; you gain 1.
#[test]
fn knights_charge_drains_each_opponent() {
    let mut g = main_phase(4);
    g.add_card_to_battlefield(0, catalog::knights_charge());
    let k = g.add_card_to_battlefield(0, catalog::worthy_knight());
    let life = g.players[0].life;
    declare(&mut g, vec![at(k, 2)]);
    assert_eq!(g.players[0].life, life + 1);
    assert!((1..4).all(|p| g.players[p].life == g.players[p].starting_life - 1));
}

/// Exsanguinator Cavalry: a Knight connecting grows and makes Blood.
#[test]
fn exsanguinator_rewards_knights_that_connect() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::exsanguinator_cavalry());
    let k = g.add_card_to_battlefield(0, catalog::worthy_knight());
    declare(&mut g, vec![at(k, 1)]);
    run_combat_out(&mut g);
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(named(&g, 0, "Blood"), 1);
}

/// Wintermoor Commander's toughness counts your Knights (itself included).
#[test]
fn wintermoor_commander_counts_knights() {
    let mut g = main_phase(2);
    let wc = g.add_card_to_battlefield(0, catalog::wintermoor_commander());
    assert_eq!(g.computed_permanent(wc).unwrap().toughness, 1);
    g.add_card_to_battlefield(0, catalog::worthy_knight());
    assert_eq!(g.computed_permanent(wc).unwrap().toughness, 2);
}

/// Xerex makes a Knight only after two spells this turn.
#[test]
fn xerex_needs_two_spells() {
    let mut g = main_phase(2);
    let x = g.add_card_to_battlefield(0, catalog::xerex_strobe_knight());
    g.clear_sickness(x);
    assert!(activate(&mut g, x, 0, &[]).is_err());
    for _ in 0..2 {
        let s = g.add_card_to_hand(0, catalog::llanowar_elves());
        cast_by(&mut g, 0, s, &[]).expect("cast");
    }
    activate(&mut g, x, 0, &[]).expect("two spells cast");
    assert_eq!(named(&g, 0, "Knight"), 1);
}

/// Worthy Knight and Vodalian Wave-Knight: a Knight spell makes a Human; a
/// draw grows the other Knights and Merfolk.
#[test]
fn worthy_and_wave_knight_triggers() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    g.add_card_to_battlefield(0, catalog::worthy_knight());
    let wave = g.add_card_to_battlefield(0, catalog::vodalian_wave_knight());
    let k = g.add_card_to_hand(0, catalog::acclaimed_contender());
    cast_by(&mut g, 0, k, &[]).expect("cast");
    assert_eq!(named(&g, 0, "Human"), 1, "a Knight spell");
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(wave).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "other");
}

/// Syr Elenora's power is your hand size.
#[test]
fn syr_elenora_counts_your_hand() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    let e = g.add_card_to_hand(0, catalog::syr_elenora_the_discerning());
    g.add_card_to_hand(0, catalog::plains());
    cast_by(&mut g, 0, e, &[]).expect("cast");
    let hand = g.players[0].hand.len() as i32;
    assert_eq!(g.computed_permanent(e).unwrap().power, hand, "the plains plus the drawn card");
}
