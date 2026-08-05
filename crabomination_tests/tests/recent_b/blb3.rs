//! Bloomburrow gap batch 3 (`decks::blb3`).

use crabomination::card::{CounterType, LandType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood_mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Ral ticks up on a noncreature cast.
#[test]
fn ral_gains_loyalty_on_noncreature_spells() {
    let mut g = main_phase();
    let ral = g.add_card_to_battlefield(0, catalog::ral_crackling_wit());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood_mana(&mut g, 0);
    let before = g.battlefield_find(ral).unwrap().counter_count(CounterType::Loyalty);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ral).unwrap().counter_count(CounterType::Loyalty),
        before + 1
    );
}

/// His ultimate hands out storm.
#[test]
fn ral_ultimate_emblem_grants_storm() {
    let mut g = main_phase();
    let ral = g.add_card_to_battlefield(0, catalog::ral_crackling_wit());
    g.battlefield_find_mut(ral).unwrap().add_counters(CounterType::Loyalty, 10);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ral,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("ultimate");
    drain_stack(&mut g);
    assert!(
        g.players[0].emblems.iter().any(|e| e.name == "Ral, Crackling Wit"),
        "the emblem landed"
    );
}

/// Eluge's flooded lands are Islands, and it sizes itself by them.
#[test]
fn eluge_makes_flooded_lands_islands() {
    let mut g = main_phase();
    // One real Island keeps the */* body alive through the SBA check.
    g.add_card_to_battlefield(0, catalog::island());
    let eluge = g.add_card_to_battlefield(0, catalog::eluge_the_shoreless_sea());
    let waste = g.add_card_to_battlefield(0, catalog::wastes());
    g.battlefield_find_mut(waste).unwrap().add_counters(CounterType::Flood, 1);
    let cp = g.computed_permanent(waste).unwrap();
    assert!(cp.subtypes.land_types.contains(&LandType::Island), "the flooded land is an Island");
    let cp = g.computed_permanent(eluge).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "two Islands → 2/2");
}

/// Its enter/attack trigger floods a land.
#[test]
fn eluge_floods_a_land_on_entry() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::island());
    let eluge = g.add_card_to_battlefield(0, catalog::eluge_the_shoreless_sea());
    g.fire_self_etb_triggers(eluge, 0);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.counter_count(CounterType::Flood) > 0),
        "a land took a flood counter"
    );
}

/// The flooded land also discounts your first spell of the turn.
#[test]
fn eluge_discounts_the_first_instant_each_turn() {
    let mut g = main_phase();
    let waste = g.add_card_to_battlefield(0, catalog::wastes());
    g.battlefield_find_mut(waste).unwrap().add_counters(CounterType::Flood, 1);
    g.add_card_to_battlefield(0, catalog::eluge_the_shoreless_sea());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let card = g.players[0].hand.iter().find(|c| c.id == bolt).unwrap().clone();
    assert_eq!(
        crabomination::game::actions::cost_reduction_for_spell(&g, 0, &card, None),
        1,
        "one flooded land shaves {{1}} off the first instant"
    );
}

/// Vren exiles the opponent's dying creatures, then turns the tally into Rats.
#[test]
fn vren_exiles_and_mints_rats_at_end_of_turn() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::vren_the_relentless());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt the bear");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "it was exiled, not buried");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let rats: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Rat").map(|c| c.id).collect();
    assert_eq!(rats.len(), 1, "one exiled creature → one Rat");
}

/// Ygra turns the board into Food and fattens on every one that dies.
#[test]
fn ygra_makes_food_and_eats_it() {
    let mut g = main_phase();
    let ygra = g.add_card_to_battlefield(0, catalog::ygra_eater_of_all());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Artifact), "the bear is Food");
    assert!(!g.granted_abilities_for(bear).is_empty(), "with the sac-for-life ability");
    let mut evs = Vec::new();
    g.destroy_permanent(bear, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ygra).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "a Food hit the graveyard → two counters"
    );
}
