//! Functionality tests for `catalog::sets::decks::recent25` — Duskmourn (DSK)
//! "Fear of …" Nightmare cycle, Eerie/attack/dies triggers.

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::TurnStep;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Sacrifice a battlefield permanent, firing dies triggers (CR 701.16).
fn kill(g: &mut GameState, id: CardId) {
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crate::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crate::effect::Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    drain_stack(g);
}

/// Attack unblocked with `atk` at player 1, dealing combat damage.
fn attack_unblocked(g: &mut GameState, atk: CardId) {
    g.clear_sickness(atk);
    advance_to(g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(g);
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("resolve combat");
    drain_stack(g);
}

/// Fear of Failed Tests draws cards equal to combat damage dealt to a player.
#[test]
fn fear_of_failed_tests_draws_on_hit() {
    let mut g = two_player_game();
    let fft = g.add_card_to_battlefield(0, catalog::fear_of_failed_tests()); // 2 power
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    attack_unblocked(&mut g, fft);
    assert_eq!(g.players[0].hand.len(), before + 2, "drew 2 = combat damage");
}

/// Fear of Surveillance has vigilance and stays untapped when it attacks.
#[test]
fn fear_of_surveillance_vigilant() {
    let mut g = two_player_game();
    let fos = g.add_card_to_battlefield(0, catalog::fear_of_surveillance());
    g.add_card_to_library(0, catalog::grizzly_bears());
    assert!(catalog::fear_of_surveillance().keywords.contains(&Keyword::Vigilance));
    attack_unblocked(&mut g, fos);
    assert!(!g.battlefield_find(fos).unwrap().tapped, "vigilance keeps it untapped");
}

/// Fear of Being Hunted ships with haste and must-be-blocked.
#[test]
fn fear_of_being_hunted_keywords() {
    let def = catalog::fear_of_being_hunted();
    assert!(def.keywords.contains(&Keyword::Haste));
    assert!(def.keywords.contains(&Keyword::MustBeBlocked));
}

/// Fear of Immobility taps and stuns an opponent's creature on ETB.
#[test]
fn fear_of_immobility_taps_and_stuns() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fi = g.add_card_to_battlefield(0, catalog::fear_of_immobility());
    g.fire_self_etb_triggers(fi, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(foe).unwrap();
    assert!(c.tapped, "opponent creature tapped");
    assert!(c.counters.get(&CounterType::Stun).copied().unwrap_or(0) >= 1, "stun counter added");
}

/// Flesh Burrower grants deathtouch to another of your creatures when it attacks.
#[test]
fn flesh_burrower_grants_deathtouch_on_attack() {
    let mut g = two_player_game();
    let fb = g.add_card_to_battlefield(0, catalog::flesh_burrower());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack_unblocked(&mut g, fb);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch),
        "ally gained deathtouch"
    );
}

/// Hardened Escort pumps and grants indestructible to another attacker.
#[test]
fn hardened_escort_pumps_ally() {
    let mut g = two_player_game();
    let he = g.add_card_to_battlefield(0, catalog::hardened_escort());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    attack_unblocked(&mut g, he);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0 ally");
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Infernal Phantom deals damage equal to its power to a player when it dies.
#[test]
fn infernal_phantom_pings_on_death() {
    let mut g = two_player_game();
    let ip = g.add_card_to_battlefield(0, catalog::infernal_phantom()); // power 2
    let before = g.players[1].life;
    // Sacrifice to fire the dies trigger; auto-target hits the opponent's face.
    kill(&mut g, ip);
    assert_eq!(g.players[1].life, before - 2, "dies ping = power 2");
}

/// Lionheart Glimmer ships with Ward {2} and pumps the team when you attack.
#[test]
fn lionheart_glimmer_team_pump() {
    let mut g = two_player_game();
    let lg = g.add_card_to_battlefield(0, catalog::lionheart_glimmer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    assert!(matches!(catalog::lionheart_glimmer().keywords[0], Keyword::Ward(_)));
    attack_unblocked(&mut g, lg);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "team +1/+1 → 3 power");
}

/// Anthropede destroys a target Room when you pay {2} on ETB.
#[test]
fn anthropede_destroys_room() {
    let mut g = two_player_game();
    let room = g.add_card_to_battlefield(1, catalog::unholy_annex_ritual_chamber());
    let ant = g.add_card_to_battlefield(0, catalog::anthropede());
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true), // pay {2}
    ]));
    g.fire_self_etb_triggers(ant, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(room).is_none(), "Room destroyed");
}

/// Living Phone digs for a small creature when it dies.
#[test]
fn living_phone_digs_on_death() {
    let mut g = two_player_game();
    let lp = g.add_card_to_battlefield(0, catalog::living_phone());
    g.add_card_to_library(0, catalog::grizzly_bears()); // 2/2 — power 2, eligible
    let hand_before = g.players[0].hand.len();
    kill(&mut g, lp);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "took the small creature");
}

/// Demonic Counsel tutors a Demon to hand (no delirium).
#[test]
fn demonic_counsel_finds_demon() {
    let mut g = two_player_game();
    let dc = g.add_card_to_hand(0, catalog::demonic_counsel());
    let demon = g.add_card_to_library(0, catalog::bloodgift_demon());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(demon)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: dc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Demonic Counsel");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == demon), "Demon tutored to hand");
}
