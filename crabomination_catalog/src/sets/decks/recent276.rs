//! MOM/BRO gap batch — an incubate body, graveyard hate, an Island-scaled
//! Wall, a graveyard-return welder, and a convoke pump. All on existing
//! primitives. Tests in `tests/recent_b/recent276.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r};

/// Converter Beast — {3}{G} 0/1 Phyrexian Beast. When it enters, incubate 5.
pub fn converter_beast() -> CardDefinition {
    CardDefinition {
        name: "Converter Beast",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Incubate { who: PlayerRef::You, amount: Value::Const(5) })],
        ..Default::default()
    }
}

/// Carrion Locust — {2}{B} 2/1 Insect Horror. Flying. When it enters, exile
/// target card from an opponent's graveyard. If it was a creature card, that
/// player loses 1 life.
pub fn carrion_locust() -> CardDefinition {
    CardDefinition {
        name: "Carrion Locust",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Exile { what: target_filtered(R::InOpponentGraveyard) },
        ]))],
        ..Default::default()
    }
}

/// Coastal Bulwark — {2} 1/3 Wall. Defender. +2/+0 while you control an Island.
/// {2}, {T}: Surveil 1.
pub fn coastal_bulwark() -> CardDefinition {
    CardDefinition {
        name: "Coastal Bulwark",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "Gets +2/+0 as long as you control an Island.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Island).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Emergency Weld — {1}{B} Sorcery. Return target artifact or creature card from
/// your graveyard to your hand. Create a 1/1 colorless Soldier artifact creature.
pub fn emergency_weld() -> CardDefinition {
    CardDefinition {
        name: "Emergency Weld",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::Artifact.or(R::Creature))),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Soldier".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        ]),
        ..Default::default()
    }
}

/// Burning Sun's Fury — {1}{R} Instant. Convoke. Up to two target creatures each
/// get +2/+0 and gain haste until end of turn.
pub fn burning_suns_fury() -> CardDefinition {
    CardDefinition {
        name: "Burning Sun's Fury",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
        ..Default::default()
    }
}
