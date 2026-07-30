//! Tribal/anthem artifacts: Coat of Arms (per-shared-type anthem, new
//! `StaticEffect::PumpPerSharedType`) and Akroma's Memorial (six-keyword team
//! anthem). Tests in `tests/recent87.rs`.

use crate::card::{
    CardDefinition, CardType, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect,
    Supertype,
};
use crate::effect::Selector;
use crate::mana::{Color, cost, generic};

/// Coat of Arms — {5} Artifact. Each creature gets +1/+1 for each other creature
/// on the battlefield that shares a creature type with it.
pub fn coat_of_arms() -> CardDefinition {
    CardDefinition {
        name: "Coat of Arms",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each creature gets +1/+1 for each other creature that shares a type with it.",
            effect: StaticEffect::PumpPerSharedType {
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Akroma's Memorial — {7} Legendary Artifact. Creatures you control have flying,
/// first strike, vigilance, trample, haste, and protection from black and from
/// red.
pub fn akromas_memorial() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    let grant = |kw: Keyword| StaticAbility {
        description: "Creatures you control gain a keyword.",
        effect: StaticEffect::GrantKeyword {
            applies_to: team(),
            keyword: kw,
        },
    };
    CardDefinition {
        name: "Akroma's Memorial",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            grant(Keyword::Flying),
            grant(Keyword::FirstStrike),
            grant(Keyword::Vigilance),
            grant(Keyword::Trample),
            grant(Keyword::Haste),
            grant(Keyword::Protection(Color::Black)),
            grant(Keyword::Protection(Color::Red)),
        ],
        ..Default::default()
    }
}
