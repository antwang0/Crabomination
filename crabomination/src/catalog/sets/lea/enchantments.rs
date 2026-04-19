use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect, StaticAbility, StaticAbilityTemplate, Subtypes, TriggerCondition, TriggeredAbility, Zone};
use crate::mana::{b, cost, generic, w};

/// Glorious Anthem — {1}{W}{W} Enchantment
/// Creatures you control get +1/+1.
pub fn glorious_anthem() -> CardDefinition {
    CardDefinition {
        name: "Glorious Anthem",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1",
            template: StaticAbilityTemplate::PumpYourCreatures { power: 1, toughness: 1 },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Animate Dead — {1}{B} Enchantment
/// When Animate Dead enters the battlefield, return target creature card from a graveyard to play.
pub fn animate_dead() -> CardDefinition {
    CardDefinition {
        name: "Animate Dead",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            condition: TriggerCondition::EntersBattlefield,
            effects: vec![SpellEffect::ReturnFromGraveyard {
                filter: SelectionRequirement::Creature,
                put_into: Zone::Battlefield,
            }],
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
