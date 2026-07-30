//! BLB gap batch — on existing primitives: Plumecreed Mentor (flying-enters
//! counter), Azure Beastbinder (evasion + attack ability-strip), Byrke
//! (ETB counters + attack counter-doubling), and Dreamdew Entrancer (ETB
//! tap + stun + reflexive draw). Tests in `crabomination/src/tests/recent181.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, Predicate, Selector};
use crate::mana::{cost, g, generic, u, w};

/// Plumecreed Mentor — {1}{W}{U} 2/3 Bird Scout with flying. Whenever this or
/// another flying creature you control enters, put a +1/+1 counter on target
/// creature you control without flying.
pub fn plumecreed_mentor() -> CardDefinition {
    CardDefinition {
        name: "Plumecreed Mentor",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                }),
            effect: Effect::AddCounter {
                what: target_filtered(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Azure Beastbinder — {1}{U} 1/3 Rat Rogue with vigilance. Can't be blocked by
/// creatures with power 2 or greater. Whenever it attacks, up to one target
/// permanent an opponent controls loses all abilities until your next turn; if
/// a creature, it's also a 2/2 until your next turn.
pub fn azure_beastbinder() -> CardDefinition {
    CardDefinition {
        name: "Azure Beastbinder",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::CantBeBlockedByPowerAtLeast(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseAllAbilities {
                    what: target_filtered(
                        R::ControlledByOpponent
                            .and(R::Artifact.or(R::Creature).or(R::Planeswalker)),
                    ),
                    duration: Duration::UntilNextTurn,
                },
                Effect::SetBasePT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::UntilNextTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Byrke, Long Ear of the Law — {4}{G}{W} 4/4 legendary Rabbit Soldier with
/// vigilance. ETB: put a +1/+1 counter on each of up to two target creatures.
/// Whenever a creature you control with a +1/+1 counter attacks, double the
/// number of +1/+1 counters on it.
pub fn byrke_long_ear_of_the_law() -> CardDefinition {
    CardDefinition {
        name: "Byrke, Long Ear of the Law",
        cost: cost(&[generic(4), g(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::WithCounter(CounterType::PlusOnePlusOne),
                    },
                ),
                effect: Effect::DoubleCountersOnEach {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        ],
        ..Default::default()
    }
}

/// Dreamdew Entrancer — {2}{G}{U} 3/4 Frog Wizard with reach. When it enters,
/// tap up to one target creature and put three stun counters on it. If you
/// control that creature, draw two cards.
pub fn dreamdew_entrancer() -> CardDefinition {
    CardDefinition {
        name: "Dreamdew Entrancer",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(R::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::ControlledByYou,
                },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}
