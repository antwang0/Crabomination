//! Functionality tests for `catalog::sets::decks::recent154`.

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::two_player_game;
use crabomination::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Harnesser of Storms impulses the top card when you cast a noncreature spell.
#[test]
fn harnesser_impulses_on_noncreature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::harnesser_of_storms());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_strike());
    fill_mana(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a noncreature spell");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == top), "impulsed the top card on a noncreature cast");
}

/// Flamecache Gecko's rummage draws for a discard.
#[test]
fn flamecache_gecko_rummage() {
    let mut g = two_player_game();
    let gecko = g.add_card_to_battlefield(0, catalog::flamecache_gecko());
    g.clear_sickness(gecko);
    g.add_card_to_hand(0, catalog::forest()); // a card to discard
    g.add_card_to_library(0, catalog::island());
    fill_mana(&mut g);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: gecko, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("rummage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "discarded one, drew one");
}

/// Intimidation Campaign's ETB drains, gains, and draws.
#[test]
fn intimidation_campaign_etb_value() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let opp = g.players[1].life;
    let me = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::intimidation_campaign());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, me + 1, "you gained 1");
    assert_eq!(g.players[0].hand.len(), hand + 1, "you drew a card");
}

/// Eddymurk Crab's ETB taps up to two creatures.
#[test]
fn eddymurk_crab_taps_on_etb() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Cards(vec![a, b])],
    ));
    g.move_card_to_battlefield_for_test(0, catalog::eddymurk_crab());
    drain_stack(&mut g);
    let tapped = [a, b].iter().filter(|&&id| g.battlefield_find(id).map(|c| c.tapped).unwrap_or(false)).count();
    assert_eq!(tapped, 2, "ETB tapped the two chosen creatures");
}
