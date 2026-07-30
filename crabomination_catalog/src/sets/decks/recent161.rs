//! A second Foundations wave — burn, Morbid, Landfall, and a green fatty. Tests
//! in `crabomination/src/tests/recent161.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Predicate, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::Effect;
use crate::effect::shortcut::target_filtered;
use crate::game::TurnStep;
use crate::mana::{cost, g, generic, r, u, w};

/// Incinerating Blast — {4}{R} Sorcery. Deal 6 damage to target creature. You
/// may discard a card; if you do, draw a card.
pub fn incinerating_blast() -> CardDefinition {
    CardDefinition {
        name: "Incinerating Blast",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(6),
            },
            Effect::MayDiscard {
                description: "Discard a card to draw a card?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: None,
            },
        ]),
        ..Default::default()
    }
}

/// Needletooth Pack — {3}{G}{G} 4/5 Dinosaur. Morbid — at the beginning of your
/// end step, if a creature died this turn, put two +1/+1 counters on target
/// creature you control.
pub fn needletooth_pack() -> CardDefinition {
    CardDefinition {
        name: "Needletooth Pack",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::ONE,
            }),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Grappling Kraken — {4}{U}{U} 5/6 Kraken. Landfall — whenever a land you
/// control enters, tap target creature an opponent controls and put a stun
/// counter on it.
pub fn grappling_kraken() -> CardDefinition {
    CardDefinition {
        name: "Grappling Kraken",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kraken],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Joust Through — {W} Instant. Deal 3 damage to target attacking or blocking
/// creature. You gain 1 life.
pub fn joust_through() -> CardDefinition {
    CardDefinition {
        name: "Joust Through",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(3),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Quakestrider Ceratops — {3}{G}{G}{G} 12/8 Dinosaur.
pub fn quakestrider_ceratops() -> CardDefinition {
    CardDefinition {
        name: "Quakestrider Ceratops",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 12,
        toughness: 8,
        ..Default::default()
    }
}
