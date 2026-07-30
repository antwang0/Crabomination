//! MKM (Murders at Karlov Manor) Case enchantments. A Case's printed first line
//! is its always-on ability; `CaseData.to_solve` is checked at the beginning of
//! the controller's end step, and `solved_*` switch on once solved. Tests in
//! `tests/recent_b/recent242.rs`.

use crate::card::{
    CardDefinition, CardType, CaseData, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{etb, investigate, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u, w};

/// Convenience: the enchantment-subtype block marking a card a Case.
fn case_subtypes() -> Subtypes {
    Subtypes {
        enchantment_subtypes: vec![EnchantmentSubtype::Case],
        ..Default::default()
    }
}

/// "You control `n` or more permanents matching `req`."
fn control_at_least(req: R, n: i32) -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(req.and(R::ControlledByYou)),
        n: Value::Const(n),
    }
}

/// Case of the Shattered Pact — {2} Enchantment — Case. ETB: fetch a basic land
/// to hand. Solve: five colors among permanents you control. Solved: at combat,
/// a creature you control gains flying, double strike, and vigilance.
pub fn case_of_the_shattered_pact() -> CardDefinition {
    CardDefinition {
        name: "Case of the Shattered Pact",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        case: Some(Box::new(CaseData {
            to_solve: Predicate::ValueAtLeast(
                Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou))),
                Value::Const(5),
            ),
            solved_triggered: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::YourControl,
                ),
                effect: Effect::GrantKeywords {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keywords: vec![Keyword::Flying, Keyword::DoubleStrike, Keyword::Vigilance],
                    duration: Duration::EndOfTurn,
                },
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Trampled Garden — {2}{G} Enchantment — Case. ETB: distribute two
/// +1/+1 counters among one or two creatures you control. Solve: creatures you
/// control have total power 8 or greater. Solved: whenever you attack, put a
/// +1/+1 counter on target attacking creature and it gains trample.
pub fn case_of_the_trampled_garden() -> CardDefinition {
    CardDefinition {
        name: "Case of the Trampled Garden",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::Const(2),
            counter: CounterType::PlusOnePlusOne,
            filter: R::Creature.and(R::ControlledByYou),
            max_targets: 2,
        })],
        case: Some(Box::new(CaseData {
            to_solve: Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8)),
            solved_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::IsAttacking),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Crimson Pulse — {2}{R} Enchantment — Case. ETB: discard a card,
/// then draw two. Solve: you have no cards in hand. Solved: at your upkeep,
/// discard your hand, then draw two.
pub fn case_of_the_crimson_pulse() -> CardDefinition {
    let discard_hand_draw_two = || {
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::HandSizeOf(PlayerRef::You),
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ])
    };
    CardDefinition {
        name: "Case of the Crimson Pulse",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
        case: Some(Box::new(CaseData {
            to_solve: Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::You), Value::Const(0)),
            solved_triggered: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: discard_hand_draw_two(),
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Filched Falcon — {U} Enchantment — Case. ETB: investigate.
/// Solve: you control three or more artifacts. Solved: {2}{U}, Sacrifice this
/// Case: put four +1/+1 counters on target noncreature artifact; it becomes a
/// 0/0 Bird creature with flying in addition to its other types.
pub fn case_of_the_filched_falcon() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Case of the Filched Falcon",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![etb(investigate(1))],
        case: Some(Box::new(CaseData {
            to_solve: control_at_least(R::Artifact, 3),
            solved_activated: vec![ActivatedAbility {
                sac_cost: true,
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Artifact.and(R::Noncreature)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(4),
                    },
                    Effect::BecomeCreature {
                        what: Selector::Target(0),
                        power: Value::Const(0),
                        toughness: Value::Const(0),
                        creature_types: vec![CreatureType::Bird],
                        keywords: vec![Keyword::Flying],
                        duration: Duration::Permanent,
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Uneaten Feast — {W} Enchantment — Case. Whenever a creature you
/// control enters, gain 1 life. Solve: you've gained 5+ life this turn. Solved:
/// Sacrifice this Case: creature cards in your graveyard gain "you may cast this
/// card from your graveyard" until end of turn (modeled as a flashback grant).
pub fn case_of_the_uneaten_feast() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Case of the Uneaten Feast",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        case: Some(Box::new(CaseData {
            to_solve: Predicate::LifeGainedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(5),
            },
            solved_activated: vec![ActivatedAbility {
                sac_cost: true,
                effect: Effect::GrantFlashbackThisTurn {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    },
                },
                ..Default::default()
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Locked Hothouse — {3}{G} Enchantment — Case. You may play an
/// additional land each turn. Solve: you control seven or more lands. Solved:
/// look at the top card any time, and play lands / cast creature and enchantment
/// spells from the top of your library.
pub fn case_of_the_locked_hothouse() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Case of the Locked Hothouse",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        case: Some(Box::new(CaseData {
            to_solve: control_at_least(R::Land, 7),
            solved_static: vec![
                StaticAbility {
                    description: "Look at the top card of your library any time.",
                    effect: StaticEffect::TopOfLibraryRevealed,
                },
                StaticAbility {
                    description: "You may play lands and cast creature and enchantment spells from the top of your library.",
                    effect: StaticEffect::PlayFromLibraryTop {
                        filter: R::Land.or(R::Creature).or(R::Enchantment),
                    },
                },
            ],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Gateway Express — {1}{W} Enchantment — Case. ETB: choose target
/// creature you don't control; each creature you control deals 1 damage to it.
/// Solve: three or more creatures attacked this turn. Solved: creatures you
/// control get +1/+0.
pub fn case_of_the_gateway_express() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Case of the Gateway Express",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![etb(Effect::EachControlledCreatureDealsDamage {
            to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            amount: Value::ONE,
        })],
        case: Some(Box::new(CaseData {
            to_solve: Predicate::ValueAtLeast(
                Value::CreaturesAttackedWithThisTurn(PlayerRef::You),
                Value::Const(3),
            ),
            solved_static: vec![StaticAbility {
                description: "Creatures you control get +1/+0.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: 1,
                    toughness: 0,
                },
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case File Auditor — {2}{W} Creature — Human Detective 1/4. On ETB and
/// whenever you solve a Case, look at the top six cards; you may reveal an
/// enchantment card and put it into your hand, rest to the bottom at random.
pub fn case_file_auditor() -> CardDefinition {
    let look_six = || Effect::LookPickToHand {
        who: PlayerRef::You,
        count: Value::Const(6),
        rest_to_graveyard: false,
        pick_filter: Some(R::Enchantment),
        take: None,
        to_battlefield: false,
        gain_life_if_pick: None,
        gain_life_greatest_power_rest: false,
        optional: true,
        picked_lands_to_battlefield: false,
        rest_bottom_random: true,
    };
    CardDefinition {
        name: "Case File Auditor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            etb(look_six()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CaseSolved, EventScope::YourControl),
                effect: look_six(),
            },
        ],
        ..Default::default()
    }
}
