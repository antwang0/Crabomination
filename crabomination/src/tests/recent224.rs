//! Functionality tests for `catalog::sets::decks::recent224`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::types::Target;
use crate::game::two_player_game;

/// Long River Lurker's ETB makes one of your creatures unblockable.
#[test]
fn long_river_lurker_makes_unblockable() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lurker = g.add_card_to_battlefield(0, catalog::long_river_lurker());
    let effect = catalog::long_river_lurker().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(ally)], ..EffectContext::for_trigger(lurker, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Unblockable), "ally can't be blocked");
    // The Lurker itself has Ward.
    assert!(g.computed_permanent(lurker).unwrap().keywords.iter().any(|k| matches!(k, Keyword::Ward(_))), "Lurker has ward");
}

/// Kolodin grants haste to Vehicles you control.
#[test]
fn kolodin_gives_vehicles_haste() {
    let mut g = two_player_game();
    let vehicle = g.add_card_to_battlefield(0, catalog::tangle_tumbler());
    assert!(!g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Haste), "no haste on its own");
    g.add_card_to_battlefield(0, catalog::kolodin_triumph_caster());
    assert!(g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Haste), "Kolodin grants haste");
}

/// Mu Yanling makes a Vehicle on entry and that Vehicle flies under her static.
#[test]
fn mu_yanling_makes_a_flying_vehicle() {
    let mut g = two_player_game();
    let mu = g.add_card_to_battlefield(0, catalog::mu_yanling_wind_rider());
    let etb = catalog::mu_yanling_wind_rider().triggered_abilities[0].effect.clone();
    g.resolve_effect(&etb, &EffectContext::for_trigger(mu, 0, None, 0)).unwrap();
    let vehicle = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Vehicle").map(|c| c.id).expect("a Vehicle token entered");
    assert!(g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Flying), "the Vehicle flies under Mu Yanling");
}
