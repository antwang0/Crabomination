//! CR 601.2c — "for each opponent, [verb] up to one target X that player
//! controls". `Effect::ForEachOpponentTarget` is the constraint that says it:
//! at most one target per controller, at most one per opponent. Two-player
//! coverage of the cards themselves lives with their sets; these are the
//! assertions only a pod can make.

use crabomination::catalog;
use crabomination::game::*;

/// CR 601.2c — at three seats Sylvan Primordial destroys one noncreature
/// permanent from **each** opponent. Seat 1 gets two candidates and seat 2
/// one, so a picker that only avoided duplicate *objects* would take both of
/// seat 1's and leave seat 2 untouched.
#[test]
fn cr_601_2c_one_target_per_opponent_not_two_from_one() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = multi_player_game(3);
    let a1 = g.add_card_to_battlefield(1, catalog::razortip_whip());
    let a2 = g.add_card_to_battlefield(1, catalog::mind_stone());
    let b1 = g.add_card_to_battlefield(2, catalog::guardian_idol());
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    let prim = g.add_card_to_battlefield(0, catalog::sylvan_primordial());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.fire_self_etb_triggers(prim, 0);
    drain_stack(&mut g);

    let gone = |g: &GameState, id| g.battlefield_find(id).is_none();
    assert!(gone(&g, b1), "seat 2's only artifact is destroyed");
    assert_eq!(
        [a1, a2].iter().filter(|id| gone(&g, **id)).count(),
        1,
        "exactly one of seat 1's two — never both",
    );
}

/// An opponent with nothing to target costs the others nothing: the clause is
/// per opponent, and an iteration with no legal target simply does nothing.
#[test]
fn cr_601_2c_an_empty_opponent_does_not_consume_another_opponents_target() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = multi_player_game(3);
    let a1 = g.add_card_to_battlefield(1, catalog::razortip_whip());
    let a2 = g.add_card_to_battlefield(1, catalog::mind_stone());
    // Seat 2 controls nothing.
    let f1 = g.add_card_to_library(0, catalog::forest());
    let prim = g.add_card_to_battlefield(0, catalog::sylvan_primordial());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(f1))]));
    g.fire_self_etb_triggers(prim, 0);
    drain_stack(&mut g);
    assert_eq!(
        [a1, a2].iter().filter(|id| g.battlefield_find(**id).is_none()).count(),
        1,
        "one of seat 1's, and still only one",
    );
}

/// Two seats is one opponent, so the wrapper must leave a duel exactly where
/// it was — this is the same assertion the per-card tests make, at the level
/// of the primitive.
#[test]
fn cr_601_2c_a_duel_still_takes_exactly_one() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let a1 = g.add_card_to_battlefield(1, catalog::razortip_whip());
    let a2 = g.add_card_to_battlefield(1, catalog::mind_stone());
    let f1 = g.add_card_to_library(0, catalog::forest());
    let prim = g.add_card_to_battlefield(0, catalog::sylvan_primordial());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(f1))]));
    g.fire_self_etb_triggers(prim, 0);
    drain_stack(&mut g);
    assert_eq!(
        [a1, a2].iter().filter(|id| g.battlefield_find(**id).is_none()).count(),
        1,
    );
}

/// Grasp of Fate — the same clause on a permanent's ETB, at four seats: one
/// nonland permanent exiled per opponent, and each from a different one.
#[test]
fn cr_601_2c_grasp_of_fate_exiles_one_per_opponent() {
    let mut g = multi_player_game(4);
    let victims: Vec<_> = (1..4)
        .map(|seat| g.add_card_to_battlefield(seat, catalog::grizzly_bears()))
        .collect();
    // A second candidate on seat 1, so doubling up is possible if unguarded.
    let spare = g.add_card_to_battlefield(1, catalog::mind_stone());
    let grasp = g.add_card_to_battlefield(0, catalog::grasp_of_fate());
    g.fire_self_etb_triggers(grasp, 0);
    drain_stack(&mut g);

    let exiled = |g: &GameState, id| g.battlefield_find(id).is_none();
    assert_eq!(
        victims.iter().filter(|id| exiled(&g, **id)).count()
            + usize::from(exiled(&g, spare)),
        3,
        "three opponents, three permanents",
    );
    for seat in 1..4 {
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == seat).count(),
            if seat == 1 { 1 } else { 0 },
            "seat {seat} kept exactly what the one-per-opponent rule leaves it",
        );
    }
}

/// CR 601.2c on the family's one *spell*: a submitted target list that names
/// two permanents the same opponent controls is rejected as the cast is
/// announced, the same place CR 115.3's distinct-*object* rule is checked.
#[test]
fn cr_601_2c_a_cast_cannot_name_one_opponent_twice() {
    let mut g = multi_player_game(3);
    let a1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let a2 = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let b1 = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::tempted_by_the_oriq());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(1);

    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(a1)),
            additional_targets: vec![Target::Permanent(a2)],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "two targets one opponent controls is not one per opponent",
    );
    // The legal pair — one from each opponent — is accepted.
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a1)),
        additional_targets: vec![Target::Permanent(b1)],
        mode: None,
        x_value: None,
    })
    .expect("one target per opponent");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a1).expect("stolen").controller, 0);
    assert_eq!(g.battlefield_find(b1).expect("stolen").controller, 0);
    assert_eq!(g.battlefield_find(a2).expect("untouched").controller, 1);
}
