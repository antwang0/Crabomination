use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::mana::{cost, generic, r, u};

/// Stormchaser Mage — {1}{U}{R} 1/3 Flying Haste Prowess.
///
/// Push XXXVIII wired Prowess as a synthetic SpellCast trigger so
/// this evasive U/R 2-drop now pumps +1/+1 EOT on every noncreature
/// IS spell cast in the same turn. Stacks with Magecraft / Opus
/// payoffs that key off the same SpellCast event.
pub fn stormchaser_mage() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser Mage",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Prowess],
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
