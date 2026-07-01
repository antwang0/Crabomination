//! Functionality tests for `catalog::sets::decks::recent64` — blue tempo/value.

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;

#[test]
fn peregrine_drake_untaps_five_lands() {
    let mut g = two_player_game();
    let lands: Vec<CardId> =
        (0..5).map(|_| g.add_card_to_battlefield(0, catalog::island())).collect();
    for &l in &lands { g.battlefield_find_mut(l).unwrap().tapped = true; }
    let drake = g.add_card_to_battlefield(0, catalog::peregrine_drake());
    g.fire_self_etb_triggers(drake, 0);
    drain_stack(&mut g);
    assert!(lands.iter().all(|&l| !g.battlefield_find(l).unwrap().tapped), "all five untapped");
}

#[test]
fn cloud_elemental_can_block_only_flying() {
    let mut g = two_player_game();
    let ce = g.add_card_to_battlefield(0, catalog::cloud_elemental());
    assert!(g.computed_permanent(ce).unwrap().keywords.contains(&Keyword::CanBlockOnlyFlying));
}

#[test]
fn thought_courier_loots() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let extra = g.add_card_to_hand(0, catalog::forest());
    let tc = g.add_card_to_battlefield(0, catalog::thought_courier());
    g.clear_sickness(tc);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![extra])]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: tc, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("loot");
    drain_stack(&mut g);
    // +1 draw, -1 discard = net unchanged.
    assert_eq!(g.players[0].hand.len(), hand);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == extra), "discarded the chosen card");
}

#[test]
fn jhessian_thief_draws_on_combat_damage() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let jt = g.add_card_to_battlefield(0, catalog::jhessian_thief());
    let hand = g.players[0].hand.len();
    g.fire_combat_damage_to_player_triggers(jt, 1, 1);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on combat damage");
}

#[test]
fn sky_spirit_flies_and_first_strikes() {
    let def = catalog::sky_spirit();
    assert!(def.keywords.contains(&Keyword::Flying) && def.keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn cephalid_broker_target_player_wheels_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    let discardable: Vec<CardId> =
        (0..2).map(|_| g.add_card_to_hand(1, catalog::mountain())).collect();
    let cb = g.add_card_to_battlefield(0, catalog::cephalid_broker());
    g.clear_sickness(cb);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(discardable.clone())]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cb, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    // Opponent drew 2 and discarded 2 → net hand unchanged.
    assert_eq!(g.players[1].hand.len(), hand);
    assert!(discardable.iter().all(|d| g.players[1].graveyard.iter().any(|c| c.id == *d)));
}

#[test]
fn riverwise_augur_draws_three_puts_two_back() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let ra = g.add_card_to_battlefield(0, catalog::riverwise_augur());
    let hand = g.players[0].hand.len();
    let lib = g.players[0].library.len();
    // Put the first two hand cards back on top.
    let picks: Vec<CardId> = g.players[0].hand.iter().take(2).map(|c| c.id).collect();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::PutOnLibrary(picks)]));
    g.fire_self_etb_triggers(ra, 0);
    drain_stack(&mut g);
    // +3 drawn, -2 put back = net +1 hand; library net -1.
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew 3, put 2 back");
    assert_eq!(g.players[0].library.len(), lib - 1);
}
