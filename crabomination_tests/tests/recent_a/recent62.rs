//! Functionality tests for `catalog::sets::decks::recent62` — Kaladesh
//! artifacts / vehicles / pilots.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;

fn servos(g: &GameState, p: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == p && c.definition.name == "Servo").count()
}

#[test]
fn servo_schematic_makes_servo_on_enter_and_death() {
    let mut g = two_player_game();
    let id = g.move_card_to_battlefield_for_test(0, catalog::servo_schematic());
    drain_stack(&mut g);
    assert_eq!(servos(&g, 0), 1, "one Servo on enter");
    // Destroy it → a second Servo on leaving the battlefield.
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(servos(&g, 0), 2, "second Servo when put into the graveyard");
}

#[test]
fn cogworkers_puzzleknot_etb_and_sac() {
    let mut g = two_player_game();
    let id = g.move_card_to_battlefield_for_test(0, catalog::cogworkers_puzzleknot());
    drain_stack(&mut g);
    assert_eq!(servos(&g, 0), 1, "ETB Servo");
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac ability");
    drain_stack(&mut g);
    assert_eq!(servos(&g, 0), 2, "sac makes a second Servo");
    assert!(g.battlefield_find(id).is_none(), "the Puzzleknot was sacrificed");
}

#[test]
fn renegade_freighter_pumps_and_tramples_on_attack() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::renegade_freighter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(veh);
    g.clear_sickness(bear);
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![bear] }).expect("crew");
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: veh, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == veh).unwrap();
    assert_eq!((v.power, v.toughness), (5, 4), "4/3 → 5/4 on attack");
    assert!(v.keywords.contains(&Keyword::Trample), "gains trample");
}

#[test]
fn bomat_bazaar_barge_draws_on_enter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    let id = g.add_card_to_battlefield(0, catalog::bomat_bazaar_barge());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on enter");
}

#[test]
fn peema_outrider_fabricate_makes_servo() {
    let mut g = two_player_game();
    // Fabricate mode 1 = create Servos instead of the +1/+1 counter.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.move_card_to_battlefield_for_test(0, catalog::peema_outrider());
    drain_stack(&mut g);
    assert_eq!(servos(&g, 0), 1, "fabricate 1 minted a Servo");
}

#[test]
fn deadeye_harpooner_destroys_tapped_creature_with_revolt() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    // Trigger revolt: a permanent left the battlefield under our control.
    let sac = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(sac);
    drain_stack(&mut g);
    let dh = g.add_card_to_battlefield(0, catalog::deadeye_harpooner());
    g.fire_self_etb_triggers(dh, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "revolt destroyed the tapped creature");
}

#[test]
fn deadeye_harpooner_no_revolt_no_destroy() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let dh = g.add_card_to_battlefield(0, catalog::deadeye_harpooner());
    g.fire_self_etb_triggers(dh, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_some(), "no revolt → nothing destroyed");
}

#[test]
fn gearshift_ace_grants_first_strike_on_crew() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::renegade_freighter());
    let ace = g.add_card_to_battlefield(0, catalog::gearshift_ace());
    g.clear_sickness(ace);
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![ace] }).expect("crew");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(veh).unwrap().keywords.contains(&Keyword::FirstStrike),
        "the crewed Vehicle gains first strike",
    );
}

#[test]
fn veteran_motorist_scries_and_pumps_crewed_vehicle() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::renegade_freighter());
    let vm = g.add_card_to_battlefield(0, catalog::veteran_motorist());
    g.clear_sickness(vm);
    g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![vm] }).expect("crew");
    drain_stack(&mut g);
    let v = g.compute_battlefield().into_iter().find(|c| c.id == veh).unwrap();
    assert_eq!((v.power, v.toughness), (5, 4), "crewed Vehicle got +1/+1");
}

#[test]
fn aether_chaser_energy_then_servo_on_attack() {
    let mut g = two_player_game();
    let ac = g.move_card_to_battlefield_for_test(0, catalog::aether_chaser());
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "ETB gave two energy");
    g.clear_sickness(ac);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ac, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(servos(&g, 0), 1, "paid {{E}}{{E}} for a Servo");
    assert_eq!(g.players[0].energy, 0, "energy spent");
}
