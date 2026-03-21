//! Theros (THS) — 2013

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Keyword};
use crate::mana::{cost, w};

/// Hopeful Eidolon — {W} Enchantment Creature — Spirit 1/1 Lifelink
pub fn hopeful_eidolon() -> CardDefinition {
    CardDefinition {
        name: "Hopeful Eidolon",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
