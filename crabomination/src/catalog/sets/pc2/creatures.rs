use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::mana::{b, cost, u};

/// Baleful Strix — {U}{B} 1/1 Flying Deathtouch
pub fn baleful_strix() -> CardDefinition {
    CardDefinition {
        name: "Baleful Strix",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}
