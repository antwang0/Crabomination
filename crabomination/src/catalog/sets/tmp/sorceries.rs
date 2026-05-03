use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, SelectionRequirement, Subtypes, Value};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost};

/// Reanimate — {B} Sorcery. Put target creature card from a graveyard onto
/// the battlefield under your control. You lose life equal to that
/// creature's mana value.
///
/// Wires `LoseLife(ManaValueOf(Target)) + Move(target → BF)`. The life
/// loss runs **before** the move so `Value::ManaValueOf` resolves while
/// the card is still in the graveyard (its zone-stable lookup falls
/// through battlefield → graveyards → exile → hands).
pub fn reanimate() -> CardDefinition {
    CardDefinition {
        name: "Reanimate",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
