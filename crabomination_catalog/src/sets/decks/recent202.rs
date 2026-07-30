//! FDN (Foundations) gap batch — spellslinger token makers, a noncombat-damage
//! draw engine, a Raid punisher, a threshold finisher, an all-types manland, and
//! a distinct-mana-value draw spell. Tests in `tests/recent202.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TokenDefinition, Value, WardCost,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, etb};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
    Selector, StaticEffect, TriggeredAbility,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u};

/// Rite of the Dragoncaller — {4}{R}{R} Enchantment. Whenever you cast an instant
/// or sorcery spell, create a 5/5 red Dragon creature token with flying.
pub fn rite_of_the_dragoncaller() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Rite of the Dragoncaller",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: dragon,
            },
        }],
        ..Default::default()
    }
}

/// Koma, World-Eater — {3}{G}{G}{U}{U} 8/12 Legendary Serpent. Can't be countered,
/// trample, ward {4}. Whenever Koma deals combat damage to a player, create four
/// 3/3 blue Serpent tokens named Koma's Coil.
pub fn koma_world_eater() -> CardDefinition {
    let coil = TokenDefinition {
        name: "Koma's Coil".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Serpent],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Koma, World-Eater",
        cost: cost(&[generic(3), g(), g(), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Serpent],
            ..Default::default()
        },
        power: 8,
        toughness: 12,
        keywords: vec![
            Keyword::CantBeCountered,
            Keyword::Trample,
            Keyword::Ward(WardCost::Mana(cost(&[generic(4)]))),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(4),
                definition: coil,
            },
        }],
        ..Default::default()
    }
}

/// Niv-Mizzet, Visionary — {4}{U}{R} 5/5 Legendary Dragon Wizard. Flying, no
/// maximum hand size. Whenever a source you control deals noncombat damage to an
/// opponent, you draw that many cards.
pub fn niv_mizzet_visionary() -> CardDefinition {
    CardDefinition {
        name: "Niv-Mizzet, Visionary",
        cost: cost(&[generic(4), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PlayerDealtNoncombatDamage,
                EventScope::YourSourceDamagedOpponent,
            ),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Perforating Artist — {1}{B}{R} 3/2 Devil. Deathtouch. Raid — At the beginning
/// of your end step, if you attacked this turn, each opponent loses 3 life unless
/// that player sacrifices a nonland permanent or discards a card.
pub fn perforating_artist() -> CardDefinition {
    CardDefinition {
        name: "Perforating Artist",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::PlayerAttackedThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::You),
                        count: Value::Const(1),
                        filter: R::Nonland,
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::Const(1),
                        random: false,
                    },
                ],
                otherwise: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Kiora, the Rising Tide — {2}{U} 3/2 Legendary Merfolk Noble. When Kiora enters,
/// draw two cards, then discard two cards. Threshold — Whenever Kiora attacks, if
/// there are seven or more cards in your graveyard, you may create Scion of the
/// Deep, a legendary 8/8 blue Octopus creature token.
pub fn kiora_the_rising_tide() -> CardDefinition {
    let scion = TokenDefinition {
        name: "Scion of the Deep".into(),
        power: 8,
        toughness: 8,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Kiora, the Rising Tide",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                    Predicate::ThresholdActive {
                        who: PlayerRef::You,
                    },
                ),
                effect: Effect::MayDo {
                    description: "Create Scion of the Deep.".into(),
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: scion,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Soulstone Sanctuary — Land. {T}: Add {C}. {4}: This land becomes a 3/3 creature
/// with vigilance and all creature types. It's still a land.
pub fn soulstone_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Soulstone Sanctuary",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![],
                    keywords: vec![Keyword::Vigilance, Keyword::Changeling],
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lunar Insight — {2}{U} Sorcery. Draw a card for each different mana value among
/// nonland permanents you control.
pub fn lunar_insight() -> CardDefinition {
    CardDefinition {
        name: "Lunar Insight",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::DistinctManaValuesAmongControlledNonland,
        },
        ..Default::default()
    }
}
