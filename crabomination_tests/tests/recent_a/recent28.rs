//! Functionality tests for `catalog::sets::decks::recent28` — Duskmourn (DSK)
//! commons on existing primitives, plus the new "whenever you attack"
//! once-per-combat trigger and the recovered recent24/recent orphans.

use crabomination::catalog;
use crabomination::card::Keyword;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;
use crabomination::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

/// Piggy Bank leaves a Treasure behind when it dies.
#[test]
fn piggy_bank_dies_makes_treasure() {
    let mut g = two_player_game();
    let pig = g.add_card_to_battlefield(0, catalog::piggy_bank());
    g.remove_to_graveyard_with_triggers(pig);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Treasure"), 1, "a Treasure on death");
}

/// Razorkin Hordecaller's "whenever you attack" fires once per combat,
/// regardless of how many attackers are declared (CR 508, new YouAttack event).
#[test]
fn razorkin_you_attack_fires_once_per_combat() {
    let mut g = two_player_game();
    let raz = g.add_card_to_battlefield(0, catalog::razorkin_hordecaller());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(raz);
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: raz, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Gremlin"), 1, "exactly one Gremlin for two attackers");
}

/// Appendage Amalgam has flash and a surveil-on-attack trigger.
#[test]
fn appendage_amalgam_flash() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::appendage_amalgam());
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2));
    assert!(cp.keywords.contains(&Keyword::Flash));
}

/// Gremlin Tamer (recovered orphan) mints a Gremlin when an enchantment you
/// control enters — the Eerie ability word.
#[test]
fn gremlin_tamer_eerie_makes_gremlin() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gremlin_tamer());
    let ench = g.add_card_to_battlefield(0, catalog::sticky_fingers());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Gremlin"), 1, "Eerie made a Gremlin");
}

/// Shepherding Spirits plainscycles to fetch a Plains.
#[test]
fn shepherding_spirits_plainscycles() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::shepherding_spirits());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Landcycle { card_id: id }).expect("plainscycle");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "fetched a Plains");
}

/// Seized from Slumber costs {3} less when it targets a tapped creature.
#[test]
fn seized_from_slumber_cheaper_vs_tapped() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == foe).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::seized_from_slumber());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // {4}{W} − {3} = {1}{W} when the target is tapped.
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast at reduced cost");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "tapped creature destroyed");
}

/// Manifest Dread puts a face-down 2/2 onto the battlefield.
#[test]
fn manifest_dread_spell_makes_face_down() {
    let mut g = two_player_game();
    for _ in 0..3 {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::manifest_dread_spell());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Manifest Dread");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.face_down), "a face-down creature");
}

/// Impossible Inferno deals 6 to a creature and, with delirium, exiles the top
/// card with a play permission.
#[test]
fn impossible_inferno_burns_and_impulses_with_delirium() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Four card types in graveyard → delirium active.
    for c in [catalog::lightning_bolt(), catalog::grizzly_bears(), catalog::ornithopter(), catalog::sticky_fingers()] {
        let id = g.next_id();
        g.players[0].send_to_graveyard(crabomination::card::CardInstance::new(id, c, 0));
    }
    let topid = g.next_id();
    g.players[0].add_to_library_top(topid, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::impossible_inferno());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "6 damage killed the bear");
    assert!(g.exile.iter().any(|c| c.id == topid && c.may_play_until.is_some()), "delirium exiled top with may-play");
}

/// Break Down the Door's first mode exiles a target artifact.
#[test]
fn break_down_the_door_exiles_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::ornithopter());
    let spell = g.add_card_to_hand(0, catalog::break_down_the_door());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(art)), additional_targets: vec![],
        mode: Some(0), x_value: None,
    }).expect("cast mode 0");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == art), "artifact exiled");
}

/// Found Footage sacrifices for surveil-2-then-draw.
#[test]
fn found_footage_sac_draws() {
    let mut g = two_player_game();
    for _ in 0..4 {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    }
    let clue = g.add_card_to_battlefield(0, catalog::found_footage());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: clue, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert!(!g.battlefield.iter().any(|c| c.id == clue), "Clue sacrificed");
}

/// Fear of Lost Teeth (recovered orphan) pings and gains life on death.
#[test]
fn fear_of_lost_teeth_dies_pings() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::fear_of_lost_teeth());
    let start = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.remove_to_graveyard_with_triggers(f);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 1, "1 damage to opponent");
    assert_eq!(g.players[0].life, start + 1, "gained 1 life");
}
