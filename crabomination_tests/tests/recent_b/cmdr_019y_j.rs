//! Commander: the 2026-10-17 (`019Y6J4R`) tenth closer batch
//! (`decks::cmdr_019y_j`).

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




















fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}



/// Lightning Reaver stores charge on connecting and burns each opponent for
/// it at your end step.
#[test]
fn lightning_reaver_burns_for_its_charge() {
    let mut g = pod(3);
    let lr = ready(&mut g, 0, catalog::lightning_reaver());
    g.battlefield_find_mut(lr).unwrap().add_counters(CounterType::Charge, 2);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
}

/// Ghoulraiser returns the only Zombie card from your graveyard.
#[test]
fn ghoulraiser_returns_a_zombie() {
    let mut g = pod(2);
    let z = g.add_card_to_graveyard(0, catalog::ghoulraiser());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let gr = g.add_card_to_hand(0, catalog::ghoulraiser());
    flood(&mut g, 0);
    cast(&mut g, gr, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == z));
}

/// Craftwork Crusher's default pair: 4 damage to a creature and a card.
#[test]
fn craftwork_crusher_bolts_and_draws() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let cc = g.add_card_to_hand(0, catalog::craftwork_crusher());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, cc, None).expect("cast");
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// Head of the Hunt exiles a dying opposing creature and makes a Wolf.
#[test]
fn head_of_the_hunt_exiles_and_makes_a_wolf() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::head_of_the_hunt());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Wolf"));
}

/// Robotics Mastery: +2/+2 and two flying Robots.
#[test]
fn robotics_mastery_builds_two_robots() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let rm = g.add_card_to_hand(0, catalog::robotics_mastery());
    flood(&mut g, 0);
    cast(&mut g, rm, Some(Target::Permanent(bear))).expect("cast");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Robot").count(), 2);
}

/// Anointer of Valor sells a counter to an attacker for {3}.
#[test]
fn anointer_of_valor_counters_an_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::anointer_of_valor());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Utrom Scientists taps and stuns an opposing creature.
#[test]
fn utrom_scientists_stun_a_creature() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let us = g.add_card_to_hand(0, catalog::utrom_scientists());
    flood(&mut g, 0);
    cast(&mut g, us, None).expect("cast");
    let c = g.battlefield_find(giant).unwrap();
    assert!(c.tapped && c.counter_count(CounterType::Stun) == 1);
}

/// Suspension Field holds a big creature until it leaves.
#[test]
fn suspension_field_holds_a_big_creature() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let sf = g.add_card_to_hand(0, catalog::suspension_field());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, sf, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == giant));
}

/// Crackdown Construct grows when you activate a creature's ability.
#[test]
fn crackdown_construct_grows_on_activations() {
    let mut g = pod(2);
    let cc = ready(&mut g, 0, catalog::crackdown_construct());
    let hh = ready(&mut g, 0, catalog::hallway_heckler());
    g.add_card_to_hand(0, catalog::forest());
    activate(&mut g, hh, 0, None).expect("loot");
    let cp = g.computed_permanent(cc).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Norwood Priestess puts a green creature into play only before attackers.
#[test]
fn norwood_priestess_cheats_a_green_creature() {
    let mut g = pod(2);
    let np = ready(&mut g, 0, catalog::norwood_priestess());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    activate(&mut g, np, 0, None).expect("main phase");
    assert!(g.battlefield_find(bear).is_some());
    g.battlefield_find_mut(np).unwrap().tapped = false;
    g.step = TurnStep::DeclareBlockers;
    assert!(activate(&mut g, np, 0, None).is_err(), "too late");
}

/// Kree Commandos and Hallway Heckler's prepared spell.
#[test]
fn hallway_heckler_casts_vicious_verse() {
    let mut g = pod(2);
    let hh = g.add_card_to_hand(0, catalog::hallway_heckler());
    flood(&mut g, 0);
    cast(&mut g, hh, None).expect("cast");
    let life = g.players[1].life;
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: hh,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("prepared");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
    let k = ready(&mut g, 0, catalog::kree_commandos());
    assert!(g.computed_permanent(k).unwrap().keywords().contains(&Keyword::Vigilance));
}
