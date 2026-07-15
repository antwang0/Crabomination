//! Artifact-matters: Thopter/Servo makers, Affinity beaters, and Saheeli.
//! Tests in `tests/recent55.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_noncreature, etb};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{cost, generic, hybrid, r, u, w, Color, ManaCost, ManaSymbol};

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    }
}

fn servo_token() -> TokenDefinition {
    TokenDefinition {
        name: "Servo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Servo], ..Default::default() },
        ..Default::default()
    }
}

/// Thopter Engineer — {2}{R} 1/3 Human Artificer. ETB create a 1/1 flying
/// Thopter; artifact creatures you control have haste.
pub fn thopter_engineer() -> CardDefinition {
    CardDefinition {
        name: "Thopter Engineer",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: thopter_token(),
        })],
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Artifact.and(R::Creature).and(R::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        ..Default::default()
    }
}

/// Maverick Thopterist — {3}{U}{R} 2/2 Human Artificer with Improvise. ETB
/// create two 1/1 flying Thopters.
pub fn maverick_thopterist() -> CardDefinition {
    CardDefinition {
        name: "Maverick Thopterist",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Improvise],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: thopter_token(),
        })],
        ..Default::default()
    }
}

/// Ingenious Smith — {1}{W} 1/1 Human Artificer. ETB look at the top four cards
/// and may put an artifact from among them into your hand. Whenever an artifact
/// you control enters, put a +1/+1 counter on this creature.
pub fn ingenious_smith() -> CardDefinition {
    CardDefinition {
        name: "Ingenious Smith",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: false,
                pick_filter: Some(R::Artifact),
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: true,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Ravenous Intruder — {1}{R} 1/2 Gremlin. Sacrifice an artifact: this creature
/// gets +2/+2 until end of turn.
pub fn ravenous_intruder() -> CardDefinition {
    CardDefinition {
        name: "Ravenous Intruder",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gremlin], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Saheeli, Sublime Artificer — {1}{U/R}{U/R} Legendary Planeswalker — Saheeli.
/// 5 loyalty. Whenever you cast a noncreature spell, create a 1/1 Servo. −2:
/// target artifact you control becomes a copy of another target artifact or
/// creature you control until end of turn.
pub fn saheeli_sublime_artificer() -> CardDefinition {
    CardDefinition {
        name: "Saheeli, Sublime Artificer",
        cost: ManaCost::new(vec![
            ManaSymbol::Generic(1),
            hybrid(Color::Blue, Color::Red),
            hybrid(Color::Blue, Color::Red),
        ]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Saheeli],
            ..Default::default()
        },
        base_loyalty: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: servo_token(),
            },
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: Effect::BecomeCopyOfFor {
                what: Selector::TargetFiltered { slot: 0, filter: R::Artifact.and(R::ControlledByYou) },
                source: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.or(R::Artifact).and(R::ControlledByYou),
                },
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
