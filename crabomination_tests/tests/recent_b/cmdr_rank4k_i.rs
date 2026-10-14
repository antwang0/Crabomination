//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_i`).

use crabomination::card::{CardId, CounterType};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}


/// Go-Shintai of Shared Purpose pays {1} for a Spirit per Shrine.
#[test]
fn go_shintai_of_shared_purpose_counts_shrines() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::go_shintai_of_shared_purpose());
    ready(&mut g, 0, catalog::sanctum_of_tranquil_light());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g);
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Spirit"), 2);
}

/// Go-Shintai of Ancient Wars burns a player for the Shrine count.
#[test]
fn go_shintai_of_ancient_wars_burns_per_shrine() {
    // Three seats: a two-seat game keeps the caster-first player pick unless
    // the seat's `hostile_player_targets` is set (every bot profile sets it).
    let mut g = pod(3);
    ready(&mut g, 0, catalog::go_shintai_of_ancient_wars());
    ready(&mut g, 0, catalog::sanctum_of_fruitful_harvest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g);
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.players[1].life + g.players[2].life, 38);
}

/// Sanctum of Tranquil Light's tap costs {1} less per Shrine.
#[test]
fn sanctum_of_tranquil_light_gets_cheaper() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::sanctum_of_tranquil_light());
    ready(&mut g, 0, catalog::go_shintai_of_boundless_vigor());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: s,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{3}{W} with two Shrines");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Sanctum of Fruitful Harvest adds mana per Shrine at your first main.
#[test]
fn sanctum_of_fruitful_harvest_adds_per_shrine() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sanctum_of_fruitful_harvest());
    ready(&mut g, 0, catalog::sanctum_of_shattered_heights());
    g.players[0].mana_pool.empty();
    step(&mut g, TurnStep::PreCombatMain);
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Fíli makes a Dwarf on entry and for another nontoken Dwarf.
#[test]
fn fili_makes_dwarves() {
    let mut g = pod(2);
    let fili = g.add_card_to_hand(0, catalog::fili_the_pathfinder());
    cast(&mut g, fili, None, None);
    assert_eq!(named(&g, 0, "Dwarf"), 1);
    let smith = g.add_card_to_hand(0, catalog::iron_hills_blacksmith());
    cast(&mut g, smith, None, None);
    assert_eq!(named(&g, 0, "Dwarf"), 2);
    assert_eq!(named(&g, 0, "Axe"), 1);
}

/// Dwarven Shortsword arrives on a fresh Dwarf.
#[test]
fn dwarven_shortsword_suits_up_a_dwarf() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::dwarven_shortsword());
    cast(&mut g, s, None, None);
    let dwarf = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Dwarf")
        .map(|c| c.id)
        .expect("Dwarf");
    let c = g.computed_permanent(dwarf).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4));
}

/// Ori destroys opponents' artifacts and enchantments, a life apiece.
#[test]
fn ori_plate_stacker_clears_and_gains() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::sol_ring());
    ready(&mut g, 2, catalog::sol_ring());
    ready(&mut g, 0, catalog::sol_ring());
    let ori = g.add_card_to_hand(0, catalog::ori_plate_stacker());
    cast(&mut g, ori, None, None);
    assert_eq!(named(&g, 1, "Sol Ring") + named(&g, 2, "Sol Ring"), 0);
    assert_eq!(named(&g, 0, "Sol Ring"), 1);
    assert_eq!(g.players[0].life, 22);
}

/// Hapatra taps and stuns one creature per opponent.
#[test]
fn hapatra_freezes_one_per_opponent() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::hapatra_the_desert_frost());
    cast(&mut g, h, None, None);
    for id in [a, b] {
        let c = g.battlefield_find(id).unwrap();
        assert!(c.tapped);
        assert_eq!(c.counter_count(CounterType::Stun), 1);
    }
}

/// The Blue Spirit flashes in the first creature spell of a turn.
#[test]
fn the_blue_spirit_flashes_the_first_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_blue_spirit());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flash");
}

/// Arwen's Gift costs {1} less with two legendary creatures.
#[test]
fn arwens_gift_discount() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::big_bertha());
    ready(&mut g, 0, catalog::the_fabulous_frog_man());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let gift = g.add_card_to_hand(0, catalog::arwens_gift());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: gift,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{2}{U}");
}
