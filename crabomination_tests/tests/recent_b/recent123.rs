//! Functionality tests for `catalog::sets::decks::recent123`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};

/// Corpseberry Cultivator forages at combat and grows from its own forage.
#[test]
fn corpseberry_cultivator_forages_and_grows() {
    let mut g = two_player_game();
    // Say "yes" to the optional forage.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let corp = g.add_card_to_battlefield(0, catalog::corpseberry_cultivator());
    // Three graveyard cards to pay the forage.
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 0, "foraged away three cards");
    assert_eq!(g.computed_permanent(corp).unwrap().power, 3, "whenever-you-forage → +1/+1");
}

/// A direct `Foraged` event fires the payoff for any forage source.
#[test]
fn foraged_event_fires_payoff() {
    let mut g = two_player_game();
    let corp = g.add_card_to_battlefield(0, catalog::corpseberry_cultivator());
    g.dispatch_triggers_for_events(&[GameEvent::Foraged { player: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(corp).unwrap().power, 3, "payoff triggers on the event");
}
