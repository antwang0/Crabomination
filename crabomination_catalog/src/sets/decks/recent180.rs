//! DSK/BLB gap batch — on existing primitives plus two small new ones
//! (`ActivatedAbility.activate_once` once-per-game gate; `BecomeColor.additive`
//! layer-5 add-color). Possessed Goat (once-per-game pump + become-black-Demon),
//! Hired Claw (Lizard attack ping + crime-gated growth), and Mistbreath Elder
//! (upkeep bounce-and-grow). Tests in `crabomination/src/tests/recent180.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{Color, cost, generic, r, w};

/// Possessed Goat — {W} 1/1 Goat. {3}, Discard a card: put three +1/+1 counters
/// on it and it becomes a black Demon in addition to its other colors and types.
/// Activate only once.
pub fn possessed_goat() -> CardDefinition {
    CardDefinition {
        name: "Possessed Goat",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goat], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            discard_cost: Some((R::Any, 1)),
            activate_once: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
                Effect::AddCreatureTypes {
                    what: Selector::This,
                    creature_types: vec![CreatureType::Demon],
                    duration: Duration::Permanent,
                },
                Effect::BecomeColor {
                    what: Selector::This,
                    colors: vec![Color::Black],
                    duration: Duration::Permanent,
                    additive: true,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hired Claw — {R} 1/2 Lizard Mercenary. Whenever you attack with one or more
/// Lizards, it deals 1 damage to target opponent. {1}{R}: put a +1/+1 counter on
/// it — only if an opponent lost life this turn, and only once each turn.
pub fn hired_claw() -> CardDefinition {
    CardDefinition {
        name: "Hired Claw",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCreatureMatching {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Lizard),
                },
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            once_per_turn: true,
            condition: Some(Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mistbreath Elder — {G} 2/2 Frog Warrior. At your upkeep, return another
/// creature you control to its owner's hand; if you do, put a +1/+1 counter on
/// it. (The "otherwise return this" fallback is approximated as a no-op.)
pub fn mistbreath_elder() -> CardDefinition {
    CardDefinition {
        name: "Mistbreath Elder",
        cost: cost(&[crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}
