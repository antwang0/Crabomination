//! Red/white aggro batch: burn-on-ETB, battalion, renown, a defender wall,
//! and a graveyard-cast punisher. Tests in `tests/recent61.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_noncreature, etb, renown, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{cost, generic, r, w};

/// Kessig Malcontents — {2}{R} 3/1 Human Warrior. ETB: deal damage to target
/// player or planeswalker equal to the number of Humans you control.
pub fn kessig_malcontents() -> CardDefinition {
    CardDefinition {
        name: "Kessig Malcontents",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Player.or(R::Planeswalker)),
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Human).and(R::ControlledByYou),
            ))),
        })],
        ..Default::default()
    }
}

/// Somberwald Vigilante — {R} 1/1 Human Warrior. When it becomes blocked by a
/// creature, it deals 1 damage to that creature.
pub fn somberwald_vigilante() -> CardDefinition {
    CardDefinition {
        name: "Somberwald Vigilante",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::BlockingCreatures,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Ash Zealot — {R}{R} 2/2 Human Warrior with first strike, haste. Whenever a
/// player casts a spell from a graveyard, deal 3 damage to that player.
pub fn ash_zealot() -> CardDefinition {
    CardDefinition {
        name: "Ash Zealot",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastFromGraveyard),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

/// Perimeter Captain — {W} 0/4 Human Soldier with defender. Whenever a creature
/// you control with defender blocks, you may gain 2 life.
pub fn perimeter_captain() -> CardDefinition {
    CardDefinition {
        name: "Perimeter Captain",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Defender),
                },
            ),
            effect: Effect::MayDo {
                description: "Gain 2 life".into(),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Firefist Striker — {1}{R} 2/1 Human Soldier. Battalion — when it and 2+
/// others attack, target creature can't block this turn.
pub fn firefist_striker() -> CardDefinition {
    CardDefinition {
        name: "Firefist Striker",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackingWithAtLeast(3)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Scab-Clan Berserker — {1}{R}{R} 2/2 Human Berserker with haste, renown 1.
/// Whenever an opponent casts a noncreature spell, if this is renowned, deal 2
/// damage to that player.
pub fn scab_clan_berserker() -> CardDefinition {
    CardDefinition {
        name: "Scab-Clan Berserker",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            renown(1),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(cast_is_noncreature()),
                effect: Effect::If {
                    cond: Predicate::SourceIsRenowned,
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Fireblade Charger — {R} 1/1 Goblin Warrior. Has haste while equipped. When it
/// dies, it deals damage equal to its power to any target.
pub fn fireblade_charger() -> CardDefinition {
    CardDefinition {
        name: "Fireblade Charger",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is equipped, it has haste.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsEquipped,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}
