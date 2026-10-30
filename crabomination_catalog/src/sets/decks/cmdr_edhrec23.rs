//! Commander: the 2026-10-13 rescan's one-card gaps that needed more than a
//! catalog entry — Raised by Giants, Kamahl's Druidic Vow, Demonic Embrace
//! and friends (`cmdr_edhrec22` is batch A). Tests in
//! `tests/recent_b/cmdr_edhrec23.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LookPick, PlayerRef};
use crate::mana::{b, cost, g, generic, r, x, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn your_commanders() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::IsCommander).and(R::OwnedByYou))
}

/// Raised by Giants — {5}{G} legendary enchantment — Background. Commander
/// creatures you own have base power and toughness 10/10 and are Giants.
pub fn raised_by_giants() -> CardDefinition {
    CardDefinition {
        name: "Raised by Giants",
        cost: cost(&[generic(5), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![
            StaticAbility {
                description: "Commander creatures you own have base power and toughness 10/10.",
                effect: StaticEffect::SetBasePtForFilter { applies_to: your_commanders(), power: 10, toughness: 10 },
            },
            StaticAbility {
                description: "Commander creatures you own are Giants in addition to their other types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: your_commanders(),
                    creature_type: CreatureType::Giant,
                },
            },
        ],
        ..Default::default()
    }
}

/// Kamahl's Druidic Vow — {X}{G}{G} legendary sorcery. Look at the top X;
/// put any number of land and/or legendary permanent cards with mana value X
/// or less onto the battlefield; the rest into your graveyard.
pub fn kamahls_druidic_vow() -> CardDefinition {
    let legendary_permanent = R::PermanentCard.and(R::HasSupertype(Supertype::Legendary));
    CardDefinition {
        name: "Kamahl's Druidic Vow",
        cost: cost(&[x(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::XFromCost,
            take: Some(Value::XFromCost),
            pick_filter: Some(R::Land.or(legendary_permanent).and(R::ManaValueAtMostXFromCost)),
            to_battlefield: true,
            rest_to_graveyard: true,
            optional: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Demonic Embrace — {1}{B}{B} Aura. Enchanted creature gets +3/+1, has
/// flying, and is a Demon too. You may cast it from your graveyard by paying
/// 3 life and discarding a card in addition to its other costs.
pub fn demonic_embrace() -> CardDefinition {
    CardDefinition {
        name: "Demonic Embrace",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            add_creature_types: vec![CreatureType::Demon],
            ..Default::default()
        }),
        static_abilities: vec![
            StaticAbility {
                description: "You may cast this card from your graveyard by paying 3 life and discarding a card in addition to paying its other costs.",
                effect: StaticEffect::CastSelfFromGraveyardWithCost(crate::card::AdditionalCastCost::PayLife { amount: 3 }),
            },
            StaticAbility {
                description: "Casting it from your graveyard also discards a card.",
                effect: StaticEffect::CastSelfFromGraveyardWithCost(crate::card::AdditionalCastCost::Discard { count: 1, filter: None }),
            },
        ],
        ..Default::default()
    }
}

/// Trepanation Blade — {3} Equipment. Whenever equipped creature attacks,
/// the defending player reveals until a land and bins it all; the creature
/// gets +1/+0 per card revealed. Equip {2}.
pub fn trepanation_blade() -> CardDefinition {
    CardDefinition {
        name: "Trepanation Blade",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::MillUntilLands { who: Selector::Player(PlayerRef::DefendingPlayer), lands: Value::ONE },
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::CardsMilledThisEffectMatching { filter: R::Any },
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                ]),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Kraven the Hunter — {1}{B}{G} 4/3 legendary Human Warrior Villain,
/// trample. Whenever a creature an opponent controls with the greatest power
/// among creatures that player controls dies, draw a card and put a +1/+1
/// counter on Kraven.
pub fn kraven_the_hunter() -> CardDefinition {
    let died = || Selector::TriggerSource;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl).with_filter(
                Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(died())),
                    Value::GreatestPowerControlled { who: PlayerRef::ControllerOf(Box::new(died())) },
                ),
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..creature(
            "Kraven the Hunter",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Villain],
            4,
            3,
        )
    }
}

/// Display of Power — {1}{R}{R} instant; can't be copied. Copy any number of
/// target instant and/or sorcery spells; you may choose new targets.
pub fn display_of_power() -> CardDefinition {
    CardDefinition {
        name: "Display of Power",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCopied],
        effect: Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            effect: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::Target(0), count: Value::ONE }),
        },
        ..Default::default()
    }
}

/// Fires of Invention — {3}{R} enchantment. You can cast spells only during
/// your turn and no more than two spells each turn. You may cast spells with
/// mana value up to the number of lands you control without paying their
/// mana costs.
pub fn fires_of_invention() -> CardDefinition {
    CardDefinition {
        name: "Fires of Invention",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "You can cast spells only during your turn and you can cast no more than two spells each turn.",
                effect: StaticEffect::ControllerCastsOnlyOnTurnUpTo { max: 2 },
            },
            StaticAbility {
                description: "You may cast spells with mana value less than or equal to the number of lands you control without paying their mana costs.",
                effect: StaticEffect::CastFilteredSpellsFree {
                    filter: R::ManaValueAtMostYourCount(Box::new(R::Land)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Interdimensional Web Watch — {4} artifact. ETB: exile the top two cards
/// of your library; you may play them until the end of your next turn. {T}:
/// add two mana in any combination of colors, spent only on spells cast from
/// exile.
pub fn interdimensional_web_watch() -> CardDefinition {
    use crate::card::{ActivatedAbility, MayPlayDuration};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Interdimensional Web Watch",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyColors(Value::Const(2))),
                    crate::mana::SpendRestriction::SpellFromExile,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Parker Luck — {2}{B} enchantment. At the beginning of your end step, two
/// target players each reveal their top card, each loses life equal to the
/// other's card's mana value, then each puts their card into their hand.
pub fn parker_luck() -> CardDefinition {
    use crate::effect::ZoneDest;
    let slot = |n: u8| {
        let filter = if n == 0 { R::Player } else { R::Player.and(R::OtherThanTargetSlot(0)) };
        Selector::TargetFiltered { slot: n, filter }
    };
    let seat = |n: u8| PlayerRef::ControllerOf(Box::new(slot(n)));
    let top_mv = |n: u8| {
        Value::ManaValueOf(Box::new(Selector::TopOfLibrary { who: seat(n), count: Value::ONE }))
    };
    let to_hand = |n: u8| Effect::Move {
        what: Selector::TopOfLibrary { who: seat(n), count: Value::ONE },
        to: ZoneDest::Hand(seat(n)),
    };
    CardDefinition {
        name: "Parker Luck",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::RevealTopOfLibrary { who: seat(0) },
                Effect::RevealTopOfLibrary { who: seat(1) },
                Effect::LoseLife { who: slot(0), amount: top_mv(1) },
                Effect::LoseLife { who: slot(1), amount: top_mv(0) },
                to_hand(0),
                to_hand(1),
            ]),
        }],
        ..Default::default()
    }
}

/// Vaevictis Asmadi, the Dire — {3}{B}{R}{G} 6/6 legendary Elder Dragon,
/// flying. Whenever it attacks, for each player, choose target permanent that
/// player controls; they sacrifice it, then reveal their top card and put it
/// onto the battlefield if it's a permanent card.
pub fn vaevictis_asmadi_the_dire() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::ForEachPlayerTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Permanent,
                effect: Box::new(Effect::Seq(vec![
                    Effect::SacrificeSelected { what: Selector::Target(0) },
                    Effect::EachPlayerDoes {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        body: Box::new(Effect::RevealTopDeployIfMatch {
                            filter: R::PermanentCard,
                            haste: false,
                            sacrifice_at_next_end_step: false,
                            miss_to_graveyard: false,
                            tapped: false,
                        }),
                    },
                ])),
            }),
        })],
        ..creature(
            "Vaevictis Asmadi, the Dire",
            cost(&[generic(3), b(), r(), g()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Desert Were-Worm — {4}{R}{R} 0/5 Dragon Wurm. +2/+0 per Mountain you
/// control. The first time each turn you attack with total power 12 or more,
/// untap the attackers; an additional combat phase follows this one.
pub fn desert_were_worm() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+0 for each Mountain you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::HasLandType(crate::card::LandType::Mountain),
                per_power: 2,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                    Predicate::AttackedWithTotalPowerAtLeast { who: PlayerRef::You, at_least: 12 },
                )
            },
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: Selector::EachPermanent(R::IsAttacking.and(R::ControlledByYou)),
                    up_to: None,
                },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..creature("Desert Were-Worm", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon, CreatureType::Wurm], 0, 5)
    }
}

/// Thicket Crasher — {3}{G} 4/3 Elemental Rhino, trample. Other Elementals
/// you control have trample.
pub fn thicket_crasher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other Elementals you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Elemental).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..creature("Thicket Crasher", cost(&[generic(3), g()]), vec![CreatureType::Elemental, CreatureType::Rhino], 4, 3)
    }
}

/// Thorn Mammoth — {5}{G}{G} 6/6 Elephant, trample. Whenever it or another
/// creature you control enters, it fights up to one target creature you don't
/// control.
pub fn thorn_mammoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Fight {
                    attacker: Selector::This,
                    defender: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                }),
            },
        }],
        ..creature("Thorn Mammoth", cost(&[generic(5), g(), g()]), vec![CreatureType::Elephant], 6, 6)
    }
}

/// Elminster's Simulacrum — {4}{U}{U} instant. For each opponent, a token
/// copy of up to one target creature that player controls.
pub fn elminsters_simulacrum() -> CardDefinition {
    CardDefinition {
        name: "Elminster's Simulacrum",
        cost: cost(&[generic(4), crate::mana::u(), crate::mana::u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::Target(0),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                }),
            }),
        },
        ..Default::default()
    }
}
