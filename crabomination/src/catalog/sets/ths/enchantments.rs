use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::mana::{cost, w};

/// Hopeful Eidolon — {W} Enchantment Creature — Spirit 1/1 Lifelink
pub fn hopeful_eidolon() -> CardDefinition {
    CardDefinition {
        name: "Hopeful Eidolon",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}
