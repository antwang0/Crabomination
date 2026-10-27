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
    g.resolve_effect(&Effect::PreventAllDamageThisTurn { target: Selector::Target(0), redirect_to: None }, &ctx).unwrap();
    // Now deal 5 noncombat damage — the shield prevents all of it.
    let dctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) }, &dctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "shielded creature survives");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "all damage prevented");
}

/// CR 614.5 — a replacement effect gets one opportunity at an event and at
/// the events that replace it. Two turn-long redirect shields pointing at
/// each other (damage to the bear goes to the giant, damage to the giant to
/// the bear): the bear's shield sends 2 to the giant, the giant's sends it
/// back, and the bear's shield has already applied, so the bear takes it.
/// Before, the redirected damage re-ran every shield and recursed until the
/// stack overflowed (a cube ladder game, 2026-10-26).
#[test]
fn cr_614_5_a_redirect_shield_does_not_reapply_to_its_own_redirected_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    for (from, to) in [(bear, giant), (giant, bear)] {
        let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(from)));
        ctx.targets.push(Target::Permanent(to));
        g.resolve_effect(&Effect::PreventAllDamageThisTurn { target: Selector::Target(0), redirect_to: Some(Selector::Target(1)) }, &ctx)
            .unwrap();
    }
    let dctx = EffectContext::for_ability(crabomination::card::CardId(0), 1, Some(Target::Permanent(bear)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) }, &dctx).unwrap();
    drain_stack(&mut g);
    let damage = |id| g.battlefield_find(id).map(|c| c.damage);
    assert_eq!(damage(giant), Some(0), "the giant's shield sent it on");
    assert_eq!(g.battlefield_find(bear).map(|c| c.damage).unwrap_or(2), 2, "the bear takes the damage its own shield already redirected");
}

/// CR 614.5 / 104.4b — a Comeuppance on each side: damage to player 1 from
/// player 0's card is reflected at player 0, player 0's Comeuppance reflects
/// it back, and player 1's shield has already applied to that chain, so
/// player 1 takes it. On paper the two shields are a mandatory loop; the
/// engine has to stop, and the per-shield rule is where it stops. Before,
/// this recursed until the stack overflowed (a cube ladder game).
#[test]
fn cr_614_5_two_comeuppances_do_not_reflect_forever() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_hand(0, catalog::comeuppance());
    let mine = g.add_card_to_hand(1, catalog::comeuppance());
    for (card, seat) in [(theirs, 0), (mine, 1)] {
        let ctx = EffectContext::for_ability(card, seat, None);
        g.resolve_effect(&Effect::PreventOpposingDamageThisTurnReflecting { target: Selector::You }, &ctx).unwrap();
    }
    let dctx = EffectContext::for_ability(theirs, 0, Some(Target::Player(1)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) }, &dctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "player 0's reflection was prevented by player 0's own shield");
    assert_eq!(g.players[1].life, 17, "the damage lands on player 1 once its shield has had its turn");
}

/// CR 702.16b — Arc Lightning's extra-target prompt does not offer a creature
/// with protection from red. The cast rejects such a slot, and a rejected
/// answer drops its decision, so offering one sent a prompting caster back
/// before the cast with the same prompt: a bot that picked Kor Firewalker
/// recast Arc Lightning until the game's action cap (cube ladder games,
/// 2026-10-27).
#[test]
fn cr_702_16b_extra_target_prompt_skips_protected_creatures() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let arc = g.add_card_to_hand(0, catalog::arc_lightning());
    let firewalker = g.add_card_to_battlefield(1, catalog::kor_firewalker()); // protection from red
    let bear = g.add_card_to_battlefield(1, catalog::gutter_skulk());
    g.perform_action(GameAction::CastSpell {
        card_id: arc,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the cast suspends on its second slot");
    let legal = match g.pending_decision.as_ref().map(|d| &d.decision) {
        Some(crabomination::decision::Decision::ChooseTarget { legal, .. }) => legal.clone(),
        other => panic!("expected the extra-slot prompt, got {other:?}"),
    };
    assert!(legal.contains(&Target::Permanent(bear)), "an unprotected creature is offered");
    assert!(!legal.contains(&Target::Permanent(firewalker)), "protection from red keeps Kor Firewalker off the list");
}
