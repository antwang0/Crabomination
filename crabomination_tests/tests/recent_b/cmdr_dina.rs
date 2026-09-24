//! Commander: the Witherbloom Pestilence precon (SOC, Dina,
//! `decks::cmdr_dina`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, seat, id, target, None)
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn bolt(g: &mut GameState, target: CardId) {
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(g, 1, b, Some(Target::Permanent(target))).expect("bolt");
}

/// Dina draws on the first creature sacrifice each turn only (CR 603.2i), and
/// its ability converts the sacrificed creature's power into life and counters.
#[test]
fn dina_draws_once_and_converts_power() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let dina = g.add_card_to_battlefield(0, catalog::dina_essence_brewer());
    g.clear_sickness(dina);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    activate(&mut g, 0, dina, 0, Some(Target::Permanent(giant)), None).expect("sac a Bears");
    assert_eq!(g.players[0].life, life + 2);
    assert_eq!(g.battlefield_find(giant).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    g.battlefield_find_mut(dina).unwrap().tapped = false;
    activate(&mut g, 0, dina, 0, Some(Target::Permanent(giant)), None).expect("sac the other");
    assert_eq!(g.players[0].hand.len(), hand + 1, "once each turn");
}

/// Creakwood Liege pumps each other creature once per matching color: a
/// black-green creature gets +2/+2.
#[test]
fn creakwood_liege_counts_each_color() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::creakwood_liege());
    let gyome = g.add_card_to_battlefield(0, catalog::gyome_master_chef());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, gyome), (7, 5));
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Defiling Daemogoth drains each opponent for the turn's life gain at your
/// end step.
#[test]
fn defiling_daemogoth_drains_the_turns_gain() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::defiling_daemogoth());
    g.players[0].life_gained_this_turn = 4;
    let (a, b) = (g.players[1].life, g.players[2].life);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (a - 4, b - 4));
}

/// Eccentric Pestfinder becomes prepared at an end step after a life gain, and
/// Turn Stones makes a Pest per opponent.
#[test]
fn eccentric_pestfinder_prepares_turn_stones() {
    let mut g = pod(4);
    let pf = g.add_card_to_battlefield(0, catalog::eccentric_pestfinder());
    g.players[0].life_gained_this_turn = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pf).unwrap().counter_count(CounterType::Prepared), 1);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pf).unwrap().counter_count(CounterType::Prepared), 1, "prepared once");
    g.step = TurnStep::PreCombatMain;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastPrepareSpell { creature_id: pf, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Turn Stones");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Pest").len(), 3);
}

/// Feral Appetite exiles a graveyard card, making a Pest only for a creature.
#[test]
fn feral_appetite_pests_only_creature_cards() {
    let mut g = pod(2);
    let fa = g.add_card_to_battlefield(0, catalog::feral_appetite());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let bolt_card = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    activate(&mut g, 0, fa, 0, Some(Target::Permanent(bolt_card)), None).expect("exile the bolt");
    assert!(named(&g, 0, "Pest").is_empty());
    activate(&mut g, 0, fa, 0, Some(Target::Permanent(bear)), None).expect("exile the Bears");
    assert_eq!(named(&g, 0, "Pest").len(), 1);
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Gorma grows on each other death, and a creature cast after deaths enters
/// with that many counters.
#[test]
fn gorma_feeds_on_deaths() {
    let mut g = pod(2);
    let gorma = g.add_card_to_battlefield(0, catalog::gorma_the_gullet());
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        bolt(&mut g, b);
    }
    assert_eq!(g.battlefield_find(gorma).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    cast(&mut g, 0, giant, None).expect("cast");
    assert_eq!(g.battlefield_find(giant).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Immoral Bargain sacrifices X creatures and destroys X targets.
#[test]
fn immoral_bargain_trades_x_for_x() {
    let mut g = pod(2);
    let mine = [g.add_card_to_battlefield(0, catalog::grizzly_bears()), g.add_card_to_battlefield(0, catalog::grizzly_bears())];
    let theirs = [g.add_card_to_battlefield(1, catalog::sol_ring()), g.add_card_to_battlefield(1, catalog::serra_angel())];
    let ib = g.add_card_to_hand(0, catalog::immoral_bargain());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: ib,
        target: Some(Target::Permanent(theirs[0])),
        additional_targets: vec![Target::Permanent(theirs[1])],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(mine.iter().all(|&c| g.battlefield_find(c).is_none()), "both sacrificed");
    assert!(theirs.iter().all(|&c| g.battlefield_find(c).is_none()), "both destroyed");
}

/// Merchant of Venom makes each player sacrifice, and grows per sacrifice.
#[test]
fn merchant_of_venom_grows_on_every_sacrifice() {
    let mut g = pod(3);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let mv = g.add_card_to_hand(0, catalog::merchant_of_venom());
    cast(&mut g, 0, mv, None).expect("cast");
    assert!(named(&g, 1, "Grizzly Bears").is_empty() && named(&g, 2, "Grizzly Bears").is_empty());
    let c = g.battlefield_find(mv).map(|c| c.counter_count(CounterType::PlusOnePlusOne));
    // The Merchant is the caster's only creature, so it sacrificed itself.
    assert!(c.is_none() || c == Some(2), "{c:?}");
}

/// Nether Traitor returns for {B} when another creature of yours dies.
#[test]
fn nether_traitor_returns_when_another_dies() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let nt = g.add_card_to_graveyard(0, catalog::nether_traitor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    bolt(&mut g, bear);
    assert!(g.battlefield_find(nt).is_some());
}

/// Pest Rescuer adds 1 to each life gain and refills a Pest at upkeep.
#[test]
fn pest_rescuer_tops_up_pests_and_life() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::pest_rescuer());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Pest").len(), 1);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Pest").len(), 1, "already has one");
    let life = g.players[0].life;
    let pest = named(&g, 0, "Pest")[0];
    bolt(&mut g, pest);
    assert_eq!(g.players[0].life, life + 2);
}

/// Ribtruss Roaster devours and makes a Pest per counter at your end step.
#[test]
fn ribtruss_roaster_devours_into_pests() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rr = g.add_card_to_hand(0, catalog::ribtruss_roaster());
    cast(&mut g, 0, rr, None).expect("cast");
    let n = g.battlefield_find(rr).unwrap().counter_count(CounterType::PlusOnePlusOne) as usize;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Pest").len(), n);
}

/// Stensian Sanguinist gives deathtouch on attack and prepares when that
/// creature connects; Exsanguinate drains every opponent.
#[test]
fn stensian_sanguinist_prepares_exsanguinate() {
    let mut g = pod(3);
    let ss = g.add_card_to_battlefield(0, catalog::stensian_sanguinist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Deathtouch));
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(ss).unwrap().counter_count(CounterType::Prepared), 1);
    let (life, a, b) = (g.players[0].life, g.players[1].life, g.players[2].life);
    flood(&mut g, 0);
    g.perform_action(GameAction::CastPrepareSpell { creature_id: ss, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("Exsanguinate");
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (a - 3, b - 3));
    assert_eq!(g.players[0].life, life + 6);
}

/// Turbulent Fen enters tapped unless opponents have eight lands.
#[test]
fn turbulent_fen_reads_opponents_lands() {
    let mut g = pod(2);
    let fen = g.add_card_to_hand(0, catalog::turbulent_fen());
    g.perform_action(GameAction::PlayLand(fen)).expect("land");
    assert!(g.battlefield_find(fen).unwrap().tapped);
}

/// Witch of the Moors, after a life gain: each opponent sacrifices, and a
/// creature card comes back to your hand.
#[test]
fn witch_of_the_moors_punishes_and_recurs() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::witch_of_the_moors());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    g.players[0].life_gained_this_turn = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(named(&g, 1, "Grizzly Bears").is_empty() && named(&g, 2, "Grizzly Bears").is_empty());
    assert!(g.players[0].hand.iter().any(|c| c.id == dead));
}
