//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Proft, Sinister
//! Mastermind's graveyard. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{b, cost, generic};

/// Proft, Sinister Mastermind — {2}{B} 5/5 legendary Human Rogue, menace.
/// Threshold: castable only with seven or more cards in your graveyard. {B},
/// discard it: target creature gets -3/-1 until end of turn.
pub fn proft_sinister_mastermind() -> CardDefinition {
    CardDefinition {
        name: "Proft, Sinister Mastermind",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Rogue], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Menace],
        cast_condition: Some(Predicate::ValueAtLeast(Value::GraveyardSizeOf(PlayerRef::You), Value::Const(7))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tyrannical Pitlord — {4}{B}{B} 6/6 Demon, flying, trample. As it enters,
/// choose another creature you control: it gets +3/+3 and flying. When the
/// Pitlord leaves, sacrifice the chosen creature.
pub fn tyrannical_pitlord() -> CardDefinition {
    CardDefinition {
        name: "Tyrannical Pitlord",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        as_enters_effect: Some(Effect::ChoosePermanentForSource {
            filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
        }),
        static_abilities: vec![
            StaticAbility {
                description: "The chosen creature gets +3/+3.",
                effect: StaticEffect::PumpPT { applies_to: Selector::ChosenPermanentOfSource, power: 3, toughness: 3 },
            },
            StaticAbility {
                description: "The chosen creature has flying.",
                effect: StaticEffect::GrantKeyword { applies_to: Selector::ChosenPermanentOfSource, keyword: Keyword::Flying },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::SacrificePermanent { what: Selector::ChosenPermanentOfSource },
        }],
        ..Default::default()
    }
}

/// Circle of Affliction — {1}{B} enchantment. As it enters, choose a color.
/// Whenever a source of that color deals damage to you, you may pay {1}: target
/// player loses 1 life and you gain 1 life.
pub fn circle_of_affliction() -> CardDefinition {
    CardDefinition {
        name: "Circle of Affliction",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::SourceDamagedYou)
                .dealt_by(R::HasChosenColorOfSource),
            effect: Effect::MayPay {
                description: "Pay {1} to drain 1?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: target_filtered(R::Player), amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Grave Strength — {1}{B} sorcery. Choose target creature; mill three, then
/// a +1/+1 counter on it per creature card in your graveyard.
pub fn grave_strength() -> CardDefinition {
    CardDefinition {
        name: "Grave Strength",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
            },
        ]),
        ..Default::default()
    }
}
