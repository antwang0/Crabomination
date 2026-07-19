//! Functionality tests for `catalog::sets::decks::recent285`.

use crabomination::catalog;
use crabomination::card::CounterType;
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game, Target};

/// Parting Gust with no gift exiles a creature and returns it with a +1/+1
/// counter at the next end step.
#[test]
fn parting_gust_no_gift_exiles_and_returns() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::parting_gust().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    // Resolve the delayed next-end-step return.
    g.fire_step_triggers(crabomination::game::TurnStep::End);
    drain_stack(&mut g);
    let returned = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
    assert!(returned.is_some(), "returned at the next end step");
    assert_eq!(
        returned.unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "returned with a +1/+1 counter",
    );
}

/// Parting Gust's gifted branch exiles the creature for good and gives the
/// opponent a tapped Fish.
#[test]
fn parting_gust_gift_exiles_permanently() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gifted = catalog::parting_gust().gift.unwrap().gifted_effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&gifted, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Fish" && c.tapped),
        "opponent got a tapped Fish",
    );
    // No delayed return: it stays exiled through the end step.
    g.fire_step_triggers(crabomination::game::TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "no return under the gift");
}

/// Starfall Invocation destroys all creatures; the gifted branch returns a
/// creature card from your graveyard.
#[test]
fn starfall_invocation_gift_reanimates() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::eagle_of_deliverance());
    g.add_card_to_library(1, catalog::forest()); // the gift draw
    g.add_card_to_graveyard(0, catalog::eagle_of_deliverance()); // a fatter body to bring back
    let gifted = catalog::starfall_invocation().gift.unwrap().gifted_effect.clone();
    let gy_eagle = g.players[0].graveyard[0].id;
    let ctx = EffectContext { targets: vec![Target::Permanent(gy_eagle)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&gifted, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mine || c.id == theirs), "board wiped");
    assert!(g.battlefield.iter().any(|c| c.id == gy_eagle && c.controller == 0), "graveyard creature reanimated");
}
