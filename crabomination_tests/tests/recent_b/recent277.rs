//! Functionality tests for `catalog::sets::decks::recent277`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{two_player_game, Target};

/// Calamity's Wake empties graveyards, locks noncreature casting, and exiles
/// itself.
#[test]
fn calamitys_wake_nukes_graveyards_and_locks() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.resolve_effect(&catalog::calamitys_wake().effect.clone(), &EffectContext::for_spell(0, None, 0, 0))
        .unwrap();
    assert!(g.players[0].graveyard.is_empty() && g.players[1].graveyard.is_empty(), "all graveyards exiled");
    assert!(g.players[0].cant_cast_noncreature_this_turn, "you locked");
    assert!(g.players[1].cant_cast_noncreature_this_turn, "opponent locked");
}

/// Attentive Skywarden's combat-damage trigger flips an Incubator token.
#[test]
fn attentive_skywarden_flips_incubator() {
    let mut g = two_player_game();
    // Give the controller an Incubator token via Converter Beast's incubate.
    let cb = g.add_card_to_battlefield(0, catalog::converter_beast());
    let inc_effect = catalog::converter_beast().triggered_abilities[0].effect.clone();
    g.resolve_effect(&inc_effect, &EffectContext::for_ability(cb, 0, None)).unwrap();
    let inc = g.battlefield.iter().find(|c| c.definition.name.contains("Incubator")).unwrap().id;
    let warden = g.add_card_to_battlefield(0, catalog::attentive_skywarden());
    let effect = catalog::attentive_skywarden().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(inc)], ..EffectContext::for_ability(warden, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    // The Incubator transformed into its Phyrexian creature back face.
    let flipped = g.battlefield_find(inc).unwrap();
    assert!(flipped.definition.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
}

/// Molten Collapse is descend-gated: one mode by default, up to both when the
/// caster descended this turn.
#[test]
fn molten_collapse_descend_widens_to_both() {
    use crabomination::effect::Effect;
    let def = catalog::molten_collapse();
    let Effect::If { cond, then, else_ } = &def.effect else { panic!("expected a descend-gated modal") };
    assert!(
        matches!(cond, crabomination::card::Predicate::DescendedThisTurn { .. }),
        "gated on descend",
    );
    assert!(matches!(**else_, Effect::ChooseModesCast { min: 1, max: 1, .. }), "one mode by default");
    let Effect::ChooseModesCast { modes, min: 1, max: 2, .. } = &**then else {
        panic!("both modes available once descended")
    };
    assert_eq!(modes.len(), 2, "the two printed destroy modes");

    // The default (non-descended) branch destroys a targeted creature.
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(creature)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&modes[0].clone(), &ctx).unwrap();
    assert!(!g.battlefield.iter().any(|c| c.id == creature), "creature destroyed by mode 0");
}
