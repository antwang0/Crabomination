//! Functionality tests for `catalog::sets::decks::recent171` — DFT
//! commons/uncommons on existing primitives.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

/// Rover Blades grants double strike to the creature it's attached to.
#[test]
fn rover_blades_grants_double_strike() {
    let mut g = two_player_game();
    let blades = g.add_card_to_battlefield(0, catalog::rover_blades());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(blades);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: blades, target: bear })
        .expect("equip the bear");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "equipped creature has double strike");
}

/// Spotcycle Scouter is a Crew 1 Vehicle whose ETB scry doesn't disturb the
/// library size.
#[test]
fn spotcycle_scouter_etb_scry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let lib_before = g.players[0].library.len();
    let v = g.move_card_to_battlefield_for_test(0, catalog::spotcycle_scouter());
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before, "scry keeps the library size");
    assert!(g.battlefield_find(v).unwrap().definition.keywords.contains(&Keyword::Crew(1)));
}

/// Veloheart Bike gains 2 life on ETB.
#[test]
fn veloheart_bike_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::veloheart_bike());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Venomsac Lagac gets +0/+3 when it attacks while saddled.
#[test]
fn venomsac_lagac_saddled_attack_pump() {
    let mut g = two_player_game();
    let lagac = g.add_card_to_battlefield(0, catalog::venomsac_lagac());
    g.clear_sickness(lagac);
    g.battlefield_find_mut(lagac).unwrap().saddled = true;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lagac, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(lagac).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 4), "saddled attack pump +0/+3");
}

/// Stall Out taps the target and lands three stun counters.
#[test]
fn stall_out_taps_and_stuns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::stall_out());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(victim)), vec![], None, None).expect("cast Stall Out");
    drain_stack(&mut g);
    let s = g.battlefield_find(victim).unwrap();
    assert!(s.tapped, "target tapped");
    assert_eq!(s.counter_count(CounterType::Stun), 3, "three stun counters");
}

/// Trip Up tucks a nonland permanent into its owner's library.
#[test]
fn trip_up_tucks_permanent() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::trip_up());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(victim)), vec![], None, None).expect("cast Trip Up");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "left the battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == victim), "tucked into owner's library");
}

/// Spikeshell Harrier bounces an opponent's creature on ETB.
#[test]
fn spikeshell_harrier_bounces_opponent_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::spikeshell_harrier());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returned to owner's hand");
}
