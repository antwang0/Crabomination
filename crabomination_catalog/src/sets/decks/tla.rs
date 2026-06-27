//! The Last Airbender (TLA) staples on existing primitives — Allies, hybrid
//! costs, attack/ETB triggers, and a defensive anthem. Tests in
//! `crabomination/src/tests/tla.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, LandType, SelectionRequirement,
    Selector, StaticAbility, StaticEffect, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{b, cost, generic, hybrid, w, Color};

/// A 1/1 white Ally creature token (Kyoshi Warriors).
fn ally_token() -> TokenDefinition {
    TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ally], ..Default::default() },
        ..Default::default()
    }
}

/// Cat-Gator — {6}{B} 3/2 Fish Crocodile. Lifelink; ETB deals damage equal to
/// the number of Swamps you control to any target.
pub fn cat_gator() -> CardDefinition {
    CardDefinition {
        name: "Cat-Gator",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish, CreatureType::Crocodile],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_any(),
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::HasLandType(LandType::Swamp),
            },
        })],
        ..Default::default()
    }
}

/// Cat-Owl — {3}{W/U} 3/3 Cat Bird. Flying; on attack, untap target artifact or
/// creature.
pub fn cat_owl() -> CardDefinition {
    CardDefinition {
        name: "Cat-Owl",
        cost: cost(&[generic(3), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Bird],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Untap {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            ),
            up_to: None,
        })],
        ..Default::default()
    }
}

/// Kyoshi Warriors — {3}{W} 3/3 Human Warrior Ally. ETB: make a 1/1 white Ally.
pub fn kyoshi_warriors() -> CardDefinition {
    CardDefinition {
        name: "Kyoshi Warriors",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: ally_token(),
        })],
        ..Default::default()
    }
}

/// The Walls of Ba Sing Se — {8} 0/30 legendary Wall. Defender; other permanents
/// you control have indestructible.
pub fn walls_of_ba_sing_se() -> CardDefinition {
    CardDefinition {
        name: "The Walls of Ba Sing Se",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 30,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "Other permanents you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Indestructible,
            },
        }],
        ..Default::default()
    }
}

/// Wandering Musicians — {3}{R/W} 2/5 Human Bard Ally. Whenever it attacks,
/// creatures you control get +1/+0 until end of turn.
pub fn wandering_musicians() -> CardDefinition {
    CardDefinition {
        name: "Wandering Musicians",
        cost: cost(&[generic(3), hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Bard, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}
