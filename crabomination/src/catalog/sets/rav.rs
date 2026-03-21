//! Ravnica: City of Guilds (RAV) — 2005

use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect};
use crate::mana::{b, cost, g, r, w};

/// Watchwolf — {G}{W} 3/3
pub fn watchwolf() -> CardDefinition {
    CardDefinition {
        name: "Watchwolf",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        power: 3, toughness: 3,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Lightning Helix — {R}{W}: deal 3 damage to any target, you gain 3 life
pub fn lightning_helix() -> CardDefinition {
    CardDefinition {
        name: "Lightning Helix",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![
            SpellEffect::DealDamage { amount: 3, target: SelectionRequirement::Any },
            SpellEffect::GainLife { amount: 3 },
        ],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Putrefy — {1}{B}{G}: destroy target creature
pub fn putrefy() -> CardDefinition {
    CardDefinition {
        name: "Putrefy",
        cost: cost(&[crate::mana::generic(1), b(), g()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DestroyCreature {
            target: SelectionRequirement::Creature,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
