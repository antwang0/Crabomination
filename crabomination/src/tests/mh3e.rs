//! Functionality tests for the MH3 batch-5 cards in `catalog::sets::mh3e`.

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// The colorless mana ability taps for {C}.
#[test]
fn bountiful_landscape_taps_for_colorless() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::bountiful_landscape());
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "one colorless produced");
}

/// The sac ability fetches an eligible basic onto the battlefield tapped.
#[test]
fn bountiful_landscape_sac_fetches_basic_tapped() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::bountiful_landscape());
    let mountain = g.add_card_to_library(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(mountain))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac to fetch");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == land) == false, "Landscape sacrificed");
    let fetched = g.battlefield.iter().find(|c| c.id == mountain).expect("Mountain fetched");
    assert!(fetched.tapped, "fetched basic enters tapped");
}

/// A non-eligible basic (Swamp — off Bountiful's F/I/M list) can't be fetched.
#[test]
fn bountiful_landscape_wont_fetch_off_color_basic() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::bountiful_landscape());
    let swamp = g.add_card_to_library(0, catalog::swamp());
    // Decider would pick the swamp, but the filter excludes it → nothing fetched.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(swamp))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac to fetch");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != swamp), "off-color Swamp not fetched");
}

/// Vega draws when you cast a spell from your graveyard (not from hand).
#[test]
fn vega_draws_on_graveyard_cast() {
    // Isolate Vega's extra draw by comparing library depletion with vs.
    // without Vega on the battlefield when flashing back the same spell.
    fn deplete_with(vega: bool) -> usize {
        let mut g = two_player_game();
        for _ in 0..20 { g.add_card_to_library(0, catalog::island()); }
        if vega {
            g.add_card_to_battlefield(0, catalog::vega_the_watcher());
        }
        let id = g.add_card_to_library(0, catalog::faithless_looting());
        let pos = g.players[0].library.iter().position(|c| c.id == id).unwrap();
        let card = g.players[0].library.remove(pos);
        g.players[0].graveyard.push(card);
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        let before = g.players[0].library.len();
        g.perform_action(GameAction::CastFlashback {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("flashback Faithless Looting");
        drain_stack(&mut g);
        before - g.players[0].library.len()
    }
    // Faithless Looting draws 2; Vega adds exactly one more.
    assert_eq!(deplete_with(false), 2, "without Vega only the spell's draws");
    assert_eq!(deplete_with(true), 3, "Vega draws one extra on the graveyard cast");
}

/// Chthonian Nightmare enters and grants three energy.
#[test]
fn chthonian_nightmare_etb_gives_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::chthonian_nightmare());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Chthonian Nightmare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 3, "ETB grants three energy");
}

/// Pay X {E} + sac a creature + bounce self to reanimate a MV-X creature.
#[test]
fn chthonian_nightmare_reanimates_by_energy_x() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let nightmare = g.add_card_to_battlefield(0, catalog::chthonian_nightmare());
    g.players[0].energy = 5;
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Target: a MV-2 creature card in the graveyard (Grizzly Bears is {1}{G}).
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: nightmare, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(dead)),
        additional_targets: vec![], x_value: Some(2),
    }).expect("activate for X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 3, "spent two energy");
    assert!(g.battlefield.iter().any(|c| c.id == dead), "MV-2 creature reanimated");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder creature sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == nightmare), "enchantment returned to hand");
}

/// Glimpse the Impossible exiles three, and uncast cards become Spawn at end.
#[test]
fn glimpse_the_impossible_exiles_then_spawns() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::glimpse_the_impossible());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Glimpse the Impossible");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3, "three cards exiled");
    // Fire the next-end-step penalty for the uncast cards.
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 3,
        "each uncast card makes an Eldrazi Spawn");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3,
        "uncast cards go to graveyard");
}

/// Cycling the land for its three colors draws a card.
#[test]
fn contaminated_landscape_cycles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::contaminated_landscape());
    let _ = g.add_card_to_library(0, catalog::forest());
    for c in [Color::White, Color::Blue, Color::Black] {
        g.players[0].mana_pool.add(c, 1);
    }
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle for {W}{U}{B}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "cycled land in graveyard");
    assert_eq!(g.players[0].hand.len(), before, "discard one, draw one");
}
