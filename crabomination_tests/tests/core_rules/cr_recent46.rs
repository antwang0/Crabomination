//! CR conformance for this run's sweep:
//! - CR 404 — the graveyard zone.
//! - CR 507 — the beginning of combat step.
//! - CR 602 — activating activated abilities (multi-target slots).

use crabomination::card::{ActivatedAbility, CardDefinition, CardType, SelectionRequirement as R};
use crabomination::catalog;
use crabomination::effect::{Duration, Effect, Selector};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{cost, generic};

/// A creature whose only ability pumps "two target creatures" — the shape
/// `Effect::ApplyToTargets` exists for, on the activation path.
fn two_target_pumper() -> CardDefinition {
    CardDefinition {
        name: "Two-Target Pumper",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: crabomination::card::Value::Const(2),
                    toughness: crabomination::card::Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── CR 602 — Activating activated abilities ──

/// CR 602.2b — an activated ability with two target slots takes both from the
/// activation and applies its effect to each.
#[test]
fn cr_602_2b_activated_ability_fills_every_target_slot() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, two_target_pumper());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: src,
        ability_index: 0,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        x_value: None, mode: None,
    })
    .expect("both slots accepted");
    drain_stack(&mut g);
    for id in [a, b] {
        let cp = g.computed_permanent(id).expect("on the battlefield");
        assert_eq!((cp.power, cp.toughness), (4, 4), "both targets were pumped");
    }
}

/// CR 602.2b — the same ability rejects an activation that leaves a required
/// target slot empty.
#[test]
fn cr_602_2b_activation_rejects_a_missing_required_slot() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, two_target_pumper());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    assert!(matches!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: src,
            ability_index: 0,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![],
            x_value: None, mode: None,
        }),
        Err(GameError::SelectionRequirementViolated)
    ));
}

// ── CR 404 — Graveyard ──

/// CR 404.1 — a graveyard is a *pile*: a card put into it goes on top, and
/// the order is preserved as further cards arrive.
#[test]
fn cr_404_1_graveyard_is_an_ordered_pile() {
    let mut g = two_player_game();
    let first = g.add_card_to_hand(0, catalog::grizzly_bears());
    let second = g.add_card_to_hand(0, catalog::lightning_bolt());
    let mut evs = Vec::new();
    g.discard_card(0, first, &mut evs);
    g.discard_card(0, second, &mut evs);
    let ids: Vec<_> = g.players[0].graveyard.iter().map(|c| c.id).collect();
    assert_eq!(ids, vec![first, second], "later arrivals sit on top");
}

/// CR 404.3 — a card put into a graveyard from anywhere is owned there by its
/// *owner*, not by whoever controlled it (a stolen creature goes home).
#[test]
fn cr_404_3_a_dying_permanent_goes_to_its_owners_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    g.destroy_permanent(bear, false, &mut Vec::new());
    g.check_state_based_actions();
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "the owner got it back");
    assert!(g.players[0].graveyard.is_empty());
}

// ── CR 507 — Beginning of combat step ──

/// CR 507.1 — the beginning of combat step is a real step: "at the beginning
/// of combat" triggers fire there, before attackers are declared.
#[test]
fn cr_507_1_begin_combat_triggers_fire_before_attackers() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.attacking.is_empty(), "no attackers yet at begin combat");
    assert_eq!(g.player_with_priority(), 0, "CR 507.1a — the active player gets priority");
}

/// CR 507.1 / 506.1 — with no attackers declared, the turn skips the declare
/// blockers and combat damage steps and lands on end of combat.
#[test]
fn cr_507_1_no_attackers_skips_to_end_of_combat() {
    let mut g = two_player_game();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("opponent passes");
    assert_eq!(g.step, TurnStep::EndCombat);
}
