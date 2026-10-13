//! Commander: The Emperor of Palamecia and Owen Grady's last gaps
//! (`decks::cmdr_edhrec21`).

use crabomination::card::{CardId, Keyword};
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
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, mode: Option<usize>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on the battlefield");
    (cp.power, cp.toughness)
}

fn has(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(&kw)
}

/// Kinetic Augur's power counts instants and sorceries in your graveyard; its
/// ETB discards up to two and draws that many.
#[test]
fn kinetic_augur_counts_spells_and_loots() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::opt());
    g.add_card_to_graveyard(0, catalog::island());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::kinetic_augur());
    cast(&mut g, id, &[]).expect("cast");
    assert_eq!(pt(&g, id), (2, 4));
    assert_eq!(g.players[0].hand.len(), 2, "hand size kept: discarded n, drew n");
}

/// Clout of the Dominus reads the enchanted creature's colours: a blue-red
/// creature gets both halves, a red one only +1/+1 and haste.
#[test]
fn clout_of_the_dominus_by_colour() {
    for (def, blue) in [(catalog::the_emperor_of_palamecia as fn() -> _, true), (catalog::goblin_guide, false)] {
        let mut g = pod(2);
        let host = ready(&mut g, 0, def());
        let (p, t) = pt(&g, host);
        let aura = g.add_card_to_hand(0, catalog::clout_of_the_dominus());
        cast(&mut g, aura, &[Target::Permanent(host)]).expect("cast");
        assert_eq!(pt(&g, host), (p + if blue { 2 } else { 1 }, t + if blue { 2 } else { 1 }));
        assert!(has(&g, host, Keyword::Haste));
        assert_eq!(has(&g, host, Keyword::Shroud), blue);
    }
}

/// Protective Bubble: unblockable and shroud.
#[test]
fn protective_bubble_grants_both() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::protective_bubble());
    cast(&mut g, aura, &[Target::Permanent(bear)]).expect("cast");
    assert!(has(&g, bear, Keyword::Unblockable) && has(&g, bear, Keyword::Shroud));
}

/// Jace, Memory Adept: +1 draws and mills the target one; 0 mills ten.
#[test]
fn jace_memory_adept_mills() {
    let mut g = pod(3);
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(2, catalog::island());
    }
    let jace = ready(&mut g, 0, catalog::jace_memory_adept());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace,
        ability_index: 0,
        target: Some(Target::Player(2)),
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    assert_eq!((g.players[0].hand.len(), g.players[2].graveyard.len()), (1, 1));
    let mut g = pod(3);
    for _ in 0..12 {
        g.add_card_to_library(2, catalog::island());
    }
    let jace = ready(&mut g, 0, catalog::jace_memory_adept());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace,
        ability_index: 1,
        target: Some(Target::Player(2)),
        x_value: None,
    })
    .expect("0");
    drain_stack(&mut g);
    assert_eq!(g.players[2].graveyard.len(), 10);
}

/// Experimental Overload: an X/X Weird for the spells in your graveyard, one
/// of them back to hand, and the spell itself exiled.
#[test]
fn experimental_overload_sizes_and_regrows() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::opt());
    g.add_card_to_graveyard(0, catalog::shock());
    let id = g.add_card_to_hand(0, catalog::experimental_overload());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id, &[]).expect("cast");
    let weird = g.battlefield.iter().find(|c| c.definition.name == "Weird").map(|c| c.id).expect("token");
    assert_eq!(pt(&g, weird), (3, 3));
    assert_eq!(g.players[0].hand.len(), 1, "one spell back");
    assert!(g.exile.iter().any(|c| c.id == id), "exiled, not in the graveyard");
}

/// Owen Grady's chosen keyword counter lands on a Dinosaur (CR 122.1b), and
/// only at sorcery speed.
#[test]
fn owen_grady_puts_keyword_counters_on_dinosaurs() {
    let mut g = pod(2);
    let owen = ready(&mut g, 0, catalog::owen_grady_raptor_trainer());
    let dino = ready(&mut g, 0, catalog::tranquil_frillback());
    activate(&mut g, owen, 0, Some(Target::Permanent(dino)), Some(1)).expect("trample");
    assert!(has(&g, dino, Keyword::Trample));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let owen2 = ready(&mut g, 0, catalog::owen_grady_raptor_trainer());
    assert!(activate(&mut g, owen2, 0, Some(Target::Permanent(bear)), Some(0)).is_err(), "Dinosaurs only");
}

/// Dino DNA exiles a creature card from any graveyard, then mints a 6/6 green
/// trampling Dinosaur copy of it.
#[test]
fn dino_dna_imprints_and_copies() {
    let mut g = pod(2);
    let dna = ready(&mut g, 0, catalog::dino_dna());
    let elf = g.add_card_to_graveyard(1, catalog::llanowar_elves());
    activate(&mut g, dna, 1, None, None).expect_err("nothing imprinted yet");
    activate(&mut g, dna, 0, Some(Target::Permanent(elf)), None).expect("imprint");
    assert!(g.players[1].graveyard.is_empty());
    activate(&mut g, dna, 1, None, None).expect("copy");
    let tok = g.battlefield.iter().find(|c| c.is_token).expect("token");
    let cp = g.computed_permanent(tok.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6));
    assert!(cp.keywords().contains(&Keyword::Trample));
    assert_eq!(tok.definition.name, "Llanowar Elves");
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Dinosaur));
    assert!(!cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Elf));
}

/// Monster Manual puts a creature from hand onto the battlefield; its
/// adventure mills five and returns a creature.
#[test]
fn monster_manual_both_halves() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    let book = g.add_card_to_hand(0, catalog::monster_manual());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAdventure {
        card_id: book,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("adventure");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert_eq!(g.players[0].graveyard.len(), 4);
    let manual = ready(&mut g, 0, catalog::monster_manual());
    activate(&mut g, manual, 0, None, None).expect("put");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Tranquil Frillback: each {G} paid buys one mode.
#[test]
fn tranquil_frillback_pays_per_mode() {
    let mut g = pod(2);
    let relic = ready(&mut g, 1, catalog::sol_ring());
    g.add_card_to_graveyard(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::tranquil_frillback());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),
    ]));
    cast(&mut g, id, &[]).expect("cast");
    assert!(g.battlefield_find(relic).is_none(), "destroyed");
    assert_eq!(g.players[1].graveyard.len(), 2, "graveyard mode declined: the Island and the Sol Ring stay");
    assert_eq!(g.players[0].life, 24);
}

/// Commune with Dinosaurs takes a Dinosaur from the top five.
#[test]
fn commune_with_dinosaurs_takes_a_dinosaur() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::tranquil_frillback());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::commune_with_dinosaurs());
    cast(&mut g, id, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].hand[0].definition.name, "Tranquil Frillback");
}

/// Savage Order needs a power-4 sacrifice and fetches an indestructible
/// Dinosaur.
#[test]
fn savage_order_sacrifices_and_tutors() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::tranquil_frillback());
    let small = ready(&mut g, 0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::savage_order());
    assert!(cast(&mut g, id, &[]).is_err(), "a 2/2 can't pay it");
    assert!(g.battlefield_find(small).is_some());
    ready(&mut g, 0, catalog::hill_giant());
    let big = ready(&mut g, 0, catalog::craw_wurm());
    cast(&mut g, id, &[]).expect("cast");
    assert!(g.battlefield_find(big).is_none(), "the 6/4 paid");
    let dino = g.battlefield.iter().find(|c| c.definition.name == "Tranquil Frillback").map(|c| c.id).expect("found");
    assert!(has(&g, dino, Keyword::Indestructible));
}
