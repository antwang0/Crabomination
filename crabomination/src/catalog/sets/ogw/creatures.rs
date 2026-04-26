use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::mana::{cost, generic, r, u};

/// Stormchaser Mage — {1}{U}{R} 1/3 Flying Haste
pub fn stormchaser_mage() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser Mage",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}
