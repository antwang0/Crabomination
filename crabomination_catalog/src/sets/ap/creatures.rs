use crate::card::{CardDefinition, CardType, CreatureType, Keyword, Subtypes};
use crate::mana::{cost, g, u};

/// Gaea's Skyfolk — {G}{U} 2/2 Merfolk Flying
pub fn gaeas_skyfolk() -> CardDefinition {
    CardDefinition {
        name: "Gaea's Skyfolk",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}
