//! Functionality tests for `catalog::sets::decks::recent238`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, SpreeMode};
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game};

fn spree_modes(def: &crabomination::card::CardDefinition) -> Vec<SpreeMode> {
    match &def.effect {
        Effect::Spree { modes } => modes.clone(),
        _ => panic!("not a spree card"),
    }
}

/// Prized Griffin is a 3/4 flier.
#[test]
fn prized_griffin_stats() {
    let def = catalog::prized_griffin();
    assert_eq!((def.power, def.toughness), (3, 4));
    assert!(def.keywords.contains(&Keyword::Flying));
}

/// Abhorrent Oculus can only be cast after exiling six graveyard cards, and
/// manifests dread on each opponent's upkeep.
#[test]
fn abhorrent_oculus_shape() {
    use crabomination::card::AdditionalCastCost;
    let def = catalog::abhorrent_oculus();
    match &def.additional_cast_cost[0] {
        AdditionalCastCost::ExileFromGraveyard { count, .. } => assert_eq!(*count, 6),
        other => panic!("unexpected cost: {other:?}"),
    }
    assert!(!def.triggered_abilities.is_empty(), "has the upkeep manifest trigger");
}

/// Lively Dirge's second mode reanimates creatures totalling MV<=4.
#[test]
fn lively_dirge_reanimates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let modes = spree_modes(&catalog::lively_dirge());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.resolve_effect(&modes[1].effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear reanimated");
}

/// Smuggler's Surprise mode 3 grants hexproof + indestructible to big creatures.
#[test]
fn smugglers_surprise_protects_big() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let modes = spree_modes(&catalog::smugglers_surprise());
    g.resolve_effect(&modes[2].effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    let kws = g.computed_permanent(big).unwrap().keywords;
    assert!(kws.contains(&Keyword::Hexproof) && kws.contains(&Keyword::Indestructible));
}

/// Prairie Dog grows at end step only if you haven't cast a spell from hand,
/// and its {4}{W} adds an extra counter to placements this turn.
#[test]
fn prairie_dog_from_hand_and_counter_bonus() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::prairie_dog());
    // No from-hand cast this turn → the end-step trigger's filter holds.
    let trig = &catalog::prairie_dog().triggered_abilities[0];
    let ctx = EffectContext::for_trigger(dog, 0, None, 0);
    assert!(
        g.evaluate_predicate(trig.event.filter.as_ref().unwrap(), &ctx),
        "haven't cast from hand → trigger fires",
    );
    // Activate {4}{W}: counter placements now get +1.
    let act = catalog::prairie_dog().activated_abilities[0].effect.clone();
    g.resolve_effect(&act, &ctx).unwrap();
    g.resolve_effect(
        &Effect::AddCounter {
            what: crabomination::effect::Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::ONE,
        },
        &ctx,
    )
    .unwrap();
    assert_eq!(
        g.battlefield_find(dog).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "1 placed + 1 bonus",
    );
}
