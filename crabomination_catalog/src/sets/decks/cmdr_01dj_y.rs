//! Commander: 2026-10-19 (`01DJHkWk`) — Jandor, Fortuned Traveler's granted
//! saddle (CR 702.171 through `GameState::effective_saddle_cost`) and
//! Wakandan Tusker.
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Predicate, Selector, StaticAbility, StaticEffect, Value,
};
use crate::mana::{Color, cost, g, generic, hybrid, r, w};
use std::sync::Arc;

/// Jandor, Fortuned Traveler — {1}{R}{G}{W} legendary 2/4 Human Citizen. Each
/// Beast, Camel, Horse, Elephant and Wolf creature you control is a Mount and
/// has saddle 2. Whenever a saddled creature you control attacks, it gets
/// +2/+2 until end of turn; draw a card; create a Food.
pub fn jandor_fortuned_traveler() -> CardDefinition {
    let riders = || {
        R::Creature.and(R::ControlledByYou).and(
            R::HasCreatureType(CreatureType::Beast)
                .or(R::HasCreatureType(CreatureType::Camel))
                .or(R::HasCreatureType(CreatureType::Horse))
                .or(R::HasCreatureType(CreatureType::Elephant))
                .or(R::HasCreatureType(CreatureType::Wolf)),
        )
    };
    CardDefinition {
        name: "Jandor, Fortuned Traveler",
        cost: cost(&[generic(1), r(), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Each Beast, Camel, Horse, Elephant, and Wolf creature you control is a Mount in addition to its other types…",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: Selector::EachPermanent(riders()),
                    creature_type: CreatureType::Mount,
                },
            },
            StaticAbility {
                description: "…and has saddle 2.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(riders()),
                    keyword: Keyword::Saddle(2),
                },
            },
            // "Whenever a saddled creature you control attacks": each creature
            // you control carries the attack trigger, gated on its own saddle.
            StaticAbility {
                description: "Whenever a saddled creature you control attacks, it gets +2/+2 until end of turn. Draw a card. Create a Food token.",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::Creature.and(R::ControlledByYou),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                            .with_filter(Predicate::SourceSaddled),
                        effect: Effect::Seq(vec![
                            Effect::PumpPT {
                                what: Selector::This,
                                power: Value::Const(2),
                                toughness: Value::Const(2),
                                duration: Duration::EndOfTurn,
                            },
                            Effect::Draw {
                                who: Selector::You,
                                amount: Value::ONE,
                            },
                            Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::ONE,
                                definition: Arc::new(crabomination_base::tokens::food_token()),
                            },
                        ]),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Wakandan Tusker — {1}{G/W} 2/3 Elephant. Whenever it becomes tapped, gain
/// 1 life and scry 1.
pub fn wakandan_tusker() -> CardDefinition {
    CardDefinition {
        name: "Wakandan Tusker",
        cost: cost(&[generic(1), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}
