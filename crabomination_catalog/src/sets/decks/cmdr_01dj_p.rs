//! Commander: 2026-10-19 (`01DJHkWk`) — single-card closers: Keldon
//! Warcaller, Mind Transfer Protocol, Donatello's Science Lesson, Agents of
//! S.H.I.E.L.D., Luke Cage, Fire of Orthanc; the attacks-alone pair
//! (Strategic Intervention, Crowd of True Believers) and Widow's Bite.
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, ModesWiden, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, Predicate, Selector, Value};
use crate::mana::{ManaCost, b, cost, generic, r, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

/// Keldon Warcaller — {1}{R} 2/2 Human Warrior. Whenever it attacks, put a
/// lore counter on target Saga you control (CR 714.2b: the chapter fires).
pub fn keldon_warcaller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    R::HasEnchantmentSubtype(EnchantmentSubtype::Saga).and(R::ControlledByYou),
                ),
                kind: CounterType::Lore,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Keldon Warcaller",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}


/// Mind Transfer Protocol — {2}{U} instant. Until end of turn, target
/// artifact or creature becomes an artifact creature with base power and
/// toughness 4/5. Draw a card.
pub fn mind_transfer_protocol() -> CardDefinition {
    spell(
        "Mind Transfer Protocol",
        cost(&[generic(2), u()]),
        true,
        Effect::Seq(vec![
            Effect::BecomeCreature {
                what: target_filtered(R::Artifact.or(R::Creature)),
                power: Value::Const(4),
                toughness: Value::Const(5),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            Effect::AddCardTypeIndefinitely {
                what: Selector::Target(0),
                card_type: CardType::Artifact,
                until_eot: true,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Donatello's Science Lesson — {2}{U} instant. Tap up to two target
/// creatures. Up to two target players each draw a card.
pub fn donatellos_science_lesson() -> CardDefinition {
    let tap = |slot: u8| Effect::Tap {
        what: Selector::TargetFiltered { slot, filter: R::Creature },
    };
    let draw = |slot: u8| Effect::Draw {
        who: Selector::TargetFiltered { slot, filter: R::Player },
        amount: Value::ONE,
    };
    spell(
        "Donatello's Science Lesson",
        cost(&[generic(2), u()]),
        true,
        Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![tap(0), tap(1), draw(2), draw(3)])),
        },
    )
}

/// Agents of S.H.I.E.L.D. — {2}{W} 2/4 Human Spy Hero. Whenever a creature
/// you control attacks alone, it gets +1/+1 until end of turn.
pub fn agents_of_shield() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::AttackingAlone),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Agents of S.H.I.E.L.D.",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            2,
            4,
        )
    }
}

/// Luke Cage, Power Man — {3}{W} legendary 2/5 Human Hero. Whenever he
/// attacks alone, he gets +2/+0 and gains indestructible until end of turn.
pub fn luke_cage_power_man() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackingAlone),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Luke Cage, Power Man",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Hero],
            2,
            5,
        )
    }
}

/// Fire of Orthanc — {3}{R} sorcery. Destroy target artifact or land.
/// Creatures without flying can't block this turn.
pub fn fire_of_orthanc() -> CardDefinition {
    spell(
        "Fire of Orthanc",
        cost(&[generic(3), r()]),
        false,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Land)),
            },
            Effect::GrantKeywordToMatchingThisTurn {
                filter: R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                keyword: Keyword::CantBlock,
            },
        ]),
    )
}


/// Strategic Intervention — {1}{W} enchantment. Whenever a creature you
/// control attacks alone, it gets +1/+1 until end of turn. Tap up to one
/// target creature defending player controls.
pub fn strategic_intervention() -> CardDefinition {
    CardDefinition {
        name: "Strategic Intervention",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::AttackingAlone),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Tap {
                        what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Crowd of True Believers — {W} 1/2 Human Citizen. {T}: target creature you
/// control that's attacking alone gets +1/+0 until end of turn. You gain 1
/// life.
pub fn crowd_of_true_believers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::IsAttackingAlone),
                    ),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Crowd of True Believers",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            1,
            2,
        )
    }
}

/// Widow's Bite — {1}{B} instant, teamwork 3. Choose one — both if cast
/// using teamwork: target creature gains deathtouch until end of turn; target
/// creature gets -2/-2 until end of turn.
pub fn widows_bite() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower {
            filter: R::Creature.and(R::ControlledByYou),
            power: 3,
        }),
        modes_widen: Some(ModesWiden::Kicked),
        ..spell(
            "Widow's Bite",
            cost(&[generic(1), b()]),
            true,
            Effect::ChooseModesCast {
                modes: vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Deathtouch,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::Const(-2),
                        toughness: Value::Const(-2),
                        duration: Duration::EndOfTurn,
                    },
                ],
                min: 1,
                max: 1,
                allow_repeats: false,
            },
        )
    }
}
