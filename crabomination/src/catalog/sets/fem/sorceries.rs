use super::no_abilities;
use crate::card::{CardDefinition, CardType, SpellEffect, Subtypes};
use crate::mana::{b, cost};

/// Hymn to Tourach — {B}{B} Sorcery: target player discards two cards at random
pub fn hymn_to_tourach() -> CardDefinition {
    CardDefinition {
        name: "Hymn to Tourach",
        cost: cost(&[b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::Discard { amount: 2 }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
