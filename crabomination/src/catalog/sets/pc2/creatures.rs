use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::mana::{b, cost, u};

/// Baleful Strix — {U}{B} 1/1 Flying Deathtouch
pub fn baleful_strix() -> CardDefinition {
    CardDefinition {
        name: "Baleful Strix",
        cost: cost(&[u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}
