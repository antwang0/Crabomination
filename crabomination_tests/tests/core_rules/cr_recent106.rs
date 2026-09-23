//! CR 508.4 — a creature put onto the battlefield attacking was never
//! declared as an attacker, so "whenever ~ attacks" abilities don't trigger
//! for it. General Kreat's "whenever a Goblin you control attacks, create a
//! Goblin that's tapped and attacking" used to fire again for each token it
//! made: 922 Goblins on one seat by turn 57 of a 15-seat pod game.
//!
//! CR 102.2 — "an opponent controls four or more nonbasic lands" asks each
//! opponent on their own (`Predicate::AnOpponentControlsAtLeast`).
//!
//! CR 506.3 — "creatures attacking you" are those whose defender is you, not
//! every attacker at the table (`SelectionRequirement::IsAttackingYou`).

use crabomination::card::SelectionRequirement;
use crabomination::catalog;
use crabomination::effect::Predicate;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

#[test]
fn cr_508_4_a_token_entering_attacking_does_not_trigger_attacks() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let kreat = g.add_card_to_battlefield(0, catalog::general_kreat_the_boltbringer());
    g.clear_sickness(kreat);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kreat,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let goblins: Vec<_> =
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Goblin").map(|c| c.id).collect();
    assert_eq!(goblins.len(), 1, "one declared Goblin, one token");
    let tok = goblins[0];
    assert!(g.battlefield_find(tok).unwrap().tapped);
    assert!(g.attacking().iter().any(|a| a.attacker == tok), "still an attacking creature");
}

#[test]
fn cr_102_2_an_opponent_controls_at_least_counts_per_opponent() {
    let mut g = multi_player_game(4);
    let pred = Predicate::AnOpponentControlsAtLeast { filter: SelectionRequirement::IsNonbasicLand, n: 4 };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    // Three opponents with two nonbasics each: six across the table, four
    // under no one.
    for seat in 1..4 {
        for _ in 0..2 {
            g.add_card_to_battlefield(seat, catalog::tundra());
        }
    }
    // Basics and your own nonbasics don't count.
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::plains());
        g.add_card_to_battlefield(0, catalog::tundra());
    }
    assert!(!g.evaluate_predicate(&pred, &ctx), "summed across the table, not per opponent");
    for _ in 0..2 {
        g.add_card_to_battlefield(2, catalog::tundra());
    }
    assert!(g.evaluate_predicate(&pred, &ctx), "one opponent has four");
}

#[test]
fn cr_506_3_attacking_you_is_not_attacking_someone_else() {
    use crabomination::card::Value;
    use crabomination::effect::Selector;
    let mut g = multi_player_game(3);
    g.active_player_idx = 1;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.set_attacking(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]);
    let count = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::IsAttackingYou)),
        filter: SelectionRequirement::Creature,
    };
    for seat in [0, 2] {
        let ctx = EffectContext::for_spell(seat, None, 0, 0);
        let one = Predicate::ValueEquals(count.clone(), Value::Const(1));
        assert!(g.evaluate_predicate(&one, &ctx), "seat {seat} is attacked by one");
    }
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let none = Predicate::ValueEquals(count, Value::Const(0));
    assert!(g.evaluate_predicate(&none, &ctx), "the attacker is attacked by none");
}
