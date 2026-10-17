//! Commander: the 2026-10-17 (`019Y6J4R`) thirteenth closer batch
//! (`decks::cmdr_019y_m`).

use crabomination::card::CardId;
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

use crabomination::card::Keyword;

fn opposing_spell(g: &mut GameState, def: crabomination::card::CardDefinition, green: u32) -> CardId {
    let id = g.add_card_to_hand(1, def);
    g.players[1].mana_pool.add(Color::Green, green);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("opposing spell");
    g.priority.player_with_priority = 0;
    id
}

/// Metalhead bounces another creature on entry, then eats an artifact for a
/// counter, menace and haste.
#[test]
fn metalhead_bounces_and_eats_artifacts() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let mh = g.add_card_to_hand(0, catalog::metalhead());
    flood(&mut g, 0);
    cast(&mut g, mh, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    let fodder = ready(&mut g, 0, catalog::lotus_ring());
    activate(&mut g, mh, 0, None).expect("sac an artifact");
    assert!(g.battlefield_find(fodder).is_none());
    let cp = g.computed_permanent(mh).unwrap();
    assert_eq!(cp.power, 5);
    assert!(cp.keywords().contains(&Keyword::Menace) && cp.keywords().contains(&Keyword::Haste));
}

/// CR 301.5 — U.S.Agent enters with a Sturdy Shield already attached.
#[test]
fn us_agent_comes_with_a_shield() {
    let mut g = pod(2);
    let us = g.add_card_to_hand(0, catalog::us_agent_john_walker());
    flood(&mut g, 0);
    cast(&mut g, us, None).expect("cast");
    let cp = g.computed_permanent(us).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sturdy Shield"));
}

/// Bladed Battle-Fan flashes onto a creature and saves it from a destroy.
#[test]
fn bladed_battle_fan_grants_indestructible() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let fan = g.add_card_to_hand(0, catalog::bladed_battle_fan());
    flood(&mut g, 0);
    g.active_player_idx = 1;
    cast(&mut g, fan, None).expect("flash");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::Indestructible));
}

/// Arnim Zola needs two creature cards in the graveyard to make a Villain.
#[test]
fn arnim_zola_needs_a_full_graveyard() {
    let mut g = pod(2);
    let az = ready(&mut g, 0, catalog::arnim_zola_bio_fanatic());
    flood(&mut g, 0);
    assert!(activate(&mut g, az, 0, None).is_err(), "empty graveyard");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    activate(&mut g, az, 0, None).expect("two creature cards");
    let v = g.battlefield.iter().find(|c| c.definition.name == "Villain").expect("Villain");
    assert!(v.tapped);
}

/// Liliana, Death Mage's −3 destroys and drains the creature's controller.
#[test]
fn liliana_death_mage_minus_three() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_death_mage());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: lili,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        x_value: None,
    })
    .expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, 18);
}

/// Tragic Lesson: with no land to return, the discard is forced.
#[test]
fn tragic_lesson_discards_without_a_land() {
    let mut g = pod(2);
    let tl = g.add_card_to_hand(0, catalog::tragic_lesson());
    flood(&mut g, 0);
    cast(&mut g, tl, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), 1, "drew two, discarded one");
}

/// Assimilate Essence counters an unpaid creature spell.
#[test]
fn assimilate_essence_counters_a_creature_spell() {
    let mut g = pod(2);
    let bear = opposing_spell(&mut g, catalog::grizzly_bears(), 2);
    let ae = g.add_card_to_hand(0, catalog::assimilate_essence());
    flood(&mut g, 0);
    cast(&mut g, ae, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
}

/// Spring Cleaning destroys its target enchantment whatever the clash says.
#[test]
fn spring_cleaning_destroys_an_enchantment() {
    let mut g = pod(2);
    let ench = ready(&mut g, 1, catalog::sigil_of_myrkul());
    let sc = g.add_card_to_hand(0, catalog::spring_cleaning());
    flood(&mut g, 0);
    cast(&mut g, sc, Some(Target::Permanent(ench))).expect("cast");
    assert!(g.battlefield_find(ench).is_none());
}

/// Flaring Cinder rummages on a big spell.
#[test]
fn flaring_cinder_rummages_on_a_big_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::flaring_cinder());
    let giant = g.add_card_to_hand(0, catalog::infinite_hourglass());
    let land = g.add_card_to_hand(0, catalog::forest());
    flood(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    cast(&mut g, giant, None).expect("cast");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == land), "discarded the land");
    assert_eq!(g.players[0].hand.len(), 1, "and drew one");
}

/// Arcane Investigator: a low roll draws a card.
#[test]
fn arcane_investigator_searches_the_room() {
    let mut g = pod(2);
    let ai = ready(&mut g, 0, catalog::arcane_investigator());
    flood(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::DieRoll(4),
    ]));
    activate(&mut g, ai, 0, None).expect("activate");
    assert_eq!(g.players[0].hand.len(), 1);
}
