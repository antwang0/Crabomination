//! Commander: 2026-10-18 (`015XaGkQ`) one-deck closers. Tests in
//! `tests/recent_b/cmdr_015x_a.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::card::MayPlayDuration;
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Effect, Predicate, PlayerRef, Selector, StaticEffect, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};
use std::sync::Arc;

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn transform_self(mana: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        effect: Effect::Transform { what: Selector::This },
        ..Default::default()
    }
}

/// April O'Neil, Hacktivist — {3}{U} legendary 1/5 Human Scientist. At the
/// beginning of your end step, draw a card for each card type among spells
/// you've cast this turn.
pub fn april_oneil_hacktivist() -> CardDefinition {
    legend(CardDefinition {
        name: "April O'Neil, Hacktivist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Scientist], ..Default::default() },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::You),
                amount: Value::CardTypesAmongSpellsCastThisTurn,
            },
        }],
        ..Default::default()
    })
}

/// The Incredible Hulk — Bruce Banner's back face, an 8/8 reach, trample
/// Gamma Berserker Hero. Enrage: a +1/+1 counter; if he's attacking, untap
/// him and there is an additional combat phase after this one (CR 500.8).
fn the_incredible_hulk() -> CardDefinition {
    legend(CardDefinition {
        name: "The Incredible Hulk",
        cost: cost(&[generic(2), r(), r(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gamma, CreatureType::Berserker, CreatureType::Hero],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking },
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::This, up_to: None },
                        Effect::AdditionalCombatPhase { count: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    })
}

/// Bruce Banner // The Incredible Hulk — {U} legendary 1/1 Human Scientist
/// Hero. {X}{X}, {T}: draw X (sorcery speed). {2}{R}{R}{G}{G}: transform
/// (sorcery speed). A modal DFC: the Hulk face may be cast too.
pub fn bruce_banner() -> CardDefinition {
    legend(CardDefinition {
        name: "Bruce Banner",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[x(), x()]),
                sorcery_speed: true,
                effect: Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::XFromCost },
                ..Default::default()
            },
            transform_self(cost(&[generic(2), r(), r(), g(), g()])),
        ],
        back_face: Some(Box::new(the_incredible_hulk())),
        ..Default::default()
    })
}

/// Amazing Spider-Man — Peter Parker's back face, a 4/4 vigilance, reach
/// Spider Human Hero. Each legendary spell you cast that's one or more colors
/// has web-slinging {G}{W}{U} (CR 702.188).
fn amazing_spider_man() -> CardDefinition {
    legend(CardDefinition {
        name: "Amazing Spider-Man",
        cost: cost(&[generic(1), g(), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Each legendary spell you cast that's one or more colors has web-slinging {G}{W}{U}.",
            effect: StaticEffect::GrantWebSlingingToSpells {
                filter: R::HasSupertype(Supertype::Legendary).and(R::Colorless.negate()),
                cost: cost(&[g(), w(), u()]),
            },
        }],
        ..Default::default()
    })
}

/// Peter Parker // Amazing Spider-Man — {1}{W} legendary 0/1 Human Scientist
/// Hero. Enters: a 2/1 green Spider token with reach. {1}{G}{W}{U}:
/// transform (sorcery speed). A modal DFC.
pub fn peter_parker() -> CardDefinition {
    legend(CardDefinition {
        name: "Peter Parker",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(TokenDefinition {
                name: "Spider".into(),
                power: 2,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
                keywords: vec![Keyword::Reach],
                ..Default::default()
            }),
        })],
        activated_abilities: vec![transform_self(cost(&[generic(1), g(), w(), u()]))],
        back_face: Some(Box::new(amazing_spider_man())),
        ..Default::default()
    })
}

/// Public Enemy — {2}{U} Aura. All creatures attack enchanted creature's
/// controller each combat if able (CR 508.1d). When enchanted creature dies,
/// draw a card.
pub fn public_enemy() -> CardDefinition {
    CardDefinition {
        name: "Public Enemy",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "All creatures attack enchanted creature's controller each combat if able.",
            effect: StaticEffect::AllCreaturesAttackEnchantedController,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Maraxus of Keld — {4}{R}{R} legendary */* Human Warrior; its power and
/// toughness are each the number of untapped artifacts, creatures, and lands
/// you control (CR 604.3 characteristic-defining).
pub fn maraxus_of_keld() -> CardDefinition {
    let untapped = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::Artifact.or(R::Creature).or(R::Land).and(R::Untapped).and(R::ControlledByYou),
        )))
    };
    legend(CardDefinition {
        name: "Maraxus of Keld",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Warrior], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Maraxus's power and toughness are each equal to the number of untapped artifacts, creatures, and lands you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: untapped(), toughness: untapped() },
        }],
        ..Default::default()
    })
}

/// Prowl, Pursuit Vehicle — Prowl's converted face, a 2/3 living metal
/// Vehicle. Another creature or Vehicle of yours entering puts a +1/+1
/// counter on it; the second resolution in a turn also converts it.
fn prowl_pursuit_vehicle() -> CardDefinition {
    let counter = || Effect::AddCounter {
        what: Selector::This,
        kind: crate::card::CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    legend(CardDefinition {
        name: "Prowl, Pursuit Vehicle",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: vec![Color::White],
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::LivingMetal],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                },
            ),
            effect: Effect::EscalatingThisTurn {
                modes: vec![
                    counter(),
                    Effect::Seq(vec![counter(), Effect::Transform { what: Selector::This }]),
                    counter(),
                ],
            },
        }],
        ..Default::default()
    })
}

/// Prowl, Stoic Strategist // Prowl, Pursuit Vehicle — {3}{W} legendary 3/3
/// artifact creature Robot; More Than Meets the Eye {2}{W}. Attacking, it
/// exiles up to one other tapped creature or Vehicle, whose owner may play it
/// while it stays exiled; a card played from among those draws you a card
/// and converts Prowl (CR 701.28).
pub fn prowl_stoic_strategist() -> CardDefinition {
    let played = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::AnyPlayer).with_filter(Predicate::TriggerCardExiledWithSource),
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
            Effect::Transform { what: Selector::This },
        ]),
    };
    legend(CardDefinition {
        name: "Prowl, Stoic Strategist",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 3,
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), w()]),
            converted: true,
            ..Default::default()
        }),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::ExileWithSource {
                            what: target_filtered(
                                R::Creature
                                    .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                                    .and(R::Tapped)
                                    .and(R::OtherThanSource),
                            ),
                        },
                        Effect::GrantMayPlay {
                            what: Selector::CardExiledWithSource,
                            duration: MayPlayDuration::WhileExiled,
                            to_owner: true,
                            exile_after: false,
                            pay_own_cost: true,
                            any_color: false,
                        },
                    ])),
                },
            },
            played(EventKind::SpellCast),
            played(EventKind::LandPlayed),
        ],
        back_face: Some(Box::new(prowl_pursuit_vehicle())),
        ..Default::default()
    })
}

/// Codie, Ravenous Codex — {3} legendary 1/4 artifact creature Book
/// Construct. Casting a prepared spell copies it (new targets allowed).
/// {W}{U}{B}{R}{G}, {T}: each creature you control with a prepare spell
/// becomes prepared.
pub fn codie_ravenous_codex() -> CardDefinition {
    legend(CardDefinition {
        name: "Codie, Ravenous Codex",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Book, CreatureType::Construct], ..Default::default() },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsPreparedSpell,
            }),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasPrepareSpell)),
                kind: crate::card::CounterType::Prepared,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    })
}
