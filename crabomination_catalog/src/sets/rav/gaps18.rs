//! Ravnica (RAV) gap wave 18: damage-into-counters replacements — Phytohydra
//! (`ReplaceDamageToSelfWithCounters`) and Szadek, Lord of Secrets
//! (`CombatDamageToPlayerBecomesCountersAndMill`). Tests in `classic_sets/rav`.

use crate::card::{CardDefinition, CardType, CreatureType, Keyword, StaticAbility, Subtypes};
use crate::effect::StaticEffect;
use crate::mana::{b, cost, g, generic, u, w};

/// Phytohydra — {2}{G}{W}{W} 1/1 Plant Hydra. If damage would be dealt to it,
/// put that many +1/+1 counters on it instead.
pub fn phytohydra() -> CardDefinition {
    CardDefinition {
        name: "Phytohydra",
        cost: cost(&[generic(2), g(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Hydra],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to this creature, put that many +1/+1 counters on it instead.",
            effect: StaticEffect::ReplaceDamageToSelfWithCounters,
        }],
        ..Default::default()
    }
}

/// Szadek, Lord of Secrets — {3}{U}{U}{B}{B} 5/5 legendary Vampire with flying.
/// Its combat damage to a player becomes that many +1/+1 counters on it and
/// that player mills that many.
pub fn szadek_lord_of_secrets() -> CardDefinition {
    CardDefinition {
        name: "Szadek, Lord of Secrets",
        cost: cost(&[generic(3), u(), u(), b(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If Szadek, Lord of Secrets would deal combat damage to a player, instead put that many +1/+1 counters on Szadek and that player mills that many cards.",
            effect: StaticEffect::CombatDamageToPlayerBecomesCountersAndMill,
        }],
        ..Default::default()
    }
}
