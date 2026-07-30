//! Gap batch — MKM detectives and clue payoffs on existing primitives. Tests in
//! `tests/recent227.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, EntersAsCopy, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{each_opponent, etb, investigate};
use crate::effect::{Duration, Effect, Predicate, Selector, Value};
use crate::mana::{b, cost, generic, u, w};

/// Persuasive Interrogators — {4}{B}{B} 5/6 Gorgon Detective. ETB: investigate.
/// Whenever you sacrifice a Clue, target opponent gets two poison counters.
pub fn persuasive_interrogators() -> CardDefinition {
    CardDefinition {
        name: "Persuasive Interrogators",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gorgon, CreatureType::Detective],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        triggered_abilities: vec![
            etb(investigate(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Clue),
                    }),
                effect: Effect::AddPoison {
                    who: each_opponent(),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Visage Bandit — {3}{U} 2/2 Shapeshifter Rogue. May enter as a copy of a
/// creature you control (staying a Shapeshifter Rogue). Plot {2}{U}.
pub fn visage_bandit() -> CardDefinition {
    CardDefinition {
        name: "Visage Bandit",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::ControlledByYou),
            extra_creature_types: vec![CreatureType::Shapeshifter, CreatureType::Rogue],
            ..Default::default()
        }),
        plot_cost: Some(cost(&[generic(2), u()])),
        ..Default::default()
    }
}

/// Perimeter Enforcer — {1}{W} 1/1 Human Detective. Flying, lifelink. Whenever
/// another Detective you control enters, this creature gets +1/+1 until end of
/// turn. (The "or is turned face up" clause is omitted.)
pub fn perimeter_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Perimeter Enforcer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Detective).and(R::OtherThanSource),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
