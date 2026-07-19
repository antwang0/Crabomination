//! Functionality tests for `catalog::sets::decks::recent276`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game, Target};

/// Converter Beast incubates 5 on entry.
#[test]
fn converter_beast_incubates() {
    let mut g = two_player_game();
    let cb = g.add_card_to_battlefield(0, catalog::converter_beast());
    let effect = catalog::converter_beast().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(cb, 0, None)).unwrap();
    let inc = g.battlefield.iter().find(|c| c.definition.name.contains("Incubator")).unwrap();
    assert_eq!(inc.counters.get(&crabomination::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0), 5);
}

/// Carrion Locust drains 1 when the exiled graveyard card is a creature.
#[test]
fn carrion_locust_drains_on_creature() {
    let mut g = two_player_game();
    let cl = g.add_card_to_battlefield(0, catalog::carrion_locust());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let effect = catalog::carrion_locust().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(dead)], ..EffectContext::for_ability(cl, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.id == dead), "the creature card is exiled");
    assert_eq!(g.players[1].life, life - 1, "owner lost 1 life for a creature card");
}

/// Coastal Bulwark grows to 3/3 while its controller has an Island.
#[test]
fn coastal_bulwark_islandwalk_pump() {
    let mut g = two_player_game();
    let cb = g.add_card_to_battlefield(0, catalog::coastal_bulwark());
    assert_eq!(g.computed_permanent(cb).unwrap().power, 1, "1/3 with no Island");
    g.add_card_to_battlefield(0, catalog::island());
    assert_eq!(g.computed_permanent(cb).unwrap().power, 3, "+2/+0 with an Island");
}

/// Emergency Weld returns a creature card and mints a Soldier.
#[test]
fn emergency_weld_returns_and_makes_token() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::emergency_weld().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned to hand");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Soldier"), "made a Soldier");
}

/// Burning Sun's Fury has convoke and pumps up to two creatures with haste.
#[test]
fn burning_suns_fury_pumps() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(a)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::burning_suns_fury().effect.clone(), &ctx).unwrap();
    let p = g.computed_permanent(a).unwrap();
    assert_eq!(p.power, 4, "+2/+0");
    assert!(p.keywords.contains(&crabomination::card::Keyword::Haste), "gains haste");
    assert!(catalog::burning_suns_fury().keywords.contains(&crabomination::card::Keyword::Convoke));
}
