use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, Effect, EventKind, EventScope, EventSpec, SelectionRequirement,
    StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
};
use crate::effect::{PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, generic, w};

/// Glorious Anthem — {1}{W}{W} Enchantment
/// Creatures you control get +1/+1.
pub fn glorious_anthem() -> CardDefinition {
    CardDefinition {
        name: "Glorious Anthem",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        start_of_game_effect: None,
    }
}

/// Animate Dead — {1}{B} Enchantment — Aura.
///
/// When Animate Dead enters the battlefield, return target creature card from
/// a graveyard to the battlefield. The real Oracle text reattaches Animate
/// Dead to the creature, sacrifices it on leave, and applies -1/-0; this
/// engine doesn't yet model the aura-attach plus leaves-the-battlefield
/// trigger plus power-modifier chain, so we keep the simplified reanimate
/// behavior but at least restrict the target to creature cards.
pub fn animate_dead() -> CardDefinition {
    CardDefinition {
        name: "Animate Dead",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature,
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        start_of_game_effect: None,
    }
}
