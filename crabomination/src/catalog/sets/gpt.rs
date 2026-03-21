//! Guildpact (GPT) — 2006

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Keyword};
use crate::mana::{b, cost, generic, w};

/// Mourning Thrull — {1}{W}{B} 1/1 Flying Lifelink
pub fn mourning_thrull() -> CardDefinition {
    CardDefinition {
        name: "Mourning Thrull",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        power: 1, toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
