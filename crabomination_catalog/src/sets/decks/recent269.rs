//! Gap batch — a stun-tapping unblockable Crab, a modal ramp/pump instant, a
//! one-or-two pump, a grindy Sloth, a fragile Illusion, and a mana-sink Elf.
//! All on existing primitives. Tests in `tests/recent_b/recent269.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u};

/// Gilded Scuttler — {2}{U} 1/3 Crab artifact creature. Can't be blocked. ETB:
/// tap target creature an opponent controls and put a stun counter on it.
pub fn gilded_scuttler() -> CardDefinition {
    CardDefinition {
        name: "Gilded Scuttler",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Crab], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Go Forth — {G} Instant. Choose one — tutor a basic land to hand, or target
/// creature gets +2/+2 until end of turn.
pub fn go_forth() -> CardDefinition {
    CardDefinition {
        name: "Go Forth",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Hearts on Fire — {1}{R} Instant. One or two target creatures each get +2/+1
/// until end of turn.
pub fn hearts_on_fire() -> CardDefinition {
    CardDefinition {
        name: "Hearts on Fire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Hungry Megasloth — {2}{G} 3/3 Sloth Beast. Reach. {2}, {T}: put a +1/+1
/// counter on this creature.
pub fn hungry_megasloth() -> CardDefinition {
    CardDefinition {
        name: "Hungry Megasloth",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sloth, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
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

/// Phantasmal Shieldback — {U} 1/3 Turtle Illusion. When it becomes the target
/// of a spell or ability, sacrifice it. When it dies, draw a card.
pub fn phantasmal_shieldback() -> CardDefinition {
    CardDefinition {
        name: "Phantasmal Shieldback",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Turtle, CreatureType::Illusion],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                effect: Effect::SacrificeSource,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Battlefield Butcher — {2}{B} 1/4 Human Soldier. {5}, {T}: each opponent
/// loses 2 life. This ability costs {1} less to activate for each creature card
/// in your graveyard.
pub fn battlefield_butcher() -> CardDefinition {
    CardDefinition {
        name: "Battlefield Butcher",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            cost_reduction_per_graveyard: Some(R::Creature),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Razorgrass Invoker — {3}{G} 4/3 Elf Scout. Vigilance. {8}: this creature and
/// up to one other target creature each get +3/+3 until end of turn.
pub fn razorgrass_invoker() -> CardDefinition {
    CardDefinition {
        name: "Razorgrass Invoker",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature.and(R::OtherThanSource),
                    effect: Box::new(Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
