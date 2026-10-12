//! Commander: one-card EDHREC gaps of EDHREC's top-600 commanders
//! (`decks::cmdr_edhrec8`).

use crabomination::card::{CardId, CardType, CounterType};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

fn activate_free(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn creature_def(name: &'static str, p: i32, t: i32, kind: crabomination::card::CreatureType) -> crabomination::card::CardDefinition {
    crabomination::card::CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: crabomination::card::Subtypes { creature_types: vec![kind], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Preston, the Vanisher: a creature put onto the battlefield (not cast) is
/// copied as a 0/1 white Illusion (CR 707.9b); a cast one is not.
#[test]
fn preston_copies_uncast_arrivals_as_illusions() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::preston_the_vanisher());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]).expect("cast");
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 0, "a cast creature isn't copied");
    let conjurer = ready(&mut g, 0, catalog::distinguished_conjurer());
    activate(&mut g, conjurer, 0, Some(perm(bear))).expect("flicker the Bears");
    let illusion = g.battlefield.iter().find(|c| c.is_token).expect("an Illusion copy").id;
    let cp = g.computed_permanent(illusion).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1));
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Illusion));
}

/// Plagon: ETB draws one per creature of yours with toughness above power.
#[test]
fn plagon_draws_for_walls() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    ready(&mut g, 0, creature_def("Wall A", 0, 4, crabomination::card::CreatureType::Wall));
    ready(&mut g, 0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let p = g.add_card_to_hand(0, catalog::plagon_lord_of_the_beach());
    cast(&mut g, p, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the Wall and Plagon itself (0/3)");
}

/// Coppercoat Vanguard: other Humans get +1/+0.
#[test]
fn coppercoat_vanguard_leads_humans() {
    let mut g = pod(2);
    let cv = ready(&mut g, 0, catalog::coppercoat_vanguard());
    let human = ready(&mut g, 0, catalog::distinguished_conjurer());
    assert_eq!(g.computed_permanent(human).unwrap().power, 2);
    assert_eq!(g.computed_permanent(cv).unwrap().power, 2, "not itself");
}

/// Court of Ardenvale: ETB monarch; upkeep returns a cheap permanent card to
/// the battlefield while you're the monarch.
#[test]
fn court_of_ardenvale_returns_while_monarch() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::court_of_ardenvale());
    cast(&mut g, c, &[]).expect("cast");
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the monarch's return goes to the battlefield");
}

/// Torens: a creature spell makes a Human Soldier.
#[test]
fn torens_makes_soldiers() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::torens_fist_of_the_angels());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]).expect("cast");
    assert_eq!(tokens_named(&g, "Human Soldier"), 1);
}

/// Engulf the Shore bounces creatures with toughness at most your Islands.
#[test]
fn engulf_the_shore_bounces_by_islands() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let small = ready(&mut g, 1, catalog::grizzly_bears());
    let big = ready(&mut g, 1, creature_def("Big", 3, 3, crabomination::card::CreatureType::Beast));
    let e = g.add_card_to_hand(0, catalog::engulf_the_shore());
    cast(&mut g, e, &[]).expect("cast");
    assert!(g.battlefield_find(small).is_none() && g.battlefield_find(big).is_some());
}

/// Solid Ground: earthbend makes a land a creature; +1/+1 counters come one
/// extra (CR 614.1a).
#[test]
fn solid_ground_adds_a_counter() {
    let mut g = pod(2);
    let land = ready(&mut g, 0, catalog::forest());
    let s = g.add_card_to_hand(0, catalog::solid_ground());
    cast(&mut g, s, &[]).expect("cast");
    drain_stack(&mut g);
    let _ = land;
    let lands_with_counters: Vec<u32> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Forest")
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .collect();
    assert_eq!(lands_with_counters, vec![4], "earthbend 3 plus Solid Ground's own extra counter");
}

/// Cracked Earth Technique: two earthbends and 3 life.
#[test]
fn cracked_earth_technique_bends_twice() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::forest());
    ready(&mut g, 0, catalog::forest());
    let life = g.players[0].life;
    let c = g.add_card_to_hand(0, catalog::cracked_earth_technique());
    cast(&mut g, c, &[]).expect("cast");
    assert_eq!(g.players[0].life, life + 3);
    // CR 115.3 — each earthbend has its own target; the same land may be
    // chosen for both.
    let counters: u32 = g.battlefield.iter().map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(counters, 6, "two earthbend 3s");
}

/// Verdant Kraken: every player's upkeep makes you a Forest Tentacle.
#[test]
fn verdant_kraken_grows_tentacles_each_upkeep() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::verdant_kraken());
    for seat in [1, 2] {
        g.active_player_idx = seat;
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Forest Tentacle" && c.controller == 0).count(), 2);
}

/// Animation Module: {3}, {T}: one more counter of a kind the target has.
#[test]
fn animation_module_adds_a_present_kind() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::animation_module());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    activate(&mut g, m, 0, Some(perm(bear))).expect("activate");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Evereth: sacrificing a Treasure grows it and grants lifelink.
#[test]
fn evereth_feeds_on_treasure() {
    let mut g = pod(2);
    let e = ready(&mut g, 0, catalog::evereth_viceroy_of_plunder());
    g.add_card_to_battlefield(0, crabomination::card::CardDefinition {
        name: "Treasure",
        card_types: vec![CardType::Artifact],
        subtypes: crabomination::card::Subtypes {
            artifact_subtypes: vec![crabomination::card::ArtifactSubtype::Treasure],
            ..Default::default()
        },
        ..Default::default()
    });
    activate(&mut g, e, 0, None).expect("sacrifice the Treasure");
    let p = g.battlefield_find(e).unwrap();
    assert_eq!(p.counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(e).unwrap().keywords().contains(&crabomination::card::Keyword::Lifelink));
}

/// Fetch Quest: mill seven, then a creature card among them enters.
#[test]
fn fetch_quest_puts_a_milled_creature_onto_the_battlefield() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let _ = bear;
    let bf = g.add_card_to_hand(0, catalog::bramble_familiar());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAdventure { card_id: bf, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("adventure");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Talent of the Telepath: cast an instant from the opponent's top seven;
/// the rest go to their graveyard.
#[test]
fn talent_of_the_telepath_steals_a_spell() {
    let mut g = pod(2);
    g.add_card_to_library(1, catalog::divination());
    library(&mut g, 1, 10);
    library(&mut g, 0, 3);
    let t = g.add_card_to_hand(0, catalog::talent_of_the_telepath());
    let before = g.players[1].library.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, t, &[Target::Player(1)]).expect("cast");
    assert_eq!(g.players[1].library.len(), before - 7);
    assert_eq!(g.players[1].graveyard.iter().filter(|c| c.definition.name == "Island").count(), 6);
}

/// Distinguished Conjurer: another creature entering gains 1.
#[test]
fn distinguished_conjurer_gains_on_arrivals() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::distinguished_conjurer());
    let life = g.players[0].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]).expect("cast");
    assert_eq!(g.players[0].life, life + 1);
}
