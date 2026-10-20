//! Commander: the 2026-10-20 (`01NkAE5B`) one-gap pair closers. Tests in
//! `tests/recent_b/cmdr_01nk_a.rs`.

use crate::card::{
    CardDefinition, CardType, EnchantmentSubtype, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, Selector};
use crate::mana::{cost, generic, u};

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// Blossombind — {1}{U} Aura. When it enters, tap enchanted creature.
/// Enchanted creature can't become untapped and can't have counters put on it.
pub fn blossombind() -> CardDefinition {
    CardDefinition {
        name: "Blossombind",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Tap { what: enchanted() })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature can't become untapped.",
                effect: StaticEffect::CantBecomeUntapped { applies_to: enchanted() },
            },
            StaticAbility {
                description: "Enchanted creature can't have counters put on it.",
                effect: StaticEffect::CantHaveCountersPutOn { applies_to: enchanted() },
            },
        ],
        ..Default::default()
    }
}

