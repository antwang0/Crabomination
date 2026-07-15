//! Functionality tests for `catalog::sets::decks::recent155` (MKM wave).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::two_player_game;
use crabomination::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Auspicious Arrival pumps a creature and makes a Clue.
#[test]
fn auspicious_arrival_pumps_and_investigates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::auspicious_arrival());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Auspicious Arrival");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((4, 4)), "+2/+2");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue" && c.controller == 0), "made a Clue");
}

/// Benthic Criminologists' ETB sacrifices an artifact to draw.
#[test]
fn benthic_criminologists_sac_for_draw() {
    let mut g = two_player_game();
    g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::benthic_criminologists());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "sacrificed an artifact and drew");
}

/// Agency Coroner sacrifices another creature to draw.
#[test]
fn agency_coroner_sac_draw() {
    let mut g = two_player_game();
    let coroner = g.add_card_to_battlefield(0, catalog::agency_coroner());
    g.clear_sickness(coroner);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    fill_mana(&mut g);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: coroner, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Agency Coroner");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the other creature");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Call a Surprise Witness reanimates a small creature with a flying counter.
#[test]
fn call_a_surprise_witness_reanimates_flying() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::call_a_surprise_witness());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Call a Surprise Witness");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "reanimated to the battlefield");
    assert!(g.computed_permanent(dead).unwrap().keywords.contains(&Keyword::Flying),
        "the flying counter grants flying");
}
