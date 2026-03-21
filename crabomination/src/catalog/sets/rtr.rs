//! Return to Ravnica (RTR) — 2012

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Keyword};
use crate::mana::{cost, generic, g, r};

/// Ghor-Clan Rampager — {2}{R}{G} 4/4 Trample
pub fn ghor_clan_rampager() -> CardDefinition {
    CardDefinition {
        name: "Ghor-Clan Rampager",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        power: 4, toughness: 4,
        keywords: vec![Keyword::Trample],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
