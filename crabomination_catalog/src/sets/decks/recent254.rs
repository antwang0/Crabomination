//! MKM (Murders at Karlov Manor) gap batch — Izzy/Rakdos legends and the
//! variable collect-evidence dragon. Tests in `tests/recent_b/recent254.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, DynamicPt, Keyword, SelectionRequirement as R,
    Subtypes, Supertype,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticAbility, StaticEffect,
    TriggeredAbility, Value,
};
use crate::mana::{cost, generic, r, u};

/// Melek, Reforged Researcher — {3}{U}{R} Legendary Creature — Weird Detective.
/// P/T = twice the instant and sorcery cards in your graveyard. The first
/// instant or sorcery spell you cast each turn costs {3} less.
pub fn melek_reforged_researcher() -> CardDefinition {
    CardDefinition {
        name: "Melek, Reforged Researcher",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Weird, CreatureType::Detective],
            ..Default::default()
        },
        dynamic_pt: Some(DynamicPt::InstantSorceryCardsInControllerGraveyard { mult: 2 }),
        static_abilities: vec![StaticAbility {
            description: "The first instant or sorcery spell you cast each turn costs {3} less.",
            effect: StaticEffect::CostReductionFirstInstantOrSorcery { amount: 3 },
        }],
        ..Default::default()
    }
}

/// Incinerator of the Guilty — {4}{R}{R} Creature — Dragon 6/6. Flying, trample.
/// When it deals combat damage to a player, you may collect evidence X; if you
/// do, it deals X damage to each creature and planeswalker that player controls.
pub fn incinerator_of_the_guilty() -> CardDefinition {
    CardDefinition {
        name: "Incinerator of the Guilty",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            // The trigger binds the damaged player to target slot 0.
            effect: Effect::CollectEvidenceX {
                then: Box::new(Effect::DealDamage {
                    to: Selector::ControlledBy {
                        who: PlayerRef::Target(0),
                        filter: R::Creature.or(R::Planeswalker),
                    },
                    amount: Value::XFromCost,
                }),
            },
        }],
        ..Default::default()
    }
}
