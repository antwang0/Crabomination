//! Functionality tests for `catalog::sets::decks::recent209`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Giant Cindermaw locks life gain for every player.
#[test]
fn giant_cindermaw_locks_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::giant_cindermaw());
    assert_eq!(g.adjust_life(0, 5), g.players[0].life, "controller can't gain");
    assert_eq!(g.adjust_life(1, 5), g.players[1].life, "opponent can't gain either");
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.players[1].life, 20);
}

/// Feldon's Cane exiles itself and shuffles the graveyard back into the library.
#[test]
fn feldons_cane_recycles_graveyard() {
    let mut g = two_player_game();
    let cane = g.add_card_to_battlefield(0, catalog::feldons_cane());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cane, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Feldon's Cane");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "graveyard emptied");
    assert_eq!(g.players[0].library.len(), lib_before + 2, "two cards returned to library");
    assert!(g.exile.iter().any(|c| c.id == cane), "the Cane is exiled");
}

/// Uncharted Haven enters tapped and records the chosen color.
#[test]
fn uncharted_haven_enters_tapped_with_chosen_color() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    let land = g.move_card_to_battlefield_for_test(0, catalog::uncharted_haven());
    drain_stack(&mut g);
    let c = g.battlefield_find(land).unwrap();
    assert!(c.tapped, "enters tapped");
    assert_eq!(c.chosen_color, Some(Color::Blue), "chose blue");
}

/// Ancestor Dragon gains 1 life per attacking creature.
#[test]
fn ancestor_dragon_gains_life_per_attacker() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::ancestor_dragon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(dragon);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: dragon, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "gained 2 life for two attackers");
}

/// Jazal Goldmane pumps attackers by the number of attackers.
#[test]
fn jazal_goldmane_pumps_by_attacker_count() {
    let mut g = two_player_game();
    let jazal = g.add_card_to_battlefield(0, catalog::jazal_goldmane());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(jazal);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: jazal, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]).expect("attack");
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jazal, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    // Two attackers → +2/+2. Grizzly Bears 2/2 → 4/4.
    let bv = g.computed_permanent(bear).unwrap();
    assert_eq!((bv.power, bv.toughness), (4, 4));
}

/// Ghitu Lavarunner grows and gains haste with two spells in the graveyard.
#[test]
fn ghitu_lavarunner_threshold() {
    let mut g = two_player_game();
    let ghitu = g.add_card_to_battlefield(0, catalog::ghitu_lavarunner());
    let base = g.computed_permanent(ghitu).unwrap();
    assert_eq!((base.power, base.toughness), (1, 2));
    assert!(!base.keywords.contains(&Keyword::Haste), "no haste with empty gy");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let boosted = g.computed_permanent(ghitu).unwrap();
    assert_eq!((boosted.power, boosted.toughness), (2, 2), "+1/+0 with 2 spells");
    assert!(boosted.keywords.contains(&Keyword::Haste), "has haste");
}

/// Mystical Teachings tutors an instant to hand.
#[test]
fn mystical_teachings_tutors_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::mystical_teachings());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mystical Teachings");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt tutored to hand");
}

/// Dragon Mage wheels both hands on combat damage.
#[test]
fn dragon_mage_wheels_on_combat_damage() {
    let mut g = two_player_game();
    let dm = g.add_card_to_battlefield(0, catalog::dragon_mage());
    g.clear_sickness(dm);
    for _ in 0..10 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dm, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 7, "P0 wheeled to seven");
    assert_eq!(g.players[1].hand.len(), 7, "P1 wheeled to seven");
}

/// Time Stop ends the turn, exiling the spell beneath it on the stack.
#[test]
fn time_stop_ends_the_turn() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    let stop = g.add_card_to_hand(0, catalog::time_stop());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: stop, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Time Stop");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the bolt was exiled by the ended turn");
    assert_eq!(g.players[1].life, 20, "bolt never resolved");
}

/// Fierce Empath tutors a big creature into hand.
#[test]
fn fierce_empath_tutors_big_creature() {
    let mut g = two_player_game();
    let djinn = g.add_card_to_library(0, catalog::mahamoti_djinn()); // MV 6
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(djinn))]));
    g.move_card_to_battlefield_for_test(0, catalog::fierce_empath());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == djinn), "MV6 creature tutored");
}

/// Obliterating Bolt exiles a creature it would kill.
#[test]
fn obliterating_bolt_exiles_lethal_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::obliterating_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Obliterating Bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear gone");
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled, not in graveyard");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
}

/// Elspeth's Smite burns an attacker and exiles it.
#[test]
fn elspeths_smite_exiles_attacker() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]).expect("attack");
    let spell = g.add_card_to_hand(0, catalog::elspeths_smite());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Elspeth's Smite");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "attacker exiled");
}

/// Taurean Mauler grows whenever an opponent casts a spell.
#[test]
fn taurean_mauler_grows_on_opponent_cast() {
    let mut g = two_player_game();
    let mauler = g.add_card_to_battlefield(0, catalog::taurean_mauler());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    drain_stack(&mut g);
    assert_eq!(*g.battlefield_find(mauler).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 1);
}
