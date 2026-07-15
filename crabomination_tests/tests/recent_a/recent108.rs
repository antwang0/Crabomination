//! Functionality tests for `catalog::sets::decks::recent108` — MH3-era
//! value staples.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::TurnStep;
use crabomination::game::*;

/// Urza's Cave sacs for a tapped land fetch.
#[test]
fn urzas_cave_fetches_tapped() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let cave = g.add_card_to_battlefield(0, catalog::urzas_cave());
    let land = g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(land))]));
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cave, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("fetch");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cave).is_none(), "cave sacrificed");
    assert!(g.battlefield_find(land).unwrap().tapped, "fetched land enters tapped");
}

/// Fallaji Archaeologist grows when the mill whiffs.
#[test]
fn fallaji_archaeologist_grows_on_whiff() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); } // all lands: whiff
    let arch = g.move_card_to_battlefield_for_test(0, catalog::fallaji_archaeologist());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(arch).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Sleep-Cursed Faerie enters tapped with three stun counters and untaps
/// for {1}{U}.
#[test]
fn sleep_cursed_faerie_stunned_start() {
    let mut g = two_player_game();
    let faerie = g.move_card_to_battlefield_for_test(0, catalog::sleep_cursed_faerie());
    drain_stack(&mut g);
    let c = g.battlefield_find(faerie).unwrap();
    assert!(c.tapped, "enters tapped");
    assert_eq!(c.counter_count(CounterType::Stun), 3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: faerie, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("self-untap");
    drain_stack(&mut g);
    // CR 122.1i — the stun counter eats the untap instead.
    let c = g.battlefield_find(faerie).unwrap();
    assert!(c.tapped, "stun counter replaced the untap");
    assert_eq!(c.counter_count(CounterType::Stun), 2, "one stun removed");
    // With the stun cleared, the activation untaps for real.
    g.battlefield_find_mut(faerie).unwrap().counters.remove(&CounterType::Stun);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: faerie, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("self-untap again");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(faerie).unwrap().tapped, "untapped once unstunned");
}

/// Manabond dumps hand lands at end step and discards the rest.
#[test]
fn manabond_dumps_lands() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::manabond());
    let l1 = g.add_card_to_hand(0, catalog::island());
    let l2 = g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(l1).is_some() && g.battlefield_find(l2).is_some(),
        "both lands deployed");
    assert!(g.players[0].hand.is_empty(), "the rest discarded");
}

/// Nissa adds mana on each land drop; the second drop digs an Elf/Elemental.
#[test]
fn nissa_resurgent_animist_ramps_and_digs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nissa_resurgent_animist());
    g.add_card_to_library(0, catalog::llanowar_elves()); // the dig hit
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].extra_land_plays = 1;
    let l1 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(l1)).expect("first land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "landfall mana");
    let hand_before = g.players[0].hand.len();
    let l2 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(l2)).expect("second land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2, "second landfall mana");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "dug up the Elf");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Llanowar Elves"));
}
