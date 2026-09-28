use crate::card::{CardDefinition, CardType};
use crate::effect::shortcut::deal;
use crate::mana::{cost, r};

/// Shock — {R}: deal 2 damage to any target
pub fn shock() -> CardDefinition {
    CardDefinition {
        name: "Shock",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: deal(2, crate::effect::shortcut::target_any()),
        ..Default::default()
    }
}
