//! CR-conformance batch — cards that exercise newly-wired rules
//! (CR 616.1c replacement ordering, CR 704.7 single loss replacement).
//! Tests in `core_rules/cr_recent42`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Selector, StaticAbility, Subtypes,
};
use crate::effect::StaticEffect;
use crate::mana::{cost, generic};

/// Lich's Mirror — CR 704.7: one replacement covers every simultaneous loss
/// state-based action, resetting you to a fresh seven and 20 life.
pub fn lichs_mirror() -> CardDefinition {
    CardDefinition {
        name: "Lich's Mirror",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If you would lose the game, instead reset to seven cards and 20 life.",
            effect: StaticEffect::ReplaceControllerLossWithReset,
        }],
        ..Default::default()
    }
}

/// Rusted Sentinel — {4} 3/4 Golem that enters tapped.
pub fn rusted_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Rusted Sentinel",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..Default::default()
    }
}
