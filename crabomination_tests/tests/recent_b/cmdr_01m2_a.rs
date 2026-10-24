//! Commander: the 2026-10-24 (`01M2Ffkh`) Isu the Abominable closers
//! (`decks::cmdr_01m2_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, GameError, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Isu plays a snow land and casts a snow spell off the top of the library
/// (never a non-snow one), and a snow arrival buys a +1/+1 counter for the
/// chosen colour.
#[test]
fn isu_plays_snow_off_the_top_and_grows() {
    let mut g = pod(2);
    let isu = ready(&mut g, 0, catalog::isu_the_abominable());
    let land = g.add_card_to_library(0, catalog::snow_covered_forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0), DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::PlayLand(land)).expect("a snow land off the top");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some());
    assert_eq!(g.battlefield_find(isu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "paid {{G}}");

    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(cast(&mut g, bears).is_err(), "not a snow spell");
    g.players[0].library.retain(|c| c.id != bears);
    let yeti = g.add_card_to_library(0, catalog::frostpeak_yeti());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2), DecisionAnswer::Bool(false)]));
    cast(&mut g, yeti).expect("a snow spell off the top");
    assert!(g.battlefield_find(yeti).is_some());
    assert_eq!(g.battlefield_find(isu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "declined");
}

/// Diamond Faerie's {1}{S} pumps snow creatures only, paid from snow lands.
#[test]
fn diamond_faerie_pumps_only_snow_creatures() {
    let mut g = pod(2);
    let faerie = ready(&mut g, 0, catalog::diamond_faerie());
    let yeti = ready(&mut g, 0, catalog::frostpeak_yeti());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::snow_covered_island());
    }
    activate(&mut g, faerie).expect("{1}{S} from two snow lands");
    assert_eq!(pt(&g, faerie), (4, 4));
    assert_eq!(pt(&g, yeti), (4, 4));
    assert_eq!(pt(&g, bears), (2, 2));
}

/// Snow Devil: flying always; first strike only while blocking with a snow
/// land under its controller.
#[test]
fn snow_devil_first_strike_needs_a_block_and_a_snow_land() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let devil = g.add_card_to_battlefield(0, catalog::snow_devil());
    g.battlefield_find_mut(devil).unwrap().attached_to = Some(bear);
    let has = |g: &GameState, k: Keyword| g.computed_permanent(bear).unwrap().keywords().contains(&k);
    assert!(has(&g, Keyword::Flying));
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::snow_covered_island());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(!has(&g, Keyword::FirstStrike), "not blocking yet");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, attacker)])).expect("block");
    drain_stack(&mut g);
    assert!(has(&g, Keyword::FirstStrike));
}

/// Glacial Plating: +3/+3 for each age counter on it.
#[test]
fn glacial_plating_scales_with_age_counters() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let plating = g.add_card_to_battlefield(0, catalog::glacial_plating());
    g.battlefield_find_mut(plating).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (2, 2));
    g.battlefield_find_mut(plating).unwrap().add_counters(CounterType::Age, 2);
    assert_eq!(pt(&g, bear), (8, 8));
}

/// CR 615 — Cover of Winter prevents X of each attacking creature's combat
/// damage, X its age counters.
#[test]
fn cover_of_winter_shaves_creature_combat_damage() {
    let mut g = pod(2);
    let cover = g.add_card_to_battlefield(0, catalog::cover_of_winter());
    g.battlefield_find_mut(cover).unwrap().add_counters(CounterType::Age, 1);
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    let life = g.players[0].life;
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    assert_eq!(g.players[0].life, life - 1, "2 power, 1 prevented");
}
