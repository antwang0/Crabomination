//! Zendikar (ZEN) — 2009

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Keyword, SpellEffect, TriggerCondition, TriggeredAbility};
use crate::mana::{cost, r};

/// Goblin Guide — {R} 2/2 Haste
/// Whenever Goblin Guide attacks, defending player reveals the top card of their library.
/// If it's a land card, that player puts it into their hand.
pub fn goblin_guide() -> CardDefinition {
    CardDefinition {
        name: "Goblin Guide",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        power: 2, toughness: 2,
        keywords: vec![Keyword::Haste],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                condition: TriggerCondition::Attacks,
                effects: vec![SpellEffect::RevealOpponentTopCard],
            },
        ],
    }
}
