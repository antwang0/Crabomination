//! Functionality tests for `catalog::sets::decks::recent205`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Don't Make a Sound counters a spell whose controller can't pay {2}.
#[test]
fn dont_make_a_sound_counters_when_unpaid() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    // P1 is now tapped out; P0 casts Don't Make a Sound at the bear.
    let counter = g.add_card_to_hand(0, catalog::dont_make_a_sound());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("soft-counter the bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear countered (no {{2}} to pay)");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "countered spell in graveyard");
}

/// Keys to the House tutors a basic land to hand for {1}, {T}, Sacrifice.
#[test]
fn keys_to_the_house_fetches_a_basic() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let forest = g.add_card_to_library(0, catalog::forest());
    let keys = g.add_card_to_battlefield(0, catalog::keys_to_the_house());
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: keys, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Keys");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic land fetched to hand");
    assert!(g.battlefield_find(keys).is_none(), "Keys sacrificed itself");
}

/// Osseous Sticktwister punishes each opponent for its power once delirium is on.
#[test]
fn osseous_sticktwister_delirium_punisher() {
    let mut g = two_player_game();
    let stick = g.add_card_to_battlefield(0, catalog::osseous_sticktwister());
    g.active_player_idx = 0;
    // Four card types in the graveyard → delirium.
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(0, catalog::island()); // land
    g.add_card_to_graveyard(0, catalog::rite_of_the_dragoncaller()); // enchantment
    g.players[1].hand.clear();
    let l1 = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let _ = stick;
    assert_eq!(g.players[1].life, l1 - 2, "opponent took 2 (this creature's power)");
}

/// No delirium → no trigger.
#[test]
fn osseous_sticktwister_needs_delirium() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::osseous_sticktwister());
    g.active_player_idx = 0;
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // only one type
    g.players[1].hand.clear();
    let l1 = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1, "no delirium → no punisher");
}
