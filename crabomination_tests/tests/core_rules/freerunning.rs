//! Functionality tests for the Freerunning batch (CR 702.179) in
//! `catalog::sets::decks::freerunning`.

use crabomination::card::{CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Drive a creature the active player (seat 0) controls into the opponent so the
/// combat-damage gate (`dealt_combat_damage_to_player_this_turn`) gets set.
fn deal_combat_damage(g: &mut GameState) {
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(g);
    advance_to(g, TurnStep::PostCombatMain);
}

/// Brotherhood Ambushers can be cast for its freerunning cost {3}{B} after a
/// creature dealt combat damage to a player this turn.
#[test]
fn brotherhood_ambushers_freeruns_after_combat_damage() {
    let mut g = two_player_game();
    deal_combat_damage(&mut g);
    assert!(g.players[0].dealt_combat_damage_to_player_this_turn, "gate set");
    let id = g.add_card_to_hand(0, catalog::brotherhood_ambushers());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{B}
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("freerunning cast legal after combat damage");
    drain_stack(&mut g);
    let r = g.battlefield_find(id).expect("Ambushers resolved");
    assert_eq!((r.power(), r.toughness()), (6, 3));
}

/// Without combat damage this turn, the freerunning cost is illegal.
#[test]
fn brotherhood_ambushers_freerun_blocked_without_combat_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::brotherhood_ambushers());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let r = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(r.is_err(), "no combat damage this turn → freerunning rejected");
}

/// Achilles Davenport has Menace and pumps other Assassins you control.
#[test]
fn achilles_davenport_pumps_other_assassins() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::merciless_harlequin()); // 2/1 Assassin
    let achilles = g.add_card_to_battlefield(0, catalog::achilles_davenport());
    assert!(g.computed_permanent(achilles).unwrap().keywords.contains(&Keyword::Menace));
    // Achilles doesn't pump itself; the other Assassin gets +1/+1 → 3/2.
    let o = g.computed_permanent(other).unwrap();
    assert_eq!((o.power, o.toughness), (3, 2));
    // Achilles stays a 3/3 (no self-pump).
    let a = g.computed_permanent(achilles).unwrap();
    assert_eq!((a.power, a.toughness), (3, 3));
}

/// Eagle Vision draws three; freerunning cost is {1}{U}.
#[test]
fn eagle_vision_draws_three() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::eagle_vision());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4); // full {4}{U}
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Eagle Vision");
    drain_stack(&mut g);
    // +1 (drew Eagle Vision into hand already counted) — net: -1 (cast) +3 drawn.
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
}

/// Chain Assassination destroys a creature and draws when another died.
#[test]
fn chain_assassination_destroys_and_conditional_draw() {
    let mut g = two_player_game();
    g.players[0].creatures_died_this_turn = 1; // one other creature already died
    g.add_card_to_library(0, catalog::island());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::chain_assassination());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chain Assassination");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target destroyed");
    // The kill makes total deaths ≥2 (prior 1 + victim), so the draw fires.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew on second death");
}

/// Distract the Guards makes three 1/1 Human Rogues.
#[test]
fn distract_the_guards_makes_three_rogues() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::distract_the_guards());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Distract the Guards");
    drain_stack(&mut g);
    let rogues = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Rogue))
        .count();
    assert_eq!(rogues, 3);
}

/// Merciless Harlequin draws a card and loses 1 life on ETB.
#[test]
fn merciless_harlequin_etb_draw_lose_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::merciless_harlequin());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew on ETB");
    assert_eq!(g.players[0].life, 19, "lost 1 life on ETB");
}

/// Viewpoint Synchronization fetches up to three basics into hand (the
/// controller picks each via `Decision::Search`).
#[test]
fn viewpoint_synchronization_fetches_basics() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let f: Vec<_> = (0..4).map(|_| g.add_card_to_library(0, catalog::forest())).collect();
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::viewpoint_synchronization());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f[0])),
        DecisionAnswer::Search(Some(f[1])),
        DecisionAnswer::Search(Some(f[2])),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Viewpoint Synchronization");
    drain_stack(&mut g);
    // Approximation routes all three fetched basics to hand: cast (-1) + 3 = +2
    // beyond the spell card itself, i.e. hand_before + 3.
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
}

/// Escape Detection bounces a creature and cantrips.
#[test]
fn escape_detection_bounces_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::escape_detection());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Escape Detection");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "bounced to owner's hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "cantrip drew (net of the cast)");
}

/// Overpowering Attack untaps attackers and grants an extra combat phase.
#[test]
fn overpowering_attack_untaps_attackers() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.battlefield_find(atk).unwrap().tapped, "attacker is tapped after combat");
    let id = g.add_card_to_hand(0, catalog::overpowering_attack());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Overpowering Attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(atk).unwrap().tapped, "attacker untapped");
}

/// Restart Sequence reanimates a creature card from your graveyard.
#[test]
fn restart_sequence_reanimates() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::restart_sequence());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Restart Sequence");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "creature returned to battlefield");
}
