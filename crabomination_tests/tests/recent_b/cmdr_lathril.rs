//! Commander: the Elven Empire precon (KHC, Lathril, `decks::cmdr_lathril`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    stock_libraries(&mut g, 10);
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn play(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> Option<(i32, i32)> {
    g.computed_permanent(id).map(|c| (c.power, c.toughness))
}

fn elf_warriors(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.is_token && c.definition.name == "Elf Warrior").count()
}

fn elves(g: &mut GameState, seat: usize, n: usize) -> Vec<CardId> {
    (0..n).map(|_| g.add_card_to_battlefield(seat, catalog::llanowar_elves())).collect()
}

/// Abomination of Llanowar counts the Elves you control (itself included) and
/// the Elf cards in your graveyard; an opponent's Elves don't count.
#[test]
fn abomination_counts_board_and_graveyard_elves() {
    let mut g = main_phase();
    let abom = g.add_card_to_battlefield(0, catalog::abomination_of_llanowar());
    elves(&mut g, 0, 2);
    elves(&mut g, 1, 3);
    g.add_card_to_graveyard(0, catalog::llanowar_elves());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, abom), Some((4, 4)));
}

/// Bounty of Skemfar takes one land (onto the battlefield tapped) and one Elf
/// card — never two of either.
#[test]
fn bounty_of_skemfar_takes_one_land_and_one_elf() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::llanowar_elves());
    }
    let bounty = g.add_card_to_hand(0, catalog::bounty_of_skemfar());
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    play(&mut g, bounty, None).expect("Bounty");
    let new_lands: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).collect();
    assert_eq!(new_lands.len(), lands + 1, "one land onto the battlefield");
    assert!(new_lands.iter().all(|c| c.tapped));
    let elves_in_hand = g.players[0].hand.iter().filter(|c| c.definition.name == "Llanowar Elves").count();
    assert_eq!(elves_in_hand, 1, "one Elf card to hand");
    assert_eq!(g.players[0].library.len(), 4, "the other four go to the bottom");
}

/// Crown of Skemfar pumps by the Elves you control and grants reach; it
/// returns from the graveyard for {2}{G}.
#[test]
fn crown_of_skemfar_pumps_per_elf_and_recurs() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    elves(&mut g, 0, 3);
    let crown = g.add_card_to_hand(0, catalog::crown_of_skemfar());
    play(&mut g, crown, Some(Target::Permanent(bear))).expect("Crown");
    assert_eq!(pt(&g, bear), Some((5, 5)));
    assert!(g.computed_permanent(bear).is_some_and(|c| c.keywords().contains(&Keyword::Reach)));
    let crown2 = g.add_card_to_graveyard(0, catalog::crown_of_skemfar());
    activate(&mut g, crown2, 0, None).expect("return the Crown");
    assert!(g.players[0].hand.iter().any(|c| c.id == crown2));
}

/// Elderfang Venom drains for each Elf of yours that dies; Eyeblight Massacre
/// kills only non-Elves.
#[test]
fn elderfang_venom_drains_and_massacre_spares_elves() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::elderfang_venom());
    let mine = elves(&mut g, 0, 1)[0];
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let massacre = g.add_card_to_hand(0, catalog::eyeblight_massacre());
    play(&mut g, massacre, None).expect("Massacre");
    assert!(g.battlefield_find(bear).is_none(), "the non-Elf died");
    assert!(g.battlefield_find(mine).is_some(), "the Elf didn't");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    play(&mut g, bolt, Some(Target::Permanent(mine))).expect("Bolt our Elf");
    assert_eq!((g.players[0].life, g.players[1].life), (21, 19), "the Elf's death drained");
}

/// Elven Ambush makes an Elf Warrior per Elf you control.
#[test]
fn elven_ambush_counts_your_elves() {
    let mut g = main_phase();
    elves(&mut g, 0, 3);
    elves(&mut g, 1, 2);
    let ambush = g.add_card_to_hand(0, catalog::elven_ambush());
    play(&mut g, ambush, None).expect("Ambush");
    assert_eq!(elf_warriors(&g, 0), 3);
}

/// Lys Alana Scarblade's cost discards an Elf card; the target gets -X/-X for
/// each Elf you control.
#[test]
fn lys_alana_scarblade_needs_an_elf_to_discard() {
    let mut g = main_phase();
    let blade = g.add_card_to_battlefield(0, catalog::lys_alana_scarblade());
    g.clear_sickness(blade);
    elves(&mut g, 0, 2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(activate(&mut g, blade, 0, Some(Target::Permanent(wurm))).is_err(), "no Elf card to discard");
    g.add_card_to_hand(0, catalog::llanowar_elves());
    activate(&mut g, blade, 0, Some(Target::Permanent(wurm))).expect("Scarblade");
    assert_eq!(pt(&g, wurm), Some((3, 1)), "three Elves: -3/-3");
}

/// Numa pays {X}{X} for X counters: eight floating mana buys four.
#[test]
fn numa_pays_x_twice() {
    let mut g = main_phase();
    let numa = g.add_card_to_battlefield(0, catalog::numa_joraga_chieftain());
    g.players[0].mana_pool.add(Color::Green, 8);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(4)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let counters: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 4);
    assert_eq!(g.players[0].mana_pool.total(), 0);
    assert!(g.battlefield_find(numa).is_some());
}

/// Return Upon the Tide adds two Elf Warriors only when the returned card is
/// an Elf.
#[test]
fn return_upon_the_tide_rewards_an_elf() {
    for (card, tokens) in [(catalog::llanowar_elves(), 2), (catalog::grizzly_bears(), 0)] {
        let mut g = main_phase();
        let dead = g.add_card_to_graveyard(0, card);
        let rut = g.add_card_to_hand(0, catalog::return_upon_the_tide());
        play(&mut g, rut, Some(Target::Permanent(dead))).expect("Return Upon the Tide");
        assert!(g.battlefield_find(dead).is_some());
        assert_eq!(elf_warriors(&g, 0), tokens);
    }
}

/// Ruthless Winnower: each upkeep the active player sacrifices a non-Elf.
#[test]
fn ruthless_winnower_takes_a_non_elf_each_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::ruthless_winnower());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elf = elves(&mut g, 1, 1)[0];
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(elf).is_some());
}

/// Serpent's Soul-Jar exiles your dying Elves and lets you cast them after
/// {T}, 2 life.
#[test]
fn serpents_soul_jar_recasts_an_exiled_elf() {
    let mut g = main_phase();
    let jar = g.add_card_to_battlefield(0, catalog::serpents_soul_jar());
    let elf = elves(&mut g, 0, 1)[0];
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    play(&mut g, bolt, Some(Target::Permanent(elf))).expect("Bolt");
    assert!(g.exile.iter().any(|c| c.id == elf), "the Elf was exiled with the Jar");
    activate(&mut g, jar, 0, None).expect("Jar");
    assert_eq!(g.players[0].life, 18);
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: elf,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the exiled Elf");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_some());
}

/// Skemfar Elderhall: -2/-2 to up to one creature you don't control, plus two
/// Elf Warriors.
#[test]
fn skemfar_elderhall_shrinks_and_makes_elves() {
    let mut g = main_phase();
    let hall = g.add_card_to_battlefield(0, catalog::skemfar_elderhall());
    g.battlefield_find_mut(hall).unwrap().tapped = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, hall, 1, Some(Target::Permanent(bear))).expect("Elderhall");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(hall).is_none(), "sacrificed");
    assert_eq!(elf_warriors(&g, 0), 2);
}

/// Wolverine Riders makes an Elf Warrior each upkeep and gains life equal to
/// an entering Elf's toughness.
#[test]
fn wolverine_riders_makes_elves_and_gains_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::wolverine_riders());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(elf_warriors(&g, 0), 1, "each upkeep, the opponent's too");
    assert_eq!(g.players[0].life, 21, "the token's toughness");
}

/// Tergrid's Shadow: each player sacrifices two creatures.
#[test]
fn tergrids_shadow_takes_two_from_each_player() {
    let mut g = main_phase();
    elves(&mut g, 0, 3);
    elves(&mut g, 1, 2);
    let shadow = g.add_card_to_hand(0, catalog::tergrids_shadow());
    play(&mut g, shadow, None).expect("Shadow");
    let count = |g: &GameState, p| g.battlefield.iter().filter(|c| c.controller == p && c.definition.is_creature()).count();
    assert_eq!((count(&g, 0), count(&g, 1)), (1, 0));
}

/// Jagged-Scar Archers is */* for your Elves and shoots only fliers.
#[test]
fn jagged_scar_archers_shoots_a_flier_for_its_power() {
    let mut g = main_phase();
    let archers = g.add_card_to_battlefield(0, catalog::jagged_scar_archers());
    g.clear_sickness(archers);
    elves(&mut g, 0, 2);
    assert_eq!(pt(&g, archers), Some((3, 3)));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    assert!(activate(&mut g, archers, 0, Some(Target::Permanent(bear))).is_err(), "no flying");
    activate(&mut g, archers, 0, Some(Target::Permanent(angel))).expect("shoot the Angel");
    assert_eq!(g.battlefield_find(angel).map(|c| c.damage), Some(3));
}
