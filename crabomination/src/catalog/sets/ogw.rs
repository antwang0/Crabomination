//! Oath of the Gatewatch (OGW) — 2016

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Keyword};
use crate::mana::{cost, generic, r, u};

/// Stormchaser Mage — {1}{U}{R} 1/3 Flying Haste
pub fn stormchaser_mage() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser Mage",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        power: 1, toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
