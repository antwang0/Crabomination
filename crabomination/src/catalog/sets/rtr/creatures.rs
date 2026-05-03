use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, Keyword, Subtypes};
use crate::mana::{cost, g, generic, r};

/// Ghor-Clan Rampager — {2}{R}{G} 4/4 Trample
pub fn ghor_clan_rampager() -> CardDefinition {
    CardDefinition {
        name: "Ghor-Clan Rampager",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
