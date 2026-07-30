//! Red burn + a green trample-anthem: Searing Wind, Lava Burst, Jagged
//! Lightning, Rain of Embers, Thunderfoot Baloth. Tests in `tests/recent88.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes,
};
use crate::effect::shortcut::{deal, target};
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, g, generic, r, x};

/// Searing Wind — {4}{R}{R} Sorcery. Deals 5 damage to any target.
pub fn searing_wind() -> CardDefinition {
    CardDefinition {
        name: "Searing Wind",
        cost: cost(&[generic(8), r()]),
        card_types: vec![CardType::Sorcery],
        effect: deal(5, target()),
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

/// Rain of Embers — {2}{R} Sorcery. Deals 1 damage to each creature without
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

/// Thunderfoot Baloth — {3}{G}{G} 5/5 Beast. Other creatures you control get
/// +2/+2 and have trample.
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
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures you control get +2/+2.",
                effect: StaticEffect::PumpPT {
                    applies_to: others(),
                    power: 2,
                    toughness: 2,
                },
            },
            StaticAbility {
                description: "Other creatures you control have trample.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others(),
                    keyword: Keyword::Trample,
                },
            },
        ],
        ..Default::default()
    }
}
