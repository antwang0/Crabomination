use crate::card::{CardDefinition, CardType, Effect, Keyword, Subtypes};
use crate::mana::{cost, g, generic, r};

/// Ghor-Clan Rampager — {2}{R}{G} 4/4 Trample
pub fn ghor_clan_rampager() -> CardDefinition {
    CardDefinition {
        name: "Ghor-Clan Rampager",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}
