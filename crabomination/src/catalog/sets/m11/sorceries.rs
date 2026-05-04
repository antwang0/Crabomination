use super::no_abilities;
use crate::card::{CardDefinition, CardType, Subtypes};
use crate::effect::shortcut::draw;
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{cost, u};

/// Preordain — {U} Sorcery: Scry 2, then draw a card.
pub fn preordain() -> CardDefinition {
    CardDefinition {
        name: "Preordain",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            draw(1),
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}
