//! Commander: Rohgahh Kher Keep Overlord's, Experiment Kraj's and Abigale
//! Eloquent First-Year's last gaps (`decks::cmdr_edhrec47`).

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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g);
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

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Rohgahh: a Kobold spell buys a Dragon, a Dragon spell a Kobold, and Kobolds
/// get +2/+2 — Venomous Changeling counts as one.
#[test]
fn rohgahh_trades_kobolds_and_dragons() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rohgahh_kher_keep_overlord());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let changeling = g.add_card_to_hand(0, catalog::venomous_changeling());
    cast(&mut g, changeling, None).expect("changeling");
    assert_eq!(g.computed_permanent(changeling).unwrap().power, 1 + 2, "a changeling Kobold");
    let dragons = named(&g, 0, "Dragon");
    assert_eq!(dragons.len(), 1, "a Kobold spell (and a Dragon spell: a Kobolds of Kher Keep too)");
    assert_eq!(named(&g, 0, "Kobolds of Kher Keep").len(), 1);
    assert_eq!(g.computed_permanent(named(&g, 0, "Kobolds of Kher Keep")[0]).unwrap().power, 2);
}

/// Rosnakht's battle cry pumps the other attackers.
#[test]
fn rosnakht_battle_cries() {
    let mut g = pod(2);
    let ros = g.add_card_to_battlefield(0, catalog::rosnakht_heir_of_rohgahh());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ros);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: ros, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    assert_eq!(g.computed_permanent(ros).unwrap().power, 0);
}

/// Gilder Bairn ({Q}) doubles every kind of counter on a permanent.
#[test]
fn gilder_bairn_doubles_counters() {
    let mut g = pod(2);
    let bairn = g.add_card_to_battlefield(0, catalog::gilder_bairn());
    g.clear_sickness(bairn);
    g.battlefield_find_mut(bairn).unwrap().tapped = true;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::Time, 1);
    activate(&mut g, bairn, 0, Some(Target::Permanent(bear))).expect("untap to double");
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 4);
    assert_eq!(b.counter_count(CounterType::Time), 2);
    assert!(!g.battlefield_find(bairn).unwrap().tapped);
}

/// Knacksaw Clique lets you play an opponent's exiled top card.
#[test]
fn knacksaw_clique_borrows_a_card() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    let clique = g.add_card_to_battlefield(0, catalog::knacksaw_clique());
    g.clear_sickness(clique);
    g.battlefield_find_mut(clique).unwrap().tapped = true;
    let exiled = g.exile.len();
    activate(&mut g, clique, 0, Some(Target::Player(2))).expect("knacksaw");
    assert_eq!(g.exile.len(), exiled + 1);
    assert_eq!(g.exile.last().unwrap().owner, 2);
}

/// Thornling: {G} indestructible, {1} +1/-1.
#[test]
fn thornling_shapes_itself() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::thornling());
    activate(&mut g, t, 2, None).expect("indestructible");
    activate(&mut g, t, 3, None).expect("+1/-1");
    let cp = g.computed_permanent(t).unwrap();
    assert!(cp.keywords().contains(&Keyword::Indestructible));
    assert_eq!((cp.power, cp.toughness), (5, 3));
}

/// Egon eats two graveyard cards each upkeep, or dies and draws.
#[test]
fn egon_feeds_on_the_graveyard() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let egon = g.add_card_to_battlefield(0, catalog::egon_god_of_death());
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(egon).is_some());
    assert!(g.players[0].graveyard.is_empty(), "two exiled");
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(egon).is_none(), "nothing to exile: sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Woolly Razorback thaws as it blocks: defender and no combat damage while
/// any ice counter remains.
#[test]
fn woolly_razorback_starts_frozen() {
    let mut g = pod(2);
    let boar = g.add_card_to_hand(0, catalog::woolly_razorback());
    cast(&mut g, boar, None).expect("razorback");
    let b = g.battlefield_find(boar).unwrap();
    assert_eq!(b.counter_count(CounterType::Ice), 3);
    let cp = g.computed_permanent(boar).unwrap();
    assert!(cp.keywords().contains(&Keyword::Defender));
    g.battlefield_find_mut(boar).unwrap().counters.insert(CounterType::Ice, 0);
    let cp = g.computed_permanent(boar).unwrap();
    assert!(!cp.keywords().contains(&Keyword::Defender));
}
