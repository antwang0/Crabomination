//! MKM (Murders at Karlov Manor) gap batch — lands, artifact value, and a
//! wither commander. Tests in `tests/recent_b/recent247.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{investigate, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::mana::{b, cost, generic};

/// Magnifying Glass — {3} Artifact. {T}: Add {C}. {4}, {T}: Investigate.
pub fn magnifying_glass() -> CardDefinition {
    CardDefinition {
        name: "Magnifying Glass",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: investigate(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Escape Tunnel — Land. {T}, Sacrifice: search your library for a basic land,
/// put it onto the battlefield tapped, then shuffle. {T}, Sacrifice: target
/// creature with power 2 or less can't be blocked this turn.
pub fn escape_tunnel() -> CardDefinition {
    CardDefinition {
        name: "Escape Tunnel",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Scene of the Crime — Artifact Land — Clue. Enters tapped. {T}: Add {C}.
/// {T}, Tap an untapped creature you control: Add one mana of any color.
/// {2}, Sacrifice this land: Draw a card.
pub fn scene_of_the_crime() -> CardDefinition {
    CardDefinition {
        name: "Scene of the Crime",
        card_types: vec![CardType::Artifact, CardType::Land],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyColors(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Massacre Girl, Known Killer — {2}{B}{B} Legendary Creature — Human Assassin
/// 4/4, menace. Creatures you control have wither. Whenever a creature an
/// opponent controls dies, if its toughness was less than 1, draw a card.
pub fn massacre_girl_known_killer() -> CardDefinition {
    CardDefinition {
        name: "Massacre Girl, Known Killer",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have wither.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Wither,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            // "if its toughness was less than 1" — the death snapshot the trigger
            // filter sees carries the dying creature's last-known toughness
            // (counter-adjusted), so `ToughnessAtMost(0)` distinguishes a
            // wither/-1-1 kill from a lethal-damage one.
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::ToughnessAtMost(0),
                }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}
