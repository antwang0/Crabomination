//! Kaldheim (KHM) — Boast (CR 702.142) creatures.
//!
//! Boast rides `shortcut::boast`: an activated ability gated on
//! `Predicate::SourceAttackedThisTurn` + `once_per_turn`, so it can only be
//! used once each turn and only if the creature attacked this turn.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Subtypes, Value,
};
use crate::effect::shortcut::boast;
use crate::mana::{cost, generic, r};

/// Dragonkin Berserker — {2}{R} 2/2 Dragon Berserker. Boast — {3}{R}: Put a
/// +1/+1 counter on this. (The "whenever you boast, make a Dragon token if
/// you control no other Dragon" payoff rider is omitted.)
pub fn dragonkin_berserker() -> CardDefinition {
    CardDefinition {
        name: "Dragonkin Berserker",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![boast(
            cost(&[generic(4), r()]),
            Effect::CreateToken {
                who: crate::effect::PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(crate::card::TokenDefinition {
                    name: "Dragon".into(),
                    power: 5,
                    toughness: 5,
                    keywords: vec![crate::card::Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Dragon],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        )],
        ..Default::default()
    }
}

/// Every KHM factory, for snapshot name→factory registration.
pub fn all_khm_card_factories() -> &'static [crate::CardFactory] {
    &[dragonkin_berserker]
}
