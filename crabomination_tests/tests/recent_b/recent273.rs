//! Functionality tests for `catalog::sets::decks::recent273`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game};

/// Academy Wall loots when you cast an instant or sorcery.
#[test]
fn academy_wall_loots_on_spell() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::academy_wall());
    g.add_card_to_library(0, catalog::forest());
    let discard = g.add_card_to_hand(0, catalog::island()); // something to discard
    // Accept the "may draw then discard" and bin the Island.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Discard(vec![discard]),
    ]));
    // Fire the trigger effect directly (a cast would drain the stack too).
    let effect = catalog::academy_wall().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(wall, 0, None);
    let hand = g.players[0].hand.len();
    g.resolve_effect(&effect, &ctx).unwrap();
    // +1 drawn, -1 discarded → net unchanged, but the loot happened.
    assert_eq!(g.players[0].hand.len(), hand, "drew then discarded");
    assert_eq!(g.players[0].graveyard.len(), 1, "one card discarded");
}

/// Battlewing Mystic wheels only when kicked.
#[test]
fn battlewing_mystic_kicked_wheels() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::mountain());
    let effect = catalog::battlewing_mystic().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_ability(crabomination::game::CardId(500), 0, None);
    ctx.kicked = true;
    g.resolve_effect(&effect, &ctx).unwrap();
    // Discarded the two-card hand, drew two fresh.
    assert_eq!(g.players[0].hand.len(), 2, "new hand of two");
    assert!(
        g.players[0].hand.iter().all(|c| c.definition.name == "Forest"),
        "the wheel replaced the old hand"
    );
}

/// Brazen Upstart digs for a creature when it dies.
#[test]
fn brazen_upstart_death_dig() {
    let mut g = two_player_game();
    let up = g.add_card_to_battlefield(0, catalog::brazen_upstart());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.battlefield_find_mut(up).unwrap().damage = 100;
    let evs = g.check_state_based_actions();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == bear),
        "revealed a creature to hand"
    );
}
