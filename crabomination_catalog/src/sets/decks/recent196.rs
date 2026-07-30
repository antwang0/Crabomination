//! OTJ gap batch on existing primitives: Slickshot Vault-Buster (crime pump),
//! Throw from the Saddle (Mount-scaled pump + one-sided fight), Shepherd of the
//! Clouds (Mount-conditional graveyard return), Sheriff of Safe Passage
//! (scaled enters-with-counters + Plot). Tests in `tests/recent196.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
};
use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, g, generic, u, w};

/// Slickshot Vault-Buster — {2}{U} 1/4 Human Rogue, Vigilance. Gets +2/+0 while
/// you've committed a crime this turn.
pub fn slickshot_vault_buster() -> CardDefinition {
    CardDefinition {
        name: "Slickshot Vault-Buster",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Gets +2/+0 as long as you've committed a crime this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::CommittedCrimeThisTurn {
                    who: PlayerRef::You,
                },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Throw from the Saddle — {1}{G} Sorcery. Target creature you control gets
/// +1/+1 until end of turn (a +1/+1 counter instead if it's a Mount), then it
/// deals damage equal to its power to target creature you don't control.
pub fn throw_from_the_saddle() -> CardDefinition {
    CardDefinition {
        name: "Throw from the Saddle",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Mount),
                },
                then: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Shepherd of the Clouds — {4}{W} 4/3 Pegasus, Flying, vigilance. ETB: return a
/// target permanent card with mana value 3 or less from your graveyard to your
/// hand — to the battlefield instead if you control a Mount.
pub fn shepherd_of_the_clouds() -> CardDefinition {
    let gy_target = || Selector::TargetFiltered {
        slot: 0,
        filter: R::Permanent
            .and(R::InYourGraveyard)
            .and(R::ManaValueAtMost(3)),
    };
    CardDefinition {
        name: "Shepherd of the Clouds",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pegasus],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Mount).and(R::ControlledByYou),
                )),
                then: Box::new(Effect::Move {
                    what: gy_target(),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
                else_: Box::new(Effect::Move {
                    what: gy_target(),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Sheriff of Safe Passage — {2}{W} 0/0 Human Knight. Enters with a +1/+1
/// counter plus one more for each other creature you control. Plot {1}{W}.
pub fn sheriff_of_safe_passage() -> CardDefinition {
    CardDefinition {
        name: "Sheriff of Safe Passage",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        plot_cost: Some(cost(&[generic(1), w()])),
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Sum(vec![
                Value::Const(1),
                Value::count(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                )),
            ]),
        )),
        ..Default::default()
    }
}
