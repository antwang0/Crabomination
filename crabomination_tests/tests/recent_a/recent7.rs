//! Functionality tests for the `catalog::sets::decks::recent7` batch.


use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

// ── White ────────────────────────────────────────────────────────────────

/// Mardu Woe-Reaper's ETB exiles a graveyard creature and gains 1 life.
#[test]
fn mardu_woe_reaper_exiles_and_gains() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::mardu_woe_reaper());
    g.players[0].mana_pool.add(Color::White, 1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(dead)),
    ]));
    cast(&mut g, id);
    assert!(g.exile.iter().any(|c| c.id == dead), "graveyard creature exiled");
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
}

// ── Blue ─────────────────────────────────────────────────────────────────

/// Peek draws a card.
#[test]
fn peek_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::peek());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand, "net hand unchanged (cast 1, drew 1)");
}

/// Pieces of the Puzzle takes up to two I/S; the rest go to the graveyard.
#[test]
fn pieces_of_the_puzzle_takes_two_instants() {
    let mut g = two_player_game();
    let b1 = catalog::lightning_bolt();
    let b2 = catalog::lightning_bolt();
    let creature = catalog::grizzly_bears();
    let i1 = g.add_card_to_library(0, b1);
    let i2 = g.add_card_to_library(0, b2);
    let c = g.add_card_to_library(0, creature);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::pieces_of_the_puzzle());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(i1))]));
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|x| x.id == i1), "first instant in hand");
    assert!(g.players[0].hand.iter().any(|x| x.id == i2), "second instant auto-filled to hand");
    assert!(g.players[0].graveyard.iter().any(|x| x.id == c), "creature went to graveyard");
}

// ── Black ────────────────────────────────────────────────────────────────

/// Ransack the Lab puts one of three to hand, the rest to the graveyard.
#[test]
fn ransack_the_lab_digs_one() {
    let mut g = two_player_game();
    let keep = g.add_card_to_library(0, catalog::grizzly_bears());
    let g1 = g.add_card_to_library(0, catalog::forest());
    let g2 = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::ransack_the_lab());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(keep))]));
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|x| x.id == keep), "kept card in hand");
    assert!(g.players[0].graveyard.iter().any(|x| x.id == g1), "rest milled");
    assert!(g.players[0].graveyard.iter().any(|x| x.id == g2), "rest milled");
}

// ── Green ────────────────────────────────────────────────────────────────

/// Leaf Gilder taps for green.
#[test]
fn leaf_gilder_taps_for_green() {
    let mut g = two_player_game();
    let dork = g.add_card_to_battlefield(0, catalog::leaf_gilder());
    g.clear_sickness(dork);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for G");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// Quirion Elves taps for green or the chosen color.
#[test]
fn quirion_elves_chosen_color_mana() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::quirion_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.fire_self_etb_triggers(elf, 0);
    drain_stack(&mut g);
    g.clear_sickness(elf);
    g.perform_action(GameAction::ActivateAbility {
        card_id: elf, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for chosen color");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "added the chosen blue");
}

/// Skyshroud Elf filters {1} into red or white.
#[test]
fn skyshroud_elf_filters_into_red() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::skyshroud_elf());
    g.clear_sickness(elf);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: elf, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("filter into red");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}

/// Briar Shield buffs +1/+1 and can be sacrificed for a +3/+3 pump.
#[test]
fn briar_shield_buffs_then_sac_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::briar_shield());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Briar Shield");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2/2 → 3/3 from the Aura");
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for +3/+3");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "base 2 + 3 after the Aura is sacrificed");
    assert!(g.battlefield_find(aura).is_none(), "Aura sacrificed");
}

/// Krosan Tusker's cycle trigger fetches a basic land to hand.
#[test]
fn krosan_tusker_cycle_fetches_basic() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::krosan_tusker());
    let plains = g.add_card_to_library(0, catalog::plains());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == plains), "fetched basic to hand");
}

// ── Lands ────────────────────────────────────────────────────────────────

/// Phyrexian Tower taps for {C}, and can sacrifice a creature for {B}{B}.
#[test]
fn phyrexian_tower_sac_for_black() {
    let mut g = two_player_game();
    let tower = g.add_card_to_battlefield(0, catalog::phyrexian_tower());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: tower, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for BB");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2, "added BB");
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
}
