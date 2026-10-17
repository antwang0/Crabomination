//! Commander: the 2026-10-17 (`yl8zjb`) two-card-gap closers
//! (`decks::cmdr_oct17_kn`).

use crabomination::card::{CardId, CounterType};
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

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })?;
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

fn shock(g: &mut GameState, at: CardId) {
    let s = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_with(g, s, Some(Target::Permanent(at)), None).expect("shock");
}

/// Elektra's reflexive trigger: the 2 to you buys 4 to a creature.
#[test]
fn elektra_pays_two_life_for_four_damage() {
    let mut g = pod(2);
    let ogre = ready(&mut g, 1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::elektra_femme_fatale());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_with(&mut g, e, None, None).expect("elektra");
    assert_eq!(g.players[0].life, life - 2);
    assert!(g.battlefield_find(ogre).is_none(), "4 damage kills the 2/2");
}

/// Increasing Confusion mills X from hand and 2X when flashed back.
#[test]
fn increasing_confusion_doubles_from_graveyard() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::increasing_confusion());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_with(&mut g, c, Some(Target::Player(1)), Some(2)).expect("cast");
    assert_eq!(g.players[1].graveyard.len(), 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastFlashback {
        card_id: c,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 6, "2 + twice 2");
}

/// Merfolk Pupil loots again from the graveyard, exiling itself.
#[test]
fn merfolk_pupil_loots_from_graveyard() {
    let mut g = pod(2);
    let p = g.add_card_to_graveyard(0, catalog::merfolk_pupil());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: p,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("graveyard loot");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != p), "exiled as a cost");
    assert_eq!(g.players[0].graveyard.len(), 1, "the discarded card");
}

/// Zalto ventures each time it's dealt damage; Venus grows by the damage.
#[test]
fn zalto_ventures_and_venus_grows_when_damaged() {
    let mut g = pod(2);
    let zalto = ready(&mut g, 0, catalog::zalto_fire_giant_duke());
    let venus = ready(&mut g, 0, catalog::venus_torn_between_worlds());
    shock(&mut g, zalto);
    assert!(g.players[0].dungeon.is_some(), "ventured");
    shock(&mut g, venus);
    assert_eq!(g.battlefield_find(venus).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Sentinel taps a creature of the active opponent at their beginning of
/// combat, and does nothing on its controller's own turn.
#[test]
fn sentinel_taps_on_each_opponents_combat() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::sentinel_of_the_eternal_watch());
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    step_to(&mut g, 2, TurnStep::DeclareAttackers);
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped at seat 2's combat");
}

/// Hollow Trees banks a storage counter each upkeep it stays tapped and
/// cashes them for {G} each.
#[test]
fn hollow_trees_stores_while_tapped() {
    let mut g = pod(2);
    let trees = ready(&mut g, 0, catalog::hollow_trees());
    g.battlefield_find_mut(trees).unwrap().tapped = true;
    g.battlefield_find_mut(trees).unwrap().add_counters(CounterType::Storage, 2);
    g.battlefield_find_mut(trees).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: trees,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("cash out");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}

/// CR 506.2 — Jurin pumps each attacker by the creatures of the player *it*
/// attacks, not Jurin's defender.
#[test]
fn jurin_pumps_each_attacker_by_its_own_defender() {
    let mut g = pod(3);
    let jurin = ready(&mut g, 0, catalog::jurin_leading_the_charge());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    ready(&mut g, 2, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: jurin, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let p = |g: &GameState, id| g.computed_permanent(id).unwrap().power;
    assert_eq!(p(&g, jurin), 4 + 3);
    assert_eq!(p(&g, bear), 2 + 1);
}
