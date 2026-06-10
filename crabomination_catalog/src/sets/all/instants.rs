use crate::card::{AlternativeCost, CardDefinition, CardType, SelectionRequirement};
use crate::effect::shortcut::counter_target_spell;
use crate::mana::{Color, ManaCost, cost, generic, u};

/// Force of Will — {3}{U}{U}: counter target spell. Alternative cost: pay 1
/// life and exile a blue card from your hand rather than pay this spell's
/// mana cost.
pub fn force_of_will() -> CardDefinition {
    CardDefinition {
        name: "Force of Will",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: counter_target_spell(),
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 1,
            exile_filter: Some(SelectionRequirement::HasColor(Color::Blue)),
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
                    exile_from_graveyard_count: 0,
                    return_to_hand: None,
                    sacrifice_permanents: None,
            effect_override: None,
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
        }),
        ..Default::default()
    }
}
