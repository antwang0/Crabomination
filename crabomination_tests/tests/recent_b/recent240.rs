//! Functionality tests for `catalog::sets::decks::recent240`.

use crabomination::card::AdditionalCastCost;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game};

/// Fear of Abduction exiles an opponent's creature until it leaves, then hands
/// it back to its owner — and it carries the exile-a-creature additional cost.
#[test]
fn fear_of_abduction_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fear = g.add_card_to_battlefield(0, catalog::fear_of_abduction());
    assert!(matches!(
        catalog::fear_of_abduction().additional_cast_cost[0],
        AdditionalCastCost::ExilePermanent { count: 1, .. }
    ));
    g.fire_self_etb_triggers(fear, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "opponent's creature exiled");
    g.remove_from_battlefield_to_graveyard_raw(fear);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returns to owner's hand on leave");
}

/// Say Its Name mills three, then returns a creature (or land) from the
/// graveyard to hand.
#[test]
fn say_its_name_mills_then_returns() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::say_its_name().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count(),
        3,
        "milled three cards"
    );
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}
