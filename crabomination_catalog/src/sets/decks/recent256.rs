//! MKM (Murders at Karlov Manor) gap batch — Golgari graveyard engine and a
//! Azorius top-of-library enchantment. Tests in `tests/recent_b/recent256.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, SelectionRequirement as R, StaticAbility,
    Subtypes, TokenDefinition,
};
use crate::effect::shortcut::grant_tap_for_any_color;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, StaticEffect,
    TriggeredAbility, Value,
};
use crate::mana::{Color, b, cost, g, generic, w};

/// Insidious Roots — {B}{G} Enchantment. Your creature tokens tap for any color.
/// Whenever one or more creature cards leave your graveyard, create a 0/1 green
/// Plant token, then put a +1/+1 counter on each Plant you control.
pub fn insidious_roots() -> CardDefinition {
    CardDefinition {
        name: "Insidious Roots",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![grant_tap_for_any_color(
            R::IsToken.and(R::Creature).and(R::ControlledByYou),
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Plant".into(),
                        colors: vec![Color::Green],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Plant],
                            ..Default::default()
                        },
                        power: 0,
                        toughness: 1,
                        ..Default::default()
                    },
                },
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Plant).and(R::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Assemble the Players — {1}{W} Enchantment. You may look at the top card of
/// your library any time. Once each turn, you may cast a creature spell with
/// power 2 or less from the top of your library.
pub fn assemble_the_players() -> CardDefinition {
    CardDefinition {
        name: "Assemble the Players",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "Once each turn, you may cast a creature spell with power 2 or \
                              less from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTopOncePerTurn {
                    filter: R::Creature.and(R::PowerAtMost(2)),
                },
            },
        ],
        ..Default::default()
    }
}
