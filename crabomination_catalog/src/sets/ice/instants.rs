use crate::card::{CardDefinition, CardType};
use crate::effect::shortcut::draw;
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{cost, u};

/// Brainstorm — {U}: draw three cards, then put two cards from your hand on top of your library.
pub fn brainstorm() -> CardDefinition {
    CardDefinition {
        name: "Brainstorm",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            draw(3),
            Effect::PutOnLibraryFromHand { who: PlayerRef::You, count: Value::Const(2) },
        ]),
        ..Default::default()
    }
}
