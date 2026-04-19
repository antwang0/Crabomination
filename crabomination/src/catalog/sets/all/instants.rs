use super::no_abilities;
use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::counter_target_spell;
use crate::mana::{cost, generic, u};

/// Force of Will — {3}{U}{U}: counter target spell
pub fn force_of_will() -> CardDefinition {
    CardDefinition {
        name: "Force of Will",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: counter_target_spell(),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
