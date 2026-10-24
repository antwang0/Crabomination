//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), fifteenth file. Tests in `tests/recent_b/cmdr_rank3k_o.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    spell(name, mana, CardType::Artifact, Effect::Noop)
}

// ── Jadzi, Steward of Fate ───────────────────────────────────────────────────

/// Rimefire Torque — {1}{U} artifact. As it enters, choose a creature type.
/// Whenever a permanent you control of the chosen type enters, a charge
/// counter on it. {T}, remove three charge counters: when you next cast an
/// instant or sorcery spell this turn, copy it (you may choose new targets).
pub fn rimefire_torque() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType {
            what: Selector::This,
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsSourceChosenCreatureType,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 3)),
            effect: Effect::CopyNextInstantSorceryThisTurn { count: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Rimefire Torque", cost(&[generic(1), u()]))
    }
}

// ── Kentaro, the Smiling Cat ─────────────────────────────────────────────────

/// Gathering Stone — {4} artifact. As it enters, choose a creature type. Spells
/// of the chosen type you cast cost {1} less. When it enters and at the
/// beginning of your upkeep, look at the top card of your library: a card of
/// the chosen type you may put into your hand; otherwise you may put it into
/// your graveyard.
pub fn gathering_stone() -> CardDefinition {
    let top = || Selector::TopOfLibrary {
        who: PlayerRef::You,
        count: Value::ONE,
    };
    let mill = || Effect::MayDo {
        description: "Put the top card of your library into your graveyard?".into(),
        body: Box::new(Effect::Move {
            what: top(),
            to: ZoneDest::Graveyard,
        }),
    };
    let look = || Effect::If {
        cond: Predicate::EntityMatches {
            what: top(),
            filter: R::IsSourceChosenCreatureType,
        },
        then: Box::new(Effect::MayDoElse {
            description: "Reveal the top card of your library and put it into your hand?".into(),
            body: Box::new(Effect::Move {
                what: top(),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(mill()),
        }),
        else_: Box::new(mill()),
    };
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType {
            what: Selector::This,
        }),
        static_abilities: vec![StaticAbility {
            description: "Spells you cast of the chosen type cost {1} less to cast.",
            effect: StaticEffect::ChosenTypeSpellCostReduction { amount: 1 },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: look(),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: look(),
            },
        ],
        ..artifact("Gathering Stone", cost(&[generic(4)]))
    }
}

/// Eiganjo Exemplar — {1}{W} 2/1 Human Samurai enchantment creature. Whenever
/// a Samurai or Warrior you control attacks alone, it gets +1/+1 until end of
/// turn.
pub fn eiganjo_exemplar() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsAttackingAlone.and(
                        R::HasCreatureType(CreatureType::Samurai)
                            .or(R::HasCreatureType(CreatureType::Warrior)),
                    ),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Eiganjo Exemplar",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            2,
            1,
        )
    }
}

// ── Lady Loki, Agent of Chaos ────────────────────────────────────────────────

/// Loki Laufeyson — {1}{R} legendary 2/1 God Sorcerer Villain. {1}, {T}: when
/// you next cast an instant or sorcery spell with mana value at most Loki's
/// power this turn, copy it (you may choose new targets). Power-up {4}{R}:
/// two +1/+1 counters. (Residual: a bigger instant or sorcery spends the
/// watch without a copy.)
pub fn loki_laufeyson() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::OnYourNextInstantSorceryThisTurn {
                    body: Box::new(Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::PowerOf(Box::new(Selector::This)),
                            Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        ),
                        then: Box::new(Effect::CopySpellMayChooseTargets {
                            what: Selector::TriggerSource,
                            count: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), r()]),
                power_up: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Loki Laufeyson",
            cost(&[generic(1), r()]),
            vec![
                CreatureType::God,
                CreatureType::Sorcerer,
                CreatureType::Villain,
            ],
            2,
            1,
        )
    }
}

/// Wanda's Vision — {3}{R}{R} enchantment. Whenever you cast your second spell
/// each turn, exile cards from the top of your library until you exile a
/// nonland card; you may cast it without paying its mana cost.
pub fn wandas_vision() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                },
            ),
            // The free cast is during the resolution (CR 608.2g), not a
            // turn-long permission.
            effect: Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::You },
                Effect::CastWithoutPayingImmediate {
                    what: Selector::one_of(Selector::ExiledThisResolution { filter: R::Nonland }),
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            ]),
        }],
        ..spell(
            "Wanda's Vision",
            cost(&[generic(3), r(), r()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Visara the Dreadful ──────────────────────────────────────────────────────

/// Sengir, the Dark Baron — {4}{B}{B} legendary 4/4 flying Vampire Noble;
/// partner. Whenever another creature dies, two +1/+1 counters on it. Whenever
/// another player loses the game, you gain life equal to that player's life
/// total as the turn began.
pub fn sengir_the_dark_baron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Partner],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::IsSource,
                    })),
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerLeftGame, EventScope::AnyPlayer),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::LifeAtTurnStart(PlayerRef::TriggerEventPlayer),
                },
            },
        ],
        ..legend(
            "Sengir, the Dark Baron",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Noble],
            4,
            4,
        )
    }
}
