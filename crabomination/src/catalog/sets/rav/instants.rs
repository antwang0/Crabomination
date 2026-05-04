use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, Subtypes};
use crate::effect::shortcut::{deal, gain_life, target, target_filtered};
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
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Putrefy — {1}{B}{G}: destroy target artifact or creature; it can't be regenerated.
///
/// Regeneration isn't modeled in this engine, so the "can't be regenerated"
/// clause is a no-op — `Effect::Destroy` already moves the target to its
/// owner's graveyard outright.
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
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
            ),
        },
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
