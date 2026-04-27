use super::no_abilities;
use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::discard;
use crate::effect::Selector;
use crate::mana::{b, cost};

/// Hymn to Tourach — {B}{B} Sorcery: target player discards two cards at random
pub fn hymn_to_tourach() -> CardDefinition {
    CardDefinition {
        name: "Hymn to Tourach",
        cost: cost(&[b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: discard(Selector::Target(0), 2, true),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand_effect: None,
    }
}
