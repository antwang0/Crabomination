//! Functionality tests for `catalog::sets::decks::recent234`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, SpreeMode};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

fn spree_modes(def: &crabomination::card::CardDefinition) -> Vec<SpreeMode> {
    match &def.effect {
        Effect::Spree { modes } => modes.clone(),
        _ => panic!("not a spree card"),
    }
}

/// Trash the Town's counter mode adds two +1/+1 counters.
#[test]
fn trash_the_town_adds_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let modes = spree_modes(&catalog::trash_the_town());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&modes[0].effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Its third mode grants a combat-damage draw trigger for the turn.
#[test]
fn trash_the_town_grants_draw_trigger() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let modes = spree_modes(&catalog::trash_the_town());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&modes[2].effect, &ctx).unwrap();
    // The until-end-of-turn grant is recorded for the creature.
    assert!(
        g.granted_triggers_eot.get(&bear).is_some_and(|v| !v.is_empty()),
        "gained an end-of-turn combat-damage draw trigger",
    );
}

/// Unfortunate Accident's first mode destroys a creature; its second makes a
/// Mercenary token.
#[test]
fn unfortunate_accident_modes() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let modes = spree_modes(&catalog::unfortunate_accident());
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&modes[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == enemy), "creature destroyed");

    g.resolve_effect(&modes[1].effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Mercenary").count(),
        1,
        "a Mercenary token appears",
    );
}

/// Thunder Lasso attaches on ETB and grants +1/+1.
#[test]
fn thunder_lasso_attaches_and_buffs() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let lasso = g.add_card_to_battlefield(0, catalog::thunder_lasso());
    let effect = catalog::thunder_lasso().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_trigger(lasso, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(lasso).unwrap().attached_to, Some(bear), "attached to the bear");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2 + 1 = 3");
    assert!(catalog::thunder_lasso().keywords.iter().any(|k| matches!(k, Keyword::Equip(_))));
}
