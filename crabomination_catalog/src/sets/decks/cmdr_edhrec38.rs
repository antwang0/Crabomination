//! Commander: three-card gaps for Jared Carthalion and Cleopatra
//! (`cmdr_edhrec37` is the previous batch of this run). Tests in
//! `tests/recent_b/cmdr_edhrec38.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies};
use crate::effect::{DamageToCounters, Effect, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, w, Color, ManaCost};
use std::sync::Arc;

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn at_your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

fn you_are_monarch() -> Predicate {
    Predicate::IsMonarch { who: PlayerRef::You }
}

// ── Jared Carthalion, True Heir ──────────────────────────────────────────────

/// Jared Carthalion, True Heir — {R}{G}{W} legendary 3/3 Human Warrior.
/// Enters: target opponent becomes the monarch, and you can't this turn
/// (CR 725). While you're the monarch, damage to it is prevented and becomes
/// that many +1/+1 counters.
pub fn jared_carthalion_true_heir() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::BecomeMonarch {
                who: PlayerRef::ControllerOf(Box::new(Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer })),
            },
            Effect::CantBecomeMonarchThisTurn { who: PlayerRef::You },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to Jared Carthalion while you're the monarch, prevent that \
                          damage and put that many +1/+1 counters on it.",
            effect: StaticEffect::WhileCondition {
                condition: you_are_monarch(),
                inner: Box::new(StaticEffect::ReplaceDamageToSelfWithCounters {
                    kind: CounterType::PlusOnePlusOne,
                    how: DamageToCounters::PreventAndPut,
                }),
            },
        }],
        ..legend(
            "Jared Carthalion, True Heir",
            cost(&[r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Court of Bounty — {2}{G}{G} enchantment. Enters: you become the monarch.
/// At your upkeep you may put a land card from your hand onto the
/// battlefield; as the monarch, a creature or land card instead.
pub fn court_of_bounty() -> CardDefinition {
    let put = |filter: R| Effect::PutFromHandOntoBattlefield {
        who: PlayerRef::You,
        filter,
        count: Value::ONE,
        tapped: false,
        haste: false,
        sacrifice_eot: false,
        return_eot: false,
        then: None,
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            at_your_upkeep(Effect::If {
                cond: you_are_monarch(),
                then: Box::new(put(R::Creature.or(R::Land))),
                else_: Box::new(put(R::Land)),
            }),
        ],
        ..enchantment("Court of Bounty", cost(&[generic(2), g(), g()]))
    }
}

/// Court of Garenbrig — {1}{G}{G} enchantment. Enters: you become the
/// monarch. At your upkeep, distribute two +1/+1 counters among up to two
/// target creatures; then, as the monarch, double the +1/+1 counters on each
/// creature you control.
pub fn court_of_garenbrig() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            at_your_upkeep(Effect::Seq(vec![
                Effect::DistributeCounters {
                    total: Value::Const(2),
                    counter: CounterType::PlusOnePlusOne,
                    filter: R::Creature,
                    max_targets: 2,
                },
                Effect::If {
                    cond: you_are_monarch(),
                    then: Box::new(Effect::DoubleCountersOnEach {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        ],
        ..enchantment("Court of Garenbrig", cost(&[generic(1), g(), g()]))
    }
}

// ── Cleopatra, Exiled Pharaoh ────────────────────────────────────────────────

/// Cleopatra, Exiled Pharaoh — {2}{B}{G} legendary 2/4 Human Noble. Your end
/// step: a +1/+1 counter on each of up to two other target legendary
/// creatures. Whenever a legendary creature with counters dies, draw a card
/// per counter on it and lose 2 life.
pub fn cleopatra_exiled_pharaoh() -> CardDefinition {
    let legendary_creature = || R::Creature.and(R::HasSupertype(Supertype::Legendary));
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: legendary_creature().and(R::OtherThanSource),
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: legendary_creature() },
                    Predicate::ValueAtLeast(
                        Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) },
                        Value::ONE,
                    ),
                ])),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) },
                    },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
        ],
        ..legend(
            "Cleopatra, Exiled Pharaoh",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Human, CreatureType::Noble],
            2,
            4,
        )
    }
}

/// Grakmaw, Skyclave Ravager — {1}{B}{G} legendary 0/0 Hydra Horror. Enters
/// with three +1/+1 counters; another creature of yours dying with a +1/+1
/// counter grows it; dies: an X/X black and green Hydra, X its counters.
pub fn grakmaw_skyclave_ravager() -> CardDefinition {
    // The dead Grakmaw, read through its death snapshot (CR 603.10a).
    let counters = || Value::CountersOn { what: Box::new(Selector::TriggerSource), kind: CounterType::PlusOnePlusOne };
    let hydra = TokenDefinition {
        name: "Hydra".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hydra], ..Default::default() },
        dynamic_pt: Some((counters(), counters())),
        ..Default::default()
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                    Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::TriggerSource),
                            kind: CounterType::PlusOnePlusOne,
                        },
                        Value::ONE,
                    ),
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            on_dies(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(hydra) }),
        ],
        ..legend(
            "Grakmaw, Skyclave Ravager",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Hydra, CreatureType::Horror],
            0,
            0,
        )
    }
}
