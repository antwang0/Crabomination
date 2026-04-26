use super::no_abilities;
use crate::card::{AlternativeCost, CardDefinition, CardType, SelectionRequirement, Subtypes};
use crate::effect::shortcut::counter_target_spell;
use crate::mana::{Color, ManaCost, cost, generic, u};

/// Force of Will — {3}{U}{U}: counter target spell. Alternative cost: pay 1
/// life and exile a blue card from your hand rather than pay this spell's
/// mana cost.
pub fn force_of_will() -> CardDefinition {
    CardDefinition {
        name: "Force of Will",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: counter_target_spell(),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 1,
            exile_filter: Some(SelectionRequirement::HasColor(Color::Blue)),
            evoke_sacrifice: false,
            not_your_turn_only: false,
        }),
    }
}
