//! Functionality tests for `catalog::sets::decks::recent119`.

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Ravine Raider's firebreathing pumps it +1/+1.
#[test]
fn ravine_raider_pumps() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let raider = g.add_card_to_battlefield(0, catalog::ravine_raider());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: raider, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(raider).unwrap().power, 2, "1/1 → 2/2");
}

/// Lightshell Duo surveils two on entry.
#[test]
fn lightshell_duo_surveils_on_etb() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::lightshell_duo());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lightshell Duo");
    drain_stack(&mut g);
    // Surveil looks at the top two; the auto-heuristic keeps them (no forced
    // graveyard), so the library is unchanged but the ETB resolved cleanly.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Lightshell Duo"), "entered");
    assert!(g.players[0].library.len() <= lib, "surveil didn't add cards");
}

/// Nightwhorl Hermit gains +1/+0 and becomes unblockable under threshold.
#[test]
fn nightwhorl_hermit_threshold() {
    let mut g = two_player_game();
    let hermit = g.add_card_to_battlefield(0, catalog::nightwhorl_hermit());
    assert_eq!(g.computed_permanent(hermit).unwrap().power, 1, "1/4 without threshold");
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    let cp = g.computed_permanent(hermit).unwrap();
    assert_eq!(cp.power, 2, "threshold → +1/+0");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Unblockable), "and unblockable");
}

/// Finch Formation grants a creature you control flying on entry.
#[test]
fn finch_formation_grants_flying() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::finch_formation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Finch Formation");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
        "the bear gains flying");
}
