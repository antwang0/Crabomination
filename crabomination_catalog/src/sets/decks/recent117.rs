//! Bloomburrow / Duskmourn / Foundations commons & uncommons that reuse
//! existing primitives (ETB bounce/impulse/modal, attack-count and
//! whenever-you-attack triggers, threshold, double-power + flashback).
//! Tests in `tests/recent117.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    MayPlayDuration, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Bigfin Bouncer — {3}{U} 3/2 Shark Pirate. ETB: return target creature an
/// opponent controls to its owner's hand.
pub fn bigfin_bouncer() -> CardDefinition {
    CardDefinition {
        name: "Bigfin Bouncer",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shark, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..Default::default()
    }
}

/// Alania's Pathmaker — {3}{R} 4/2 Otter Wizard. ETB: exile the top card of
/// your library; you may play it until the end of your next turn.
pub fn alanias_pathmaker() -> CardDefinition {
    CardDefinition {
        name: "Alania's Pathmaker",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(1),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Apothecary Stomper — {4}{G}{G} 4/4 Elephant with vigilance. ETB, choose one:
/// two +1/+1 counters on target creature you control, or gain 4 life.
pub fn apothecary_stomper() -> CardDefinition {
    CardDefinition {
        name: "Apothecary Stomper",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
        ]))],
        ..Default::default()
    }
}

/// Armasaur Guide — {4}{W} 4/4 Dinosaur with vigilance. Whenever you attack
/// with three or more creatures, put a +1/+1 counter on target creature you
/// control.
pub fn armasaur_guide() -> CardDefinition {
    CardDefinition {
        name: "Armasaur Guide",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCountAtLeast {
                    who: PlayerRef::ActivePlayer,
                    at_least: 3,
                },
            ),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Battlesong Berserker — {3}{R} 3/4 Human Berserker. Whenever you attack,
/// target creature you control gets +1/+0 and gains menace until end of turn.
pub fn battlesong_berserker() -> CardDefinition {
    CardDefinition {
        name: "Battlesong Berserker",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Billowing Shriekmass — {3}{B} 2/3 Spirit with flying. ETB: mill three.
/// Threshold — +2/+1 while seven or more cards are in your graveyard.
pub fn billowing_shriekmass() -> CardDefinition {
    CardDefinition {
        name: "Billowing Shriekmass",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::You,
            amount: Value::Const(3),
        })],
        static_abilities: vec![StaticAbility {
            description: "Threshold — gets +2/+1 while seven or more cards are in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ThresholdActive {
                    who: PlayerRef::You,
                },
                power: 2,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Bulk Up — {1}{R} instant. Double target creature's power until end of turn.
/// Flashback {4}{R}{R}.
pub fn bulk_up() -> CardDefinition {
    CardDefinition {
        name: "Bulk Up",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r(), r()]))],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::PowerOf(Box::new(Selector::Target(0))),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
