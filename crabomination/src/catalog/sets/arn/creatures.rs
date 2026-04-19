use super::no_abilities;
use crate::card::{CardDefinition, CardType, CreatureType, SpellEffect, Subtypes, TriggerCondition, TriggeredAbility};
use crate::mana::{b, cost, generic};

/// Juzám Djinn — {2}{B}{B} 5/5
/// At the beginning of your upkeep, Juzám Djinn deals 1 damage to you.
pub fn juzam_djinn() -> CardDefinition {
    CardDefinition {
        name: "Juzám Djinn",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Djinn], ..Default::default() },
        power: 5, toughness: 5,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            condition: TriggerCondition::BeginningOfUpkeep,
            effects: vec![SpellEffect::LoseLife { amount: 1 }],
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
