use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::discard;
use crate::effect::Selector;
use crate::mana::{b, cost};

/// Hymn to Tourach — {B}{B} Sorcery: target player discards two cards at random
pub fn hymn_to_tourach() -> CardDefinition {
    CardDefinition {
        name: "Hymn to Tourach",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: discard(Selector::Target(0), 2, true),
        triggered_abilities: vec![],
        ..Default::default()
    }
}
