use super::no_abilities;
use crate::card::{CardDefinition, CardType, SpellEffect, Subtypes};
use crate::mana::{cost, u};

/// Preordain — {U} Sorcery: Scry 2, then draw a card.
pub fn preordain() -> CardDefinition {
    CardDefinition {
        name: "Preordain",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![
            SpellEffect::Scry { amount: 2 },
            SpellEffect::DrawCards { amount: 1 },
        ],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
