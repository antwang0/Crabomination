//! Radha's, Martok's, Dr. Pulaski's and Sela's decks (`decks::cmdr_01wr2y_a` / `_b`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on battlefield");
    (c.power, c.toughness)
}

/// Veteran's Voice taps Radha for its cost; Radha's domain trigger (three
/// basic types: +3/+3) and the Voice (+2/+1, never the tapped host) both land
/// on the other creature.
#[test]
fn veterans_voice_taps_radha_for_both_pumps() {
    let mut g = pod(3);
    for land in [catalog::forest(), catalog::mountain(), catalog::island()] {
        g.add_card_to_battlefield(0, land);
    }
    let radha = g.add_card_to_battlefield(0, catalog::radha_coalition_warlord());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let voice = g.add_card_to_hand(0, catalog::veterans_voice());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, voice, Some(Target::Permanent(radha)), None);
    activate(&mut g, voice, Some(Target::Permanent(bears))).expect("tap Radha");
    assert!(g.battlefield_find(radha).is_some_and(|c| c.tapped));
    assert_eq!(pt(&g, bears), (7, 6));
    assert_eq!(pt(&g, radha), (3, 3));
}

/// CR 302.6 — "Tap an untapped creature you control" isn't {T}: a
/// summoning-sick Bramblesnap taps itself for +1/+1.
#[test]
fn cr_302_6_bramblesnap_taps_itself_while_summoning_sick() {
    let mut g = pod(2);
    let snap = g.add_card_to_battlefield(0, catalog::bramblesnap());
    activate(&mut g, snap, None).expect("tap itself");
    assert!(g.battlefield_find(snap).is_some_and(|c| c.tapped));
    assert_eq!(pt(&g, snap), (2, 2));
    assert!(activate(&mut g, snap, None).is_err(), "no untapped creature left");
}

/// Encounter the Unknown's second mode: 2 damage to each creature and each
/// player in a four-seat pod.
#[test]
fn encounter_the_unknown_shoot_to_kill_hits_every_seat() {
    let mut g = pod(4);
    let bears = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::encounter_the_unknown());
    g.players[0].mana_pool.add(Color::Red, 3);
    let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    cast(&mut g, spell, None, Some(1));
    assert!(g.players.iter().zip(before).all(|(p, b)| p.life == b - 2));
    assert!(g.battlefield_find(bears).is_none());
}

/// 70,000 Light-Years from Home at four seats: the exiled creature's
/// controller (seat 2), not the caster, searches for the basic.
#[test]
fn seventy_thousand_light_years_gives_the_basic_to_the_exiled_creatures_controller() {
    let mut g = pod(4);
    let bears = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.add_card_to_library(2, catalog::plains());
    let spell = g.add_card_to_hand(0, catalog::seventy_thousand_light_years_from_home());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, spell, Some(Target::Permanent(bears)), None);
    assert!(g.exile.iter().any(|c| c.id == bears));
    assert!(g.players[2].hand.iter().any(|c| c.definition.name == "Plains"));
    assert!(g.players[0].hand.is_empty());
}

/// Salt Vampire: the chosen nonland card is exiled and its owner draws a
/// replacement; the land stays in hand.
#[test]
fn salt_vampire_exiles_a_nonland_card_and_its_owner_draws() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::mountain());
    g.add_card_to_library(1, catalog::island());
    let vamp = g.add_card_to_hand(0, catalog::salt_vampire());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, vamp, None, None);
    assert!(g.exile.iter().any(|c| c.id == bolt));
    let names: Vec<&str> = g.players[1].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Mountain") && names.contains(&"Island"));
}

/// A Good Day to Die: +2/+0, and when that creature dies this turn its
/// caster draws.
#[test]
fn a_good_day_to_die_draws_when_the_creature_dies() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::a_good_day_to_die());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, spell, Some(Target::Permanent(bears)), None);
    assert_eq!(pt(&g, bears), (4, 2));
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bears)), None);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Island"));
}

/// Type-2 Phaser's entry taps an opposing creature and stuns it (CR 122.1d).
#[test]
fn type_2_phaser_taps_and_stuns_on_entry() {
    use crabomination::card::CounterType;
    let mut g = pod(3);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let phaser = g.add_card_to_hand(0, catalog::type_2_phaser());
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, phaser, None, None);
    let b = g.battlefield_find(bears).unwrap();
    assert!(b.tapped);
    assert_eq!(b.counter_count(CounterType::Stun), 1);
}

/// Worf's cost taps two untapped artifacts and/or creatures you control —
/// Worf himself may be one, summoning sick or not (no {T} symbol, CR 302.6).
#[test]
fn worf_taps_himself_and_an_artifact_for_a_counter() {
    let mut g = pod(2);
    let worf = g.add_card_to_battlefield(0, catalog::worf_honor_bound_warrior());
    let rock = g.add_card_to_battlefield(0, catalog::sol_ring());
    activate(&mut g, worf, None).expect("tap two");
    assert!(g.battlefield_find(worf).is_some_and(|c| c.tapped));
    assert!(g.battlefield_find(rock).is_some_and(|c| c.tapped));
    assert_eq!(pt(&g, worf), (2, 2));
}

/// Shields Up! on a creature: indestructible through a Doom Blade-style
/// destroy and a +1/+1 counter; on a noncreature artifact, no counter.
#[test]
fn shields_up_counters_only_a_creature() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rock = g.add_card_to_battlefield(0, catalog::sol_ring());
    for (id, counters) in [(bears, 1), (rock, 0)] {
        let s = g.add_card_to_hand(0, catalog::shields_up());
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::White, 2);
        cast(&mut g, s, Some(Target::Permanent(id)), None);
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), counters);
    }
    let blade = g.add_card_to_hand(1, catalog::murder());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 3);
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: blade,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "hexproof or indestructible keeps it");
}

/// The Measure of a Man: chapter I exiles an opponent's creature, chapter III
/// returns it under its owner's control (and the Saga's controller gains 2).
#[test]
fn the_measure_of_a_man_returns_the_exiled_card_to_its_owner() {
    use crabomination::game::effects::EffectContext;
    let mut g = pod(3);
    let saga = g.add_card_to_battlefield(0, catalog::the_measure_of_a_man());
    let bears = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let def = catalog::the_measure_of_a_man();
    let chapter = |n: u32| def.saga_chapters.iter().find(|(k, _)| *k == n).unwrap().1.clone();
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(bears)), 0, 0);
    ctx.source = Some(saga);
    g.resolve_effect(&chapter(1), &ctx).expect("I");
    assert!(g.exile.iter().any(|c| c.id == bears));
    let life = g.players[0].life;
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(saga);
    g.resolve_effect(&chapter(3), &ctx).expect("III");
    assert!(g.battlefield_find(bears).is_some_and(|c| c.controller == 2));
    assert_eq!(g.players[0].life, life + 2);
}

/// CR 500.7 — H.M.S. Bounty's combat-damage trigger (station 8+) takes an
/// extra turn only while you control a Whale and haven't begun one this game;
/// beginning the extra turn is what spends it.
#[test]
fn cr_500_7_hms_bounty_takes_an_extra_turn_once_per_game() {
    use crabomination::game::effects::EffectContext;
    let mut g = pod(3);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let ship = g.add_card_to_battlefield(0, catalog::h_m_s_bounty_unlikely_timeship());
    let def = catalog::h_m_s_bounty_unlikely_timeship();
    let warp = def.station[0].triggers[0].effect.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(ship);
    g.resolve_effect(&warp, &ctx).expect("no Whale");
    assert_eq!(g.players[0].extra_turns, 0, "no Whale, no extra turn");
    g.add_card_to_battlefield(0, catalog::flying_dolphin_fish());
    g.resolve_effect(&warp, &ctx).expect("Whale");
    assert_eq!(g.players[0].extra_turns, 1);
    // End the turn: seat 0 begins the extra turn.
    let turn = g.turn_number;
    while g.turn_number == turn {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.current_turn_is_extra && g.active_player_idx == 0);
    assert_eq!(g.players[0].extra_turns_begun, 1);
    g.resolve_effect(&warp, &ctx).expect("second hit");
    assert_eq!(g.players[0].extra_turns, 0, "one extra turn a game");
}

/// Chekov and McCoy draws as many cards as the counters you put on it, once
/// a turn: a second Shields Up! the same turn draws nothing.
#[test]
fn chekov_and_mccoy_draws_per_counter_once_each_turn() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let cm = g.add_card_to_battlefield(0, catalog::chekov_and_mccoy());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    for drawn in [1, 1] {
        let s = g.add_card_to_hand(0, catalog::shields_up());
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::White, 2);
        cast(&mut g, s, Some(Target::Permanent(cm)), None);
        assert_eq!(g.players[0].hand.len(), drawn);
    }
}

/// Zefram Cochrane: a Spacecraft you control enters with two more charge
/// counters (U.S.S. Cerritos, cast from hand).
#[test]
fn zefram_cochrane_spacecraft_enter_with_two_charge_counters() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::zefram_cochrane_warp_pioneer());
    let ship = g.add_card_to_hand(0, catalog::u_s_s_cerritos_following_up());
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, ship, None, None);
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 2);
}

/// U.S.S. Defiant phases out with up to one creature you control (CR 702.26).
#[test]
fn uss_defiant_phases_out_with_a_creature() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ship = g.add_card_to_hand(0, catalog::u_s_s_defiant_mirror_migrant());
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, ship, None, None);
    assert!(g.battlefield_find(ship).is_none() && g.battlefield_find(bears).is_none());
    assert!(g.phased_out.iter().any(|c| c.id == ship) && g.phased_out.iter().any(|c| c.id == bears));
}
