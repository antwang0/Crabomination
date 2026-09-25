//! Commander: the cards the **Death Toll** precon (DSC, Winter, Cynical
//! Opportunist) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Winter** — the engine picks the exiled set (the greatest-mana-value
//!   permanent card plus the cheapest cards covering four card types), and
//!   the finality counter is added as the card enters rather than with it.
//! - **Cemetery Tampering** — a hidden land is put onto the battlefield
//!   rather than played (it doesn't use the land drop).
//! - **Polluted Cistern** — counts milled cards, not every card put into
//!   your graveyard from your library.
//! - **Demonic Covenant** — "attack a player" also fires on an attack at a
//!   planeswalker.
//! - **Into the Pit** — the sacrifice is paid as the cast completes.
//! - **Old Stickfingers** — reveals creature by creature, bottoming each
//!   run of misses before the next.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, RoomDoor,
    RoomDoors, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
    RevealMissDest, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, x};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn token(
    name: &str,
    color: Color,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn make(count: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition: std::sync::Arc::new(def),
    }
}

fn mill(n: i32) -> Effect {
    Effect::Mill { who: Selector::You, amount: Value::Const(n) }
}

fn delirium() -> Predicate {
    Predicate::DeliriumActive { who: PlayerRef::You }
}

fn step(s: TurnStep, scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(s), scope), effect }
}

fn plus_one(what: Selector) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }
}

/// A card with two or more card types (Rendmaw's "a card with two or more
/// card types"), as the pairs a real card can print.
fn multi_typed() -> R {
    let t = R::HasCardType;
    let pair = |a: CardType, b: CardType| t(a).and(t(b));
    pair(CardType::Artifact, CardType::Creature)
        .or(pair(CardType::Enchantment, CardType::Creature))
        .or(pair(CardType::Artifact, CardType::Land))
        .or(pair(CardType::Enchantment, CardType::Artifact))
        .or(pair(CardType::Land, CardType::Creature))
        .or(pair(CardType::Enchantment, CardType::Land))
        .or(pair(CardType::Planeswalker, CardType::Creature))
        .or(pair(CardType::Instant, CardType::Sorcery))
        .or(t(CardType::Kindred))
}

/// Winter, Cynical Opportunist — deathtouch; mills three on attack;
/// delirium end step: exile cards with four card types from your graveyard
/// to return a permanent card from among them with a finality counter.
///
/// Approximation: the engine picks the exiled set.
pub fn winter_cynical_opportunist() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            on_attack(mill(3)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(delirium()),
                effect: Effect::ExileTypeSpreadReturnPermanent { min_types: 4 },
            },
        ],
        ..creature(
            "Winter, Cynical Opportunist",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            2,
            5,
        )
    })
}

/// Carrion Grub — +X/+0 for the greatest power among creature cards in your
/// graveyard; mills four on entry.
pub fn carrion_grub() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +X/+0, where X is the greatest power among creature cards in your graveyard.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::GreatestPowerAmongCards(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Creature,
                })),
                toughness: Value::Const(0),
            },
        }],
        triggered_abilities: vec![etb(mill(4))],
        ..creature("Carrion Grub", cost(&[generic(3), b()]), vec![CreatureType::Insect], 0, 5)
    }
}

/// Cemetery Tampering — hideaway 5; each upkeep you may mill three, then
/// with twenty or more cards in your graveyard play the hidden card free.
///
/// Approximation: a hidden land is put onto the battlefield, not played.
pub fn cemetery_tampering() -> CardDefinition {
    let hidden = || Selector::one_of(Selector::CardExiledWithSource);
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Hideaway { count: Value::Const(5) }),
            step(
                TurnStep::Upkeep,
                EventScope::YourControl,
                Effect::Seq(vec![
                    Effect::MayDo { description: "Mill three cards?".into(), body: Box::new(mill(3)) },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::GraveyardSizeOf(PlayerRef::You),
                            Value::Const(20),
                        ),
                        then: Box::new(Effect::If {
                            cond: Predicate::EntityMatches { what: hidden(), filter: R::Land },
                            then: Box::new(Effect::MayDo {
                                description: "Put the hidden land onto the battlefield?".into(),
                                body: Box::new(Effect::Move {
                                    what: hidden(),
                                    to: ZoneDest::Battlefield {
                                        controller: PlayerRef::You,
                                        tapped: false,
                                    },
                                }),
                            }),
                            else_: Box::new(Effect::CastWithoutPayingImmediate {
                                what: hidden(),
                                source_zone: Zone::Exile,
                                exile_after: false,
                                copy: false,
                                reduce_generic: 0,
                                pay_own_cost: false,
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            ),
        ],
        ..spell("Cemetery Tampering", cost(&[generic(2), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Convert to Slime — destroy up to one artifact, creature and enchantment;
/// delirium: an X/X Ooze, X the destroyed permanents' total mana value.
pub fn convert_to_slime() -> CardDefinition {
    let x = Value::TotalManaValueOf(Box::new(Selector::DestroyedThisResolution { filter: R::Any }));
    let slot = |slot: u8, filter: R| Effect::Destroy {
        what: Selector::TargetFiltered { slot, filter },
    };
    spell(
        "Convert to Slime",
        cost(&[generic(3), b(), g()]),
        CardType::Sorcery,
        Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                slot(0, R::Artifact),
                slot(1, R::Creature),
                slot(2, R::Enchantment),
                Effect::If {
                    cond: delirium(),
                    then: Box::new(make(
                        1,
                        TokenDefinition {
                            dynamic_pt: Some((x.clone(), x)),
                            ..token("Ooze", Color::Green, vec![CreatureType::Ooze], 0, 0, vec![])
                        },
                    )),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
    )
}

/// Deathcap Cultivator — {T}: {B} or {G}; delirium: deathtouch.
pub fn deathcap_cultivator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Delirium — deathtouch with four or more card types in your graveyard.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Deathtouch,
                condition: delirium(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Black, Color::Green], Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Deathcap Cultivator",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            2,
            1,
        )
    }
}

/// Deluge of Doom — all creatures get -X/-X, X the card types in your
/// graveyard.
pub fn deluge_of_doom() -> CardDefinition {
    let x = || Value::Negate(Box::new(Value::CardTypesInGraveyard(PlayerRef::You)));
    spell(
        "Deluge of Doom",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: x(),
            toughness: x(),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Demolisher Spawn — trample, haste; delirium attack: other attackers get
/// +4/+4.
pub fn demolisher_spawn() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(delirium()),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsAttacking).and(R::OtherThanSource),
                ),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Demolisher Spawn", cost(&[generic(5), g(), g()]), vec![CreatureType::Horror], 7, 7)
    }
}

/// Demonic Covenant — Demons attacking draw you a card for 1 life; each end
/// step a 5/5 flying Demon, then mill two, sacrificing it if the two share
/// all their card types.
///
/// Approximation: an attack at a planeswalker also draws.
pub fn demonic_covenant() -> CardDefinition {
    CardDefinition {
        name: "Demonic Covenant",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                    Predicate::AttackedWithCreatureMatching {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Demon),
                    },
                ),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
            step(
                TurnStep::End,
                EventScope::YourControl,
                Effect::Seq(vec![
                    make(
                        1,
                        token("Demon", Color::Black, vec![CreatureType::Demon], 5, 5, vec![
                            Keyword::Flying,
                        ]),
                    ),
                    mill(2),
                    Effect::If {
                        cond: Predicate::TwoShareAllCardTypes(Selector::LastMoved),
                        then: Box::new(Effect::Sacrifice {
                            who: Selector::You,
                            count: Value::ONE,
                            filter: R::IsSource,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Into the Pit — look at your top card any time; cast spells from the top
/// by also sacrificing a nonland permanent.
///
/// Approximation: the sacrifice is paid as the cast completes.
pub fn into_the_pit() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast spells from the top of your library by sacrificing a nonland permanent in addition to paying their other costs.",
                effect: StaticEffect::PlayFromLibraryTopBySacrificing {
                    filter: R::Nonland,
                    sacrifice: R::Permanent.and(R::Nonland),
                },
            },
        ],
        ..spell("Into the Pit", cost(&[generic(2), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Ishkanah, Grafwidow — reach; delirium entry: three 1/2 reach Spiders;
/// {6}{B}: target opponent loses 1 life per Spider you control.
pub fn ishkanah_grafwidow() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(delirium()),
            effect: make(
                3,
                token("Spider", Color::Green, vec![CreatureType::Spider], 1, 2, vec![Keyword::Reach]),
            ),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), b()]),
            effect: Effect::LoseLife {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::PermanentCountControlledByMatching(
                    PlayerRef::You,
                    R::HasCreatureType(CreatureType::Spider),
                ),
            },
            ..Default::default()
        }],
        ..creature("Ishkanah, Grafwidow", cost(&[generic(4), g()]), vec![CreatureType::Spider], 3, 5)
    })
}

/// Moldgraf Monstrosity — trample; on death, exile it and return two
/// creature cards at random from your graveyard.
pub fn moldgraf_monstrosity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                Effect::Move {
                    what: Selector::TakeRandom {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        }),
                        count: Box::new(Value::Const(2)),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
        }],
        ..creature(
            "Moldgraf Monstrosity",
            cost(&[generic(4), g(), g(), g()]),
            vec![CreatureType::Insect],
            8,
            8,
        )
    }
}

/// Obsessive Skinner — a +1/+1 counter on entry; delirium, each opponent's
/// upkeep: another.
pub fn obsessive_skinner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(plus_one(target_filtered(R::Creature))),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::OpponentControl,
                )
                .with_filter(delirium()),
                effect: plus_one(target_filtered(R::Creature)),
            },
        ],
        ..creature(
            "Obsessive Skinner",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Old Stickfingers — cast: reveal until X creature cards, those to your
/// graveyard, the rest to the bottom; */* for the creature cards in your
/// graveyard.
///
/// Approximation: one reveal-until per creature card.
pub fn old_stickfingers() -> CardDefinition {
    legendary(CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusCreaturesInControllerGraveyard { base: 0 }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Repeat {
                count: Value::XFromCost,
                body: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature,
                    to: ZoneDest::Graveyard,
                    cap: Value::Const(60),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                }),
            },
        }],
        ..creature(
            "Old Stickfingers",
            cost(&[x(), b(), g()]),
            vec![CreatureType::Horror],
            0,
            0,
        )
    })
}

/// Polluted Cistern // Dim Oubliette — Cistern: one or more cards put into
/// your graveyard from your library (milled or surveiled) drain each opponent
/// 1 per card type among them. Oubliette: on unlock, mill three, then return
/// a creature card from your graveyard.
pub fn polluted_cistern_dim_oubliette() -> CardDefinition {
    CardDefinition {
        name: "Polluted Cistern // Dim Oubliette",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Room],
            ..Default::default()
        },
        room: Some(Box::new(RoomDoors {
            left: RoomDoor {
                name: "Polluted Cistern".into(),
                cost: cost(&[generic(1), b()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::CardMilled, EventScope::YourControl)
                        .once_per_batch_counting_card_types(),
                    effect: Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::TriggerEventAmount,
                    },
                }],
                ..Default::default()
            },
            right: RoomDoor {
                name: "Dim Oubliette".into(),
                cost: cost(&[generic(4), b()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
                    effect: Effect::Seq(vec![
                        mill(3),
                        Effect::Move {
                            what: Selector::one_of(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Graveyard,
                                filter: R::Creature,
                            }),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                    ]),
                }],
                ..Default::default()
            },
        })),
        ..Default::default()
    }
}

/// Rendmaw, Creaking Nest — reach, menace; on entry and whenever you play a
/// card with two or more card types, each player makes a tapped 2/2 flying
/// Bird goaded for the game.
pub fn rendmaw_creaking_nest() -> CardDefinition {
    let birds = Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::EachPlayer,
            count: Value::ONE,
            definition: std::sync::Arc::new(TokenDefinition {
                tapped: true,
                ..token("Bird", Color::Black, vec![CreatureType::Bird], 2, 2, vec![Keyword::Flying])
            }),
        },
        Effect::GoadForTheGame { what: Selector::LastCreatedTokens },
    ]);
    let played = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl).with_filter(Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: multi_typed(),
        }),
        effect: birds.clone(),
    };
    legendary(CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Reach, Keyword::Menace],
        triggered_abilities: vec![
            etb(birds.clone()),
            played(EventKind::SpellCast),
            played(EventKind::LandPlayed),
        ],
        ..creature(
            "Rendmaw, Creaking Nest",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Scarecrow],
            5,
            5,
        )
    })
}

/// Suspicious Bookcase — defender; {3}, {T}: a creature can't be blocked.
pub fn suspicious_bookcase() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Suspicious Bookcase", cost(&[generic(2)]), vec![CreatureType::Wall], 0, 4)
    }
}

/// Titania, Nature's Force — Forests from your graveyard; a 5/3 Elemental
/// per Forest entering; an Elemental dying may mill three.
pub fn titania_natures_force() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may play Forests from your graveyard.",
            effect: StaticEffect::MayPlayLandsFromGraveyardMatching(R::HasLandType(
                LandType::Forest,
            )),
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasLandType(LandType::Forest),
                    }),
                effect: make(
                    1,
                    token("Elemental", Color::Green, vec![CreatureType::Elemental], 5, 3, vec![]),
                ),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Elemental),
                    }),
                effect: Effect::MayDo { description: "Mill three cards?".into(), body: Box::new(mill(3)) },
            },
        ],
        ..creature(
            "Titania, Nature's Force",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Elemental],
            6,
            6,
        )
    })
}

/// Wrenn and Seven — +1: lands from the top four to hand, the rest to the
/// graveyard; 0: any lands from hand tapped; −3: a reach Treefolk as big as
/// your land count; −8: permanent cards back to hand, no maximum hand size.
pub fn wrenn_and_seven() -> CardDefinition {
    let lands = || Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land);
    CardDefinition {
        name: "Wrenn and Seven",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Wrenn],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::RevealTopTakeMatchingRestToGraveyard {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    filter: R::Land,
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::PutLandsFromHandOntoBattlefieldTapped {
                    count: Value::HandSizeOf(PlayerRef::You),
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: make(
                    1,
                    TokenDefinition {
                        static_abilities: vec![StaticAbility {
                            description: "This token's power and toughness are each equal to the number of lands you control.",
                            effect: StaticEffect::SelfBasePtFromValue { power: lands(), toughness: lands() },
                        }],
                        ..token("Treefolk", Color::Green, vec![CreatureType::Treefolk], 0, 0, vec![
                            Keyword::Reach,
                        ])
                    },
                ),
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::PermanentCard,
                        },
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Wrenn and Seven".into(),
                        triggered: vec![],
                        statics: vec![StaticAbility {
                            description: "You have no maximum hand size.",
                            effect: StaticEffect::NoMaximumHandSize,
                        }],
                    },
                ]),
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}
