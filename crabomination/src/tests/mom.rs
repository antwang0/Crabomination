//! Functionality tests for the March of the Machine **Battle — Siege** batch
//! and the CR 310 battle rules (defense counters, protector, attack-your-own,
//! defeat → transform).

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, TurnStep};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Seed a battle the way its ETB would (3 defense, protected by the opponent),
/// returning the battle id. A bare-bones 6/6 attacker for player 0 is also
/// created and de-sicked.
fn seeded_battle_and_attacker(g: &mut GameState, def_fac: fn() -> crate::card::CardDefinition,
    defense: u32) -> (CardId, CardId) {
    let battle = g.add_card_to_battlefield(0, def_fac());
    {
        let b = g.battlefield_find_mut(battle).unwrap();
        b.counters.insert(CounterType::Defense, defense);
        b.protected_by = Some(1);
    }
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    (battle, atk)
}

/// A Siege cast from hand enters with its printed defense counters, a protector
/// (the lone opponent), and resolves its ETB (Zendikar ramps two basics).
#[test]
fn siege_enters_with_defense_and_protector() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    let battle = g.add_card_to_hand(0, catalog::invasion_of_zendikar());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: battle, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Invasion of Zendikar");
    drain_stack(&mut g);
    let b = g.battlefield_find(battle).expect("battle on the battlefield");
    assert_eq!(b.counter_count(CounterType::Defense), 3, "enters with 3 defense");
    assert_eq!(b.protected_by, Some(1), "the lone opponent protects it");
    let forests = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Forest").count();
    assert_eq!(forests, 2, "ETB ramped two basics onto the battlefield");
}

/// Attacking your own battle removes defense counters equal to the damage; a
/// non-lethal hit leaves it in play with fewer counters.
#[test]
fn attacking_own_battle_removes_defense() {
    let mut g = two_player_game();
    let (battle, atk) = seeded_battle_and_attacker(&mut g, catalog::invasion_of_zendikar, 5);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Battle(battle),
    }])).expect("attack your own Siege");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("opponent declines to block");
    advance_to(&mut g, TurnStep::PostCombatMain);
    let b = g.battlefield_find(battle).expect("battle survives a non-lethal hit");
    assert_eq!(b.counter_count(CounterType::Defense), 3, "2 damage removed 2 of 5 defense");
}

/// Removing the last defense counter defeats the Siege: it's exiled and its
/// transformed back face enters under its controller's control.
#[test]
fn defeated_siege_transforms_to_back() {
    let mut g = two_player_game();
    let (battle, atk) = seeded_battle_and_attacker(&mut g, catalog::invasion_of_zendikar, 2);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Battle(battle),
    }])).expect("attack your own Siege");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    advance_to(&mut g, TurnStep::PostCombatMain);
    // The defeated Siege is exiled and re-enters transformed (same engine id,
    // a new game object): the back face is now what's on the battlefield.
    let back = g.battlefield_find(battle).expect("the transformed back face is in play");
    assert_eq!(back.definition.name, "Awakened Skyclave", "transformed into its back face");
    assert!(back.transformed, "showing the back face");
    assert_eq!(back.controller, 0, "under the Siege controller's control");
    assert!(back.summoning_sick, "entered as a new object");
}

/// CR 508.4 — a player can't attack a battle they protect.
#[test]
fn cannot_attack_battle_you_protect() {
    let mut g = two_player_game();
    let battle = g.add_card_to_battlefield(0, catalog::invasion_of_zendikar());
    {
        let b = g.battlefield_find_mut(battle).unwrap();
        b.counters.insert(CounterType::Defense, 3);
        b.protected_by = Some(0); // the active player protects it
    }
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Battle(battle),
    }]));
    assert!(res.is_err(), "you can't attack a battle you protect");
}
