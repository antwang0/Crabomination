use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect, Subtypes, Zone};
use crate::mana::{b, cost};

/// Reanimate — {B} Sorcery: put target creature card from a graveyard onto the battlefield
pub fn reanimate() -> CardDefinition {
    CardDefinition {
        name: "Reanimate",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::ReturnFromGraveyard {
            filter: SelectionRequirement::Creature,
            put_into: Zone::Battlefield,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
