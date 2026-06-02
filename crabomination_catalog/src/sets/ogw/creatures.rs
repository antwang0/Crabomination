use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::effect::shortcut::prowess_trigger;
use crate::mana::{cost, r, u};

/// Stormchaser Mage — {1}{U}{R} 1/3 Flying Haste Prowess
pub fn stormchaser_mage() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser Mage",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Prowess],
        effect: Effect::Noop,
        triggered_abilities: vec![prowess_trigger()],
        ..Default::default()
    }
}
