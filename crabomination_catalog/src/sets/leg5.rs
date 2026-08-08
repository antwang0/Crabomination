//! Legends (LEG) wave 6 — the block-and-kill creatures, the prevention
//! bodies and the set's remaining legends, artifacts and spells. Tests in
//! `classic_sets/leg5`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{gain_life, target, target_any, target_filtered, you},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, c, types, p, t) }
}

fn upkeep(scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), scope)
}

/// "Whenever this blocks or becomes blocked by a [filter] creature, destroy
/// that creature at end of combat" — the Legends fight-and-kill shape.
fn kills_what_it_meets(filter: R) -> Vec<TriggeredAbility> {
    let body = || Effect::AtEndOfCombat {
        body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
    };
    [EventKind::Blocks, EventKind::BecomesBlocked]
        .into_iter()
        .map(|kind| TriggeredAbility {
            event: EventSpec::new(kind, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: filter.clone() },
            ),
            effect: body(),
        })
        .collect()
}

/// Abomination — anything green or white it meets in combat dies.
pub fn abomination() -> CardDefinition {
    CardDefinition {
        triggered_abilities: kills_what_it_meets(
            R::HasColor(Color::Green).or(R::HasColor(Color::White)),
        ),
        ..creature("Abomination", cost(&[generic(3), b(), b()]), vec![CreatureType::Horror], 2, 6)
    }
}

/// Infernal Medusa — the same, but Walls survive blocking it.
pub fn infernal_medusa() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Wall),
                    }))),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                },
            },
        ],
        ..creature("Infernal Medusa", cost(&[generic(3), b(), b()]), vec![CreatureType::Gorgon], 2, 4)
    }
}

/// Aisling Leprechaun — everything it meets turns green, for good.
pub fn aisling_leprechaun() -> CardDefinition {
    CardDefinition {
        triggered_abilities: [EventKind::Blocks, EventKind::BecomesBlocked]
            .into_iter()
            .map(|kind| TriggeredAbility {
                event: EventSpec::new(kind, EventScope::SelfSource),
                effect: Effect::BecomeColor {
                    what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                    colors: vec![Color::Green],
                    duration: Duration::Permanent,
                    additive: false,
                },
            })
            .collect(),
        ..creature("Aisling Leprechaun", cost(&[g()]), vec![CreatureType::Faerie], 1, 1)
    }
}

// ── Prevention bodies ──────────────────────────────────────────────────────

/// Enchanted Being — enchanted creatures can't hurt it.
pub fn enchanted_being() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to this creature by \
                          enchanted creatures.",
            effect: StaticEffect::PreventCombatDamageToThisFromMatching { filter: R::IsEnchanted },
        }],
        ..creature("Enchanted Being", cost(&[generic(1), w(), w()]), vec![CreatureType::Human], 2, 2)
    }
}

/// Marble Priest — every Wall that can block it must, and none of them can
/// hurt it.
pub fn marble_priest() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::AllMustBlock],
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to this creature by \
                          Walls.",
            effect: StaticEffect::PreventCombatDamageToThisFromMatching {
                filter: R::HasCreatureType(CreatureType::Wall),
            },
        }],
        ..creature("Marble Priest", cost(&[generic(5)]), vec![CreatureType::Cleric], 3, 3)
    }
}

/// Clergy of the Holy Nimbus — free regeneration, which your opponents can
/// switch off for {1}. (The printed replacement is modelled as a `{0}`
/// regenerate the controller activates.)
pub fn clergy_of_the_holy_nimbus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
            ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            opponents_only: true,
            effect: Effect::CantBeRegeneratedThisTurn { what: Selector::This },
            ..Default::default()
            },
        ],
        ..creature(
            "Clergy of the Holy Nimbus",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Elder Spawn — feed it an Island every upkeep or it takes you with it.
pub fn elder_spawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::Not(Box::new(R::HasColor(
            Color::Red,
        )))))],
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::MayDoElse {
                description: "Sacrifice an Island to keep Elder Spawn?".into(),
                body: Box::new(Effect::Sacrifice {
                    who: you(),
                    count: Value::ONE,
                    filter: R::Land.and(R::HasLandType(LandType::Island)),
                }),
                else_: Box::new(Effect::Seq(vec![
                    Effect::SacrificePermanent { what: Selector::This },
                    Effect::DealDamage { to: you(), amount: Value::Const(6) },
                ])),
            },
        }],
        ..creature("Elder Spawn", cost(&[generic(4), u(), u(), u()]), vec![CreatureType::Horror], 6, 6)
    }
}

/// Firestorm Phoenix — it goes to hand instead of dying. (The printed
/// reveal-and-can't-play rider isn't modelled.)
pub fn firestorm_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If this creature would die, return it to its owner's hand instead.",
            effect: StaticEffect::DiesToOwnersHandInstead {
                filter: R::HasName("Firestorm Phoenix".into()),
            },
        }],
        ..creature(
            "Firestorm Phoenix",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Phoenix],
            3,
            2,
        )
    }
}

/// The Wretched — everything that blocked it changes sides.
pub fn the_wretched() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::GainControlWhileSourceRemains {
                what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
            },
        }],
        ..creature("The Wretched", cost(&[generic(3), b(), b()]), vec![CreatureType::Demon], 2, 5)
    }
}

/// Lesser Werewolf — trade its own power for -0/-1 counters on its dance
/// partner.
pub fn lesser_werewolf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            condition: Some(Predicate::ValueAtLeast(
                Value::PowerOf(Box::new(Selector::This)),
                Value::ONE,
            )),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(-1),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::InCombatWithSource)),
                    kind: CounterType::MinusZeroMinusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Lesser Werewolf", cost(&[generic(3), b()]), vec![CreatureType::Werewolf], 2, 4)
    }
}

/// Axelrod Gunnarson — every creature he finishes off pays him.
pub fn axelrod_gunnarson() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::Seq(vec![
                gain_life(1),
                Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ]),
        }],
        ..legend(
            "Axelrod Gunnarson",
            cost(&[generic(4), b(), b(), r(), r()]),
            vec![CreatureType::Giant],
            5,
            5,
        )
    }
}

/// Rohgahh of Kher Keep — a Kobold lord you have to keep paying.
pub fn rohgahh_of_kher_keep() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control named Kobolds of Kher Keep get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasName("Kobolds of Kher Keep".into())),
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                all_players: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {R}{R}{R} to keep your Kobolds?".into(),
                mana_cost: cost(&[r(), r(), r()]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Seq(vec![
                    Effect::Tap {
                        what: Selector::EachPermanent(
                            R::ControlledByYou.and(
                                R::IsSource.or(R::HasName("Kobolds of Kher Keep".into())),
                            ),
                        ),
                    },
                    Effect::GainControl {
                        what: Selector::EachPermanent(
                            R::ControlledByYou.and(
                                R::IsSource.or(R::HasName("Kobolds of Kher Keep".into())),
                            ),
                        ),
                        to: Some(PlayerRef::EachOpponent),
                        duration: Duration::Permanent,
                    },
                ]))),
            },
        }],
        ..legend(
            "Rohgahh of Kher Keep",
            cost(&[generic(2), b(), b(), r(), r()]),
            vec![CreatureType::Kobold],
            5,
            5,
        )
    }
}

/// Stangg — arrives with a twin, and the two live and die together.
pub fn stangg() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Stangg Twin".into(),
                        power: 3,
                        toughness: 4,
                        colors: vec![Color::Red, Color::Green],
                        supertypes: vec![Supertype::Legendary],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Exile { what: Selector::TokensCreatedBySource },
            },
        ],
        ..legend(
            "Stangg",
            cost(&[generic(4), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            4,
        )
    }
}

/// Wood Elemental — as big as the Forests you feed it.
pub fn wood_elemental() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Land.and(R::HasLandType(LandType::Forest)).and(R::Untapped),
            per_each: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        }),
        ..creature("Wood Elemental", cost(&[generic(3), g()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Primordial Ooze — it grows every upkeep, and burns you if you stop paying.
pub fn primordial_ooze() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttack],
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::UnlessPlayerPays {
                    who: PlayerRef::You,
                    cost: crate::card::WardCost::GenericCountersOnSource(
                        CounterType::PlusOnePlusOne,
                    ),
                    if_paid: None,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::This },
                        Effect::DealDamage {
                            to: you(),
                            amount: Value::CountersOn {
                                what: Box::new(Selector::This),
                                kind: CounterType::PlusOnePlusOne,
                            },
                        },
                    ])),
                },
            ]),
        }],
        ..creature("Primordial Ooze", cost(&[r()]), vec![CreatureType::Ooze], 1, 1)
    }
}

// ── Lands and artifacts ────────────────────────────────────────────────────

/// Urborg — black mana, or strip one evasion keyword.
pub fn urborg() -> CardDefinition {
    CardDefinition {
        name: "Urborg",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Black]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::ChooseMode(vec![
                    Effect::LoseKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::LoseKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Landwalk(LandType::Swamp),
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Alchor's Tomb — repaint one of your permanents, indefinitely.
pub fn alchors_tomb() -> CardDefinition {
    CardDefinition {
        name: "Alchor's Tomb",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeChosenColor {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Divine Intervention — two upkeeps, then nobody wins.
pub fn divine_intervention() -> CardDefinition {
    CardDefinition {
        name: "Divine Intervention",
        cost: cost(&[generic(6), w(), w()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Intervention, Value::Const(2))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: upkeep(EventScope::YourControl),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Intervention,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterRemoved(CounterType::Intervention),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::ValueAtMost(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Intervention,
                    },
                    Value::ZERO,
                )),
                effect: Effect::GameIsADraw,
            },
        ],
        ..Default::default()
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Alabaster Potion — X life, or X damage stopped.
pub fn alabaster_potion() -> CardDefinition {
    CardDefinition {
        name: "Alabaster Potion",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::XFromCost,
            },
            Effect::PreventNextDamage { target: target_any(), amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}

/// Rapid Fire — first strike now, and rampage if it hasn't got any.
pub fn rapid_fire() -> CardDefinition {
    CardDefinition {
        name: "Rapid Fire",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        cast_only_before_blockers: true,
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: target(),
                    filter: R::HasKeyword(Keyword::Rampage(2)),
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::GrantKeyword {
                    what: target(),
                    keyword: Keyword::Rampage(2),
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Recall — trade X cards from hand for X out of your graveyard.
pub fn recall() -> CardDefinition {
    CardDefinition {
        name: "Recall",
        cost: cost(&[x(), x(), u()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::Discard { who: you(), amount: Value::XFromCost, random: false },
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Any,
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}
