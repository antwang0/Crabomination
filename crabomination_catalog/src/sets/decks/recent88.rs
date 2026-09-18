//! Red burn + a green trample-anthem: Searing Wind, Lava Burst, Jagged
//! Lightning, Rain of Embers, Thunderfoot Baloth. Tests in `tests/recent88.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes,
};
use crate::effect::shortcut::{deal, target};
use crate::effect::{Effect, Predicate, PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, r, x};

/// Searing Wind — {8}{R} Sorcery. Deals 10 damage to any target.
pub fn searing_wind() -> CardDefinition {
    CardDefinition {
        name: "Searing Wind",
        cost: cost(&[generic(8), r()]),
        card_types: vec![CardType::Instant],
        effect: deal(10, target()),
        ..Default::default()
    }
}

/// Lava Burst — {X}{R} Sorcery. Deals X damage to any target. (The "damage
/// can't be prevented" rider is dropped.)
pub fn lava_burst() -> CardDefinition {
    CardDefinition {
        name: "Lava Burst",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target(),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Jagged Lightning — {3}{R}{R} Sorcery. Deals 3 damage to each of up to two
/// target creatures.
pub fn jagged_lightning() -> CardDefinition {
    CardDefinition {
        name: "Jagged Lightning",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(3),
            }),
        },
        ..Default::default()
    }
}

/// Rain of Embers — {1}{R} Sorcery. Deals 1 damage to each creature without
/// flying.
pub fn rain_of_embers() -> CardDefinition {
    CardDefinition {
        name: "Rain of Embers",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            ),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(1),
            }),
        },
        ..Default::default()
    }
}

/// Thunderfoot Baloth — {4}{G}{G} 5/5 Beast with trample. Other creatures you
/// control get +2/+2 and have trample.
pub fn thunderfoot_baloth() -> CardDefinition {
    let others =
        || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        name: "Thunderfoot Baloth",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![
            // Lieutenant (CR 207.2c — an ability word; the condition is the
            // printed text). All three halves are gated on controlling your
            // commander; the anthem used to apply unconditionally, which made
            // the card strictly better than printed.
            StaticAbility {
                description: "Lieutenant — this creature gets +2/+2.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
            },
            StaticAbility {
                description: "Lieutenant — other creatures you control get +2/+2.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::PumpPT {
                        applies_to: others(),
                        power: 2,
                        toughness: 2,
                    }),
                },
            },
            StaticAbility {
                description: "Lieutenant — other creatures you control have trample.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: others(),
                        keyword: Keyword::Trample,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}
