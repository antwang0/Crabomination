use crate::card::{CardDefinition, CardType};
use crate::effect::shortcut::draw;
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{cost, u};

/// Preordain — {U} Sorcery: Scry 2, then draw a card.
pub fn preordain() -> CardDefinition {
    CardDefinition {
        name: "Preordain",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            draw(1),
        ]),
        ..Default::default()
    }
}
