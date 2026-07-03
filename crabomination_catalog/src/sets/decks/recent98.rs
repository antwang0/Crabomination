//! Kamigawa: Neon Dynasty batch 4 — Vehicles, artifact/enchantment-gated
//! keywords, more Ninjutsu and graveyard-hate. Rides existing primitives. Tests
//! in `tests/recent98.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Nezumi Bladeblesser — {2}{B} 3/2 Rat Samurai. Has deathtouch while you
/// control an artifact and menace while you control an enchantment.
pub fn nezumi_bladeblesser() -> CardDefinition {
    let gated = |kw: Keyword, filter: R, desc: &'static str| StaticAbility {
        description: desc,
        effect: StaticEffect::PumpSelfIf {
            condition: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(filter.and(R::ControlledByYou)),
                n: Value::Const(1),
            },
            power: 0,
            toughness: 0,
            keywords: vec![kw],
        },
    };
    CardDefinition {
        name: "Nezumi Bladeblesser",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Samurai],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        static_abilities: vec![
            gated(Keyword::Deathtouch, R::Artifact, "Deathtouch while you control an artifact."),
            gated(Keyword::Menace, R::Enchantment, "Menace while you control an enchantment."),
        ],
        ..Default::default()
    }
}

/// Iron Apprentice — {1} 0/0 Artifact Construct. Enters with a +1/+1 counter.
/// When it dies, move its counters to target creature you control.
pub fn iron_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Iron Apprentice",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![on_dies(Effect::MoveAllCounters {
            from: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..Default::default()
    }
}

/// Circuit Mender — {3} 2/3 Artifact Insect. ETB: gain 2 life. When it leaves
/// the battlefield, draw a card.
pub fn circuit_mender() -> CardDefinition {
    CardDefinition {
        name: "Circuit Mender",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Dragonfly Suit — {2}{W} 3/2 Vehicle, flying. Crew 1.
pub fn dragonfly_suit() -> CardDefinition {
    CardDefinition {
        name: "Dragonfly Suit",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        ..Default::default()
    }
}

/// Moon-Circuit Hacker — {1}{U} 2/1 Human Ninja enchantment creature. Ninjutsu
/// {U}. Combat damage: you may draw a card. (The "discard unless it entered this
/// turn" clause is omitted — Ninjutsu'd copies would skip it anyway.)
pub fn moon_circuit_hacker() -> CardDefinition {
    CardDefinition {
        name: "Moon-Circuit Hacker",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Ninjutsu(cost(&[u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

/// Kaito's Pursuit — {2}{B} Sorcery. Target player discards two cards. Ninjas
/// and Rogues you control gain menace until end of turn.
pub fn kaitos_pursuit() -> CardDefinition {
    CardDefinition {
        name: "Kaito's Pursuit",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: false,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ninja)
                        .or(R::HasCreatureType(CreatureType::Rogue))
                        .and(R::ControlledByYou),
                ),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Bearer of Memory — {2}{G} 3/2 Human Monk enchantment creature. {5}{G}: put a
/// +1/+1 counter on target enchantment creature; it gains trample until end of
/// turn.
pub fn bearer_of_memory() -> CardDefinition {
    CardDefinition {
        name: "Bearer of Memory",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Enchantment.and(R::Creature)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dokuchi Shadow-Walker — {4}{B}{B} 5/5 Ogre Ninja. Ninjutsu {3}{B}.
pub fn dokuchi_shadow_walker() -> CardDefinition {
    CardDefinition {
        name: "Dokuchi Shadow-Walker",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Ninja],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(3), b()]))],
        ..Default::default()
    }
}

/// Reito Sentinel — {3} 3/3 Artifact Construct, defender. ETB: target player
/// mills three. {3}: put target card from a graveyard on the bottom of its
/// owner's library.
pub fn reito_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Reito Sentinel",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(3),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Akki Ronin — {1}{R} 1/3 Goblin Samurai. Whenever a Samurai or Warrior you
/// control attacks alone, you may discard a card; if you do, draw a card.
pub fn akki_ronin() -> CardDefinition {
    CardDefinition {
        name: "Akki Ronin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Samurai],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::AttackingAlone,
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Samurai)
                            .or(R::HasCreatureType(CreatureType::Warrior)),
                    },
                ]),
            ),
            effect: Effect::MayDiscard {
                description: "Discard a card to draw a card?".into(),
                count: Value::Const(1),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
