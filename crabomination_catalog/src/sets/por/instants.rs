use crate::card::{CardDefinition, CardType};
use crate::effect::shortcut::{deal, target};
use crate::mana::{cost, r};

/// Shock — {R}: deal 2 damage to any target
pub fn shock() -> CardDefinition {
    CardDefinition {
        name: "Shock",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: deal(2, target()),
        ..Default::default()
    }
}
