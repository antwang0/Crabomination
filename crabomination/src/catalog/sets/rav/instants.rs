use super::no_abilities;
use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::{deal, destroy_target, gain_life, target};
use crate::effect::Effect;
use crate::mana::{b, cost, g, generic, r, w};

/// Lightning Helix — {R}{W}: deal 3 damage to any target, you gain 3 life
pub fn lightning_helix() -> CardDefinition {
    CardDefinition {
        name: "Lightning Helix",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![deal(3, target()), gain_life(3)]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Putrefy — {1}{B}{G}: destroy target creature
pub fn putrefy() -> CardDefinition {
    CardDefinition {
        name: "Putrefy",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: destroy_target(),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
