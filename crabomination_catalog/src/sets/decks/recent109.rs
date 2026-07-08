//! CR 104.3 can't-lose/can't-win cluster (TODO.md deferred list): Angel's
//! Grace, Platinum Angel, Abyssal Persecutor, Worship. Tests in
//! `tests/recent109.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, StaticAbility, Subtypes,
};
use crate::effect::{Effect, StaticEffect};
use crate::mana::{b, cost, generic, w};

/// Angel's Grace — {W} Instant. Split second. You can't lose the game this
/// turn and your opponents can't win; damage that would drop you below 1
/// life drops you to 1 instead.
pub fn angels_grace() -> CardDefinition {
    CardDefinition {
        name: "Angel's Grace",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::CantLoseThisTurn { damage_floor: true },
        ..Default::default()
    }
}

/// Platinum Angel — {7} 4/4 Artifact Creature — Angel. Flying. You can't
/// lose the game and your opponents can't win the game.
pub fn platinum_angel() -> CardDefinition {
    CardDefinition {
        name: "Platinum Angel",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You can't lose the game and your opponents can't win the game.",
            effect: StaticEffect::ControllerCantLoseGame,
        }],
        ..Default::default()
    }
}

/// Abyssal Persecutor — {2}{B}{B} 6/6 Demon. Flying, trample. You can't win
/// the game and your opponents can't lose the game.
pub fn abyssal_persecutor() -> CardDefinition {
    CardDefinition {
        name: "Abyssal Persecutor",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "You can't win the game and your opponents can't lose the game.",
            effect: StaticEffect::ControllerCantWinGame,
        }],
        ..Default::default()
    }
}

/// Worship — {3}{W} Enchantment. If you control a creature, damage that
/// would reduce your life total to less than 1 reduces it to 1 instead.
pub fn worship() -> CardDefinition {
    CardDefinition {
        name: "Worship",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If you control a creature, damage that would reduce your life total to less than 1 reduces it to 1 instead.",
            effect: StaticEffect::DamageWontReduceControllerLifeBelowOne { requires_creature: true },
        }],
        ..Default::default()
    }
}
