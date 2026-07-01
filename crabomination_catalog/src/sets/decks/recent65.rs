//! Black aggro/aristocrats batch: low-life payoffs, a drain outlet, exalted,
//! and a death-token maker. Tests in `tests/recent65.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{exalted, target_filtered};
use crate::effect::PlayerRef;
use crate::mana::{b, cost, generic, hybrid, Color};

fn low_life() -> Predicate {
    Predicate::PlayerLifeAtMost { who: PlayerRef::EachOpponent, life: 10 }
}

/// Ruthless Cullblade — {1}{B} 2/1 Vampire Warrior. +2/+1 as long as an opponent
/// has 10 or less life.
pub fn ruthless_cullblade() -> CardDefinition {
    CardDefinition {
        name: "Ruthless Cullblade",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "+2/+1 while an opponent has 10 or less life.",
            effect: StaticEffect::PumpSelfIf {
                condition: low_life(),
                power: 2,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Guul Draz Vampire — {B} 1/1 Vampire Rogue. While an opponent has 10 or less
/// life, it gets +2/+1 and has intimidate.
pub fn guul_draz_vampire() -> CardDefinition {
    CardDefinition {
        name: "Guul Draz Vampire",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "+2/+1 and intimidate while an opponent has 10 or less life.",
            effect: StaticEffect::PumpSelfIf {
                condition: low_life(),
                power: 2,
                toughness: 1,
                keywords: vec![Keyword::Intimidate],
            },
        }],
        ..Default::default()
    }
}

/// Bloodrite Invoker — {2}{B} 3/1 Vampire Shaman. {8}: Target player loses 3
/// life and you gain 3 life.
pub fn bloodrite_invoker() -> CardDefinition {
    CardDefinition {
        name: "Bloodrite Invoker",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::Drain {
                from: target_filtered(R::Player),
                to: Selector::You,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nip Gwyllion — {W/B} 1/1 Hag with lifelink.
pub fn nip_gwyllion() -> CardDefinition {
    CardDefinition {
        name: "Nip Gwyllion",
        cost: cost(&[hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hag], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    }
}

/// Barony Vampire — {2}{B} 3/2 Vampire.
pub fn barony_vampire() -> CardDefinition {
    CardDefinition {
        name: "Barony Vampire",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 2,
        ..Default::default()
    }
}

/// Nested Shambler — {B} 1/1 Zombie. When it dies, create X tapped 1/1 green
/// Squirrels, where X is its power.
pub fn nested_shambler() -> CardDefinition {
    let squirrel = TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        name: "Nested Shambler",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::TriggerSource)),
                definition: squirrel,
            },
        }],
        ..Default::default()
    }
}

/// Duty-Bound Dead — {B} 0/2 Skeleton with exalted. {3}{B}: Regenerate this
/// creature.
pub fn duty_bound_dead() -> CardDefinition {
    CardDefinition {
        name: "Duty-Bound Dead",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        power: 0,
        toughness: 2,
        triggered_abilities: vec![exalted()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}
