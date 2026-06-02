//! Kaldheim (KHM) — Boast (CR 702.142) creatures.
//!
//! Boast rides `shortcut::boast`: an activated ability gated on
//! `Predicate::SourceAttackedThisTurn` + `once_per_turn`, so it can only be
//! used once each turn and only if the creature attacked this turn.

use crate::card::{CardDefinition, CardType, CounterType, CreatureType, Effect, Selector, Subtypes, Value};
use crate::effect::shortcut::boast;
use crate::mana::{cost, generic, r};

/// Dragonkin Berserker — {2}{R} 2/2 Dragon Berserker. Boast — {3}{R}: Put a
/// +1/+1 counter on this. (The "whenever you boast, make a Dragon token if
/// you control no other Dragon" payoff rider is omitted.)
pub fn dragonkin_berserker() -> CardDefinition {
    CardDefinition {
        name: "Dragonkin Berserker",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![boast(
            cost(&[generic(3), r()]),
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        )],
        ..Default::default()
    }
}

/// Every KHM factory, for snapshot name→factory registration.
pub fn all_khm_card_factories() -> &'static [crate::CardFactory] {
    &[dragonkin_berserker]
}
