use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::mana::{cost, g, u};

/// Gaea's Skyfolk — {G}{U} 2/2 Merfolk Flying
pub fn gaeas_skyfolk() -> CardDefinition {
    CardDefinition {
        name: "Gaea's Skyfolk",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}
