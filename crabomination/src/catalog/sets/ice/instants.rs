use super::no_abilities;
use crate::card::{CardDefinition, CardType, SpellEffect, Subtypes};
use crate::mana::{cost, u};

/// Brainstorm — {U}: draw three cards, then put two cards from your hand on top of your library
/// (Simplified: draw three cards.)
pub fn brainstorm() -> CardDefinition {
    CardDefinition {
        name: "Brainstorm",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DrawCards { amount: 3 }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
