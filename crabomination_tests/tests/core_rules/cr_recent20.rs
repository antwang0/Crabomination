//! CR conformance for rules exercised by this run's GTC wave 15:
//! CR 702.15 (lifelink life gain — via Alms Beast's granted lifelink, near the
//! combat-static work), CR 302.6 (a {T} ability needs the creature to shake off
//! summoning sickness — random), and CR 601.2d (divided-damage distribution —
//! the Tier-2 "divided damage" roadmap item, still 🟡).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameError};

/// CR 702.15b — combat damage from a source with (granted) lifelink makes its
/// controller gain that much life.
#[test]
fn cr_702_15_granted_lifelink_gains_life() {
    let mut g = two_player_game();
    let beast = g.add_card_to_battlefield(0, catalog::alms_beast()); // 6/6
    g.clear_sickness(beast);
    let blocker = g.add_card_to_battlefield(1, catalog::gutter_skulk()); // 2/2
    let p1_life = g.players[1].life;
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: beast, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // The blocker is now in combat with Alms Beast and has lifelink.
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, beast)])).expect("block");
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    // The blocker dealt 2 combat damage to Alms Beast; its lifelink gains P1 2.
    assert_eq!(g.players[1].life, p1_life + 2, "granted lifelink gained 2 life");
}

/// CR 302.6 — a creature's {T} activated ability can't be activated the turn it
/// enters (summoning sickness); it works once the creature has been controlled
/// continuously since the turn began.
#[test]
fn cr_302_6_tap_ability_needs_no_sickness() {
    let mut g = two_player_game();
    let tiger = g.add_card_to_battlefield(0, catalog::zarichi_tiger());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let sick = g.perform_action(GameAction::ActivateAbility {
        card_id: tiger, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(matches!(sick, Err(GameError::SummoningSickness(_))),
        "a summoning-sick creature can't use its {{T}} ability");
    // Once it's no longer sick the same ability resolves.
    g.clear_sickness(tiger);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tiger, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate once not sick");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// CR 601.2d — a divided-damage spell splits its damage among the chosen
/// targets (AutoDecider spreads Forked Bolt's 2 evenly, one to each).
#[test]
fn cr_601_2d_divided_damage_spreads() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::gnarlwood_dryad()); // 1/1
    let b = g.add_card_to_battlefield(1, catalog::gnarlwood_dryad()); // 1/1
    let bolt = g.add_card_to_hand(0, catalog::forked_bolt());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.cast_spell(bolt, Some(Target::Permanent(a)), vec![Target::Permanent(b)], None, None).expect("cast");
    drain_stack(&mut g);
    let _ = CounterType::Charge;
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
        "1 damage to each 1/1 killed both");
}
