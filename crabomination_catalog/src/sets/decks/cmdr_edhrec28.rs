//! Commander: the last claimed two-card gaps — Samwise Gamgee's Campsite
//! Cuisine and The Ring Goes South, Miles "Tails" Prower's The Thanos-Copter
//! and Lita (`cmdr_edhrec27` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec28.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, w};
use std::sync::Arc;

fn vehicle() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
}

fn food() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::food_token()),
    }
}

// ── Samwise Gamgee ───────────────────────────────────────────────────────────

/// Campsite Cuisine — {1}{G} enchantment. It or a legendary creature of yours
/// entering makes a Food. Whenever you attack, you may sacrifice X Foods;
/// when you do, up to X target attacking creatures each get +3/+3, trample
/// and indestructible until end of turn.
pub fn campsite_cuisine() -> CardDefinition {
    let boost = |kw| Effect::GrantKeyword { what: Selector::Target(0), keyword: kw, duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Campsite Cuisine",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(food()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                    },
                ),
                effect: food(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::SacrificeAnyNumber {
                        who: PlayerRef::You,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Food),
                        per_each: Box::new(Effect::Noop),
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::SacrificedCount, Value::ONE),
                        then: Box::new(Effect::WithX {
                            x: Value::SacrificedCount,
                            body: Box::new(Effect::ReflexiveTrigger {
                                body: Box::new(Effect::CapTargetsAtX {
                                    body: Box::new(Effect::ApplyToTargets {
                                        max_targets: 8,
                                        min_targets: 0,
                                        filter: R::Creature.and(R::IsAttacking),
                                        effect: Box::new(Effect::Seq(vec![
                                            Effect::PumpPT {
                                                what: Selector::Target(0),
                                                power: Value::Const(3),
                                                toughness: Value::Const(3),
                                                duration: Duration::EndOfTurn,
                                            },
                                            boost(Keyword::Trample),
                                            boost(Keyword::Indestructible),
                                        ])),
                                    }),
                                }),
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// The Ring Goes South — {3}{G} sorcery. The Ring tempts you. Reveal until X
/// lands (X = legendary creatures you control): the lands enter tapped, the
/// rest go to the bottom in a random order.
pub fn the_ring_goes_south() -> CardDefinition {
    CardDefinition {
        name: "The Ring Goes South",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::RingTempts { who: PlayerRef::You },
            Effect::RevealUntilLandsToBattlefield {
                count: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                )),
                tapped: true,
            },
        ]),
        ..Default::default()
    }
}

// ── Miles "Tails" Prower ─────────────────────────────────────────────────────

/// The Thanos-Copter — {6} legendary artifact — Vehicle 5/5, flying, haste,
/// crew 2. ETB: your Vehicles become artifact creatures until end of turn. A
/// Vehicle of yours hitting a player draws a card.
pub fn the_thanos_copter() -> CardDefinition {
    CardDefinition {
        name: "The Thanos-Copter",
        cost: cost(&[generic(6)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Crew(2)],
        triggered_abilities: vec![
            etb(Effect::AnimateAsCreature {
                what: Selector::EachPermanent(vehicle().and(R::ControlledByYou)),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).dealt_by(vehicle()),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Lita, Mechanical Engineer — {2}{W} 3/3 legendary artifact creature
/// Artificer, vigilance. Your end step untaps your other artifact creatures.
/// {3}{W}, {T}: a 5/5 colorless Vehicle artifact token named Zeppelin with
/// flying and crew 3.
pub fn lita_mechanical_engineer() -> CardDefinition {
    let zeppelin = TokenDefinition {
        name: "Zeppelin".into(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Lita, Mechanical Engineer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Artificer], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    R::Artifact.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                up_to: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            tap_cost: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(zeppelin) },
            ..Default::default()
        }],
        ..Default::default()
    }
}
