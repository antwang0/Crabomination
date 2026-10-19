//! Commander: the 2026-10-19 (`016Xh16H`) closers (`decks::cmdr_016x_a`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

/// CR 601.2b-c — Cruel Alliance's teamwork is announced before its target is
/// chosen: unteamed it can't reach a mana-value-4 creature; teamed (a 2-power
/// creature tapped) it exiles one and gains 3 life.
#[test]
fn cruel_alliance_teamwork_widens_the_target() {
    let mut g = pod(3);
    let giant = ready(&mut g, 2, catalog::hill_giant());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ca = g.add_card_to_hand(0, catalog::cruel_alliance());
    flood(&mut g, 0);
    assert!(cast(&mut g, ca, Some(Target::Permanent(giant))).is_err(), "MV 4 > 3 unteamed");
    g.perform_action(GameAction::CastSpellKicked {
        card_id: ca,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("teamwork cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant));
    assert_eq!(g.players[0].life, 23);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the bear paid teamwork");
}

/// Cruel Alliance unteamed exiles a small creature and gains no life.
#[test]
fn cruel_alliance_unteamed_exiles_a_small_creature() {
    let mut g = pod(3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ca = g.add_card_to_hand(0, catalog::cruel_alliance());
    flood(&mut g, 0);
    cast(&mut g, ca, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, 20);
}

/// Glyph Keeper counters the first spell to target it each turn; the second
/// resolves.
#[test]
fn glyph_keeper_counters_the_first_targeting_spell() {
    let mut g = pod(2);
    let gk = ready(&mut g, 1, catalog::glyph_keeper());
    flood(&mut g, 0);
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, b1, Some(Target::Permanent(gk))).expect("bolt 1");
    assert_eq!(g.battlefield_find(gk).map(|c| c.damage), Some(0), "countered");
    cast(&mut g, b2, Some(Target::Permanent(gk))).expect("bolt 2");
    assert!(g.battlefield_find(gk).is_none(), "the second bolt kills it");
}

/// CR 702.128a — Vizier of the Anointed entombs a creature card with embalm,
/// and activating that embalm draws a card (the trait reads the activation,
/// not the token it makes).
#[test]
fn vizier_of_the_anointed_entombs_and_draws_on_embalm() {
    let mut g = pod(2);
    let gk = g.add_card_to_library(0, catalog::glyph_keeper());
    let vz = g.add_card_to_hand(0, catalog::vizier_of_the_anointed());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, vz, None).expect("vizier");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == gk), "entombed");
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: gk,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
        mode: None,
    })
    .expect("embalm");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "embalm draws");
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Glyph Keeper").expect("token");
    assert_eq!(tok.definition.printed_colors(), vec![Color::White], "embalm makes it white");
}

/// CR 700.2 — Splinter & Leo's default line: the counters on your own team,
/// the Mutant to a different player (each mode targets a different player).
#[test]
fn splinter_and_leo_counters_your_team() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sl = g.add_card_to_hand(0, catalog::splinter_leo_father_son());
    flood(&mut g, 0);
    cast(&mut g, sl, None).expect("cast");
    let bear_counters = g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne);
    assert_eq!(bear_counters, 1);
    assert_eq!(g.battlefield_find(sl).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 0, "other creatures");
    let mutants: Vec<usize> = g.battlefield.iter().filter(|c| c.definition.name == "Mutant").map(|c| c.controller).collect();
    assert!(mutants.iter().all(|&s| s != 0), "the token mode went to another player: {mutants:?}");
}

/// CR 615 — Djeru shaves 1 off damage to a planeswalker you control.
#[test]
fn djeru_shields_your_planeswalkers() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    ready(&mut g, 1, catalog::djeru_with_eyes_open());
    let lili = ready(&mut g, 1, catalog::liliana_death_mage());
    let start = g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(lili))).expect("bolt");
    assert_eq!(g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty), start - 2);
}

/// Elspeth's Talent grants "[+1]: three Soldiers"; activating it (a loyalty
/// ability of the enchanted planeswalker) pumps the team +2/+2 and vigilance.
#[test]
fn elspeths_talent_grants_soldiers_and_pumps_on_loyalty() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_death_mage());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let et = g.add_card_to_hand(0, catalog::elspeths_talent());
    flood(&mut g, 0);
    cast(&mut g, et, Some(Target::Permanent(lili))).expect("cast");
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lili, ability_index: 3, target: None, x_value: None })
        .expect("granted +1");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 3);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4));
    assert!(b.has_keyword(&crabomination::card::Keyword::Vigilance));
}
