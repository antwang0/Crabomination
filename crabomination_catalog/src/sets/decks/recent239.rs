//! DSK (Duskmourn) gap batch — Survival creatures, Delirium payoffs, a
//! modal redirect, and Norin's blocked-creature blink. Tests in
//! `tests/recent239.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    MayPlayDuration, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{deal, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
    ZoneDest,
};
use crate::mana::{cost, g, generic, r};

/// Betrayer's Bargain — {1}{R} Instant. Additional cost: sacrifice a creature
/// or enchantment or pay {2}. Deal 5 to target creature; if it would die this
/// turn, exile it instead.
pub fn betrayers_bargain() -> CardDefinition {
    CardDefinition {
        name: "Betrayer's Bargain",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay {
            filter: R::Creature.or(R::Enchantment),
            pay: 2,
        }],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            deal(5, target_filtered(R::Creature)),
        ]),
        ..Default::default()
    }
}

/// Untimely Malfunction — {1}{R} Instant. Choose one — destroy target artifact;
/// choose new targets for target spell; or one or two target creatures can't
/// block this turn.
pub fn untimely_malfunction() -> CardDefinition {
    CardDefinition {
        name: "Untimely Malfunction",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::ChooseNewTargetsForSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Omnivorous Flytrap — {2}{G} Plant 2/4. Delirium — Whenever it enters or
/// attacks, if 4+ card types in your graveyard, distribute two +1/+1 counters
/// among one or two target creatures; then if 6+ types, double the +1/+1
/// counters on those creatures.
pub fn omnivorous_flytrap() -> CardDefinition {
    let delirium_body = || {
        Effect::Seq(vec![
            Effect::DistributeCounters {
                total: Value::Const(2),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 2,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardTypesInGraveyard(PlayerRef::You),
                    Value::Const(6),
                ),
                then: Box::new(Effect::DoubleCountersOnEach {
                    what: Selector::AllTargets,
                    kind: CounterType::PlusOnePlusOne,
                }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    let delirium = || Predicate::DeliriumActive { who: PlayerRef::You };
    CardDefinition {
        name: "Omnivorous Flytrap",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(delirium()),
                effect: delirium_body(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                    .with_filter(delirium()),
                effect: delirium_body(),
            },
        ],
        ..Default::default()
    }
}

/// Norin, Swift Survivalist — {R} Human Coward 2/1. Can't block. Whenever a
/// creature you control becomes blocked, you may exile it, then play it from
/// exile this turn.
pub fn norin_swift_survivalist() -> CardDefinition {
    CardDefinition {
        name: "Norin, Swift Survivalist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Coward],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exile it, then play it from exile this turn".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}
