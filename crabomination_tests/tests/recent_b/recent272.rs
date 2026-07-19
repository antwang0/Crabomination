//! Functionality tests for `catalog::sets::decks::recent272`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

/// Ambitious Assault pumps the team and draws when a modified creature is out.
#[test]
fn ambitious_assault_draws_when_modified() {
    // No modified creature → no draw.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let effect = catalog::ambitious_assault().effect;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let hand = g.players[0].hand.len();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "team +2/+0");
    assert_eq!(g.players[0].hand.len(), hand, "no modified creature → no draw");

    // Put a +1/+1 counter on the bear (a modification) → the draw happens.
    g.battlefield_find_mut(bear)
        .unwrap()
        .add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    let hand = g.players[0].hand.len();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "modified creature → draw");
}

/// Revenge of the Drowned tucks a creature and makes a decayed Zombie.
#[test]
fn revenge_of_the_drowned_tucks_and_spawns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lib_before = g.players[1].library.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // top
    let effect = catalog::revenge_of_the_drowned().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature tucked away");
    assert_eq!(g.players[1].library.len(), lib_before + 1, "returned to its owner's library");
    let zombie = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Zombie")
        .expect("made a Zombie");
    assert!(zombie.definition.keywords.contains(&crabomination::card::Keyword::Decayed));
}
