use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, Subtypes};
use crate::mana::{cost, g, w};

/// Watchwolf — {G}{W} 3/3
pub fn watchwolf() -> CardDefinition {
    CardDefinition {
        name: "Watchwolf",
        cost: cost(&[g(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 3, toughness: 3,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
