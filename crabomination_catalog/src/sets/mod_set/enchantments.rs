//! Pillowfort / tax enchantments — "creatures can't attack you unless their
//! controller pays a tax." Powered by `StaticEffect::AttackTaxToController`,
//! which `GameState::declare_attackers` charges per attacking creature.

use crate::card::{CardDefinition, CardType, StaticAbility, StaticEffect};
use crate::mana::{cost, generic, u, w};

/// Propaganda — {2}{U} Enchantment. "Creatures can't attack you unless their
/// controller pays {2} for each of those creatures."
pub fn propaganda() -> CardDefinition {
    CardDefinition {
        name: "Propaganda",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you unless their controller pays {2} for each of those creatures.",
            effect: StaticEffect::AttackTaxToController { amount: 2 },
        }],
        ..Default::default()
    }
}

/// Ghostly Prison — {2}{W} Enchantment. White mirror of Propaganda:
/// "Creatures can't attack you unless their controller pays {2} for each of
/// those creatures."
pub fn ghostly_prison() -> CardDefinition {
    CardDefinition {
        name: "Ghostly Prison",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you unless their controller pays {2} for each of those creatures.",
            effect: StaticEffect::AttackTaxToController { amount: 2 },
        }],
        ..Default::default()
    }
}
