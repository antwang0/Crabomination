use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, Subtypes};
use crate::effect::{PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost};

/// Reanimate — {B} Sorcery: put target creature card from a graveyard onto the battlefield
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
        effect: Effect::Move {
            what: Selector::Target(0),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
