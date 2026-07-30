//! FDN/DSK gap batch 2 — death-reanimation with grafted counters/types, a
//! finality burn spell, a delirium combat trick, a life-scaling legend, and a
//! combat-counter Wurm. Tests in `tests/recent203.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype,
};
use crate::effect::shortcut::{deal, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
    StaticEffect, TriggeredAbility, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, w};

/// Valkyrie's Call — {3}{W}{W} Enchantment. Whenever a nontoken, non-Angel
/// creature you control dies, return that card to the battlefield with a +1/+1
/// counter on it. It has flying and is an Angel in addition to its other types.
pub fn valkyries_call() -> CardDefinition {
    CardDefinition {
        name: "Valkyrie's Call",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken.and(R::HasCreatureType(CreatureType::Angel).negate()),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::AddKeywordCounter {
                    what: Selector::LastMoved,
                    keyword: Keyword::Flying,
                    amount: Value::ONE,
                },
                Effect::AddCreatureTypes {
                    what: Selector::LastMoved,
                    creature_types: vec![CreatureType::Angel],
                    duration: Duration::Permanent,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Infernal Vessel — {2}{B} 2/1 Human Cleric. When this dies, if it wasn't a
/// Demon, return it to the battlefield with two +1/+1 counters on it. It's a
/// Demon in addition to its other types.
pub fn infernal_vessel() -> CardDefinition {
    CardDefinition {
        name: "Infernal Vessel",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Demon).negate(),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::AddCreatureTypes {
                    what: Selector::LastMoved,
                    creature_types: vec![CreatureType::Demon],
                    duration: Duration::Permanent,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Fiery Annihilation — {2}{R} Instant. Deals 5 damage to target creature; if it
/// would die this turn, exile it instead. (The exile-attached-Equipment rider is
/// approximated away — no equipment second-target slot yet.)
pub fn fiery_annihilation() -> CardDefinition {
    CardDefinition {
        name: "Fiery Annihilation",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        // Install the die→exile replacement first, then deal the damage, so a
        // creature this kills is exiled rather than buried.
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: target_filtered(R::Creature),
            },
            deal(5, target_filtered(R::Creature)),
        ]),
        ..Default::default()
    }
}

/// Violent Urge — {R} Instant. Target creature gets +1/+0 and gains first strike
/// until end of turn. Delirium — if four or more card types are among cards in
/// your graveyard, it gains double strike until end of turn instead.
pub fn violent_urge() -> CardDefinition {
    CardDefinition {
        name: "Violent Urge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::DeliriumActive {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Elenda, Saint of Dusk — {2}{W}{B} 4/4 Legendary Vampire Knight, Lifelink. Gets
/// +1/+1 and menace while your life is above your starting total, and an
/// additional +5/+5 while it's at least 10 above. (Hexproof from instants is
/// approximated away — no from-instants hexproof keyword yet.)
pub fn elenda_saint_of_dusk() -> CardDefinition {
    CardDefinition {
        name: "Elenda, Saint of Dusk",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![
            StaticAbility {
                description: "+1/+1 and menace while above your starting life.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::PlayerLifeAtLeastAboveStarting {
                        who: PlayerRef::You,
                        delta: 1,
                    },
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Menace],
                },
            },
            StaticAbility {
                description: "+5/+5 more while 10+ above your starting life.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::PlayerLifeAtLeastAboveStarting {
                        who: PlayerRef::You,
                        delta: 10,
                    },
                    power: 5,
                    toughness: 5,
                    keywords: vec![],
                },
            },
        ],
        ..Default::default()
    }
}

/// Quilled Greatwurm — {4}{G}{G} 7/7 Wurm, Trample. Whenever a creature you
/// control deals combat damage to a player during your turn, put that many +1/+1
/// counters on it. (The graveyard-cast-by-removing-counters rider is approximated
/// away.)
pub fn quilled_greatwurm() -> CardDefinition {
    CardDefinition {
        name: "Quilled Greatwurm",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}
