//! CR conformance: 602.5b cost batches, 508.1a attack restrictions,
//! 701.3c re-attaching an Aura, and 603.10 Aura death LKI.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn end_turn(g: &mut GameState) {
    let started = g.turn_number;
    while g.turn_number == started {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// CR 602.5b — every permanent sacrificed to pay one activation's cost feeds
/// the same batch, so the ability body reads the total and not the last one.
#[test]
fn cr_602_5b_cost_sacrifice_batch_totals_reach_resolution() {
    let mut g = main_phase();
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_the_ages());
    g.battlefield_find_mut(sword).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::savannah_lions());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sword,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 7, "3 + 2 + 2, not just the last body");
}

/// CR 602.5b — "any number" includes zero, so an empty board still pays.
#[test]
fn cr_602_5b_any_number_sacrifice_accepts_an_empty_batch() {
    let mut g = main_phase();
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_the_ages());
    g.battlefield_find_mut(sword).unwrap().tapped = false;
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sword,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "no creatures, no damage");
    assert!(g.exile.iter().any(|c| c.id == sword), "the Sword still exiles itself");
}

/// CR 508.1a — a restriction keyed on the *previous* turn only bites on the
/// controller's own next turn, and lifts the turn after.
#[test]
fn cr_508_1a_attacked_last_turn_restriction_lifts_after_one_turn() {
    let mut g = main_phase();
    let turtle = g.add_card_to_battlefield(0, catalog::giant_turtle());
    g.clear_sickness(turtle);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: turtle, target: AttackTarget::Player(1) }])
        .expect("first swing");
    end_turn(&mut g); // seat 1
    end_turn(&mut g); // back to seat 0 — the ban is live
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: turtle, target: AttackTarget::Player(1) }])
            .is_err()
    );
    end_turn(&mut g);
    end_turn(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: turtle, target: AttackTarget::Player(1) }])
            .is_ok(),
        "it sat one turn out"
    );
}

/// CR 701.3c — a re-attached Aura must still satisfy its own enchant filter;
/// an illegal host leaves it where it was.
#[test]
fn cr_701_3c_reattach_rejects_an_illegal_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let aura = g.add_card_to_hand(0, catalog::spirit_link());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let spell = g.add_card_to_hand(0, catalog::enchantment_alteration());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(aura)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    });
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(aura).unwrap().attached_to,
        Some(bear),
        "enchant creature can't move to a land"
    );
}

/// CR 603.10 — "when enchanted creature dies" reads the Auras that were on it,
/// on the sacrifice path as well as the lethal-damage one.
#[test]
fn cr_603_10_enchanted_dies_trigger_fires_on_a_sacrifice() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::puppet_master());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        bear,
        1,
        Some(Target::Permanent(bear)),
    );
    let events = g
        .resolve_effect(
            &crabomination::effect::Effect::SacrificePermanent {
                what: crabomination::effect::Selector::Target(0),
            },
            &ctx,
        )
        .expect("sacrifice");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let _ = aura;
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "returned by the Aura");
}
