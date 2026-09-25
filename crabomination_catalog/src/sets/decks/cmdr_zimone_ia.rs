//! Commander: the cards the **Quandrix Unlimited** precon (SOC, Zimone,
//! Infinite Analyst) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_zimone_ia.rs`.
//!
//! Residuals (each also on its card):
//! - **Kinetic Ooze** — at X 10 or more it doubles the counters on each
//!   other creature you control rather than on the targets you choose.
//! - **Primo, the Unbounded** — the Fractal's counters are the batch's first
//!   damage event's amount when several base-power-0 creatures connect.
//! - **Unbound Flourishing** — activated abilities with {X} aren't copied.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EntersAsCopy, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, u, x, Color, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn p1p1(what: Selector, amount: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount }
}

fn x_counters() -> Option<(CounterType, Value)> {
    Some((CounterType::PlusOnePlusOne, Value::XFromCost))
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// "Whenever you cast your first spell with {X} in its mana cost each turn".
fn first_x_spell(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellFirstMatchingThisTurn(R::HasXInCost)),
        effect,
    }
}

fn fractal() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Fractal".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fractal], ..Default::default() },
        ..Default::default()
    })
}

/// A 0/0 Fractal with `counters` +1/+1 counters.
fn make_fractal(counters: Value) -> Effect {
    Effect::Seq(vec![
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: fractal() },
        p1p1(Selector::LastCreatedToken, counters),
    ])
}

fn becomes_prepared() -> Effect {
    Effect::AddCounterCapped { what: Selector::This, kind: CounterType::Prepared, amount: Value::ONE, cap: Value::ONE }
}

/// Zimone, Infinite Analyst — your first {X} spell each turn costs {1} less
/// per +1/+1 counter on her, and puts two on her.
pub fn zimone_infinite_analyst() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "The first spell you cast with {X} in its mana cost each turn costs {1} less to cast for \
                          each +1/+1 counter on Zimone.",
            effect: StaticEffect::FirstMatchingSpellEachTurnCostsLessPerCounter {
                filter: R::HasXInCost,
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        triggered_abilities: vec![first_x_spell(p1p1(Selector::This, Value::Const(2)))],
        ..legend(
            "Zimone, Infinite Analyst",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            4,
        )
    }
}

/// Alchemist's Refuge — {C}; {G}{U}, {T}: cast spells this turn as though
/// they had flash.
pub fn alchemists_refuge() -> CardDefinition {
    CardDefinition {
        name: "Alchemist's Refuge",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[g(), u()]),
                tap_cost: true,
                effect: Effect::GrantSpellsFlashThisTurn { who: PlayerRef::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Altered Ego — can't be countered; may enter as a copy of any creature
/// with X additional +1/+1 counters.
pub fn altered_ego() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        enters_with_counters: x_counters(),
        // The X counters carry through the copy (`enters_with_counters` is
        // read off the entering card, not the copied one).
        enters_as_copy: Some(EntersAsCopy { filter: R::Creature, ..Default::default() }),
        ..creature("Altered Ego", cost(&[x(), generic(2), g(), u()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

/// Benevolent Hydra — X counters; +1/+1 counters on your other creatures get
/// one more; tap and remove a counter to give one to another creature.
pub fn benevolent_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: x_counters(),
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on another creature you control, that many \
                          plus one +1/+1 counters are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCountersMatching { filter: R::Creature.and(R::OtherThanSource) },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: p1p1(target_filtered(yours().and(R::OtherThanSource)), Value::ONE),
            ..Default::default()
        }],
        ..creature("Benevolent Hydra", cost(&[x(), g(), g()]), vec![CreatureType::Hydra], 1, 1)
    }
}

/// Brass Infiniscope — {T}: add {C}{C}; your next {X} spell this turn draws
/// a card and gains half X life.
pub fn brass_infiniscope() -> CardDefinition {
    CardDefinition {
        name: "Brass Infiniscope",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(2)) },
                Effect::OnYourNextSpellMatchingThisTurn {
                    filter: R::HasXInCost,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::GainLife { who: Selector::You, amount: Value::HalfDown(Box::new(Value::XFromCost)) },
                    ])),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Entrancing Melody — gain control of a creature with mana value X.
pub fn entrancing_melody() -> CardDefinition {
    spell(
        "Entrancing Melody",
        cost(&[x(), u(), u()]),
        CardType::Sorcery,
        Effect::GainControl {
            what: target_filtered(R::Creature.and(R::ManaValueExactlyXFromCost)),
            to: None,
            duration: Duration::Permanent,
        },
    )
}

/// Expansion Algorithm — proliferate X times.
pub fn expansion_algorithm() -> CardDefinition {
    spell(
        "Expansion Algorithm",
        cost(&[x(), u(), u()]),
        CardType::Sorcery,
        Effect::Repeat { count: Value::XFromCost, body: Box::new(Effect::Proliferate) },
    )
}

/// Kinetic Ooze — X counters; on entering, destroys an artifact or
/// enchantment with mana value X or less, draws at X 5+, doubles counters
/// at X 10+.
///
/// Residual: the doubling hits each other creature you control rather than
/// targets you choose.
pub fn kinetic_ooze() -> CardDefinition {
    CardDefinition {
        enters_with_counters: x_counters(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Artifact.or(R::Enchantment).and(R::ManaValueAtMostXFromCost),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(10)),
                then: Box::new(Effect::DoubleCountersOnEach {
                    what: Selector::EachPermanent(yours().and(R::OtherThanSource)),
                    kind: CounterType::PlusOnePlusOne,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Kinetic Ooze", cost(&[x(), g()]), vec![CreatureType::Ooze], 0, 0)
    }
}

/// Lattice Library — X study counters; entering and your first {X} spell
/// each turn make a Fractal with a counter per study counter.
pub fn lattice_library() -> CardDefinition {
    let fractal = || make_fractal(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Study });
    CardDefinition {
        name: "Lattice Library",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Study, Value::XFromCost)),
        triggered_abilities: vec![etb(fractal()), first_x_spell(fractal())],
        ..Default::default()
    }
}

/// Nev, the Practical Dean — your creatures with counters have trample;
/// your first {X} spell each turn puts X counters on Nev.
pub fn nev_the_practical_dean() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with counters on them have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours().and(R::WithAnyCounter)),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![first_x_spell(p1p1(Selector::This, Value::XFromCost))],
        ..legend(
            "Nev, the Practical Dean",
            cost(&[generic(2), g()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Nexus Mentality — move all counters between two of your nonland
/// permanents, or remove them all and draw a card per counter; both with a
/// commander.
pub fn nexus_mentality() -> CardDefinition {
    let nonland = || R::Permanent.and(R::Not(Box::new(R::Land))).and(R::ControlledByYou);
    let modes = || {
        vec![
            Effect::MoveAllCounters {
                from: target_filtered(nonland()),
                to: Selector::TargetFiltered { slot: 1, filter: nonland() },
            },
            Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::TotalCountersOn { what: Box::new(target_filtered(nonland())) },
                },
                Effect::RemoveAllCounters { what: Selector::Target(0) },
            ]),
        ]
    };
    spell(
        "Nexus Mentality",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
    )
}

/// Owlin Spiralmancer — flying, vigilance; your first {X} spell each turn
/// may be copied.
pub fn owlin_spiralmancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![first_x_spell(Effect::MayDo {
            description: "Copy that spell?".into(),
            body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
        })],
        ..creature("Owlin Spiralmancer", cost(&[generic(3), u()]), vec![CreatureType::Bird, CreatureType::Wizard], 3, 4)
    }
}

/// Ozolith, the Shattered Spire — +1/+1 counters on your artifacts and
/// creatures get one more; {1}{G}, {T}: a counter on one; cycling {2}.
pub fn ozolith_the_shattered_spire() -> CardDefinition {
    let artifact_or_creature = || R::Artifact.or(R::Creature);
    CardDefinition {
        name: "Ozolith, the Shattered Spire",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on an artifact or creature you control, that \
                          many plus one +1/+1 counters are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCountersMatching { filter: artifact_or_creature() },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: p1p1(target_filtered(artifact_or_creature().and(R::ControlledByYou)), Value::ONE),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Primo, the Unbounded — trample; enters with twice X counters; base-power-0
/// creatures of yours connecting make a Fractal the size of the damage.
///
/// Residual: with several such creatures connecting at once, the Fractal
/// reads the first one's damage.
pub fn primo_the_unbounded() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::BasePowerIs(0) })
                .once_per_batch(),
            effect: make_fractal(Value::TriggerEventAmount),
        }],
        ..legend(
            "Primo, the Unbounded",
            cost(&[x(), g(), g(), u()]),
            vec![CreatureType::Fractal, CreatureType::Wolf],
            0,
            0,
        )
    }
}

/// Primordial Hydra — X counters, doubled each upkeep; trample at ten.
pub fn primordial_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: x_counters(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::PlusOnePlusOne },
        }],
        static_abilities: vec![StaticAbility {
            description: "This creature has trample as long as it has ten or more +1/+1 counters on it.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                    Value::Const(10),
                ),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..creature("Primordial Hydra", cost(&[x(), g(), g()]), vec![CreatureType::Hydra], 0, 0)
    }
}

fn forest_island(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Forest, LandType::Island], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add(Color::Green), crate::sets::tap_add(Color::Blue)],
        ..Default::default()
    }
}

/// Rain-Slicked Copse — Forest Island; enters tapped; cycling {2}.
pub fn rain_slicked_copse() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..forest_island("Rain-Slicked Copse")
    }
}

/// Turbulent Wilderness — Forest Island; enters tapped unless your
/// opponents control eight or more lands.
pub fn turbulent_wilderness() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless your opponents control eight or more lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                    n: Value::Const(8),
                },
            },
        }],
        ..forest_island("Turbulent Wilderness")
    }
}

/// Steelbane Hydra — X counters; {2}{G}, remove a counter: destroy an
/// artifact or enchantment.
pub fn steelbane_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: x_counters(),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..creature("Steelbane Hydra", cost(&[x(), g(), g()]), vec![CreatureType::Turtle, CreatureType::Hydra], 0, 0)
    }
}

/// Run the Play — Striding Shotcaller's prepared spell: a counter and
/// flying on each of up to X creatures, then draw.
fn run_the_play() -> CardDefinition {
    spell(
        "Run the Play",
        cost(&[x(), g(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::CapTargetsAtX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        p1p1(Selector::Target(0), Value::ONE),
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Flying,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                }),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Striding Shotcaller — reach; your creatures connecting prepare it (Run
/// the Play).
pub fn striding_shotcaller() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
            effect: becomes_prepared(),
        }],
        prepare_spell: Some(Arc::new(run_the_play())),
        ..creature(
            "Striding Shotcaller",
            cost(&[g(), u()]),
            vec![CreatureType::Troll, CreatureType::Druid],
            0,
            4,
        )
    }
}

/// Unbound Flourishing — your {X} permanent spells get double X; your {X}
/// instants and sorceries are copied.
///
/// Residual: activated abilities with {X} aren't copied.
pub fn unbound_flourishing() -> CardDefinition {
    // `CastSpellHasX`, not `HasXInCost` inside `CastSpellMatches`: on the
    // stack a spell's cost reads with its X filled in (CR 202.3e).
    let on_cast = |filter: R, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::All(vec![Predicate::CastSpellHasX, Predicate::CastSpellMatches(filter)])),
        effect,
    };
    CardDefinition {
        name: "Unbound Flourishing",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            on_cast(R::PermanentCard, Effect::DoubleXOfSpell { what: Selector::TriggerSource }),
            on_cast(
                instant_or_sorcery(),
                Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
            ),
        ],
        ..Default::default()
    }
}

/// Yavimaya Bloomsage — your end step puts a counter on a creature of yours
/// and prepares it (Channel) once that creature has power 7 or more.
pub fn yavimaya_bloomsage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                p1p1(target_filtered(yours()), Value::ONE),
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::PowerOf(Box::new(Selector::Target(0))),
                        Value::Const(7),
                    ),
                    then: Box::new(becomes_prepared()),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        prepare_spell: Some(Arc::new(crate::sets::decks::channel())),
        ..creature("Yavimaya Bloomsage", cost(&[generic(2), g()]), vec![CreatureType::Dryad, CreatureType::Druid], 2, 2)
    }
}
