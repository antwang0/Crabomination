//! Ravnica (RAV) gap wave 18: damage-into-counters replacements — Phytohydra
//! (`ReplaceDamageToSelfWithCounters`) and Szadek, Lord of Secrets
//! (`CombatDamageToPlayerBecomesCountersAndMill`). Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes,
};
use crate::effect::{Duration, Effect, Selector, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w};

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

/// Sabertooth Alley Cat — {1}{R}{R} 2/1 Cat that attacks each combat if able.
/// {1}{R}: creatures without defender can't block it this turn.
pub fn sabertooth_alley_cat() -> CardDefinition {
    CardDefinition {
        name: "Sabertooth Alley Cat",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::MustAttack],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Defender))),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
