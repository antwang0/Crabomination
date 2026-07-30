//! MKM (Murders at Karlov Manor) gap batch — the artifact-sacrifice payoff and
//! a modal removal spell. Tests in `tests/recent_b/recent248.rs`.

use crate::card::{CardDefinition, CardType, CounterType, Keyword, SelectionRequirement as R};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, Value};
use crate::mana::{b, cost, generic, r};

/// Suspicious Detonation — {4}{R} Sorcery. Costs {3} less if you've sacrificed an
/// artifact this turn. Can't be countered. Deal 4 damage to target creature.
pub fn suspicious_detonation() -> CardDefinition {
    CardDefinition {
        name: "Suspicious Detonation",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        self_cost_reduction_if_sacrificed_artifact: Some(3),
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Deadly Complication — {1}{B}{R} Sorcery. Choose one or both — destroy target
/// creature; and/or put a +1/+1 counter on a target suspected creature you
/// control and you may have it become no longer suspected.
pub fn deadly_complication() -> CardDefinition {
    CardDefinition {
        name: "Deadly Complication",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Destroy {
                    what: target_filtered(R::Creature),
                },
                Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(
                            R::Creature.and(R::IsSuspected).and(R::ControlledByYou),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::MayDo {
                        description: "Have it become no longer suspected?".into(),
                        body: Box::new(Effect::ClearSuspected {
                            what: crate::effect::Selector::Target(0),
                        }),
                    },
                ]),
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}
