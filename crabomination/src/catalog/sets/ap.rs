//! Apocalypse (AP) — 2001

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Keyword};
use crate::mana::{cost, g, u};

/// Gaea's Skyfolk — {G}{U} 2/2 Flying
pub fn gaeas_skyfolk() -> CardDefinition {
    CardDefinition {
        name: "Gaea's Skyfolk",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        power: 2, toughness: 2,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
