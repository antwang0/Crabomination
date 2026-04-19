use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, Subtypes};
use crate::effect::shortcut::draw;
use crate::mana::{cost, u};

/// Brainstorm — {U}: draw three cards, then put two cards from your hand on top of your library
/// (Simplified: draw three cards.)
pub fn brainstorm() -> CardDefinition {
    CardDefinition {
        name: "Brainstorm",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: draw(3),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
