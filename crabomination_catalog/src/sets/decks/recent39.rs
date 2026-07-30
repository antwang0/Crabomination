//! Defensive walls, including two that exercise the new
//! `StaticEffect::PreventAllCombatDamageToThis` (CR 615). Tests in
//! `tests/recent39.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, StaticAbility, StaticEffect, Subtypes,
};
use crate::mana::{cost, generic, u, w};

/// Wall of Denial — {1}{W}{U} 0/8 Wall. Defender, flying, shroud.
pub fn wall_of_denial() -> CardDefinition {
    CardDefinition {
        name: "Wall of Denial",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 8,
        keywords: vec![Keyword::Defender, Keyword::Flying, Keyword::Shroud],
        ..Default::default()
    }
}

/// Fog Bank — {1}{U} 0/2 Wall. Defender, flying. Prevent all combat damage to
/// and dealt by it.
pub fn fog_bank() -> CardDefinition {
    CardDefinition {
        name: "Fog Bank",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![
            Keyword::Defender,
            Keyword::Flying,
            Keyword::DealsNoCombatDamage,
        ],
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to this creature.",
            effect: StaticEffect::PreventAllCombatDamageToThis,
        }],
        ..Default::default()
    }
}

/// Guard Gomazoa — {2}{U} 1/3 Jellyfish. Defender, flying. Prevent all combat
/// damage dealt to it.
pub fn guard_gomazoa() -> CardDefinition {
    CardDefinition {
        name: "Guard Gomazoa",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to this creature.",
            effect: StaticEffect::PreventAllCombatDamageToThis,
        }],
        ..Default::default()
    }
}
