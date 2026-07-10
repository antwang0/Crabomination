//! Functionality tests for the MH3 batch-4 cards in `catalog::sets::mh3d`.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
}

fn cast(g: &mut GameState, id: crate::card::CardId, target: Option<Target>) {
    fill_mana(g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(g);
}

/// Ugin's Binding returns a nonland permanent an opponent controls to hand.
#[test]
fn ugins_binding_bounces_opponent_permanent() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::ugins_binding());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced to owner's hand");
}

/// Abstruse Appropriation exiles a nonland permanent and grants a cast
/// permission that lasts as long as it stays exiled.
#[test]
fn abstruse_appropriation_exiles_and_grants_recast() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::abstruse_appropriation());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("bear exiled");
    assert!(exiled.may_play_until.is_some(), "granted a cast permission");
}

/// Expel the Unworthy exiles a small creature; its controller gains life equal
/// to its mana value (Grizzly Bears = MV 2).
#[test]
fn expel_the_unworthy_exiles_small_and_gains_life() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::expel_the_unworthy());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    assert_eq!(g.players[1].life, life + 2, "controller gains life = mana value");
}

/// Kicked, Expel the Unworthy can exile a creature of any mana value.
#[test]
fn expel_the_unworthy_kicked_hits_large_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    let life = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::expel_the_unworthy());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == angel), "large creature exiled when kicked");
    assert_eq!(g.players[1].life, life + 5, "controller gains life = mana value 5");
}

/// Twisted Riddlekeeper's cast trigger taps two permanents and stuns each.
#[test]
fn twisted_riddlekeeper_taps_and_stuns_two() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::twisted_riddlekeeper());
    fill_mana(&mut g);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(a)),
        DecisionAnswer::Target(Target::Permanent(b)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("hardcast");
    drain_stack(&mut g);
    for id in [a, b] {
        let p = g.battlefield_find(id).unwrap();
        assert!(p.tapped, "target tapped");
        assert_eq!(p.counter_count(CounterType::Stun), 1, "stun counter added");
    }
}

/// Depth Defiler cast unkicked resolves a single chosen mode (bounce).
#[test]
fn depth_defiler_unkicked_bounces_one_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::depth_defiler());
    fill_mana(&mut g);
    // Mode 0 = bounce, then target the only creature.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(0),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast unkicked");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "unkicked bounce mode");
}

/// Kicked, Depth Defiler performs both modes: bounce and draw-two-discard-one.
#[test]
fn depth_defiler_kicked_does_both() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let junk = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let spell = g.add_card_to_hand(0, catalog::depth_defiler());
    fill_mana(&mut g);
    // The bounce leg auto-targets the only creature; only the discard needs a
    // scripted answer.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![junk])]));
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "kicked bounce leg");
    // Started with hand_before (incl. junk), removed Depth Defiler on cast,
    // drew 2, discarded 1 → net +1 over the post-cast hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew two, discarded one");
}

/// Dog Umbra's umbra armor redirects a lethal damage marking to itself.
#[test]
fn dog_umbra_saves_enchanted_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let umbra = g.add_card_to_hand(0, catalog::dog_umbra());
    cast(&mut g, umbra, Some(Target::Permanent(bear)));
    g.battlefield_find_mut(bear).unwrap().damage = 5;
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "creature saved by umbra armor");
    assert!(g.battlefield_find(umbra).is_none(), "aura destroyed instead");
}

/// Thief of Existence exiles an opponent's small noncreature permanent and
/// gains a leaves-the-battlefield draw trigger.
#[test]
fn thief_of_existence_exiles_and_grants_ltb_draw() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone()); // MV 2 artifact
    g.add_card_to_library(0, catalog::grizzly_bears());
    let thief = g.add_card_to_hand(0, catalog::thief_of_existence());
    fill_mana(&mut g);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(stone)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: thief, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == stone), "opponent artifact exiled");
    let hand_before = g.players[0].hand.len();
    // Kill the Thief (3/4) with lethal damage; its granted LTB trigger draws.
    g.battlefield_find_mut(thief).unwrap().damage = 4;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "LTB trigger drew a card");
}
