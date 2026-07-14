//! Functionality tests for `catalog::sets::decks::recent185` (BLB gaps).

use crate::catalog;
use crate::game::two_player_game;
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Fill `p`'s graveyard with `n` vanilla cards (to toggle threshold).
fn fill_graveyard(g: &mut GameState, p: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_graveyard(p, catalog::grizzly_bears());
    }
}

/// Thought Shucker's threshold ability grows it and draws — once only, and only
/// with seven+ cards in the graveyard.
#[test]
fn thought_shucker_threshold_activate_once() {
    let mut g = two_player_game();
    let shucker = g.add_card_to_battlefield(0, catalog::thought_shucker());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // < 7 cards in graveyard → gated.
    fill_graveyard(&mut g, 0, 6);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: shucker,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .is_err(),
        "threshold not met → activation rejected",
    );
    // Reach threshold and activate.
    fill_graveyard(&mut g, 0, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: shucker,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Thought Shucker");
    drain_stack(&mut g);
    let cp = g.computed_permanent(shucker).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "+1/+1 counter");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    // Activate-once: a second attempt is rejected even with mana + threshold.
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: shucker,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .is_err(),
        "activate only once",
    );
}

/// Shoreline Looter draws on combat damage and skips the discard at threshold.
#[test]
fn shoreline_looter_loots_at_threshold() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let looter = g.add_card_to_battlefield(0, catalog::shoreline_looter());
    g.clear_sickness(looter);
    g.add_card_to_library(0, catalog::grizzly_bears());
    fill_graveyard(&mut g, 0, 7); // threshold active → no discard
    let hand_before = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: looter,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew, no discard at threshold");
}

/// Below threshold, the looter draws then discards — hand size unchanged, a card
/// enters the graveyard.
#[test]
fn shoreline_looter_discards_below_threshold() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let looter = g.add_card_to_battlefield(0, catalog::shoreline_looter());
    g.clear_sickness(looter);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: looter,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "draw then discard nets zero");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "discarded into graveyard");
}

/// Ruthless Negotiation makes a target opponent exile a hand card; the graveyard
/// flashback also draws (cast-from-graveyard rider).
#[test]
fn ruthless_negotiation_flashback_draws() {
    let mut g = two_player_game();
    let spell = g.add_card_to_graveyard(0, catalog::ruthless_negotiation());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // Flashback cost {4}{B}.
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_hand_before = g.players[1].hand.len();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastFlashback {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback Ruthless Negotiation");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opponent exiled a hand card");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "cast-from-graveyard drew a card");
    assert!(g.exile.iter().any(|c| c.id == spell), "flashback exiles the spell");
}

/// Seasoned Warrenguard pumps only when you control a token as it attacks.
#[test]
fn seasoned_warrenguard_token_gated_pump() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let guard = g.add_card_to_battlefield(0, catalog::seasoned_warrenguard());
    g.clear_sickness(guard);
    // No token yet → attacking gives no bonus.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: guard,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(guard).unwrap().power, 1, "no token → no pump");

    // Fresh turn with a token controlled → +2/+0.
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let guard = g.add_card_to_battlefield(0, catalog::seasoned_warrenguard());
    g.clear_sickness(guard);
    let tok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: guard,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(guard).unwrap().power, 3, "token → +2/+0");
}

/// Valley Flamecaller adds 1 to the damage its typed creatures deal.
#[test]
fn valley_flamecaller_boosts_typed_damage() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.players[1].life = 20;
    let flamecaller = g.add_card_to_battlefield(0, catalog::valley_flamecaller());
    g.clear_sickness(flamecaller);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flamecaller,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "3 power + 1 = 4 combat damage");
}
