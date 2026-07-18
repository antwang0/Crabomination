//! Functionality tests for `catalog::sets::decks::recent263`
//! (Glacial Dragonhunt's filtered reflexive discard).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::two_player_game;

/// Discarding a nonland card fires the reflexive 3-damage bolt (a 2/2 dies).
#[test]
fn glacial_dragonhunt_bolts_on_nonland_discard() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_hand(0, catalog::serra_angel()); // MV 5 nonland — auto-discarded
    g.add_card_to_library(0, catalog::forest()); // the "draw a card"
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let effect = catalog::glacial_dragonhunt().effect;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "3 damage killed the 2/2");
}

/// Discarding a land does not fire the bolt.
#[test]
fn glacial_dragonhunt_no_bolt_on_land_discard() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest()); // only a land to discard
    g.add_card_to_library(0, catalog::forest()); // drawn card is also a land
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let effect = catalog::glacial_dragonhunt().effect;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_some(), "land discard deals no damage");
}
