//! Commander: EDHREC ranks 1501-2000, three-card gaps (`decks::cmdr_rank2k_f`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize, target: Option<Target>) {
    act(g, GameAction::ActivateAbility { card_id: id, ability_index, target, additional_targets: vec![], x_value: None, mode: None });
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Scrap Welder returns an artifact card with mana value below the
/// sacrificed artifact's — never one as large.
#[test]
fn scrap_welder_returns_a_smaller_artifact() {
    let mut g = pod(2);
    let welder = ready(&mut g, 0, catalog::scrap_welder());
    let big = g.add_card_to_graveyard(0, catalog::solemn_simulacrum());
    let small = g.add_card_to_graveyard(0, catalog::sol_ring());
    ready(&mut g, 0, catalog::thran_temporal_gateway());
    activate(&mut g, welder, 0, None);
    assert!(g.battlefield_find(small).is_some(), "Sol Ring (1) under the Gateway (4)");
    assert!(g.battlefield_find(big).is_none(), "Solemn (4) is not less than 4");
}

/// Kaalia, Zenith Seeker takes one Angel, one Demon and one Dragon.
#[test]
fn kaalia_takes_one_of_each() {
    let mut g = pod(2);
    for def in [
        catalog::serra_angel(),
        catalog::guardian_of_ghirapur(),
        catalog::archwing_dragon(),
        catalog::island(),
        catalog::island(),
        catalog::island(),
    ] {
        g.add_card_to_library(0, def);
    }
    let kaalia = g.add_card_to_hand(0, catalog::kaalia_zenith_seeker());
    cast(&mut g, kaalia, None);
    let names: Vec<_> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(names.len(), 2, "one Angel and the Dragon: {names:?}");
    assert!(names.contains(&"Archwing Dragon"));
}

/// Dragonspark Reactor counts artifacts entering, then fires that much.
#[test]
fn dragonspark_reactor_charges_and_fires() {
    let mut g = pod(2);
    let reactor = g.add_card_to_hand(0, catalog::dragonspark_reactor());
    cast(&mut g, reactor, None);
    let purse = g.add_card_to_hand(0, catalog::nobles_purse());
    cast(&mut g, purse, None);
    assert_eq!(g.battlefield_find(reactor).unwrap().counter_count(CounterType::Charge), 2);
    let life = g.players[1].life;
    activate(&mut g, reactor, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 2);
}

/// Slagstone Refinery makes a tapped Powerstone when a nontoken artifact of
/// yours dies, itself included.
#[test]
fn slagstone_refinery_mints_powerstones() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::slagstone_refinery());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    cast(&mut g, shatter, Some(Target::Permanent(ring)));
    let stone = g.battlefield.iter().find(|c| c.definition.name == "Powerstone").expect("a Powerstone");
    assert!(stone.tapped);
}

/// Bartered Cow makes a Food when discarded.
#[test]
fn bartered_cow_feeds_on_discard() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::bartered_cow());
    let looter = g.add_card_to_hand(0, catalog::tolarian_winds());
    cast(&mut g, looter, None);
    assert_eq!(named(&g, "Food"), 1);
}

/// Zephyr Sentinel grows only when it returns a Soldier.
#[test]
fn zephyr_sentinel_rewards_a_soldier() {
    let mut g = pod(2);
    let soldier = ready(&mut g, 0, catalog::yotian_tactician());
    let sentinel = g.add_card_to_hand(0, catalog::zephyr_sentinel());
    cast(&mut g, sentinel, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == soldier), "returned");
    assert_eq!(g.battlefield_find(sentinel).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Fortified Beachhead enters untapped beside a Soldier, tapped without one.
#[test]
fn fortified_beachhead_wants_a_soldier() {
    for (soldier, tapped) in [(true, false), (false, true)] {
        let mut g = pod(2);
        if soldier {
            ready(&mut g, 0, catalog::yotian_tactician());
        }
        let land = g.add_card_to_hand(0, catalog::fortified_beachhead());
        act(&mut g, GameAction::PlayLand(land));
        assert_eq!(g.battlefield_find(land).unwrap().tapped, tapped);
    }
}

/// Fathom Fleet Swordjack hits the attacked player for your artifact count.
#[test]
fn fathom_fleet_swordjack_counts_artifacts() {
    let mut g = pod(3);
    let jack = ready(&mut g, 0, catalog::fathom_fleet_swordjack());
    ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::nobles_purse());
    let life = g.players[2].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: jack, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[2].life, life - 2);
    assert_eq!(g.players[1].life, life, "not the other opponent");
}
