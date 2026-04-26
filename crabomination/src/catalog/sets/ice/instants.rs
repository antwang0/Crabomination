use super::no_abilities;
use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::draw;
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{cost, u};

/// Brainstorm — {U}: draw three cards, then put two cards from your hand on top of your library.
pub fn brainstorm() -> CardDefinition {
    CardDefinition {
        name: "Brainstorm",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            draw(3),
            Effect::PutOnLibraryFromHand { who: PlayerRef::You, count: Value::Const(2) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        start_of_game_effect: None,
    }
}
