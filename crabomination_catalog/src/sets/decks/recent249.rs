//! MKM (Murders at Karlov Manor) gap batch — suspect + artifact Detectives and a
//! during-your-turn body. Tests in `tests/recent_b/recent249.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, investigate};
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{b, cost, generic, g, u};

/// Clandestine Meddler — {2}{B} Creature — Vampire Rogue 3/2. ETB suspect up to
/// one other target creature you control. Whenever one or more suspected
/// creatures you control attack, surveil 1.
pub fn clandestine_meddler() -> CardDefinition {
    CardDefinition {
        name: "Clandestine Meddler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Suspect {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    },
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                    Predicate::AttackedWithCreatureMatching {
                        who: PlayerRef::You,
                        filter: R::IsSuspected,
                    },
                ),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Forensic Gadgeteer — {2}{U} Creature — Vedalken Artificer Detective 2/3.
/// Whenever you cast an artifact spell, investigate. (The "activated abilities of
/// artifacts you control cost {1} less" static is not modeled.)
pub fn forensic_gadgeteer() -> CardDefinition {
    CardDefinition {
        name: "Forensic Gadgeteer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Vedalken,
                CreatureType::Artificer,
                CreatureType::Detective,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: investigate(1),
        }],
        ..Default::default()
    }
}

/// Pompous Gadabout — {2}{G} Creature — Human Citizen 4/2. During your turn, this
/// creature has hexproof. (The "can't be blocked by creatures that don't have a
/// name" rider is not modeled — every non-token creature has a name.)
pub fn pompous_gadabout() -> CardDefinition {
    CardDefinition {
        name: "Pompous Gadabout",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has hexproof",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Hexproof,
                condition: R::ControllersTurn,
            },
        }],
        ..Default::default()
    }
}
