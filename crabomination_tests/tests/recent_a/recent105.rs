//! Functionality tests for `catalog::sets::decks::recent105` — legacy
//! prison/combo staples (Squee, Dark Depths, Smokestack, Tangle Wire) and
//! the Dimir Charm mode-3 primitive.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

/// Squee recasts from the graveyard and from exile for its mana cost.
#[test]
fn squee_recasts_from_graveyard_and_exile() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // From the graveyard (ability 0).
    let squee = g.add_card_to_graveyard(0, catalog::squee_the_immortal());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: squee, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("recast from graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(squee).is_some(), "back from the graveyard");
    // Exile it, then recast from exile (ability 1).
    g.remove_from_battlefield_to_exile(squee);
    assert!(g.exile.iter().any(|c| c.id == squee));
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: squee, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("recast from exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(squee).is_some(), "back from exile");
}

/// Dark Depths enters with ten ice counters; removing the last one
/// sacrifices it for Marit Lage.
#[test]
fn dark_depths_hatches_marit_lage() {
    let mut g = two_player_game();
    let depths = g.move_card_to_battlefield_for_test(0, catalog::dark_depths());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(depths).unwrap().counter_count(CounterType::Ice), 10);
    g.battlefield_find_mut(depths).unwrap().counters.insert(CounterType::Ice, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: depths, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("remove the last ice counter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(depths).is_none(), "Dark Depths sacrificed");
    let lage = g.battlefield.iter().find(|c| c.definition.name == "Marit Lage")
        .expect("Marit Lage token");
    assert_eq!((lage.power(), lage.toughness()), (20, 20));
}

/// Smokestack accrues soot at your upkeep and makes each player sacrifice
/// per counter at their upkeep.
#[test]
fn smokestack_taxes_each_upkeep() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let stack = g.add_card_to_battlefield(0, catalog::smokestack());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(stack).unwrap().counter_count(CounterType::Soot), 1,
        "soot added at your upkeep");
    // Opponent's upkeep: they sacrifice one permanent.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed their only permanent");
}

/// Tangle Wire taps the active player's permanents per fade counter.
#[test]
fn tangle_wire_taps_per_fade_counter() {
    let mut g = two_player_game();
    let wire = g.move_card_to_battlefield_for_test(0, catalog::tangle_wire());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wire).unwrap().counter_count(CounterType::Fade), 4,
        "Fading 4 enters with four fade counters");
    g.battlefield_find_mut(wire).unwrap().counters.insert(CounterType::Fade, 2);
    let l1 = g.add_card_to_battlefield(1, catalog::island());
    let l2 = g.add_card_to_battlefield(1, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // Two fade counters → two taps, lands preferred.
    assert!(g.battlefield_find(l1).unwrap().tapped && g.battlefield_find(l2).unwrap().tapped);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "creature spared (lands first)");
}

/// Dimir Charm mode 3 keeps the opponent's worst card on top and bins the
/// other two.
#[test]
fn dimir_charm_mode_three_strands_the_worst_card() {
    let mut g = two_player_game();
    // Opponent library (top → bottom): Serra Angel (MV 5), Island, Bears (MV 2).
    g.add_card_to_library(1, catalog::serra_angel());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let charm = g.add_card_to_hand(0, catalog::dimir_charm());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: charm, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("mode 3 at the opponent");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), 1, "one card kept on top");
    assert_eq!(g.players[1].library[0].definition.name, "Island",
        "the lowest-MV card stays");
    assert_eq!(g.players[1].graveyard.len(), 2, "the rest milled");
}
