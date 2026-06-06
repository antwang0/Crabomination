//! Murders at Karlov Manor (MKM) — 2024. Detective set introducing the
//! Suspect (CR 701.60) and Collect Evidence (CR 701.59) keyword actions.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::card::ActivatedAbility;
use crate::effect::shortcut::{draw, etb, lose_life, on_attack, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate};
use crate::mana::{b, cost, g, generic, u};

/// Repeat Offender — {1}{B} 2/1 Human Assassin. "{2}{B}: If this creature is
/// suspected, put a +1/+1 counter on it. Otherwise, suspect it."
pub fn repeat_offender() -> CardDefinition {
    CardDefinition {
        name: "Repeat Offender",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::If {
                cond: Predicate::SourceIsSuspected,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Suspect { what: Selector::This }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reasonable Doubt — {1}{U} Instant. "Counter target spell unless its
/// controller pays {2}. Suspect up to one target creature." (The "up to one"
/// rider is modeled as a required creature target.)
pub fn reasonable_doubt() -> CardDefinition {
    CardDefinition {
        name: "Reasonable Doubt",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
            },
            Effect::Suspect {
                what: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Creature },
            },
        ]),
        ..Default::default()
    }
}

/// Sample Collector — {2}{G} 2/3 Troll Detective. "Whenever this attacks, you
/// may collect evidence 3. When you do, put a +1/+1 counter on target
/// creature you control."
pub fn sample_collector() -> CardDefinition {
    CardDefinition {
        name: "Sample Collector",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::CollectEvidence {
            amount: Value::Const(3),
            then: Box::new(Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Barbed Servitor — {3}{B} 1/1 Artifact Creature — Construct. Indestructible;
/// ETB suspect itself; combat damage to a player → draw + lose 1 life; when
/// dealt damage, each opponent loses that much life (modeled as each opponent
/// rather than a single target).
pub fn barbed_servitor() -> CardDefinition {
    CardDefinition {
        name: "Barbed Servitor",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![
            etb(Effect::Suspect { what: Selector::This }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![draw(1), lose_life(1, Selector::You)]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..Default::default()
    }
}
