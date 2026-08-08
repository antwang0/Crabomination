//! A cross-set wave (BLB / DSK / OTJ) of catalog gaps riding conditional-static
//! and life-matters primitives — including the new
//! `Predicate::PlayerGainedLifeThisTurn`. Tests in
//! `crabomination/src/tests/recent158.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, w};

/// Starlit Soothsayer — {2}{B} 2/2 Bat Cleric with flying. At the beginning of
/// your end step, if you gained or lost life this turn, surveil 1.
pub fn starlit_soothsayer() -> CardDefinition {
    CardDefinition {
        name: "Starlit Soothsayer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::Any(vec![
                    Predicate::PlayerGainedLifeThisTurn {
                        who: PlayerRef::You,
                    },
                    Predicate::PlayerLostLifeThisTurn {
                        who: PlayerRef::You,
                    },
                ]),
                then: Box::new(Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Omenport Vigilante — {1}{W} 2/2 Human Mercenary. Has double strike as long as
/// you've committed a crime this turn.
pub fn omenport_vigilante() -> CardDefinition {
    CardDefinition {
        name: "Omenport Vigilante",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Double strike as long as you've committed a crime this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::CommittedCrimeThisTurn {
                    who: PlayerRef::You,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::DoubleStrike],
            },
        }],
        ..Default::default()
    }
}

/// Essence Channeler — {1}{W} 2/1 Bat Cleric. Has flying and vigilance as long
/// as you've lost life this turn. Whenever you gain life, put a +1/+1 counter on
/// it. When it dies, put its counters on target creature you control.
pub fn essence_channeler() -> CardDefinition {
    CardDefinition {
        name: "Essence Channeler",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Flying and vigilance as long as you've lost life this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::PlayerLostLifeThisTurn {
                    who: PlayerRef::You,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying, Keyword::Vigilance],
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            on_dies(Effect::MoveAllCounters {
                from: Selector::This,
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
            }),
        ],
        ..Default::default()
    }
}

/// Cactarantula — {4}{G}{G} 6/5 Plant Spider with reach. Costs {1} less if you
/// control a Desert. Whenever it becomes the target of a spell or ability an
/// opponent controls, you may draw a card.
pub fn cactarantula() -> CardDefinition {
    CardDefinition {
        name: "Cactarantula",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Spider],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Costs {1} less to cast if you control a Desert.",
            effect: StaticEffect::SelfCostReducedIfControlEach {
                filters: vec![R::HasLandType(LandType::Desert)],
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                actor_is_opponent: true,
                ..EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource)
            },
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Inventive Wingsmith — {2}{W} 2/4 Dwarf Artificer. At the beginning of your
/// end step, if you haven't cast a spell this turn and it isn't already flying
/// (its flying counter), put a flying counter on it.
pub fn inventive_wingsmith() -> CardDefinition {
    CardDefinition {
        name: "Inventive Wingsmith",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::All(vec![
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(0),
                },
                Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::HasKeyword(Keyword::Flying).negate(),
                },
            ])),
            effect: Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Flying,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Mourner's Surprise — {1}{B} Sorcery. Return up to one target creature card
/// from your graveyard to your hand, then create a 1/1 red Mercenary token with
/// "{T}: Target creature you control gets +1/+0 until end of turn. Activate only
/// as a sorcery."
pub fn mourners_surprise() -> CardDefinition {
    let mercenary = || TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Mourner's Surprise",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(mercenary()),
            },
        ]),
        ..Default::default()
    }
}
