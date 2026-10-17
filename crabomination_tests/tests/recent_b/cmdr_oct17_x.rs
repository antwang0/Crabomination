//! Commander: the 2026-10-17 (`3chree`) fifth closer batch
//! (`decks::cmdr_oct17_x`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Selector, Value};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
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
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn attack_unblocked(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

fn named_count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}

/// Ant-Man's Army's second mode makes a Treasure, not a Food.
#[test]
fn ant_mans_army_makes_a_treasure() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let army = g.add_card_to_hand(0, catalog::ant_mans_army());
    flood(&mut g, 0);
    cast(&mut g, army, None).expect("army");
    assert_eq!(named_count(&g, 0, "Treasure"), 1);
    assert_eq!(named_count(&g, 0, "Food"), 0);
}

/// Bob attacking alone bounces himself and drains each opponent for 2.
#[test]
fn bob_attacks_alone_bounces_and_drains() {
    let mut g = pod(2);
    let bob = ready(&mut g, 0, catalog::bob_reluctant_hydra_agent());
    attack_unblocked(&mut g, vec![(bob, 1)]);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == bob),
        "Bob back in hand"
    );
    assert_eq!(g.players[1].life, 18, "drained 2, no combat damage");
    assert_eq!(g.players[0].life, 22);
}

/// Stolen Stark Tech attaches on entry and makes the creature indestructible.
#[test]
fn stolen_stark_tech_attaches_and_protects() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let tech = g.add_card_to_hand(0, catalog::stolen_stark_tech());
    flood(&mut g, 0);
    cast(&mut g, tech, None).expect("tech");
    assert_eq!(
        g.battlefield_find(tech).and_then(|c| c.attached_to),
        Some(bear)
    );
    assert!(g.permanent_has_keyword(bear, &Keyword::Indestructible));
    assert_eq!(pt(&g, bear), (3, 2));
}

/// The Death of Gwen Stacy: I destroys a creature; II costs a handless player 3 life.
#[test]
fn death_of_gwen_stacy_destroys_then_taxes_empty_hands() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let saga = g.add_card_to_hand(0, catalog::the_death_of_gwen_stacy());
    flood(&mut g, 0);
    cast(&mut g, saga, None).expect("saga");
    assert!(g.battlefield_find(bear).is_none(), "chapter I");
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Upkeep);
    g.players[1].hand.clear();
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert_eq!(g.players[1].life, 17, "no card to discard");
}

/// Roxxon Brutes: only the second draw of the turn adds a counter.
#[test]
fn roxxon_brutes_grows_on_second_draw() {
    let mut g = pod(2);
    let brutes = ready(&mut g, 0, catalog::roxxon_brutes());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let draw = Effect::Draw {
        who: Selector::You,
        amount: Value::ONE,
    };
    for n in 0..2 {
        let evs = g.resolve_effect(&draw, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let counters = g
            .battlefield_find(brutes)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(counters, n, "after draw {}", n + 1);
    }
}

/// Whiplash enters tapped; an equipped attack drains per Equipment.
#[test]
fn whiplash_drains_per_equipment() {
    let mut g = pod(2);
    let wl = g.add_card_to_hand(0, catalog::whiplash_vengeful_engineer());
    flood(&mut g, 0);
    cast(&mut g, wl, None).expect("whiplash");
    assert!(g.battlefield_find(wl).unwrap().tapped, "enters tapped");
    g.battlefield_find_mut(wl).unwrap().tapped = false;
    g.clear_sickness(wl);
    let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield_find_mut(eq).unwrap().attached_to = Some(wl);
    attack_unblocked(&mut g, vec![(wl, 1)]);
    assert_eq!(g.players[0].life, 21, "drained 1");
    assert_eq!(g.players[1].life, 20 - 1 - 4, "drain + 4 combat");
}

/// Winter Soldier returns from the graveyard with a finality counter and gets
/// +2/+0 per attached Equipment.
#[test]
fn winter_soldier_returns_and_scales_with_equipment() {
    let mut g = pod(2);
    let ws = g.add_card_to_graveyard(0, catalog::winter_soldier_icy_assassin());
    flood(&mut g, 0);
    activate(&mut g, ws, None).expect("return");
    let c = g.battlefield_find(ws).expect("on battlefield");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
    let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield_find_mut(eq).unwrap().attached_to = Some(ws);
    assert_eq!(
        pt(&g, ws),
        (6, 2),
        "2 base + 2 Winter Soldier + 2 Bonesplitter"
    );
}

/// Ms. Marvel draws only once per turn for pumped attackers that connect.
#[test]
fn ms_marvel_draws_once_per_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ms_marvel_elastic_ally());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    for id in [a, b] {
        g.battlefield_find_mut(id)
            .unwrap()
            .counters
            .insert(CounterType::PlusOnePlusOne, 1);
    }
    let before = g.players[0].hand.len();
    attack_unblocked(&mut g, vec![(a, 1), (b, 1)]);
    assert_eq!(g.players[1].life, 14);
    assert_eq!(g.players[0].hand.len(), before + 1, "only once each turn");
}

/// Bereaved Survivor transforms when another creature dies; Dauntless Avenger
/// brings the dead 2-drop back attacking.
#[test]
fn bereaved_survivor_transforms_and_reanimates_attacking() {
    let mut g = pod(2);
    let surv = ready(&mut g, 0, catalog::bereaved_survivor());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    cast(&mut g, m, Some(Target::Permanent(bear))).expect("murder");
    assert!(g.battlefield_find(surv).unwrap().transformed);
    attack_unblocked(&mut g, vec![(surv, 1)]);
    assert!(g.battlefield_find(bear).is_some(), "bear returned");
    assert_eq!(g.players[1].life, 15, "3 + 2 combat");
}

/// Skinshifter's Rhino mode makes a 4/4 trampler; a second activation fails.
#[test]
fn skinshifter_rhino_once_per_turn() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::skinshifter());
    flood(&mut g, 0);
    let act = |g: &mut GameState, mode: usize| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: s,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: Some(mode),
        })
    };
    act(&mut g, 0).expect("rhino");
    drain_stack(&mut g);
    assert_eq!(pt(&g, s), (4, 4));
    let c = g.computed_permanent(s).unwrap();
    assert!(c.keywords().contains(&Keyword::Trample));
    assert!(
        c.subtypes()
            .creature_types
            .contains(&crabomination::card::CreatureType::Rhino)
    );
    assert!(act(&mut g, 2).is_err(), "only once each turn");
}

/// Turning Belltoll Dragon face up pumps the other Dragons.
#[test]
fn belltoll_dragon_unmorph_pumps_dragons() {
    let mut g = pod(2);
    let shivan = ready(&mut g, 0, catalog::shivan_dragon());
    let bd = g.add_card_to_hand(0, catalog::belltoll_dragon());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: bd })
        .expect("face down");
    drain_stack(&mut g);
    flood(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: bd })
        .expect("face up");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bd)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1,
        "megamorph"
    );
    assert_eq!(
        g.battlefield_find(shivan)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Robot Domination: three creature cards hitting the graveyard draw three,
/// cost 3 life, then trade the Plan for three Robot Villains.
#[test]
fn robot_domination_completes_its_plan() {
    let mut g = pod(2);
    let rd = ready(&mut g, 0, catalog::robot_domination());
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let m = g.add_card_to_hand(0, catalog::murder());
        flood(&mut g, 0);
        cast(&mut g, m, Some(Target::Permanent(bear))).expect("murder");
    }
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].life, 17);
    assert!(g.battlefield_find(rd).is_none(), "sacrificed");
    assert_eq!(named_count(&g, 0, "Robot Villain"), 3);
}

/// Armor Wars I: draw per artifact, each opponent draws one.
#[test]
fn armor_wars_chapter_one_draws_per_artifact() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::ornithopter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let aw = g.add_card_to_hand(0, catalog::armor_wars());
    flood(&mut g, 0);
    cast(&mut g, aw, None).expect("saga");
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Portal Manipulator: flashed in during declare attackers, it turns an
/// attack aimed at its controller onto another player (not a new attack).
#[test]
fn portal_manipulator_redirects_an_attack() {
    let mut g = pod(3);
    step_to(&mut g, 1, TurnStep::DeclareAttackers);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    let pm = g.add_card_to_hand(0, catalog::portal_manipulator());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: pm, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("flash");
    drain_stack(&mut g);
    let target = g.attacking.iter().find(|a| a.attacker == bear).map(|a| a.target);
    assert_eq!(target, Some(AttackTarget::Player(2)));
}
