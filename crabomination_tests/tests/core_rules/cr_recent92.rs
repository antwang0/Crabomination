//! CR conformance for this run:
//! - CR 508.1a / 509.1b — a board-state attack/block restriction is checked
//!   against the *defending* (resp. attacking) player, not the whole table.
//! - CR 122.5 — moving counters is relocation, not creation, so a
//!   counter-doubling replacement doesn't apply.
//! - CR 611.2c — a "for as long as this Aura is attached" control effect ends
//!   the moment the Aura comes off.
//! - CR 607.2b — a linked exile-and-return pair only returns the cards the
//!   linked ability itself exiled.
//! - CR 705.2 — one coin flip per iteration of a repeated effect.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 508.1a — the land-lead restriction compares against the defending
/// player only; a third seat's lands are irrelevant.
#[test]
fn cr_508_1a_land_restriction_reads_the_defending_player() {
    let mut g = two_player_game();
    let hound = ready(&mut g, 0, catalog::monstrous_hound());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let attack = |g: &mut GameState| {
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: hound,
            target: AttackTarget::Player(1),
        }]))
    };
    assert!(attack(&mut g).is_err(), "one land each is not a lead");
    g.add_card_to_battlefield(0, catalog::forest());
    attack(&mut g).expect("2 > 1");
}

/// CR 509.1b — the mirror restriction on blocking reads the attacking
/// player's land count.
#[test]
fn cr_509_1b_land_restriction_reads_the_attacking_player() {
    let mut g = two_player_game();
    let hound = ready(&mut g, 1, catalog::monstrous_hound());
    let attacker = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(hound, attacker)])).is_err(),
        "level on lands"
    );
    g.add_card_to_battlefield(1, catalog::forest());
    g.perform_action(GameAction::DeclareBlockers(vec![(hound, attacker)])).expect("2 > 1");
}

/// CR 122.5 — Doubling Season doesn't apply to counters that are *moved*: the
/// pile Spike Cannibal absorbs arrives at its original size.
#[test]
fn cr_122_5_moving_counters_is_not_creating_them() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doubling_season());
    let donor = g.add_card_to_battlefield_with_counters(1, catalog::spike_rogue());
    assert_eq!(g.battlefield_find(donor).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let etb = catalog::spike_cannibal().triggered_abilities[0].effect.clone();
    let cannibal = g.add_card_to_battlefield(0, catalog::spike_cannibal());
    let ctx = crabomination::game::effects::EffectContext::for_ability(cannibal, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    assert_eq!(
        g.battlefield_find(cannibal).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "moved, not doubled"
    );
}

/// CR 611.2c — the Licid's steal is scoped to the Aura staying attached, so
/// paying its end cost hands the creature straight back.
#[test]
fn cr_611_2c_control_ends_when_the_aura_detaches() {
    let mut g = two_player_game();
    let licid = ready(&mut g, 0, catalog::dominating_licid());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: licid,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("attach");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0);

    g.battlefield_find_mut(licid).unwrap().attached_to = None;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "handed back");
}

/// CR 607.2b — Wall of Nets' return ability is linked to its own exile, so a
/// card exiled by something else stays exiled when the Wall leaves.
#[test]
fn cr_607_2b_linked_return_only_takes_its_own_exiles() {
    let mut g = two_player_game();
    let wall = ready(&mut g, 0, catalog::wall_of_nets());
    let blocked = ready(&mut g, 1, catalog::grizzly_bears());
    let stranger = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.move_card_to(
        stranger,
        &crabomination::effect::ZoneDest::Exile,
        &crabomination::game::effects::EffectContext::for_ability(wall, 0, None),
        &mut events,
    );

    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: blocked,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, blocked)])).expect("block");
    advance_to(&mut g, TurnStep::End);
    assert!(g.battlefield_find(blocked).is_none(), "the Wall's victim is exiled");

    let mut events = Vec::new();
    g.destroy_permanent(wall, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocked).is_some(), "linked return");
    assert!(g.battlefield_find(stranger).is_none(), "not linked to the Wall");
}

/// CR 705.2 — a per-object repeated flip makes one flip per object, so two
/// blockers get two independent results.
#[test]
fn cr_705_2_one_flip_per_blocking_creature() {
    let mut g = two_player_game();
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    // A 6/6 (0/0 + six counters) survives one blocker so the damage is
    // readable.
    let attacker = g.add_card_to_battlefield_with_counters(0, catalog::spike_hatcher());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(a, attacker), (b, attacker)]))
        .expect("gang block");
    // Win the first flip, lose the second: only the first blocker is fogged.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    let spell = g.add_card_to_hand(0, catalog::fighting_chance());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("fighting chance");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(attacker).unwrap().damage,
        2,
        "one blocker was fogged, the other wasn't"
    );
}
