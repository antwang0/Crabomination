//! Portal (POR) — 1997

use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect};
use crate::mana::{cost, r};

/// Shock — {R}: deal 2 damage to any target
pub fn shock() -> CardDefinition {
    CardDefinition {
        name: "Shock",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DealDamage {
            amount: 2,
            target: SelectionRequirement::Any,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
