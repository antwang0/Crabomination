//! CR conformance for the combat-restriction rules exercised by the GTC wave:
//! CR 509.1b ("can't be blocked except by"), CR 508.1d ("attacks each combat
//! if able"), and CR 615.6 (a prevention shield stops damage to a creature).

use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::game::GameAction;

/// CR 509.1b — Deathcult Rogue "can't be blocked except by Rogues": a non-Rogue
/// blocker is illegal, a Rogue blocker is legal.
#[test]
fn cr_509_1b_block_restriction_by_subtype() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield(0, catalog::deathcult_rogue());
    let non_rogue = g.add_card_to_battlefield(1, catalog::gutter_skulk()); // Zombie Rat
    let a_rogue = g.add_card_to_battlefield(1, catalog::syndicate_enforcer()); // Human Rogue
    g.clear_sickness(rogue);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: rogue, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(non_rogue, rogue)])).is_err(),
        "a non-Rogue can't block Deathcult Rogue");
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(a_rogue, rogue)])).is_ok(),
        "a Rogue may block it");
}

/// CR 508.1d — a creature with "attacks each combat if able" (granted by
/// Hellraiser Goblin) must be in the declared attack set.
#[test]
fn cr_508_1d_must_attack_is_enforced() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(0, catalog::hellraiser_goblin());
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    // Hellraiser grants haste, so both are able to attack this turn.
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(g.declare_attackers(vec![]).is_err(), "declaring zero attackers is illegal while able MustAttack creatures exist");
    g.declare_attackers(vec![
        Attack { attacker: goblin, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]).expect("declaring all able attackers is legal");
}

/// CR 615.6 — Shielded Passage's "prevent all damage to target creature this
/// turn" shield soaks a later damage event.
#[test]
fn cr_615_6_prevention_shield_soaks_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    // Resolve Shielded Passage's effect on the bear.
    let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&Effect::PreventAllDamageThisTurn { target: Selector::Target(0) }, &ctx).unwrap();
    // Now deal 5 noncombat damage — the shield prevents all of it.
    let dctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) }, &dctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "shielded creature survives");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "all damage prevented");
}
