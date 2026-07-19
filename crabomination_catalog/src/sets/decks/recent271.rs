//! SNC/DMU gap batch — an aristocrats Devil, a shield-counter trick, a hybrid
//! pump Wall, a death-shield Soldier, a protect trick, a kicker edict, and a
//! modal keyword-granting artifact. All on existing primitives. Tests in
//! `tests/recent_b/recent271.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{b, cost, generic, hybrid, r, u, w, Color};

/// Body Dropper — {B}{R} 2/2 Devil Warrior. Whenever you sacrifice another
/// creature, put a +1/+1 counter on it. {B}{R}, Sacrifice another creature:
/// gains menace until end of turn.
pub fn body_dropper() -> CardDefinition {
    CardDefinition {
        name: "Body Dropper",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), r()]),
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Boon of Safety — {W} Instant. Put a shield counter on target creature; scry 1.
pub fn boon_of_safety() -> CardDefinition {
    CardDefinition {
        name: "Boon of Safety",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::Shield,
                amount: Value::ONE,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Brokers Initiate — {W} 0/4 Cat Citizen. {4}{G/U}: this creature has base
/// power and toughness 5/5 until end of turn.
pub fn brokers_initiate() -> CardDefinition {
    CardDefinition {
        name: "Brokers Initiate",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Citizen],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), hybrid(Color::Green, Color::Blue)]),
            effect: Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Brokers Veteran — {1}{U} 2/1 Human Soldier. When it dies, put a shield
/// counter on target creature you control.
pub fn brokers_veteran() -> CardDefinition {
    CardDefinition {
        name: "Brokers Veteran",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::Shield,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Battle-Rage Blessing — {1}{B} Instant. Target creature gains deathtouch and
/// indestructible until end of turn.
pub fn battle_rage_blessing() -> CardDefinition {
    CardDefinition {
        name: "Battle-Rage Blessing",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Benalish Sleeper — {1}{W} 3/1 Phyrexian Human Soldier, kicker {B}. When it
/// enters, if it was kicked, each player sacrifices a creature.
pub fn benalish_sleeper() -> CardDefinition {
    CardDefinition {
        name: "Benalish Sleeper",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Human,
                CreatureType::Soldier,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Kicker(cost(&[b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::ONE,
                filter: R::Creature,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Argivian Avenger — {6} 5/5 Shapeshifter artifact creature. {1}: until end of
/// turn, this gets -1/-1 and gains your choice of flying, vigilance,
/// deathtouch, or haste.
pub fn argivian_avenger() -> CardDefinition {
    let grant = |k: Keyword| Effect::GrantKeyword {
        what: Selector::This,
        keyword: k,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Argivian Avenger",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::ChooseMode(vec![
                    grant(Keyword::Flying),
                    grant(Keyword::Vigilance),
                    grant(Keyword::Deathtouch),
                    grant(Keyword::Haste),
                ]),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
