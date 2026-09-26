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

/// Dragonkin Berserker — {1}{R} 2/2 Human Berserker, first strike. Boast —
/// {4}{R}: create a 5/5 red flying Dragon; boasts cost {1} less per Dragon you
/// control.
pub fn dragonkin_berserker() -> CardDefinition {
    let dragon = crate::card::TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        keywords: vec![crate::card::Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        ..Default::default()
    };
    let make_dragon = Effect::CreateToken {
        who: crate::effect::PlayerRef::You,
        count: Value::Const(1),
        definition: std::sync::Arc::new(dragon),
    };
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
        keywords: vec![crate::card::Keyword::FirstStrike],
        // "Boast abilities you activate cost {1} less to activate for each
        // Dragon you control" — its own boast is the only one in reach.
        activated_abilities: vec![crate::card::ActivatedAbility {
            cost_reduction_per: Some(crate::card::SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
            ..boast(cost(&[generic(4), r()]), make_dragon)
        }],
        ..Default::default()
    }
}

/// Every KHM factory, for snapshot name→factory registration.
pub fn all_khm_card_factories() -> &'static [crate::CardFactory] {
    &[dragonkin_berserker]
}
