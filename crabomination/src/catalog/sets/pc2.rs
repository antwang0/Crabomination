//! Planechase 2012 (PC2) — 2012

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Keyword};
use crate::mana::{b, cost, u};

/// Baleful Strix — {U}{B} 1/1 Flying Deathtouch
pub fn baleful_strix() -> CardDefinition {
    CardDefinition {
        name: "Baleful Strix",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        power: 1, toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
