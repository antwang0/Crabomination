//! Aetherdrift (DFT) gap cards unblocked by four small engine primitives:
//! `Predicate::SacrificedWasVehicle` (Hellish Sideswipe), `StaticEffect::
//! SelfIsCreatureIf` (Midnight Mangler), `Effect::SetSaddled` + `Effect::
//! AnimateAsCreature` (Guidelight Matrix), and `StaticEffect::
//! SelfCrewsSaddlesWithToughness` (Interface Ace). Tests in
//! `crabomination/src/tests/recent168.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{b, cost, generic, u, w};

/// Hellish Sideswipe — {B} Sorcery. Sacrifice an artifact or creature as an
/// additional cost. Destroy target creature or Vehicle; if the sacrificed
/// permanent was a Vehicle, draw a card.
pub fn hellish_sideswipe() -> CardDefinition {
    CardDefinition {
        name: "Hellish Sideswipe",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.or(R::Artifact),
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                ),
            },
            Effect::If {
                cond: Predicate::SacrificedWasVehicle,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Midnight Mangler — {1}{U} Artifact — Vehicle 3/3. During turns other than
/// yours, it is an artifact creature. Crew 2.
pub fn midnight_mangler() -> CardDefinition {
    CardDefinition {
        name: "Midnight Mangler",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        static_abilities: vec![StaticAbility {
            description: "During turns other than yours, this Vehicle is an artifact creature.",
            effect: StaticEffect::SelfIsCreatureIf {
                condition: Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
            },
        }],
        ..Default::default()
    }
}

/// Guidelight Matrix — {2} Artifact. ETB: draw. `{2},{T}: Target Mount you
/// control becomes saddled (sorcery speed).` `{2},{T}: Target Vehicle you
/// control becomes an artifact creature until end of turn.`
pub fn guidelight_matrix() -> CardDefinition {
    CardDefinition {
        name: "Guidelight Matrix",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::SetSaddled {
                    what: target_filtered(
                        R::HasCreatureType(CreatureType::Mount).and(R::ControlledByYou),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::AnimateAsCreature {
                    what: target_filtered(
                        R::HasArtifactSubtype(ArtifactSubtype::Vehicle).and(R::ControlledByYou),
                    ),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Interface Ace — {1}{W} Artifact Creature — Robot Pilot 0/4. Crews Vehicles
/// and saddles Mounts using its toughness rather than its power. Whenever it
/// becomes tapped during your turn, untap it (once each turn).
pub fn interface_ace() -> CardDefinition {
    CardDefinition {
        name: "Interface Ace",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Pilot],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Crews Vehicles and saddles Mounts using its toughness rather than its power.",
            effect: StaticEffect::SelfCrewsSaddlesWithToughness,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You))
                .once_per_turn(),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..Default::default()
    }
}
