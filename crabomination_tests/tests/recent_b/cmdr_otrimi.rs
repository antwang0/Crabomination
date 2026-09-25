//! Commander: the Enhanced Evolution precon (C20, Otrimi, `decks::cmdr_otrimi`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn loyalty(g: &mut GameState, id: CardId, idx: usize, targets: &[Target]) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: id,
        ability_index: idx,
        target: targets.first().cloned(),
        x_value: None,
    })
    .expect("loyalty");
    drain_stack(g);
}

fn on_bf(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).is_some()
}

/// Otrimi can be the commander (CR 903.3); its combat damage returns a
/// mutate creature card from the graveyard (CR 702.140).
#[test]
fn otrimi_recurs_mutate_cards() {
    assert!(catalog::otrimi_the_ever_playful().can_be_commander);
    let mut g = pod(2);
    let o = g.add_card_to_battlefield(0, catalog::otrimi_the_ever_playful());
    let dead = g.add_card_to_graveyard(0, catalog::pouncing_shoreshark());
    g.clear_sickness(o);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: o,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.players[0].hand.iter().any(|c| c.id == dead));
}

/// Animist's Awakening puts the revealed lands in tapped, untapped under
/// spell mastery (CR 207.2c).
#[test]
fn animists_awakening_ramps() {
    for mastery in [false, true] {
        let mut g = pod(2);
        if mastery {
            g.add_card_to_graveyard(0, catalog::lightning_bolt());
            g.add_card_to_graveyard(0, catalog::divination());
        }
        let land = g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let aa = g.add_card_to_hand(0, catalog::animists_awakening());
        cast_x(&mut g, aa, &[], Some(2));
        assert_eq!(g.battlefield_find(land).map(|c| c.tapped), Some(!mastery));
    }
}

/// Boneyard Mycodrax counts creature cards in the graveyard; scavenging it
/// (CR 702.96) moves the others' count.
#[test]
fn mycodrax_scavenges() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let m = g.add_card_to_battlefield(0, catalog::boneyard_mycodrax());
    assert_eq!(g.computed_permanent(m).map(|c| c.power), Some(3));
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let m = g.add_card_to_graveyard(0, catalog::boneyard_mycodrax());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: m,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
    .expect("scavenge");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Hungering Hydra enters with X counters and grows by the damage it
/// survives.
#[test]
fn hungering_hydra_grows() {
    let mut g = pod(2);
    let h = g.add_card_to_hand(0, catalog::hungering_hydra());
    cast_x(&mut g, h, &[], Some(3));
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_x(&mut g, shock, &[Target::Permanent(h)], None);
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// Nissa enters with X loyalty (CR 306.5b); her 0 puts a small enough
/// creature onto the battlefield.
#[test]
fn nissa_x_loyalty_and_zero() {
    let mut g = pod(2);
    let n = g.add_card_to_hand(0, catalog::nissa_steward_of_elements());
    cast_x(&mut g, n, &[], Some(3));
    assert_eq!(g.battlefield_find(n).unwrap().counter_count(CounterType::Loyalty), 3);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    loyalty(&mut g, n, 1, &[]);
    assert!(on_bf(&g, bear));
}

/// Manascape Refractor borrows a land's activated abilities.
#[test]
fn manascape_refractor_borrows_land_abilities() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::endless_sands());
    let r = g.add_card_to_battlefield(0, catalog::manascape_refractor());
    assert!(g.granted_abilities_for(r).len() >= 3);
}

/// Endless Sands exiles your creature and later returns it (CR 610.3).
#[test]
fn endless_sands_blinks() {
    let mut g = pod(2);
    let es = g.add_card_to_battlefield(0, catalog::endless_sands());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility { card_id: es, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("exile");
    drain_stack(&mut g);
    assert!(!on_bf(&g, bear));
    g.battlefield_find_mut(es).unwrap().tapped = false;
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility { card_id: es, ability_index: 2, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("return");
    drain_stack(&mut g);
    assert!(on_bf(&g, bear));
}

/// Ukkima leaving drains its last-known power (CR 603.10).
#[test]
fn ukkima_drains_on_leaving() {
    let mut g = pod(2);
    let u = g.add_card_to_battlefield(0, catalog::ukkima_stalking_shadow());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast_x(&mut g, m, &[Target::Permanent(u)], None);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 2);
    assert_eq!(g.players[0].life - g.players[0].starting_life, 2);
}

/// Villainous Wealth casts the opponent's exiled spells free.
#[test]
fn villainous_wealth_steals_spells() {
    let mut g = pod(2);
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    let vw = g.add_card_to_hand(0, catalog::villainous_wealth());
    cast_x(&mut g, vw, &[Target::Player(1)], Some(2));
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));
}

/// Tidal Barracuda: no opponent casts on your turn.
#[test]
fn tidal_barracuda_locks_your_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::tidal_barracuda());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
}
