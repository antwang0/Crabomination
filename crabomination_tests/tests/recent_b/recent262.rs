//! Functionality tests for `catalog::sets::decks::recent262`
//! (Worldsoul's Rage + the deploy-lands-from-hand-and-graveyard primitive).

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::two_player_game;

/// X damage hits the chosen target and up to X lands come back tapped, drawn
/// preferentially from the graveyard so hand lands stay playable.
#[test]
fn worldsouls_rage_burns_and_ramps() {
    let mut g = two_player_game();
    // Two graveyard lands + one hand land available; X = 2 deploys the two
    // graveyard lands and leaves the hand land untouched.
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::mountain());
    let hand_land = g.add_card_to_hand(0, catalog::forest());
    let start_life = g.players[1].life;

    let effect = catalog::worldsouls_rage().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 2);
    g.resolve_effect(&effect, &ctx).unwrap();

    assert_eq!(g.players[1].life, start_life - 2, "X=2 damage to the player");
    let lands_out = g.battlefield.iter().filter(|c| c.controller == 0 && c.tapped).count();
    assert_eq!(lands_out, 2, "two graveyard lands deployed tapped");
    assert!(g.players[0].graveyard.iter().all(|c| !c.definition.is_land()), "graveyard lands consumed");
    assert!(g.players[0].hand.iter().any(|c| c.id == hand_land), "hand land untouched at X=2");
}

/// Discarding a 5-drop and a 2-drop deals 5 (the greatest MV, not the
/// last-discarded 2) to each creature — a 4/4 dies.
#[test]
fn ill_timed_explosion_scales_by_greatest_discarded_mv() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    // Hand's two highest-MV cards (auto-discarded): a 5-drop + a 2-drop.
    g.add_card_to_hand(0, catalog::serra_angel()); // MV 5
    g.add_card_to_hand(0, catalog::grizzly_bears()); // MV 2
    // Library feeds the "draw two" so the discard picks from the intended pair.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let effect = catalog::ill_timed_explosion().effect;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "5 damage (greatest MV) killed the 4/4");
}

/// With no graveyard lands, deploy pulls from hand instead.
#[test]
fn worldsouls_rage_deploys_from_hand_when_graveyard_empty() {
    let mut g = two_player_game();
    let hand_land = g.add_card_to_hand(0, catalog::mountain());
    let effect = catalog::worldsouls_rage().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 1);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield.iter().any(|c| c.id == hand_land && c.tapped), "hand land deployed tapped");
}
