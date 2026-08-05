//! Alliances (ALL) — `catalog::sets::all2`.

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
        card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn next_upkeep(g: &mut GameState) {
    g.active_player_idx = 1;
    g.turn_number += 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// Agent of Stromgald launders red into black.
#[test]
fn agent_of_stromgald_filters_red_into_black() {
    let mut g = main_phase();
    let agent = g.add_card_to_battlefield(0, catalog::agent_of_stromgald());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: agent, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("filter");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0);
}

/// Arcane Denial pays its victim two cards and you one, both next upkeep.
#[test]
fn arcane_denial_refunds_both_sides_next_upkeep() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::shock());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bolt");
    let denial = g.add_card_to_hand(0, catalog::arcane_denial());
    let (mine, theirs) = (g.players[0].hand.len(), g.players[1].hand.len());
    cast(&mut g, 0, denial, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "countered");
    next_upkeep(&mut g);
    assert_eq!(g.players[0].hand.len(), mine - 1 + 1, "you drew one");
    assert_eq!(g.players[1].hand.len(), theirs + 2, "they drew two");
}

/// Astrolabe cashes out two of a colour and a card.
#[test]
fn astrolabe_pays_two_mana_and_a_card() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(0, catalog::astrolabe());
    g.clear_sickness(rock);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: rock, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("crack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2);
    assert!(g.battlefield_find(rock).is_none(), "sacrificed");
    next_upkeep(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Balduvian Horde eats itself with an empty hand.
#[test]
fn balduvian_horde_needs_a_card_to_pitch() {
    let mut g = main_phase();
    let id = g.add_card_to_hand(0, catalog::balduvian_horde());
    cast(&mut g, 0, id, None);
    assert!(g.battlefield_find(id).is_none(), "nothing to discard");

    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::balduvian_horde());
    cast(&mut g, 0, id, None);
    assert!(g.battlefield_find(id).is_some(), "paid with the Bears");
}

/// Carrier Pigeons mails a card on the next upkeep, not this one.
#[test]
fn carrier_pigeons_deliver_next_upkeep() {
    let mut g = main_phase();
    let id = g.add_card_to_hand(0, catalog::carrier_pigeons());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, id, None);
    assert_eq!(g.players[0].hand.len(), hand - 1, "no draw yet");
    next_upkeep(&mut g);
    assert_eq!(g.players[0].hand.len(), hand);
}

/// Enslaved Scout buys mountainwalk for a turn.
#[test]
fn enslaved_scout_buys_mountainwalk() {
    let mut g = main_phase();
    let scout = g.add_card_to_battlefield(0, catalog::enslaved_scout());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: scout, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("walk");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(scout)
            .expect("scout")
            .keywords
            .contains(&crabomination::card::Keyword::Landwalk(
                crabomination::card::LandType::Mountain
            ))
    );
}

/// Errand of Duty mints a banding Knight.
#[test]
fn errand_of_duty_mints_a_banding_knight() {
    let mut g = main_phase();
    let id = g.add_card_to_hand(0, catalog::errand_of_duty());
    cast(&mut g, 0, id, None);
    let token = g.battlefield.iter().find(|c| c.is_token).expect("Knight");
    assert!(token.definition.keywords.contains(&crabomination::card::Keyword::Banding));
}

/// Feast or Famine's second mode is unconditional removal.
#[test]
fn feast_or_famine_kills_on_mode_one() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::feast_or_famine());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Fyndhorn Druid only pays out if it was blocked.
#[test]
fn fyndhorn_druid_pays_out_only_when_blocked() {
    let mut g = main_phase();
    let druid = g.add_card_to_battlefield(0, catalog::fyndhorn_druid());
    let mut events = Vec::new();
    g.destroy_permanent(druid, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "never blocked");

    let mut g = main_phase();
    let druid = g.add_card_to_battlefield(0, catalog::fyndhorn_druid());
    g.clear_sickness(druid);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: druid, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, druid)])).expect("block");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.destroy_permanent(druid, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 24);
}

/// Gift of the Woods fires when its host becomes blocked.
#[test]
fn gift_of_the_woods_pays_when_blocked() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let gift = g.add_card_to_hand(0, catalog::gift_of_the_woods());
    cast(&mut g, 0, gift, Some(Target::Permanent(bear)));
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bear)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
    assert_eq!(g.computed_permanent(bear).expect("bear").toughness, 5);
}

/// Inheritance turns each death into a card, for {3}.
#[test]
fn inheritance_buys_a_card_per_death() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::inheritance());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let hand = g.players[0].hand.len();
    let mut events = Vec::new();
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Juniper Order Advocate's anthem switches off when it taps.
#[test]
fn juniper_order_advocate_anthem_needs_to_stand() {
    let mut g = main_phase();
    let advocate = g.add_card_to_battlefield(0, catalog::juniper_order_advocate());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green 2/2
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 3);
    g.battlefield_find_mut(advocate).unwrap().tapped = true;
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 2);
}

/// Kaysa pumps the green team unconditionally.
#[test]
fn kaysa_pumps_the_green_team() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::kaysa());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 3);
}
