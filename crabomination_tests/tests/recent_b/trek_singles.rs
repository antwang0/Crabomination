//! Star Trek: the EDHREC single-commander closers (`decks::trek_singles`).

use crabomination::card::{CardId, CounterType, CreatureType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
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

fn play_land(g: &mut GameState, seat: usize) {
    let land = g.add_card_to_hand(seat, catalog::mountain());
    g.players[seat].lands_played_this_turn = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(g);
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// Captain Burnham: each landfall pings every opponent; only the second
/// resolution this turn grows her.
#[test]
fn burnham_grows_on_the_second_landfall_only() {
    let mut g = pod(3);
    let burnham = g.add_card_to_battlefield(0, catalog::captain_michael_burnham());
    let life = g.players[1].life;
    play_land(&mut g, 0);
    assert_eq!((g.players[1].life, g.players[2].life), (life - 1, life - 1));
    assert_eq!(plus(&g, burnham), 0, "first resolution");
    play_land(&mut g, 0);
    assert_eq!(plus(&g, burnham), 1, "second resolution");
    play_land(&mut g, 0);
    assert_eq!(plus(&g, burnham), 1, "third resolution");
}

/// Mr. Spock: a draw becomes two.
#[test]
fn mr_spock_doubles_draws() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mr_spock_science_officer());
    let hand = g.players[0].hand.len();
    let mut ev = Vec::new();
    g.draw_one(0, &mut ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Dr. McCoy (CR 614.8): a countered creature of yours that would be
/// destroyed sheds its +1/+1 counters instead; one without counters dies;
/// McCoy doesn't save himself.
#[test]
fn dr_mccoy_trades_counters_for_survival() {
    let mut g = pod(2);
    let mccoy = g.add_card_to_battlefield(0, catalog::dr_mccoy_surly_surgeon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plain = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(mccoy).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let wrath = g.add_card_to_hand(1, catalog::wrath_of_god());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    cast(&mut g, wrath, None);
    assert!(g.battlefield_find(bear).is_some(), "shed its counters instead");
    assert_eq!(plus(&g, bear), 0);
    assert!(g.battlefield_find(plain).is_none());
    assert!(g.battlefield_find(mccoy).is_none(), "\"another creature\"");
}

/// Dr. McCoy also replaces lethal-damage destruction, healing the damage.
#[test]
fn dr_mccoy_heals_lethal_damage() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::dr_mccoy_surly_surgeon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    let c = g.battlefield_find(bear).expect("survives");
    assert_eq!(c.damage, 0);
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Deanna Troi: another creature of yours enters with a +1/+1 counter.
#[test]
fn deanna_troi_adds_a_counter_to_entrants() {
    let mut g = pod(2);
    let troi = g.add_card_to_battlefield(0, catalog::deanna_troi_psychic_counsel());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    cast(&mut g, bear, None);
    assert_eq!(plus(&g, bear), 1);
    assert_eq!(plus(&g, troi), 0);
}

/// Grand Nagus Zek: a Treasure on the first other death each turn, and +1/+1
/// per Treasure.
#[test]
fn grand_nagus_zek_profits_once_a_turn() {
    let mut g = pod(2);
    let zek = g.add_card_to_battlefield(0, catalog::grand_nagus_zek());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::savannah_lions());
    for id in [a, b] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        flood(&mut g, 0);
        cast(&mut g, bolt, Some(Target::Permanent(id)));
    }
    let treasures = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").count();
    assert_eq!(treasures, 1, "only once each turn");
    let c = g.computed_permanent(zek).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2));
}

/// Starship Enterprise: landfall stations it four at a time.
#[test]
fn starship_enterprise_charges_on_landfall() {
    let mut g = pod(2);
    let ship = g.add_card_to_battlefield(0, catalog::starship_enterprise());
    play_land(&mut g, 0);
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 4);
}

/// Deep Space 9: the permanent cards among the top four enter; the rest are
/// exiled.
#[test]
fn deep_space_9_deploys_permanents_from_the_top_four() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let forest = g.add_card_to_library(0, catalog::forest());
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    let fifth = g.add_card_to_library(0, catalog::island());
    let ds9 = g.add_card_to_hand(0, catalog::deep_space_9());
    flood(&mut g, 0);
    cast(&mut g, ds9, None);
    for id in [bear, forest, ring] {
        assert!(g.battlefield_find(id).is_some());
    }
    assert!(g.exile.iter().any(|c| c.id == bolt));
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(fifth));
}

/// Worf, Intelligence Contractor: one cheap creature from each opponent,
/// untapped, hasty and a Warrior until end of turn.
#[test]
fn worf_borrows_a_creature_from_each_opponent() {
    let mut g = pod(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::savannah_lions());
    let big = g.add_card_to_battlefield(2, catalog::craw_wurm());
    g.battlefield_find_mut(a).unwrap().tapped = true;
    let worf = g.add_card_to_hand(0, catalog::worf_intelligence_contractor());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: worf,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        let c = g.computed_permanent(id).unwrap();
        assert_eq!(c.controller, 0);
        assert!(c.subtypes().creature_types.contains(&CreatureType::Warrior));
        assert!(!g.battlefield_find(id).unwrap().tapped);
    }
    assert_eq!(g.computed_permanent(big).unwrap().controller, 2, "mana value 6");
}

/// The Doctor, EMH: a counter on the target, then one more on each of your
/// countered creatures (the target included).
#[test]
fn the_doctor_spreads_counters() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lions = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let other = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(lions).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let etb = catalog::the_doctor_emh().triggered_abilities[0].effect.clone();
    g.resolve_effect(&etb, &crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0))
        .expect("resolve");
    let total: u32 = [bear, lions, other].iter().map(|&id| plus(&g, id)).sum();
    // One new counter on the target, then +1 on the two countered creatures.
    assert_eq!(total, 1 + 1 + 2);
}
