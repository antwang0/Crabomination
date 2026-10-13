//! Commander: Zuko's and Keleth + Yoshimaru's last gaps
//! (`decks::cmdr_edhrec49`).

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

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Zuko makes a Soldier per red spell; Ghitu Firebreathing pumps and comes
/// back to hand.
#[test]
fn zuko_and_ghitu_firebreathing() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::zuko_avatar_hunter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let breath = g.add_card_to_hand(0, catalog::ghitu_firebreathing());
    cast(&mut g, breath, Some(Target::Permanent(bear))).expect("aura");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count(), 1, "a red spell");
    activate(&mut g, breath, 0, None).expect("firebreathe");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    activate(&mut g, breath, 1, None).expect("return");
    assert!(g.players[0].hand.iter().any(|c| c.id == breath));
}

/// CR 903.3 — Keleth counts only a commander you control attacking;
/// Yoshimaru adds one to each +1/+1 placement.
#[test]
fn cr_903_3_keleth_grows_attacking_commanders() {
    let mut g = pod(2);
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let card = g.players[0].command.remove(pos);
    g.battlefield.push(card);
    g.clear_sickness(cmd);
    let other = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(other);
    g.add_card_to_battlefield(0, catalog::keleth_sunmane_familiar());
    g.add_card_to_battlefield(0, catalog::yoshimaru_beloved_companion());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: cmd, target: AttackTarget::Player(1) },
        Attack { attacker: other, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cmd).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "one plus Yoshimaru's");
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Elspeth Resplendent: +1 is a +1/+1 counter and a chosen keyword counter;
/// −3 puts a cheap permanent onto the battlefield with a shield counter.
#[test]
fn elspeth_resplendent_counters_and_shields() {
    let mut g = pod(2);
    let elspeth = g.add_card_to_battlefield(0, catalog::elspeth_resplendent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(0)]));
    loyalty(&mut g, elspeth, 0, Some(Target::Permanent(bear))).expect("+1");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::Flying));
    g.battlefield_find_mut(elspeth).unwrap().loyalty_uses_this_turn = 0;
    g.add_card_to_library(0, catalog::grizzly_bears());
    loyalty(&mut g, elspeth, 1, None).expect("-3");
    let fresh = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears" && c.id != bear).map(|c| c.id).next();
    let fresh = fresh.expect("the dug-up bear");
    assert_eq!(g.battlefield_find(fresh).unwrap().counter_count(CounterType::Shield), 1);
}
