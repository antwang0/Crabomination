//! CR conformance for rules wired by this run's WAR bomb batch:
//! CR 122.3 (removing counters — the new `GameEvent::CounterRemoved` /
//! `EventKind::CounterRemoved`, exercised by Chandra, Fire Artisan's
//! loyalty-removal trigger), CR 508.4 (a creature put onto the battlefield
//! attacking is never *declared* as an attacker, so its own "when this
//! attacks" ability doesn't fire — Ilharg, the Raze-Boar), and CR 701.16
//! (mass sacrifice keeps one — Single Combat's "sacrifices the rest").

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 122.3 — removing loyalty counters is a counter-removal event that
/// "whenever counters are removed from this" triggers see, with the count as
/// the trigger amount (Chandra deals that much to an opponent).
#[test]
fn cr_122_3_counter_removal_fires_trigger_with_amount() {
    let mut g = two_player_game();
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_fire_artisan());
    let opp = g.players[1].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(chandra), 3, None, &mut evs);
    assert!(
        evs.iter().any(|e| matches!(e, GameEvent::CounterRemoved { counter_type: CounterType::Loyalty, count: 3, .. })),
        "a CounterRemoved(Loyalty, 3) event is emitted",
    );
    g.dispatch_triggers_for_events(&evs);
    while !g.stack.is_empty() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, opp - 3, "trigger dealt 3 = the loyalty removed");
}

/// CR 508.4 — a creature put onto the battlefield attacking was not *declared*
/// as an attacker, so its own on-attack ability never triggers. Rubblebelt
/// Rioters (a "whenever this attacks" pump) deployed via Ilharg stays a 0/4.
#[test]
fn cr_508_4_deployed_attacker_skips_its_own_attack_trigger() {
    let mut g = two_player_game();
    let ilharg = g.add_card_to_battlefield(0, catalog::ilharg_the_raze_boar());
    g.clear_sickness(ilharg);
    let rioters = g.add_card_to_hand(0, catalog::rubblebelt_rioters()); // 0/4, on-attack pump
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![rioters])]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ilharg, target: AttackTarget::Player(1) }])).expect("attack");
    while !g.stack.is_empty() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.attacking.iter().any(|a| a.attacker == rioters), "deployed and attacking");
    assert_eq!(g.computed_permanent(rioters).unwrap().power, 0, "its own on-attack pump did not fire (never declared)");
}

/// CR 701.16 — Single Combat: each player chooses one creature/planeswalker and
/// sacrifices the rest.
#[test]
fn cr_701_16_single_combat_keeps_one_sacrifices_rest() {
    let mut g = two_player_game();
    let keep = g.add_card_to_battlefield(0, catalog::primordial_wurm());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sc = g.add_card_to_hand(0, catalog::single_combat());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: sc, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    while !g.stack.is_empty() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let creatures: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).collect();
    assert_eq!(creatures.len(), 1, "kept exactly one");
    assert_eq!(creatures[0].id, keep, "kept the highest mana value");
}
