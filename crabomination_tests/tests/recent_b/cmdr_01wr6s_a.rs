//! Sulu, Heroic Helmsman's, Commander Spock's and Seven of Nine's decks
//! (`decks::cmdr_01wr6s_a`).

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
    g
}

fn ready(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// CR 702.122 — Sulu crews as though his power were tripled: his 2 power
/// crews Marshals' Pathcruiser's Crew 5 alone, where Grizzly Bears can't.
#[test]
fn cr_702_122_sulu_crews_with_tripled_power() {
    let mut g = pod(2);
    let bears = ready(&mut g, catalog::grizzly_bears());
    let car = g.add_card_to_battlefield(0, catalog::marshals_pathcruiser());
    assert!(g.perform_action(GameAction::Crew { vehicle: car, crew_creatures: vec![bears] }).is_err());
    let sulu = ready(&mut g, catalog::sulu_heroic_helmsman());
    g.perform_action(GameAction::Crew { vehicle: car, crew_creatures: vec![sulu] }).expect("Sulu crews 5");
    let cp = g.computed_permanent(car).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
}

/// CR 702.184a — Sulu stations as though his power were tripled: six charge
/// counters from a 2-power Sulu.
#[test]
fn cr_702_184a_sulu_stations_with_tripled_power() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ship = g.add_card_to_battlefield(0, catalog::uss_enterprise_d_galaxy_class());
    ready(&mut g, catalog::sulu_heroic_helmsman());
    activate(&mut g, ship, None).expect("station");
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 6);
}

/// DOT-7 Repair Squad untaps a creature once each turn.
#[test]
fn dot_7_untaps_once_each_turn() {
    let mut g = pod(2);
    let dot = ready(&mut g, catalog::dot_7_repair_squad());
    let bears = ready(&mut g, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    activate(&mut g, dot, Some(Target::Permanent(bears))).expect("untap");
    assert!(!g.battlefield_find(bears).unwrap().tapped);
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    assert!(activate(&mut g, dot, Some(Target::Permanent(bears))).is_err(), "only once each turn");
}

/// Commander Spock enters with three differently named lands: a revealed
/// 2-drop is cast free (so nothing is drawn), a revealed 5-drop stays on top
/// and is drawn, and a revealed land enters tapped (nothing drawn).
#[test]
fn commander_spock_casts_deploys_or_draws_the_top_card() {
    for (top, cast_free, drawn) in [
        (catalog::grizzly_bears(), true, false),
        (catalog::shivan_dragon(), false, true),
        (catalog::swamp(), true, false),
    ] {
        let mut g = pod(2);
        for land in [catalog::forest(), catalog::plains(), catalog::island()] {
            g.add_card_to_battlefield(0, land);
        }
        let top = g.add_card_to_library(0, top);
        g.add_card_to_library(0, catalog::mountain());
        let spock = g.add_card_to_hand(0, catalog::commander_spock());
        cast(&mut g, spock, None, None);
        let on_bf = g.battlefield_find(top);
        assert_eq!(on_bf.is_some(), cast_free, "{top:?} put onto the battlefield");
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == top), drawn);
        if let Some(c) = on_bf.filter(|c| c.definition.is_land()) {
            assert!(c.tapped, "the land enters tapped");
        }
    }
}

/// Captain Kirk's enters trigger takes a basic land from the graveyard (no
/// library search); Spacecraft creatures get +3/+3.
#[test]
fn captain_kirk_fetches_a_basic_from_the_graveyard() {
    let mut g = pod(2);
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    let kirk = g.add_card_to_hand(0, catalog::captain_kirk_boldly_going());
    let ship = g.add_card_to_battlefield(0, catalog::uss_enterprise_d_galaxy_class());
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 7);
    cast(&mut g, kirk, None, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
    let cp = g.computed_permanent(ship).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 8), "a Spacecraft creature gets +3/+3");
}

/// CR 700.2 — choosing Resistance Is Futile's mode is facing a dilemma:
/// Seven of Nine draws, Melkotian Adjudicator gets a counter.
#[test]
fn cr_700_2_seven_of_nine_and_adjudicator_hear_a_modal_spell() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    ready(&mut g, catalog::seven_of_nine());
    let adj = ready(&mut g, catalog::melkotian_adjudicator());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::resistance_is_futile());
    let hand = g.players[0].hand.len();
    cast(&mut g, spell, Some(Target::Permanent(bears)), Some(0));
    assert!(g.battlefield_find(bears).is_none(), "destroyed");
    assert_eq!(g.players[0].hand.len(), hand, "spell left, one card drawn");
    assert_eq!(g.battlefield_find(adj).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Resistance Is Futile's second mode returns an artifact creature card with
/// mana value 3 or less.
#[test]
fn resistance_is_futile_reanimates_a_small_artifact_creature() {
    let mut g = pod(2);
    let thopter = g.add_card_to_graveyard(0, catalog::ornithopter());
    let spell = g.add_card_to_hand(0, catalog::resistance_is_futile());
    cast(&mut g, spell, Some(Target::Permanent(thopter)), Some(1));
    assert!(g.battlefield_find(thopter).is_some());
}

/// Tell Me What You See: the opponent either has you draw or takes 4.
#[test]
fn tell_me_what_you_see_punishes_or_draws() {
    let mut g = pod(3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ench = g.add_card_to_hand(0, catalog::tell_me_what_you_see());
    let hand = g.players[0].hand.len();
    cast(&mut g, ench, Some(Target::Player(1)), None);
    let drew = g.players[0].hand.len() == hand;
    let lost = g.players[1].life == g.players[1].starting_life - 4;
    assert!(drew != lost, "exactly one branch: drew {drew}, lost {lost}");
    assert_eq!(g.players[2].life, g.players[2].starting_life, "only the target");
}
