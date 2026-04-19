use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect, Subtypes};
use crate::mana::{b, cost, g, generic, r, w};

/// Lightning Helix — {R}{W}: deal 3 damage to any target, you gain 3 life
pub fn lightning_helix() -> CardDefinition {
    CardDefinition {
        name: "Lightning Helix",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![
            SpellEffect::DealDamage { amount: 3, target: SelectionRequirement::Any },
            SpellEffect::GainLife { amount: 3 },
        ],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Putrefy — {1}{B}{G}: destroy target creature
pub fn putrefy() -> CardDefinition {
    CardDefinition {
        name: "Putrefy",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DestroyCreature {
            target: SelectionRequirement::Creature,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
