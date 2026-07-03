//! Kamigawa: Neon Dynasty batch 5 — small, self-contained rides: a Ninja lord,
//! an enchantment-cast payoff, a death-counter Kirin, and a mana-value pump.
//! Tests in `tests/recent99.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_other_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect};
use crate::mana::{b, cost, g, generic, u, w};

/// Guardian Kirin — {3}{W} 2/3 Kirin, flying. Whenever another creature you
/// control dies, put a +1/+1 counter on it.
pub fn guardian_kirin() -> CardDefinition {
    CardDefinition {
        name: "Guardian Kirin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kirin], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_other_dies(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silver-Fur Master — {U}{B} 2/2 Rat Ninja. Ninjutsu {U}{B}. Other Ninja and
/// Rogue creatures you control get +1/+1. (The "your Ninjutsu costs {1} less"
/// rider is omitted — no activated-cost-reduction static for Ninjutsu yet.)
pub fn silver_fur_master() -> CardDefinition {
    CardDefinition {
        name: "Silver-Fur Master",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Ninja],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ninjutsu(cost(&[u(), b()]))],
        static_abilities: vec![crate::effect::StaticAbility {
            description: "Other Ninja and Rogue creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ninja)
                        .or(R::HasCreatureType(CreatureType::Rogue))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Generous Visitor — {G} 1/1 Spirit. Whenever you cast an enchantment spell,
/// put a +1/+1 counter on target creature.
pub fn generous_visitor() -> CardDefinition {
    CardDefinition {
        name: "Generous Visitor",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasCardType(CardType::Enchantment))),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Boon of Boseiju — {1}{G} Instant. Target creature gets +X/+X until end of
/// turn, where X is the greatest mana value among permanents you control, then
/// untap it.
pub fn boon_of_boseiju() -> CardDefinition {
    let x = || {
        Value::HighestManaValueAmong(Box::new(Selector::ControlledBy {
            who: PlayerRef::You,
            filter: R::Permanent,
        }))
    };
    CardDefinition {
        name: "Boon of Boseiju",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: x(),
                toughness: x(),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}
