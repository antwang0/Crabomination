//! Commander: the Painbow precon (DMC, Jared Carthalion, `decks::cmdr_jared`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
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

fn named(g: &GameState, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == name).map(|c| c.id).collect()
}

/// Jared (CR 903.3a — it can be your commander): +1 makes an all-colors Kavu
/// (CR 105.2c) that Iridian Maelstrom spares; −3 counts each creature's
/// colors.
#[test]
fn jared_kavu_survives_the_maelstrom() {
    let mut g = pod(2);
    assert!(catalog::jared_carthalion().can_be_commander);
    let j = g.add_card_to_battlefield(0, catalog::jared_carthalion());
    loyalty(&mut g, j, 0, &[]);
    let kavu = named(&g, "Kavu")[0];
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let im = g.add_card_to_hand(0, catalog::iridian_maelstrom());
    cast(&mut g, im, &[]);
    assert!(g.battlefield_find(kavu).is_some());
    assert!(g.battlefield_find(bear).is_none());
    g.battlefield_find_mut(j).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, j, 1, &[Target::Permanent(kavu)]);
    assert_eq!(g.battlefield_find(kavu).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// Jenson and Mana Cannons count a multicolored spell's colors; an
/// all-colors one makes Jenson's Angel.
#[test]
fn multicolored_casts_pay_off() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::jenson_carthalion_druid_exile());
    g.add_card_to_battlefield(0, catalog::mana_cannons());
    let fe = g.add_card_to_hand(0, catalog::fusion_elemental());
    cast(&mut g, fe, &[]);
    assert_eq!(named(&g, "Angel").len(), 1);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 5, "five colors at any target");
}

/// Knight of New Alara pumps each other multicolored creature per color.
#[test]
fn knight_of_new_alara_counts_colors() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::knight_of_new_alara());
    let fe = g.add_card_to_battlefield(0, catalog::fusion_elemental());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(fe).map(|c| c.power), Some(13));
    assert_eq!(g.computed_permanent(bear).map(|c| c.power), Some(2));
}

/// Archelos: untapped, other permanents enter untapped (CR 614.1c) — even an
/// enters-tapped artifact; tapped, they enter tapped.
#[test]
fn archelos_sets_the_entry_state() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::archelos_lagoon_mystic());
    let ob = g.add_card_to_hand(0, catalog::obsidian_obelisk());
    cast(&mut g, ob, &[]);
    assert!(!g.battlefield_find(ob).unwrap().tapped);
    g.battlefield_find_mut(a).unwrap().tapped = true;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Rienne returns a dead multicolored creature at the next end step.
#[test]
fn rienne_brings_them_back() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rienne_angel_of_rebirth());
    let fe = g.add_card_to_battlefield(0, catalog::fusion_elemental());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(fe)]);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fe));
}

/// Zaxara: an X spell makes a Hydra with X counters — the cast trigger reads
/// the spell's X (it used to read 0, so the 0/0 Hydra died).
#[test]
fn zaxara_hatches_hydras() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::zaxara_the_exemplary());
    let fb = g.add_card_to_hand(0, catalog::fireball());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: fb,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("fireball");
    drain_stack(&mut g);
    let hydra = named(&g, "Hydra")[0];
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Xyris: an opponent's draw outside their first draw-step draw makes a
/// Snake.
#[test]
fn xyris_punishes_extra_draws() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::xyris_the_writhing_storm());
    g.add_card_to_library(1, catalog::island());
    let d = g.add_card_to_hand(1, catalog::divination());
    for c in [Color::Blue] {
        g.players[1].mana_pool.add(c, 5);
    }
    g.players[1].mana_pool.add_colorless(5);
    g.add_card_to_library(1, catalog::island());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: d,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("divination");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Snake").len(), 2);
}
