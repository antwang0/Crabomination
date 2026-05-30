use crate::card::{CardDefinition, CardType, CreatureType, Effect, Subtypes};
use crate::mana::{cost, g, w};

/// Watchwolf — {G}{W} 3/3
pub fn watchwolf() -> CardDefinition {
    CardDefinition {
        name: "Watchwolf",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wolf],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}
