//! Commander: the 2026-10-17 (`01Htv6cF`) frequency scan's short cards
//! (`decks::cmdr_oct17_g`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_with(g, id, target, vec![])
}

fn hand(g: &GameState, seat: usize) -> usize {
    g.players[seat].hand.len()
}

/// Balance of Power draws the difference only when the target opponent holds
/// more cards (a non-negative difference).
#[test]
fn balance_of_power_draws_the_hand_difference() {
    for (theirs, drawn) in [(5usize, 4usize), (0, 0)] {
        let mut g = pod(3);
        for _ in 0..theirs {
            g.add_card_to_hand(1, catalog::forest());
        }
        let s = g.add_card_to_hand(0, catalog::balance_of_power());
        g.add_card_to_hand(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, s, Some(Target::Player(1))).expect("Balance of Power");
        assert_eq!(hand(&g, 0), 1 + drawn, "opponent held {theirs}");
    }
}

/// Boulder Dash: 2 damage to one target and 1 to another.
#[test]
fn boulder_dash_splits_two_and_one() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let s = g.add_card_to_hand(0, catalog::boulder_dash());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast_with(&mut g, s, Some(Target::Permanent(bears)), vec![Target::Player(1)]).expect("Boulder Dash");
    assert!(g.battlefield_find(bears).is_none(), "2 damage kills the 2/2");
    assert_eq!(g.players[1].life, life - 1);
}

/// Dreaded Bat-Cloud costs {3} less once a creature has died this turn.
#[test]
fn dreaded_bat_cloud_is_cheaper_after_a_death() {
    let mut g = pod(2);
    let bat = g.add_card_to_hand(0, catalog::dreaded_bat_cloud());
    g.players[0].mana_pool.add(Color::Black, 2);
    assert!(cast(&mut g, bat, None).is_err(), "{{4}}{{B}} with two mana");
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bears))).expect("bolt");
    cast(&mut g, bat, None).expect("{1}{B} after a creature died");
    assert!(g.battlefield_find(bat).is_some());
}

/// Malakir Cullblade grows only on an opponent's creature dying.
#[test]
fn malakir_cullblade_counts_opponent_deaths() {
    let mut g = pod(2);
    let blade = ready(&mut g, 0, catalog::malakir_cullblade());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    for t in [mine, theirs] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Permanent(t))).expect("bolt");
    }
    assert_eq!(g.battlefield_find(blade).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Crystalline Entity wipes nonartifact creatures only when cast (CR
/// 603.4 intervening "if you cast it").
#[test]
fn crystalline_entity_wipes_only_when_cast() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let golem = ready(&mut g, 1, catalog::precursor_golem());
    ready(&mut g, 0, catalog::crystalline_entity());
    assert!(g.battlefield_find(bears).is_some(), "put onto the battlefield, not cast");
    let e = g.add_card_to_hand(0, catalog::crystalline_entity());
    g.players[0].mana_pool.add_colorless(8);
    cast(&mut g, e, None).expect("Entity");
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(golem).is_some(), "artifact creatures survive");
}

/// Ray of Erasure mills now and draws at the next upkeep.
#[test]
fn ray_of_erasure_draws_at_the_next_upkeep() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::ray_of_erasure());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, s, Some(Target::Player(1))).expect("Ray");
    assert_eq!(g.players[1].graveyard.len(), 1);
    let before = hand(&g, 0);
    let mut guard = 0;
    while !(g.active_player_idx == 1 && g.step == TurnStep::Draw) && guard < 200 {
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
        guard += 1;
    }
    assert_eq!(hand(&g, 0), before + 1, "drew at the opponent's upkeep");
}

/// Deglamer shuffles the artifact into its owner's library.
#[test]
fn deglamer_shuffles_into_the_owners_library() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let lib = g.players[1].library.len();
    let s = g.add_card_to_hand(0, catalog::deglamer());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, s, Some(Target::Permanent(ring))).expect("Deglamer");
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.players[1].library.len(), lib + 1);
}

/// Bad Deal: you draw two, each opponent discards two, everyone loses 2.
#[test]
fn bad_deal_costs_everyone() {
    let mut g = pod(3);
    for seat in 1..3 {
        for _ in 0..3 {
            g.add_card_to_hand(seat, catalog::forest());
        }
    }
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let s = g.add_card_to_hand(0, catalog::bad_deal());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, s, None).expect("Bad Deal");
    assert_eq!(hand(&g, 0), 2);
    for (p, life) in g.players.iter().zip(&lives) {
        assert_eq!(p.life, life - 2);
    }
    assert_eq!((hand(&g, 1), hand(&g, 2)), (1, 1));
}

/// Swaggering Corsair's raid counter needs an attack this turn.
#[test]
fn swaggering_corsair_raid_counter() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::swaggering_corsair());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, c, None).expect("Corsair");
    assert_eq!(g.battlefield_find(c).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Dreadwaters mills X, X = the lands you control.
#[test]
fn dreadwaters_mills_per_land() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::island());
    }
    let s = g.add_card_to_hand(0, catalog::dreadwaters());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, s, Some(Target::Player(1))).expect("Dreadwaters");
    assert_eq!(g.players[1].graveyard.len(), 3);
}
