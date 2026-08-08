//! Mercadian Masques (MMQ) gap closure, fourth wave. Tests in
//! `classic_sets/mmq4`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::{target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Cho-Manno, Revolutionary — {2}{W}{W} 2/2 that simply can't be damaged.
pub fn cho_manno_revolutionary() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to this creature.",
            effect: StaticEffect::PreventAllDamageToThis,
        }],
        ..creature(
            "Cho-Manno, Revolutionary",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            2,
        )
    }
}

/// Drake Hatchling — {2}{U} 1/3 flier with one pump a turn.
pub fn drake_hatchling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Drake Hatchling", cost(&[generic(2), u()]), vec![CreatureType::Drake], 1, 3)
    }
}

/// Pious Warrior — {3}{W} 2/3 that converts combat damage into life.
pub fn pious_warrior() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtCombatDamage, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Pious Warrior",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Quagmire Lamprey — {2}{B} 1/1 that shrinks whatever blocks it.
pub fn quagmire_lamprey() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::BlockingCreatures,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Quagmire Lamprey", cost(&[generic(2), b()]), vec![CreatureType::Fish], 1, 1)
    }
}

/// Saber Ants — {3}{G} 2/3 that spawns an Insect per point of damage taken.
pub fn saber_ants() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Create that many 1/1 green Insect creature tokens".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: Box::new(TokenDefinition {
                        name: "Insect".into(),
                        power: 1,
                        toughness: 1,
                        colors: vec![Color::Green],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Insect],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                }),
            },
        }],
        ..creature("Saber Ants", cost(&[generic(3), g()]), vec![CreatureType::Insect], 2, 3)
    }
}

/// Pangosaur — {2}{G}{G} 6/6 that bounces itself on every land drop.
pub fn pangosaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::AnyPlayer),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..creature("Pangosaur", cost(&[generic(2), g(), g()]), vec![CreatureType::Dinosaur], 6, 6)
    }
}

/// Saprazzan Breaker — {4}{U} 3/3 that mills its way past blockers.
pub fn saprazzan_breaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CardsMilledThisEffectMatching { filter: R::Land },
                        Value::ONE,
                    ),
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Saprazzan Breaker", cost(&[generic(4), u()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Sustenance — {1}{G}. Trade lands for a pump.
pub fn sustenance() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Sustenance", cost(&[generic(1), g()]))
    }
}

/// Pretender's Claim — {1}{B} Aura. Blocking the host costs the defender their
/// whole mana base for the turn.
pub fn pretenders_claim() -> CardDefinition {
    CardDefinition {
        name: "Pretender's Claim",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::Tap {
                    what: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Puffer Extract — {5}. Blow a creature up now, lose it at end of turn.
pub fn puffer_extract() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact],
        name: "Puffer Extract",
        cost: cost(&[generic(5)]),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::XFromCost,
                    toughness: Value::XFromCost,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Territorial Dispute — {4}{R}{R}. Nobody plays lands, and it eats one of
/// yours each upkeep.
pub fn territorial_dispute() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can't play lands.",
            effect: StaticEffect::NoPlayerCanPlayLands,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::SacrificeSourceUnlessSacrifice { filter: R::Land },
        }],
        ..enchantment("Territorial Dispute", cost(&[generic(4), r(), r()]))
    }
}

/// Righteous Indignation — {2}{W}. Blocking a black or red creature pays.
pub fn righteous_indignation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::BlockedAttacker,
                    filter: R::HasColor(Color::Black).or(R::HasColor(Color::Red)),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..enchantment("Righteous Indignation", cost(&[generic(2), w()]))
    }
}

/// Rock Badger's Merfolk cousin — Sand Squid {3}{U} 2/2 islandwalker that pins
/// a creature down for as long as it stays tapped.
pub fn sand_squid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TapAndLockWhileSourcePresent {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Sand Squid",
            cost(&[generic(3), u()]),
            vec![CreatureType::Squid, CreatureType::Beast],
            2,
            2,
        )
    }
}
