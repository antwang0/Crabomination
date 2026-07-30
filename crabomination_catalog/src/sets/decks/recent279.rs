//! MID/VOW gap batch — two day/night creatures, a land-ramp dig, and a modal
//! Vampire pump. All on existing primitives. Tests in
//! `tests/recent_b/recent279.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::card::{CounterType, EventKind, EventScope, EventSpec, Predicate};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, r, w};

/// Day/night ETB: becomes day if it's neither day nor night.
fn day_on_etb() -> TriggeredAbility {
    etb(Effect::If {
        cond: Predicate::Not(Box::new(Predicate::Any(vec![
            Predicate::IsDay,
            Predicate::IsNight,
        ]))),
        then: Box::new(Effect::BecomeDay),
        else_: Box::new(Effect::Noop),
    })
}

/// Sunrise Cavalier — {1}{R}{W} 3/3 Human Knight. Trample, haste. Sets day on
/// entry; whenever day/night flips, put a +1/+1 counter on a creature you control.
pub fn sunrise_cavalier() -> CardDefinition {
    CardDefinition {
        name: "Sunrise Cavalier",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![
            day_on_etb(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DayNightChanged, EventScope::AnyPlayer),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Celestus Sanctifier — {2}{W} 3/2 Human Cleric. Sets day on entry; whenever
/// day/night flips, look at the top two cards and put one into your graveyard.
pub fn celestus_sanctifier() -> CardDefinition {
    CardDefinition {
        name: "Celestus Sanctifier",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            day_on_etb(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DayNightChanged, EventScope::AnyPlayer),
                effect: Effect::LookTopKeepOneRestToGraveyard {
                    count: Value::Const(2),
                    who: Some(PlayerRef::You),
                    exile_rest: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Cartographer's Survey — {3}{G} Sorcery. Look at the top seven cards; put up
/// to two land cards onto the battlefield tapped, the rest on the bottom in a
/// random order.
pub fn cartographers_survey() -> CardDefinition {
    CardDefinition {
        name: "Cartographer's Survey",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::Const(7),
            filter: R::Land,
            then: None,
            max: Some(2),
            tapped: true,
            exile_rest: false,
        },
        ..Default::default()
    }
}

/// Markov Retribution — {2}{R} Sorcery. Choose one or both — creatures you
/// control get +1/+0 until end of turn; and/or target Vampire you control deals
/// damage equal to its power to another target creature.
pub fn markov_retribution() -> CardDefinition {
    CardDefinition {
        name: "Markov Retribution",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: crate::effect::Duration::EndOfTurn,
                },
                Effect::DealDamageEqualToPower {
                    source: target_filtered(
                        R::HasCreatureType(CreatureType::Vampire).and(R::ControlledByYou),
                    ),
                    target: target_filtered(R::Creature),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}
