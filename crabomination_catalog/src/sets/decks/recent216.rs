//! Mixed gap batch — Bloomburrow expend payoffs (Teapot Slinger, Byway
//! Barterer), Foundations' Wick's Patrol (riding the new
//! `Value::GreatestManaValueInGraveyard`), and Tarkir's Maha (riding the new
//! `StaticEffect::SetBaseToughnessForMatching`). Tests in `tests/recent216.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
    StaticEffect, Value,
};
use crate::mana::{b, cost, generic, r};

/// Teapot Slinger — {3}{R} 3/4 Raccoon Warrior. Menace. Whenever you expend 4,
/// deal 2 damage to each opponent.
pub fn teapot_slinger() -> CardDefinition {
    CardDefinition {
        name: "Teapot Slinger",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                .with_filter(Predicate::ExpendReached(4)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Byway Barterer — {2}{R} 3/3 Raccoon Rogue. Menace. Whenever you expend 4, you
/// may discard your hand; if you do, draw two cards.
pub fn byway_barterer() -> CardDefinition {
    CardDefinition {
        name: "Byway Barterer",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                .with_filter(Predicate::ExpendReached(4)),
            effect: Effect::MayDo {
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::HandSizeOf(PlayerRef::You),
                        random: false,
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ])),
                description: "Discard your hand, then draw two?".into(),
            },
        }],
        ..Default::default()
    }
}

/// Wick's Patrol — {4}{B}{B} 5/3 Rat Warlock. When it enters, mill three cards,
/// then a target creature an opponent controls gets -X/-X until end of turn,
/// where X is the greatest mana value among cards in your graveyard.
pub fn wicks_patrol() -> CardDefinition {
    CardDefinition {
        name: "Wick's Patrol",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Warlock],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Times(
                    Box::new(Value::GreatestManaValueInGraveyard(PlayerRef::You)),
                    Box::new(Value::Const(-1)),
                ),
                toughness: Value::Times(
                    Box::new(Value::GreatestManaValueInGraveyard(PlayerRef::You)),
                    Box::new(Value::Const(-1)),
                ),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Maha, Its Feathers Night — {3}{B}{B} 6/5 Elemental Bird. Flying, trample,
/// Ward—Discard a card. Creatures your opponents control have base toughness 1.
pub fn maha_its_feathers_night() -> CardDefinition {
    CardDefinition {
        name: "Maha, Its Feathers Night",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Bird],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Ward(WardCost::Discard(1))],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control have base toughness 1.",
            effect: StaticEffect::SetBaseToughnessForMatching {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}
