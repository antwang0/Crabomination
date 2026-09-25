//! Commander: the Living Energy precon (DRC, Saheeli, `decks::cmdr_saheeli_radiant`).

use crabomination::card::{CardId, CardType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, mode: Option<usize>, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Nissa, Worldsoul Speaker (CR 118.9): eight energy instead of the mana cost.
#[test]
fn nissa_casts_permanents_for_energy() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::nissa_worldsoul_speaker());
    g.players[0].energy = 9;
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: wurm,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("pay eight energy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some());
    assert_eq!(g.players[0].energy, 1);
}

/// Stridehangar Automaton: a Treasure comes with a Thopter, which it pumps.
#[test]
fn stridehangar_adds_a_thopter_to_artifact_tokens() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::stridehangar_automaton());
    let vault = g.add_card_to_battlefield(0, catalog::treasure_vault());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vault,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("vault");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Treasure").len(), 2);
    let thopters = named(&g, 0, "Thopter");
    assert_eq!(thopters.len(), 1, "one extra Thopter per batch");
    assert_eq!(g.computed_permanent(thopters[0]).unwrap().power, 2);
}

/// Aetheric Amplifier's second mode doubles your energy.
#[test]
fn aetheric_amplifier_doubles_your_counters() {
    let mut g = main_phase(2);
    let amp = g.add_card_to_battlefield(0, catalog::aetheric_amplifier());
    g.players[0].energy = 5;
    g.players[0].experience = 2;
    activate(&mut g, amp, 1, Some(1), &[]).expect("double");
    assert_eq!((g.players[0].energy, g.players[0].experience), (10, 4));
}

/// Confiscation Coup: four energy, then the target's mana value in energy
/// takes it.
#[test]
fn confiscation_coup_pays_energy_to_steal() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let coup = g.add_card_to_hand(0, catalog::confiscation_coup());
    cast_at(&mut g, coup, &[Target::Permanent(giant)]).expect("cast");
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 0);
    assert_eq!(g.players[0].energy, 4 - 4);
}

/// Saheeli: three energy at combat makes a hasty 5/5 artifact copy that is
/// sacrificed at the end step.
#[test]
fn saheeli_copies_a_permanent_as_a_5_5_artifact() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::saheeli_radiant_creator());
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.players[0].energy = 3;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    fire(&mut g, TurnStep::BeginCombat);
    let copies: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.controller == 0).map(|c| c.id).collect();
    assert_eq!(copies.len(), 1);
    let cp = g.computed_permanent(copies[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.card_types().contains(&CardType::Artifact));
    assert_eq!(g.players[0].energy, 0);
    fire(&mut g, TurnStep::End);
    assert!(g.battlefield_find(copies[0]).is_none(), "sacrificed at the end step");
}

/// Territorial Aetherkite: the energy paid is the damage to each other creature.
#[test]
fn territorial_aetherkite_spends_energy_as_damage() {
    let mut g = main_phase(2);
    g.players[0].energy = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let kite = g.add_card_to_hand(0, catalog::territorial_aetherkite());
    cast_at(&mut g, kite, &[]).expect("cast");
    assert!(g.battlefield_find(bear).is_none(), "three damage (1 + 2 energy)");
    assert!(g.battlefield_find(wurm).is_some());
    assert!(g.battlefield_find(kite).is_some(), "not itself");
}

/// Pia Nalaar: the end-step energy becomes an X/X flying Vehicle.
#[test]
fn pia_nalaar_builds_an_aetherjet() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::pia_nalaar_chief_mechanic());
    g.players[0].energy = 4;
    fire(&mut g, TurnStep::End);
    let jet = named(&g, 0, "Nalaar Aetherjet");
    assert_eq!(jet.len(), 1);
    let c = g.battlefield_find(jet[0]).unwrap();
    assert_eq!((c.definition.power, c.definition.toughness), (4, 4));
}

/// Aetherflux Conduit: energy equal to the mana spent on each spell.
#[test]
fn aetherflux_conduit_counts_mana_spent() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::aetherflux_conduit());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    cast_at(&mut g, wurm, &[]).expect("cast");
    assert_eq!(g.players[0].energy, 6);
}
