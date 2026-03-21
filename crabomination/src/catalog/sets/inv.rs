//! Invasion (INV) — 2000

use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect};
use crate::mana::{b, cost, r};

/// Terminate — {B}{R}: destroy target creature
pub fn terminate() -> CardDefinition {
    CardDefinition {
        name: "Terminate",
        cost: cost(&[b(), r()]),
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
