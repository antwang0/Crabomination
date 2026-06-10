use crate::card::{CardDefinition, CardType};
use crate::effect::shortcut::discard;
use crate::effect::Selector;
use crate::mana::{b, cost};

/// Hymn to Tourach — {B}{B} Sorcery: target player discards two cards at random
pub fn hymn_to_tourach() -> CardDefinition {
    CardDefinition {
        name: "Hymn to Tourach",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: discard(Selector::Target(0), 2, true),
        ..Default::default()
    }
}
