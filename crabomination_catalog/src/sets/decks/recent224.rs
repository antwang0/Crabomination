//! BLB + DFT gap batch — a Ward-granting Frog, a Mount/Vehicle Pilot lord, and
//! a Vehicle-flying planeswalker-esque legend. Tests in `tests/recent224.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::etb;
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
    StaticAbility, StaticEffect, Value,
};
use crate::mana::{cost, generic, r, u, w};

fn ward_1() -> Keyword {
    Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))
}

/// Long River Lurker — {2}{U} 2/3 Frog Scout. Ward {1}; other Frogs you control
/// have ward {1}; when it enters, a target creature you control can't be blocked
/// this turn. (The "blink it after it connects" rider is not modeled.)
pub fn long_river_lurker() -> CardDefinition {
    CardDefinition {
        name: "Long River Lurker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![ward_1()],
        static_abilities: vec![StaticAbility {
            description: "Other Frogs you control have ward {1}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Frog)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                keyword: ward_1(),
            },
        }],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByYou),
            },
            keyword: Keyword::Unblockable,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Kolodin, Triumph Caster — {R}{W} 2/3 Human Pilot. Mounts and Vehicles you
/// control have haste; a Mount you control that enters becomes saddled; a
/// Vehicle you control that enters becomes an artifact creature until end of turn.
pub fn kolodin_triumph_caster() -> CardDefinition {
    let mount_or_vehicle =
        R::HasCreatureType(CreatureType::Mount).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle));
    CardDefinition {
        name: "Kolodin, Triumph Caster",
        cost: cost(&[r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Mounts and Vehicles you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    mount_or_vehicle.clone().and(R::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Mount),
                    }),
                effect: Effect::SetSaddled {
                    what: Selector::TriggerSource,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                    }),
                effect: Effect::AnimateAsCreature {
                    what: Selector::TriggerSource,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

fn vehicle_3_2_crew1() -> TokenDefinition {
    TokenDefinition {
        name: "Vehicle".to_string(),
        power: 3,
        toughness: 2,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        keywords: vec![Keyword::Crew(1)],
        ..Default::default()
    }
}

/// Mu Yanling, Wind Rider — {2}{U}{U} 2/4 Human Wizard Pilot. Enters making a
/// 3/2 colorless Vehicle with crew 1; Vehicles you control have flying; when one
/// or more of your flyers hit a player, draw a card.
pub fn mu_yanling_wind_rider() -> CardDefinition {
    CardDefinition {
        name: "Mu Yanling, Wind Rider",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Wizard,
                CreatureType::Pilot,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Vehicles you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasArtifactSubtype(ArtifactSubtype::Vehicle).and(R::ControlledByYou),
                ),
                keyword: Keyword::Flying,
            },
        }],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(vehicle_3_2_crew1()),
            }),
            // `once_per_turn` approximates "one or more … deal combat damage".
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                )
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Flying),
                })
                .once_per_turn(),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}
