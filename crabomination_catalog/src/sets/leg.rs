//! Legends (LEG) — opened with the CR 702.22 "bands with other" cycle: the
//! five legendary-band lands, Master of the Hunt and the two band-hosers.
//! Tests in `classic_sets/leg`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value,
    shortcut::target_filtered,
};
use crate::mana::{Color, ManaCost, cost, g, generic};

/// "Bands with other legendary creatures" — the quality the five Legends
/// band lands hand out.
fn bands_with_legends() -> Keyword {
    Keyword::BandsWithOther(Box::new(
        R::Creature.and(R::HasSupertype(Supertype::Legendary)),
    ))
}

/// The Legends band-land cycle: one colour's legendary creatures band together.
fn band_land(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "Legendary creatures you control of this land's color have \
                          \"bands with other legendary creatures.\"",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasSupertype(Supertype::Legendary))
                        .and(R::HasColor(color)),
                ),
                keyword: bands_with_legends(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Adventurers' Guildhouse — the green band land.
pub fn adventurers_guildhouse() -> CardDefinition {
    band_land("Adventurers' Guildhouse", Color::Green)
}

/// Cathedral of Serra — the white band land.
pub fn cathedral_of_serra() -> CardDefinition {
    band_land("Cathedral of Serra", Color::White)
}

/// Mountain Stronghold — the red band land.
pub fn mountain_stronghold() -> CardDefinition {
    band_land("Mountain Stronghold", Color::Red)
}

/// Seafarer's Quay — the blue band land.
pub fn seafarers_quay() -> CardDefinition {
    band_land("Seafarer's Quay", Color::Blue)
}

/// Unholy Citadel — the black band land.
pub fn unholy_citadel() -> CardDefinition {
    band_land("Unholy Citadel", Color::Black)
}

/// Master of the Hunt — a wolf factory whose tokens band with each other.
pub fn master_of_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "Master of the Hunt",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Wolves of the Hunt".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Wolf],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::BandsWithOther(Box::new(R::HasName(
                        "Wolves of the Hunt".into(),
                    )))],
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shelkin Brownie — strips a creature's "bands with other".
pub fn shelkin_brownie() -> CardDefinition {
    CardDefinition {
        name: "Shelkin Brownie",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ouphe], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LoseKeyword {
                what: target_filtered(R::Creature),
                keyword: bands_with_legends(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tolaria — blue mana, or a band-hosing tap during any upkeep.
pub fn tolaria() -> CardDefinition {
    CardDefinition {
        name: "Tolaria",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Blue]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::LoseKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Banding,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::LoseKeyword {
                        what: target_filtered(R::Creature),
                        keyword: bands_with_legends(),
                        duration: Duration::EndOfTurn,
                    },
                ]),
                condition: Some(crate::card::Predicate::CurrentStepIs(
                    crate::game::types::TurnStep::Upkeep,
                )),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
