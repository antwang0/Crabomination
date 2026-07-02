//! Kamigawa: Neon Dynasty batch 2 — Channel spells, enchantment-matters, and
//! more Ninjutsu / Reconfigure. Rides existing primitives. Tests in
//! `tests/recent96.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, Subtypes, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Jukai Naturalist — {G}{W} 2/2 Human Monk Enchantment Creature, lifelink.
/// Enchantment spells you cast cost {1} less to cast.
pub fn jukai_naturalist() -> CardDefinition {
    CardDefinition {
        name: "Jukai Naturalist",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Enchantment spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Enchantment, amount: 1 },
        }],
        ..Default::default()
    }
}

/// Ironhoof Boar — {5}{R} 5/4 Boar artifact creature, trample & haste. Channel —
/// {1}{R}, Discard this card: target creature gets +3/+1 and gains trample until
/// end of turn.
pub fn ironhoof_boar() -> CardDefinition {
    CardDefinition {
        name: "Ironhoof Boar",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(3),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
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

/// Reinforced Ronin — {R} 2/2 Human Samurai artifact creature, haste. At the
/// beginning of your end step, return it to its owner's hand. Channel — {1}{R},
/// Discard this card: draw a card.
pub fn reinforced_ronin() -> CardDefinition {
    use crate::effect::shortcut::draw;
    CardDefinition {
        name: "Reinforced Ronin",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::YourControl),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Colossal Skyturtle — {4}{G}{G}{U} 6/5 Turtle Enchantment Creature, flying,
/// ward {2}. Channel — {2}{G}: return target card from your graveyard to your
/// hand. Channel — {1}{U}: return target creature to its owner's hand.
pub fn colossal_skyturtle() -> CardDefinition {
    CardDefinition {
        name: "Colossal Skyturtle",
        cost: cost(&[generic(4), g(), g(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Turtle], ..Default::default() },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                from_hand: true,
                discard_self_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::InYourGraveyard),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                from_hand: true,
                discard_self_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kami of Transience — {1}{G} 2/2 Spirit, trample. Whenever you cast an
/// enchantment spell, put a +1/+1 counter on it. (The graveyard-recursion end-
/// step trigger is dropped.)
pub fn kami_of_transience() -> CardDefinition {
    CardDefinition {
        name: "Kami of Transience",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Rabbit Battery — {R} 1/1 Equipment Rabbit artifact creature, haste. Equipped
/// creature gets +1/+1 and has haste. Reconfigure {R}.
pub fn rabbit_battery() -> CardDefinition {
    CardDefinition {
        name: "Rabbit Battery",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Reconfigure(cost(&[r()]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Haste],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Nezumi Prowler — {1}{B} 3/1 Rat Ninja artifact creature. Ninjutsu {1}{B}.
/// ETB: target creature you control gains deathtouch and lifelink until end of
/// turn.
pub fn nezumi_prowler() -> CardDefinition {
    CardDefinition {
        name: "Nezumi Prowler",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Ninja],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), b()]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Invigorating Hot Spring — {1}{R}{G} Enchantment. Enters with four +1/+1
/// counters. Modified creatures you control have haste. Remove a +1/+1 counter
/// from this: put a +1/+1 counter on target creature you control. Sorcery-speed,
/// once each turn.
pub fn invigorating_hot_spring() -> CardDefinition {
    CardDefinition {
        name: "Invigorating Hot Spring",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        static_abilities: vec![StaticAbility {
            description: "Modified creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::IsModified),
                ),
                keyword: Keyword::Haste,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            once_per_turn: true,
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
