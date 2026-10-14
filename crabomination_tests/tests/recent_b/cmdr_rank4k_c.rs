//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_c`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

/// Eye of Jace surveils each upkeep and pops at seven cards in the graveyard.
#[test]
fn eye_of_jace_pops_at_seven() {
    let mut g = pod(3);
    let eye = ready(&mut g, 0, catalog::eye_of_jace());
    g.add_card_to_library(0, catalog::island());
    step(&mut g, TurnStep::Upkeep);
    assert!(g.battlefield_find(eye).is_some());
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::island());
    step(&mut g, TurnStep::Upkeep);
    assert!(g.battlefield_find(eye).is_none());
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
    assert_eq!(g.players[0].life, 22);
}

/// Tarox Bladewing's grandeur doubles its power.
#[test]
fn tarox_grandeur_doubles_power() {
    let mut g = pod(2);
    let tarox = ready(&mut g, 0, catalog::tarox_bladewing());
    g.add_card_to_hand(0, catalog::tarox_bladewing());
    activate(&mut g, tarox, 0, None);
    let c = g.computed_permanent(tarox).unwrap();
    assert_eq!((c.power, c.toughness), (8, 7));
}

/// CR 702.87-style Saga flip: Behold the Unspeakable's chapter III returns
/// it as a Spirit sized by your hand.
#[test]
fn behold_the_unspeakable_becomes_its_vision() {
    let mut g = pod(2);
    let saga = g.add_card_to_hand(0, catalog::behold_the_unspeakable());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    cast(&mut g, saga, None, None);
    assert_eq!(g.computed_permanent(foe).unwrap().power, 0, "chapter I");
    for _ in 0..2 {
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let lore = Effect::AddCounter {
            what: Selector::ExactObjects(vec![saga]),
            kind: CounterType::Lore,
            amount: Value::ONE,
        };
        let evs = g.resolve_effect(&lore, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let vision = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Vision of the Unspeakable")
        .map(|c| c.id)
        .expect("transformed");
    let hand = g.players[0].hand.len() as i32;
    assert!(hand >= 4);
    assert_eq!(g.computed_permanent(vision).unwrap().power, hand);
}

/// Spore Cloud fogs combat and freezes the combatants.
#[test]
fn spore_cloud_freezes_the_combatants() {
    let mut g = pod(2);
    let att = ready(&mut g, 1, catalog::grizzly_bears());
    let blk = ready(&mut g, 0, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: att,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, att)])).expect("block");
    drain_stack(&mut g);
    let cloud = g.add_card_to_hand(0, catalog::spore_cloud());
    cast(&mut g, cloud, None, None);
    assert!(g.battlefield_find(blk).unwrap().tapped);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(att).is_some() && g.battlefield_find(blk).is_some());
}

/// Does Machines at level 3 animates a noncreature artifact into a 3/3.
#[test]
fn does_machines_animates_an_artifact() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let class = g.add_card_to_hand(0, catalog::does_machines());
    cast(&mut g, class, None, None);
    activate(&mut g, class, 0, None);
    activate(&mut g, class, 1, None);
    step(&mut g, TurnStep::BeginCombat);
    let c = g.computed_permanent(ring).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3));
}

/// Quantum Entanglement's reflexive blink resets a creature's counters.
#[test]
fn quantum_entanglement_blinks() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let qe = g.add_card_to_hand(0, catalog::quantum_entanglement());
    cast(&mut g, qe, None, None);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .expect("back");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Halsin makes a token a green 4/4 Bear in addition to its colors.
#[test]
fn halsin_bears_a_token() {
    let mut g = pod(2);
    let halsin = ready(&mut g, 0, catalog::halsin_emerald_archdruid());
    let kiora = g.add_card_to_hand(0, catalog::kiora_of_fire_and_ashes());
    cast(&mut g, kiora, None, None);
    let dragon = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Dragon")
        .map(|c| c.id)
        .expect("Dragon");
    activate(&mut g, halsin, 0, Some(Target::Permanent(dragon)));
    let c = g.computed_permanent(dragon).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.colors.contains(Color::Green) && c.colors.contains(Color::Red));
}
