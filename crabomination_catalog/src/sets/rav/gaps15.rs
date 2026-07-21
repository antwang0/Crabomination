//! Ravnica (RAV) gap wave 15: a flash Equipment and Indentured Oaf's
//! color-scoped damage prevention. Tests in `classic_sets/rav`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, Selector, StaticEffect};
use crate::mana::{cost, generic, r, Color};

/// Grifter's Blade — {3} Equipment with flash. As it enters, attach it to a
/// creature you control. Equipped creature gets +1/+1. Equip {1}.
pub fn grifters_blade() -> CardDefinition {
    CardDefinition {
        name: "Grifter's Blade",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..Default::default()
    }
}

/// Indentured Oaf — {3}{R} 4/3 Ogre Warrior. Prevent all damage that this
/// creature would deal to red creatures.
pub fn indentured_oaf() -> CardDefinition {
    CardDefinition {
        name: "Indentured Oaf",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage this creature would deal to red creatures.",
            effect: StaticEffect::PreventThisDamageToColor(Color::Red),
        }],
        ..Default::default()
    }
}
