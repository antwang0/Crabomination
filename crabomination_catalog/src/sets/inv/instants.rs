use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::{destroy_target_no_regen, draw};
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{b, cost, r, u};

/// Terminate — {B}{R}: destroy target creature. It can't be regenerated.
/// (CR 701.15g — wired via `Effect::DestroyNoRegen`.)
pub fn terminate() -> CardDefinition {
    CardDefinition {
        name: "Terminate",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: destroy_target_no_regen(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Opt — {U} Instant: Scry 1, then draw a card.
pub fn opt() -> CardDefinition {
    CardDefinition {
        name: "Opt",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            draw(1),
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}
