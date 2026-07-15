//! Functionality tests for `catalog::sets::decks::recent227`.

use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::two_player_game;

/// Persuasive Interrogators poisons an opponent when you sacrifice a Clue.
#[test]
fn persuasive_interrogators_poisons_on_clue_sac() {
    let mut g = two_player_game();
    let pi = g.add_card_to_battlefield(0, catalog::persuasive_interrogators());
    // The Clue-sac trigger (index 1) adds two poison to the opponent.
    let effect = catalog::persuasive_interrogators().triggered_abilities[1].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(pi, 0, None, 1)).unwrap();
    assert_eq!(g.players[1].poison_counters, 2, "opponent got two poison");
}

/// Perimeter Enforcer grows when another Detective enters.
#[test]
fn perimeter_enforcer_grows_on_detective_enter() {
    let mut g = two_player_game();
    let pe = g.add_card_to_battlefield(0, catalog::perimeter_enforcer());
    let effect = catalog::perimeter_enforcer().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(pe, 0, None, 0)).unwrap();
    assert_eq!(g.computed_permanent(pe).unwrap().power, 2, "1 + 1 = 2");
}
