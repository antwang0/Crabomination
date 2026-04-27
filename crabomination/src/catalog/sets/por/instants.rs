use super::no_abilities;
use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::{deal, target};
use crate::mana::{cost, r};

/// Shock — {R}: deal 2 damage to any target
pub fn shock() -> CardDefinition {
    CardDefinition {
        name: "Shock",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: deal(2, target()),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
