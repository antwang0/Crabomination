//! CR conformance for this run:
//! - CR 701.19a — a regeneration shield removes marked damage, taps, and
//!   pulls the permanent out of combat.
//! - CR 604.4 — moving an Aura/Equipment stops its static from modifying the
//!   old host and starts it on the new one.
//! - CR 611.2c — a continuous effect from a resolving spell locks its set of
//!   affected objects when it begins; a static ability's doesn't.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::effect::{Duration, Effect, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// A blocker that regenerates is tapped, healed, and out of combat — so it
/// deals no combat damage and takes none.
#[test]
fn cr_701_19a_regeneration_heals_taps_and_removes_from_combat() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let blocker = g.add_card_to_battlefield(1, catalog::gorilla_chieftain()); // 3/3
    g.clear_sickness(attacker);
    g.battlefield_find_mut(blocker).unwrap().damage = 2;
    g.battlefield_find_mut(blocker).unwrap().regeneration_shields = 1;

    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");

    let ctx = EffectContext::for_ability(blocker, 1, Some(Target::Permanent(blocker)));
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);

    let c = g.battlefield_find(blocker).expect("the shield saved it");
    assert!(c.tapped);
    assert_eq!(c.damage, 0, "marked damage is removed");
    assert!(!g.block_map.contains_key(&blocker), "removed from combat");

    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "the attacker stays blocked (CR 509.1b)");
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 0, "and took nothing");
    assert_eq!(g.computed_permanent(attacker).unwrap().toughness, 2);
    assert_eq!(g.battlefield_find(attacker).unwrap().damage, 0);
}

/// Moving an Equipment takes its bonus off the old bearer and puts it on the
/// new one — no targeting, no re-attachment trigger needed.
#[test]
fn cr_604_4_moving_an_equipment_moves_its_static() {
    let mut g = two_player_game();
    let sword = g.add_card_to_battlefield(0, catalog::bonesplitter()); // +2/+0
    let first = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let second = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(first);
    assert_eq!(g.computed_permanent(first).unwrap().power, 4);
    assert_eq!(g.computed_permanent(second).unwrap().power, 2);

    g.battlefield_find_mut(sword).unwrap().attached_to = Some(second);
    assert_eq!(g.computed_permanent(first).unwrap().power, 2, "the old bearer is plain again");
    assert_eq!(g.computed_permanent(second).unwrap().power, 4);
}

/// A resolving spell's "creatures you control get +1/+1" fixes its set of
/// creatures on resolution; an anthem static keeps picking up newcomers.
#[test]
fn cr_611_2c_a_resolved_pump_locks_its_set_but_a_static_does_not() {
    let mut g = two_player_game();
    let early = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_ability(early, 0, None);
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::ControlledBy {
                who: crabomination::effect::PlayerRef::You,
                filter: crabomination::card::SelectionRequirement::Creature,
            },
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("pump");
    assert_eq!(g.computed_permanent(early).unwrap().power, 3);

    let late = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(late).unwrap().power, 2, "it wasn't there when it resolved");

    g.add_card_to_battlefield(0, catalog::glorious_anthem());
    assert_eq!(g.computed_permanent(late).unwrap().power, 3, "the static sees it");
    assert_eq!(g.computed_permanent(early).unwrap().power, 4);
    assert!(!g.computed_permanent(late).unwrap().keywords.contains(&Keyword::Flying));
}

/// Invasion Plans hands the block declaration to the attacker; the bot on
/// that side submits only the blocks CR 509.1c forces.
#[test]
fn cr_509_1c_the_attacking_block_chooser_submits_only_forced_blocks() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::invasion_plans());
    let attacker = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flier
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::wind_drake());
    g.clear_sickness(attacker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);

    assert_eq!(g.block_chooser(), Some(0), "the attacker declares blocks");
    let blocks = crabomination::server::bot::forced_blocks_for_test(&g);
    assert_eq!(blocks, vec![(flier, attacker)], "only the creature that can block is forced in");
    let _ = ground;
    g.perform_action(GameAction::DeclareBlockers(blocks)).expect("blocks");
}
