//! Functionality tests for `catalog::sets::decks::recent248` (artifact-sacrifice
//! payoffs + modal removal) and the per-turn artifact-sacrifice tracker.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Sacrificing an artifact bumps the per-turn artifact-sacrifice tally; a
/// creature sacrifice does not.
#[test]
fn artifact_sacrifice_tracker_counts_only_artifacts() {
    let mut g = two_player_game();
    let clue_src = g.add_card_to_battlefield(0, catalog::magnifying_glass()); // an artifact
    let mut evs = vec![];
    g.sacrifice_one(clue_src, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.players[0].artifacts_sacrificed_this_turn, 1, "artifact counted");
    // A creature sacrifice bumps the permanent tally but not the artifact one.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut evs = vec![];
    g.sacrifice_one(bear, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.players[0].artifacts_sacrificed_this_turn, 1, "creature not counted");
    assert_eq!(g.players[0].permanents_sacrificed_this_turn, 2, "both permanents counted");
}

/// Suspicious Detonation costs {3} less once you've sacrificed an artifact this
/// turn — castable for {1}{R} instead of {4}{R}.
#[test]
fn suspicious_detonation_cost_reduction_after_artifact_sac() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].artifacts_sacrificed_this_turn = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::suspicious_detonation());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1); // only {1}{R} — the reduced cost
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast for the reduced {1}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "4 damage killed the 2/2");
}

/// Furtive Courier is unblockable only while you've sacrificed an artifact this
/// turn.
#[test]
fn furtive_courier_unblockable_after_artifact_sac() {
    let mut g = two_player_game();
    let courier = g.add_card_to_battlefield(0, catalog::furtive_courier());
    assert!(
        !g.computed_permanent(courier).unwrap().keywords.contains(&Keyword::Unblockable),
        "not unblockable without an artifact sacrifice"
    );
    g.players[0].artifacts_sacrificed_this_turn = 1;
    assert!(
        g.computed_permanent(courier).unwrap().keywords.contains(&Keyword::Unblockable),
        "unblockable once an artifact was sacrificed"
    );
}

/// Deadly Complication's destroy mode kills a target creature.
#[test]
fn deadly_complication_destroys() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::deadly_complication());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Deadly Complication");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "destroy mode killed the creature");
}
