use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::mana::{Color, cost, generic, hybrid};

/// Mourning Thrull — {1}{W/B} 1/1 Flying Lifelink
pub fn mourning_thrull() -> CardDefinition {
    CardDefinition {
        name: "Mourning Thrull",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thrull],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        enters_with_counters: None,
    }
}
