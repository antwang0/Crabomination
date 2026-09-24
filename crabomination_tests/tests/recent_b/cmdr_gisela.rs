//! Commander: the Angels Secret Lair precon (SLD, Gisela, `decks::cmdr_gisela`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn attack_unblocked(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn melded(g: &GameState) -> Option<CardId> {
    g.battlefield.iter().find(|c| c.definition.name == "Brisela, Voice of Nightmares").map(|c| c.id)
}

/// CR 903.3 / 701.37 (Gisela's ruling) — melded with Bruna, the commander is
/// Brisela: "if you control your commander" holds, Brisela's combat damage
/// counts on Gisela's tally, and when Brisela dies only the commander card
/// may go to the command zone.
#[test]
fn a_melded_commander_is_still_the_commander() {
    let mut g = main_phase(2);
    let gisela = g.seat_commanders(0, vec![catalog::gisela_the_broken_blade()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: gisela,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast Gisela");
    drain_stack(&mut g);
    let bruna = g.add_card_to_battlefield(0, catalog::bruna_the_fading_light());
    to_end_step(&mut g);
    let brisela = melded(&g).expect("melded at the end step");
    assert!(g.battlefield_find(gisela).is_none() && g.battlefield_find(bruna).is_none());
    assert!(g.is_commander(brisela));
    assert!(g.is_own_commander_object(0, brisela), "you control your commander");

    let mut g = g;
    g.step = TurnStep::PreCombatMain;
    attack_unblocked(&mut g, &[brisela], 1);
    assert_eq!(g.commander_damage.get(&(1, gisela)).copied(), Some(9), "tallied against Gisela");

    g.remove_from_battlefield_to_graveyard_raw(brisela);
    g.check_state_based_actions();
    assert!(g.players[0].command.iter().any(|c| c.id == gisela), "Gisela goes home");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bruna), "Bruna stays in the graveyard");
}

/// Brisela locks out an opponent's cheap spells.
#[test]
fn brisela_stops_spells_of_mana_value_three_or_less() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::brisela_voice_of_nightmares());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    assert!(g
        .perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err());
}

#[test]
fn ajani_counts_creatures_and_planeswalkers_and_exiles_at_fifteen_over() {
    let mut g = main_phase(2);
    let ajani = g.add_card_to_battlefield(0, catalog::ajani_strength_of_the_pride());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ajani, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "one creature + one planeswalker");

    let mut g = main_phase(2);
    let ajani = g.add_card_to_battlefield(0, catalog::ajani_strength_of_the_pride());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rock = g.add_card_to_battlefield(1, catalog::ornithopter());
    g.players[0].life = g.players[0].starting_life + 15;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ajani, ability_index: 2, target: None, x_value: None })
        .expect("0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ajani).is_none() && g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(rock).is_none());
}

/// A creature you control connecting gains you both life; at the end step at
/// 15 over, the player it attacked loses.
#[test]
fn angel_of_destiny_ends_the_game_for_the_player_it_attacked() {
    let mut g = main_phase(3);
    let angel = g.add_card_to_battlefield(0, catalog::angel_of_destiny());
    g.players[0].life = g.players[0].starting_life + 10;
    attack_unblocked(&mut g, &[angel], 2);
    // Double strike: two 2-damage hits, each gaining both players 2.
    assert_eq!(g.players[0].life, g.players[0].starting_life + 14);
    assert_eq!(g.players[2].life, 20);
    g.players[0].life += 1;
    to_end_step(&mut g);
    assert!(!g.players[2].is_alive(), "the attacked player loses");
    assert!(g.players[1].is_alive(), "the other opponent doesn't");
}

#[test]
fn arch_of_orazca_draws_with_the_citys_blessing() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::plains());
    let arch = g.add_card_to_battlefield(0, catalog::arch_of_orazca());
    flood(&mut g, 0);
    let draw = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: arch,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(draw(&mut g).is_err(), "no blessing with two permanents");
    for _ in 0..9 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    draw(&mut g).expect("ten permanents: ascend, then draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

#[test]
fn arden_angel_climbs_back_on_a_one() {
    let mut g = main_phase(2);
    let angel = g.add_card_to_graveyard(0, catalog::arden_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)]));
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(angel).is_some());
}

/// A paired creature that dies comes back at your next upkeep.
#[test]
fn breathkeeper_seraph_brings_its_partner_back() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let seraph = g.add_card_to_hand(0, catalog::breathkeeper_seraph());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    cast(&mut g, seraph, None);
    assert_eq!(g.battlefield_find(seraph).and_then(|c| c.soulbond_partner), Some(bear), "paired");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
    // To the next turn's upkeep.
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::plains());
        }
    }
    let mut guard = 0;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Draw) && guard < 40 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        guard += 1;
    }
    assert!(g.battlefield_find(bear).is_some(), "back at your next upkeep");
}

#[test]
fn bruna_returns_an_angel_as_it_is_cast() {
    let mut g = main_phase(2);
    let dead = g.add_card_to_graveyard(0, catalog::serra_angel());
    let bruna = g.add_card_to_hand(0, catalog::bruna_the_fading_light());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, bruna, None);
    assert!(g.battlefield_find(dead).is_some());
}

#[test]
fn cosmos_elixir_draws_above_starting_life_and_heals_below() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::cosmos_elixir());
    let life = g.players[0].life;
    to_end_step(&mut g);
    assert_eq!(g.players[0].life, life + 2, "at the starting life: gain 2");
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::cosmos_elixir());
    g.players[0].life += 1;
    to_end_step(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "above it: draw");
}

/// Each side's creature card returns under its owner; the controller only
/// takes the swap when it gets at least as much as it gives.
#[test]
fn dawnbreak_reclaimer_swaps_graveyard_creatures() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::dawnbreak_reclaimer());
    let mine = g.add_card_to_graveyard(0, catalog::serra_angel());
    let theirs = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    let bigger = g.add_card_to_graveyard(1, catalog::hill_giant());
    to_end_step(&mut g);
    assert_eq!(g.battlefield_find(mine).map(|c| c.controller), Some(0));
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(2), "the cheapest opposing creature card");
    assert!(g.battlefield_find(bigger).is_none());
}

#[test]
fn keeper_of_the_accord_catches_up_on_an_opponents_end_step() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::keeper_of_the_accord());
    g.add_card_to_library(0, catalog::plains());
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::forest());
    }
    g.active_player_idx = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    to_end_step(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.is_token && c.definition.name == "Soldier"));
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Plains" && c.tapped));
}

#[test]
fn thalias_lancers_tutors_a_legend() {
    let mut g = main_phase(2);
    let legend = g.add_card_to_library(0, catalog::gisela_the_broken_blade());
    g.add_card_to_library(0, catalog::plains());
    let l = g.add_card_to_hand(0, catalog::thalias_lancers());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, l, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == legend));
}

/// CR 104.3d — the Book's Angel makes its controller unable to lose, and the
/// grant outlives the enlightened counter (the counter is a memory aid).
#[test]
fn the_book_of_exalted_deeds_keeps_you_alive() {
    let mut g = main_phase(2);
    let book = g.add_card_to_battlefield(0, catalog::the_book_of_exalted_deeds());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: book,
        ability_index: 0,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(book).is_none(), "exiled as a cost");
    assert_eq!(g.battlefield_find(angel).expect("angel").counter_count(CounterType::Enlightened), 1);
    assert!(g.permanent_has_keyword(angel, &Keyword::ControllerCantLoseGame));
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(g.players[0].is_alive());
}

#[test]
fn the_book_of_exalted_deeds_makes_an_angel_after_three_life() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::the_book_of_exalted_deeds());
    g.players[0].life_gained_this_turn = 3;
    to_end_step(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Angel"));
}

/// CR 903.9b — Brisela bounced: its cards go to hand one by one, and the
/// commander card may take the command-zone replacement; Bruna goes to hand.
#[test]
fn a_bounced_melded_commander_can_go_home() {
    let mut g = main_phase(2);
    let gisela = g.seat_commanders(0, vec![catalog::gisela_the_broken_blade()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: gisela,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast Gisela");
    drain_stack(&mut g);
    let bruna = g.add_card_to_battlefield(0, catalog::bruna_the_fading_light());
    to_end_step(&mut g);
    let brisela = melded(&g).expect("melded");
    let mut g = g;
    g.step = TurnStep::PreCombatMain;
    let bounce = g.add_card_to_hand(0, catalog::unsummon());
    cast(&mut g, bounce, Some(Target::Permanent(brisela)));
    assert!(g.players[0].command.iter().any(|c| c.id == gisela), "Gisela took the replacement");
    assert!(g.players[0].hand.iter().any(|c| c.id == bruna), "Bruna went to hand");
}
