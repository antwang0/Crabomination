use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect, Subtypes};
use crate::mana::{cost, generic, u};

/// Force of Will — {3}{U}{U}: counter target spell
pub fn force_of_will() -> CardDefinition {
    CardDefinition {
        name: "Force of Will",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::CounterSpell {
            target: SelectionRequirement::Any,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
