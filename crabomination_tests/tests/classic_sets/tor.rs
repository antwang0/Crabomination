//! Torment (TOR) — the Cephalid self-mill shell, Madness and Threshold.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
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

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Seven cards in the graveyard turns Threshold on.
fn fill_graveyard(g: &mut GameState, seat: usize) {
    for _ in 0..7 {
        g.add_card_to_graveyard(seat, catalog::forest());
    }
}

/// Aquamoeba flips its stats for a card.
#[test]
fn aquamoeba_switches_power_and_toughness() {
    let mut g = main_phase();
    let moeba = g.add_card_to_battlefield(0, catalog::aquamoeba());
    g.add_card_to_hand(0, catalog::forest());
    let cp = g.computed_permanent(moeba).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3));
    activate(&mut g, 0, moeba, 0, None);
    let cp = g.computed_permanent(moeba).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1));
}

/// Pointing anything at a Cephalid fills its controller's graveyard.
#[test]
fn cephalid_illusionist_mills_on_being_targeted() {
    let mut g = main_phase();
    let ceph = g.add_card_to_battlefield(0, catalog::cephalid_illusionist());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, 0, pump, Some(Target::Permanent(ceph)));
    assert_eq!(g.players[0].graveyard.len(), 4, "three milled plus the spell");
}

/// Cephalid Vandal mills one more each upkeep.
#[test]
fn cephalid_vandal_accelerates_each_upkeep() {
    let mut g = main_phase();
    let vandal = g.add_card_to_battlefield(0, catalog::cephalid_vandal());
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert_eq!(g.battlefield_find(vandal).unwrap().counter_count(CounterType::Shred), 1);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 3, "two more on the second tick");
}

/// Cephalid Sage draws three past Threshold and nothing before it.
#[test]
fn cephalid_sage_draws_past_threshold() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    fill_graveyard(&mut g, 0);
    let sage = g.add_card_to_hand(0, catalog::cephalid_sage());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, sage, None);
    // -1 for the cast, +3 drawn, -2 discarded.
    assert_eq!(g.players[0].hand.len(), before - 1 + 3 - 2);
}

/// Boneshard Slasher swells but becomes brittle past Threshold.
#[test]
fn boneshard_slasher_grows_and_gets_brittle() {
    let mut g = main_phase();
    let slasher = g.add_card_to_battlefield(0, catalog::boneshard_slasher());
    let cp = g.computed_permanent(slasher).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(slasher).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, 0, pump, Some(Target::Permanent(slasher)));
    assert!(g.battlefield_find(slasher).is_none(), "targeting it kills it past Threshold");
}

/// Cabal Torturer's bigger shrink needs Threshold.
#[test]
fn cabal_torturer_second_ability_needs_threshold() {
    let mut g = main_phase();
    let torturer = g.add_card_to_battlefield(0, catalog::cabal_torturer());
    g.battlefield_find_mut(torturer).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: torturer,
            ability_index: 1,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the -2/-2 is Threshold-gated"
    );
    fill_graveyard(&mut g, 0);
    activate(&mut g, 0, torturer, 1, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 killed the 2/2");
}

/// Circular Logic taxes the countered spell by your whole graveyard.
#[test]
fn circular_logic_taxes_by_your_graveyard() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(1, catalog::giant_growth());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pump,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the pump");
    let logic = g.add_card_to_hand(0, catalog::circular_logic());
    // Seat 1 is tapped out, so the {7} tax can't be paid.
    g.players[1].mana_pool = Default::default();
    cast(&mut g, 0, logic, Some(Target::Permanent(pump)));
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == pump),
        "countered — they could not pay the tax"
    );
}

/// Ambassador Laquatus mills three for {3}.
#[test]
fn ambassador_laquatus_mills_three() {
    let mut g = main_phase();
    let laq = g.add_card_to_battlefield(0, catalog::ambassador_laquatus());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::forest());
    }
    activate(&mut g, 0, laq, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 3);
}

/// Chainer reanimates a corpse as a Nightmare under his control.
#[test]
fn chainer_reanimates_as_a_nightmare() {
    let mut g = main_phase();
    let chainer = g.add_card_to_battlefield(0, catalog::chainer_dementia_master());
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, 0, chainer, 0, Some(Target::Permanent(corpse)));
    let cp = g.computed_permanent(corpse).expect("reanimated");
    assert_eq!(cp.controller, 0);
    assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Nightmare));
}

/// Coral Net taxes its host a card every upkeep.
#[test]
fn coral_net_taxes_a_card_each_upkeep() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let net = g.add_card_to_hand(0, catalog::coral_net());
    cast(&mut g, 0, net, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(net).unwrap().attached_to, Some(bear));
    // With an empty hand the host can't pay, so it dies.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "nothing to discard");
}

/// Compulsion loots for {1}{U}.
#[test]
fn compulsion_loots() {
    let mut g = main_phase();
    let comp = g.add_card_to_battlefield(0, catalog::compulsion());
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, 0, comp, 0, None);
    assert_eq!(g.players[0].graveyard.len(), 1, "the discard");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Acorn Harvest makes two Squirrels, and again from the graveyard.
#[test]
fn acorn_harvest_makes_squirrels_twice() {
    let mut g = main_phase();
    let harvest = g.add_card_to_hand(0, catalog::acorn_harvest());
    cast(&mut g, 0, harvest, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(), 2);
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: harvest,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(), 4);
}

/// Churning Eddy bounces a creature and a land.
#[test]
fn churning_eddy_bounces_both() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let eddy = g.add_card_to_hand(0, catalog::churning_eddy());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: eddy,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(land).is_none());
}

/// CR 611.2b — Chainer's permanent Nightmare stamp outlives him, so his own
/// leaves-the-battlefield trigger exiles what he reanimated.
#[test]
fn chainer_exiles_his_nightmares_when_he_dies() {
    let mut g = main_phase();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let chainer = g.add_card_to_battlefield(0, catalog::chainer_dementia_master());
    activate(&mut g, 0, chainer, 0, Some(Target::Permanent(corpse)));
    assert!(g.battlefield_find(corpse).is_some(), "reanimated");
    let mut events = Vec::new();
    g.destroy_permanent(chainer, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(corpse).is_none(), "the Nightmare was exiled");
    assert!(g.exile.iter().any(|c| c.id == corpse));
}
