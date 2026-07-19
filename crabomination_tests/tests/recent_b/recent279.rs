//! Functionality tests for `catalog::sets::decks::recent279`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{two_player_game, Target};

/// Sunrise Cavalier's day/night flip grows a creature.
#[test]
fn sunrise_cavalier_flip_adds_counter() {
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::sunrise_cavalier());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // The second triggered ability is the day/night payoff.
    let effect = catalog::sunrise_cavalier().triggered_abilities[1].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_ability(cav, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2/2 → 3/3");
}

/// Celestus Sanctifier bins one of the top two on a day/night flip.
#[test]
fn celestus_sanctifier_flip_mills_choice() {
    let mut g = two_player_game();
    let cel = g.add_card_to_battlefield(0, catalog::celestus_sanctifier());
    let keep = g.add_card_to_library(0, catalog::forest());
    let bin = g.add_card_to_library(0, catalog::island());
    let effect = catalog::celestus_sanctifier().triggered_abilities[1].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(cel, 0, None)).unwrap();
    // Exactly one of the top two is binned; the other stays in the library.
    let two = [keep, bin];
    assert_eq!(g.players[0].graveyard.len(), 1, "one of the two binned");
    let binned = two.iter().filter(|id| g.players[0].graveyard.iter().any(|c| c.id == **id)).count();
    let kept = two.iter().filter(|id| g.players[0].library.iter().any(|c| c.id == **id)).count();
    assert_eq!((binned, kept), (1, 1), "one binned, one kept on top");
}

/// Cartographer's Survey ramps up to two lands tapped.
#[test]
fn cartographers_survey_ramps_lands() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let bf = g.battlefield.len();
    g.resolve_effect(&catalog::cartographers_survey().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    let lands_in = g.battlefield.len() - bf;
    assert!(lands_in >= 1 && lands_in <= 2, "put up to two lands onto the battlefield");
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Forest").all(|c| c.tapped), "entered tapped");
}

/// Markov Retribution's team-pump mode buffs your board.
#[test]
fn markov_retribution_team_pump_mode() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Resolve mode 0 (team +1/+0) directly.
    let crabomination::effect::Effect::ChooseModesCast { modes, .. } = &catalog::markov_retribution().effect
    else {
        panic!("modal");
    };
    g.resolve_effect(&modes[0].clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2/2 → 3/2");
}
