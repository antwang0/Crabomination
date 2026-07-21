//! Ravnica (RAV) gap wave 13: a Radiance prevention cleric and Pariah's Shield.
//! Wojek Apothecary reuses `Selector::RadianceGroup`; Pariah's Shield rides the
//! new `RedirectControllerDamageToEquippedCreature` static. Tests in
//! `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, Selector, StaticEffect};
use crate::mana::{cost, generic, w};

/// Wojek Apothecary — {2}{W}{W} 1/1 Human Cleric. Radiance — {T}: Prevent the
/// next 1 damage that would be dealt to target creature and each other creature
/// that shares a color with it this turn.
pub fn wojek_apothecary() -> CardDefinition {
    CardDefinition {
        name: "Wojek Apothecary",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: Selector::RadianceGroup {
                    subject: Box::new(target_filtered(R::Creature)),
                },
                amount: crate::effect::Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pariah's Shield — {5} Equipment. All damage that would be dealt to you is
/// dealt to equipped creature instead. Equip {3}.
pub fn pariahs_shield() -> CardDefinition {
    CardDefinition {
        name: "Pariah's Shield",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![StaticAbility {
            description: "All damage that would be dealt to you is dealt to equipped creature instead.",
            effect: StaticEffect::RedirectControllerDamageToEquippedCreature,
        }],
        ..Default::default()
    }
}
