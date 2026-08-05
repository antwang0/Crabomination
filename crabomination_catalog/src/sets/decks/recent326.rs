//! Aetherdrift (DFT) build-around gaps — the legends, the Vehicles and the
//! exhaust/speed payoffs. Tests in `tests/recent_b/recent326.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, DynamicPt, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, SpendRestriction, b, cost, g, generic, r, u, w, x};

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

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, c, types, p, t)
    }
}

fn vehicle(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Wickerfolk Indomitable — {3}{B} 4/3 Scarecrow that keeps coming back: cast
/// it from your graveyard for 2 life plus an artifact or creature.
pub fn wickerfolk_indomitable() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::GraveyardCast],
        flashback_additional_cost: vec![
            AdditionalCastCost::PayLife { amount: 2 },
            AdditionalCastCost::SacrificePermanent {
                filter: R::Artifact.or(R::Creature).and(R::ControlledByYou),
                count: 1,
            },
        ],
        ..creature(
            "Wickerfolk Indomitable",
            cost(&[generic(3), b()]),
            vec![CreatureType::Scarecrow],
            4,
            3,
        )
    }
}

/// Daretti, Rocketeer Engineer — {4}{R} */5 whose power is your biggest
/// artifact; entering or attacking rebuys one from the graveyard for another.
pub fn daretti_rocketeer_engineer() -> CardDefinition {
    let rebuy = || Effect::MaySacrifice {
        description: "Sacrifice an artifact to return the chosen card?".into(),
        filter: R::Artifact.and(R::ControlledByYou),
        count: Value::ONE,
        then: Box::new(Effect::Move {
            what: Selector::TargetFiltered { slot: 0, filter: R::Artifact },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        }),
        else_: None,
    };
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusGreatestOtherArtifactMv { base_p: 0, base_t: 5 }),
        triggered_abilities: vec![etb(rebuy()), on_attack(rebuy())],
        ..legend(
            "Daretti, Rocketeer Engineer",
            cost(&[generic(4), r()]),
            vec![CreatureType::Goblin, CreatureType::Artificer],
            0,
            5,
        )
    }
}

/// Mendicant Core, Guidelight — {W}{U} */3 Robot; at max speed your artifact
/// spells can be copied for {1}.
pub fn mendicant_core_guidelight() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 0, base_t: 3 }),
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact,
                    },
                    Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                ]),
            ),
            effect: Effect::MayPay {
                description: "Pay {1} to copy that artifact spell?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::CopySpell {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                }),
                else_: None,
            },
        }],
        ..legend(
            "Mendicant Core, Guidelight",
            cost(&[w(), u()]),
            vec![CreatureType::Robot],
            0,
            3,
        )
    }
}

/// Oviya, Automech Artisan — {3}{G} 1/2; your attackers trample and {G},{T}
/// drops a creature or Vehicle from hand (artifacts arrive with two counters).
pub fn oviya_automech_artisan() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature that's attacking one of your opponents has trample.",
            effect: StaticEffect::GrantKeywordToAttackers { keyword: Keyword::Trample },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: Some(Box::new(Effect::AddCounter {
                    what: Selector::MatchingAmong {
                        inner: Box::new(Selector::LastMoved),
                        filter: R::Artifact,
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                })),
            },
            ..Default::default()
        }],
        ..legend(
            "Oviya, Automech Artisan",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            2,
        )
    }
}

/// Sita Varma, Masked Racer — {G}{U} 2/3. Exhaust: grow by X, then optionally
/// flatten every other creature you control to her power.
pub fn sita_varma_masked_racer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), g(), g(), u()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
                Effect::MayDo {
                    description: "Set your other creatures' base P/T to Sita Varma's power?".into(),
                    body: Box::new(Effect::SetBasePT {
                        what: Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou).and(R::Not(Box::new(R::IsSource))),
                        ),
                        power: Value::PowerOf(Box::new(Selector::This)),
                        toughness: Value::PowerOf(Box::new(Selector::This)),
                        duration: Duration::EndOfTurn,
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Sita Varma, Masked Racer",
            cost(&[g(), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            3,
        )
    }
}

/// Winter, Cursed Rider — {U}{B} 3/2. Ward—Pay 2 life, shared with your
/// artifacts; exhaust exiles artifacts from your graveyard for a board sweep.
pub fn winter_cursed_rider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Life(2))],
        static_abilities: vec![StaticAbility {
            description: "Artifacts you control have \"Ward—Pay 2 life.\"",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                keyword: Keyword::Ward(WardCost::Life(2)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), b()]),
            tap_cost: true,
            exhaust: true,
            exile_other_filter: Some((R::Artifact, 1)),
            exile_other_x: true,
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::Not(Box::new(R::Artifact)))
                        .and(R::Not(Box::new(R::IsSource))),
                ),
                power: Value::Negate(Box::new(Value::XFromCost)),
                toughness: Value::Negate(Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Winter, Cursed Rider",
            cost(&[u(), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            2,
        )
    }
}

/// Redshift, Rocketeer Chief — {R}{G} 2/3 vigilance. Taps for power-many
/// ability mana; exhaust dumps your hand's permanents onto the battlefield.
pub fn redshift_rocketeer_chief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::PowerOf(Box::new(
                            Selector::This,
                        )))),
                        SpendRestriction::AbilitiesOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(10), r(), g()]),
                exhaust: true,
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Permanent,
                    count: Value::Const(99),
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Redshift, Rocketeer Chief",
            cost(&[r(), g()]),
            vec![CreatureType::Goblin, CreatureType::Pilot],
            2,
            3,
        )
    }
}

/// Demonic Junker — {6}{B} 4/3 Vehicle with affinity for artifacts; its ETB
/// sweeps one creature per player and grows on your own casualty.
pub fn demonic_junker() -> CardDefinition {
    let mine = Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) };
    let theirs =
        Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) };
    CardDefinition {
        affinity_filter: Some(R::Artifact.and(R::ControlledByYou)),
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Destroy { what: theirs },
                Effect::If {
                    cond: Predicate::SelectorExists(mine.clone()),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Destroy { what: mine },
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(2),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..vehicle("Demonic Junker", cost(&[generic(6), b()]), 4, 3)
    }
}

/// Rise from the Wreck — {2}{G} Sorcery. Four separately-filtered graveyard
/// slots, each optional.
pub fn rise_from_the_wreck() -> CardDefinition {
    let slot = |n: u8, filter: R| Effect::Move {
        what: Selector::TargetFiltered { slot: n, filter },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Rise from the Wreck",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                slot(0, R::Creature),
                slot(1, R::HasCreatureType(CreatureType::Mount)),
                slot(2, R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                slot(3, R::Creature.and(R::HasNoAbilities)),
            ])),
        },
        ..Default::default()
    }
}

/// Riptide Gearhulk — {1}{W}{W}{U}{U} 2/5 double strike prowess; its ETB
/// buries one nonland permanent per opponent third from the top.
pub fn riptide_gearhulk() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::DoubleStrike, Keyword::Prowess],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Not(Box::new(R::Land)).and(R::ControlledByOpponent),
                },
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::FromTop(2),
                },
            }),
        })],
        ..creature(
            "Riptide Gearhulk",
            cost(&[generic(1), w(), w(), u(), u()]),
            vec![CreatureType::Construct],
            2,
            5,
        )
    }
}

/// Radiant Lotus — {6} Artifact. Sacrifice artifacts to hand a player three
/// mana of one color for each.
pub fn radiant_lotus() -> CardDefinition {
    CardDefinition {
        name: "Radiant Lotus",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_any_number_filter: Some(R::Artifact.and(R::ControlledByYou)),
            effect: Effect::AddMana {
                who: PlayerRef::Target(0),
                pool: ManaPayload::AnyOneColor(Value::Times(
                    Box::new(Value::SacrificedCount),
                    Box::new(Value::Const(3)),
                )),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skyseer's Chariot — {1}{W} 3/3 flying Vehicle that taxes every activated
/// ability of the card name it names on the way in.
pub fn skyseers_chariot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        as_enters_effect: Some(Effect::NameCard {
            what: Selector::This,
            restrict_to: Some(R::Not(Box::new(R::Land))),
        }),
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of sources with the chosen name cost {2} more to \
                          activate.",
            effect: StaticEffect::NamedSourcesActivationTax { amount: 2 },
        }],
        ..vehicle("Skyseer's Chariot", cost(&[generic(1), w()]), 3, 3)
    }
}

/// Push the Limit — {5}{R}{R} Sorcery. Every Mount and Vehicle in your
/// graveyard comes back hasty for one attack.
pub fn push_the_limit() -> CardDefinition {
    CardDefinition {
        name: "Push the Limit",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ReturnAllMatchingFromGraveyardToBattlefield {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Mount)
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                sacrifice_eot: true,
            },
            Effect::AnimateAsCreature {
                what: Selector::EachPermanent(
                    R::HasArtifactSubtype(ArtifactSubtype::Vehicle).and(R::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Lifecraft Engine — {3} 4/4 Vehicle. Names a creature type on the way in;
/// your Vehicles join it, and everything else of that type grows.
pub fn lifecraft_engine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Crew(3)],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                // Scoped to your Vehicles rather than your *crewed* Vehicles:
                // the layer walker reads printed types. Nothing is visible in
                // play — the anthem below only reaches creatures anyway.
                description: "Vehicle creatures you control are the chosen creature type in \
                              addition to their other types.",
                effect: StaticEffect::MatchingAreChosenTypeToo {
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
                        .and(R::ControlledByYou),
                },
            },
            StaticAbility {
                description: "Each creature you control of the chosen type other than this \
                              Vehicle gets +1/+1.",
                effect: StaticEffect::AnthemForChosenType {
                    power: 1,
                    toughness: 1,
                    exclude_source: true,
                    opponents: false,
                    all_players: false,
                    per_counter: None,
                },
            },
        ],
        ..vehicle("Lifecraft Engine", cost(&[generic(3)]), 4, 4)
    }
}

/// Cursecloth Wrappings — {2}{B}{B} Artifact. A Zombie anthem that hands your
/// graveyard's creatures embalm at their own mana cost.
pub fn cursecloth_wrappings() -> CardDefinition {
    CardDefinition {
        name: "Cursecloth Wrappings",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Zombies you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Zombie),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantEmbalmThisTurn {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Samut, the Driving Force — {3}{R}{G}{W} 4/5. Your speed pumps the rest of
/// the team and discounts your noncreature spells.
pub fn samut_the_driving_force() -> CardDefinition {
    let speed = || Value::PlayerSpeed(PlayerRef::You);
    CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Vigilance,
            Keyword::Haste,
            Keyword::StartYourEngines,
        ],
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures you control get +X/+0, where X is your speed.",
                effect: StaticEffect::PumpPTByValue {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    power: speed(),
                    toughness: Value::Const(0),
                },
            },
            StaticAbility {
                description: "Noncreature spells you cast cost {X} less to cast, where X is \
                              your speed.",
                effect: StaticEffect::CostReductionByValue {
                    filter: R::Not(Box::new(R::Creature)),
                    amount: speed(),
                },
            },
        ],
        ..legend(
            "Samut, the Driving Force",
            cost(&[generic(3), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Cleric],
            4,
            5,
        )
    }
}

/// A 1/1 colorless Pilot that crews and saddles as though its power were 2
/// greater — Aetherdrift's Pilot token.
fn pilot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pilot],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This token saddles Mounts and crews Vehicles as though its power \
                          were 2 greater.",
            effect: StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
        }],
        ..Default::default()
    }
}

/// Valor's Flagship — {4}{W}{W}{W} 7/7 Vehicle. Cycle it for {X}{2}{W} and X
/// Pilots show up to crew whatever is left.
pub fn valors_flagship() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::Flying,
            Keyword::FirstStrike,
            Keyword::Lifelink,
            Keyword::Crew(3),
            Keyword::Cycling(cost(&[x(), generic(2), w()])),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: pilot_token(),
            },
        }],
        ..vehicle("Valor's Flagship", cost(&[generic(4), w(), w(), w()]), 7, 7)
    }
}

/// A 3/2 colorless Vehicle with crew 1 — Chandra, Spark Hunter's 0.
fn spark_hunter_vehicle_token() -> TokenDefinition {
    TokenDefinition {
        name: "Vehicle".into(),
        power: 3,
        toughness: 2,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        keywords: vec![Keyword::Crew(1)],
        ..Default::default()
    }
}

/// Chandra, Spark Hunter — {3}{R} loyalty 4. Crews for free each combat, +2
/// rummages off an artifact or card, 0 builds a Vehicle, −7 turns artifacts
/// into Bolts.
pub fn chandra_spark_hunter() -> CardDefinition {
    CardDefinition {
        name: "Chandra, Spark Hunter",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Chandra],
            ..Default::default()
        },
        base_loyalty: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::AnimateAsCreature {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
                                .and(R::ControlledByYou),
                        },
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        }],
        loyalty_abilities: vec![
            crate::card::LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::MaySacrifice {
                    description: "Sacrifice an artifact to draw a card?".into(),
                    filter: R::Artifact.and(R::ControlledByYou),
                    count: Value::ONE,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Some(Box::new(Effect::MayDo {
                        description: "Discard a card to draw a card?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::Discard {
                                who: Selector::You,
                                amount: Value::ONE,
                                random: false,
                            },
                            Effect::Draw { who: Selector::You, amount: Value::ONE },
                        ])),
                    })),
                },
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: spark_hunter_vehicle_token(),
                },
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Chandra, Spark Hunter".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::EntersBattlefield,
                            EventScope::YourControl,
                        )
                        .with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Artifact,
                        }),
                        effect: Effect::DealDamage {
                            to: Selector::TargetFiltered { slot: 0, filter: R::Any },
                            amount: Value::Const(3),
                        },
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ancient Vendetta — {3}{B} Sorcery. Name a card and strip four copies out of
/// an opponent's graveyard, hand and library.
pub fn ancient_vendetta() -> CardDefinition {
    CardDefinition {
        name: "Ancient Vendetta",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::NameCardThenExileFromZones { who: PlayerRef::Target(0), count: 4 },
        ..Default::default()
    }
}

/// Ketramose, the New Dawn — {1}{W}{B} 4/4 God. Locked out of combat until
/// exile is seven deep; every exile off a graveyard or the battlefield on your
/// turn draws you a card.
pub fn ketramose_the_new_dawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Menace,
            Keyword::Lifelink,
            Keyword::Indestructible,
            Keyword::CantAttackOrBlockUnlessCardsInExile(7),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CardExiledFromPlayOrGraveyard,
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..legend(
            "Ketramose, the New Dawn",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::God],
            4,
            4,
        )
    }
}

/// Captain Howler, Sea Scourge — {2}{U}{R} 5/4. Ward—{2}, Pay 2 life. Every
/// discard pumps a creature and turns it into a cantrip on connection.
pub fn captain_howler_sea_scourge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::ManaAndLife(cost(&[generic(2)]), 2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                    power: Value::Times(
                        Box::new(Value::TriggerEventAmount),
                        Box::new(Value::Const(2)),
                    ),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::WhenTargetDealsCombatDamageToPlayerThisTurn {
                    slot: 0,
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ]),
        }],
        ..legend(
            "Captain Howler, Sea Scourge",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Shark, CreatureType::Pirate],
            5,
            4,
        )
    }
}

/// The Aetherspark — {4} legendary Equipment planeswalker, loyalty 4. It
/// equips itself off the +1, grows on combat damage, and ultimates into mana.
pub fn the_aetherspark() -> CardDefinition {
    CardDefinition {
        name: "The Aetherspark",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Planeswalker],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        base_loyalty: 4,
        // The "can't be attacked while attached" half needs a planeswalker
        // attack restriction the engine doesn't model; everything else is here.
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: [
                EventKind::DealsCombatDamageToPlayer,
                EventKind::DealsCombatDamageToCreature,
            ]
            .map(|kind| TriggeredAbility {
                event: EventSpec::new(kind, EventScope::SelfSource)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::AddCounter {
                    what: Selector::AttachmentGranting,
                    kind: CounterType::Loyalty,
                    amount: Value::TriggerEventAmount,
                },
            })
            .to_vec(),
            ..Default::default()
        }),
        loyalty_abilities: vec![
            crate::card::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::AttachSourceTo {
                            host: Selector::TargetFiltered {
                                slot: 0,
                                filter: R::Creature.and(R::ControlledByYou),
                            },
                        },
                        Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                    ])),
                },
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(10)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Webstrike Elite — {G}{G} 3/3 reach. Cycle it for {X}{G}{G} to blow up an
/// artifact or enchantment with mana value X.
pub fn webstrike_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Cycling(cost(&[x(), g(), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Destroy {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Artifact
                            .or(R::Enchantment)
                            .and(R::ManaValueEqualsTriggerAmount),
                    },
                }),
            },
        }],
        ..creature(
            "Webstrike Elite",
            cost(&[g(), g()]),
            vec![CreatureType::Insect, CreatureType::Archer],
            3,
            3,
        )
    }
}

/// Pit Automaton — {2} 0/4 defender. Taps for ability-only mana; {2},{T} arms
/// a copy of your next exhaust activation.
pub fn pit_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::Const(2))),
                        SpendRestriction::AbilitiesOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::OnYourNextExhaustActivationThisTurn {
                    body: Box::new(Effect::CopyActivatedAbilityMayChooseTargets),
                },
                ..Default::default()
            },
        ],
        ..creature("Pit Automaton", cost(&[generic(2)]), vec![CreatureType::Construct], 0, 4)
    }
}
