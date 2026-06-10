use crate::card::{CardDefinition, CardType, CreatureType, Keyword, Subtypes};
use crate::mana::{Color, cost, generic, hybrid};

/// Mourning Thrull — {1}{W/B} 1/1 Flying Lifelink
pub fn mourning_thrull() -> CardDefinition {
    CardDefinition {
        name: "Mourning Thrull",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thrull],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        ..Default::default()
    }
}
