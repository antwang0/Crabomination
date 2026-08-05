//! The BLB/DSK/OTJ one-primitive backlog — each card here was blocked on a
//! single engine primitive. Tests in `tests/recent_b/recent329.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    LoyaltyAbility,
    CreatureType, Keyword, PlaneswalkerSubtype, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{eerie, etb, on_attack, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, EventKind, EventScope, EventSpec, MillShareAxis,
    PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{cost, b, generic, r, u, w, x};

fn legend(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Victor, Valgavoth's Seneschal — {1}{W}{B} 3/3. Eerie escalates: surveil 2,
/// then an opponent discard, then a reanimation.
pub fn victor_valgavoths_seneschal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: eerie(Effect::NthResolutionThisTurn {
            branches: vec![
                Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                    random: false,
                },
                Effect::PutGraveyardCardOntoBattlefield { filter: R::Creature },
            ],
        }),
        ..legend(
            "Victor, Valgavoth's Seneschal",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Alania, Divergent Storm — {3}{U}{R} 3/5. The turn's first instant, sorcery
/// or other Otter spell may be copied by gifting an opponent a card.
pub fn alania_divergent_storm() -> CardDefinition {
    let first = |f: R| Predicate::CastSpellFirstMatchingThisTurn(f);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::Any(vec![
                    first(R::HasCardType(CardType::Instant)),
                    first(R::HasCardType(CardType::Sorcery)),
                    first(
                        R::HasCreatureType(CreatureType::Otter)
                            .and(R::HasName("Alania, Divergent Storm".into()).negate()),
                    ),
                ]),
            ),
            effect: Effect::MayDo {
                description: "Have target opponent draw a card and copy that spell?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: crate::effect::shortcut::target_filtered(R::OpponentPlayer),
                        amount: Value::ONE,
                    },
                    Effect::CopySpellMayChooseTargets {
                        what: Selector::TriggerSource,
                        count: Value::ONE,
                    },
                ])),
            },
        }],
        ..legend(
            "Alania, Divergent Storm",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Otter, CreatureType::Wizard],
            3,
            5,
        )
    }
}

/// Heirloom Epic — {1} Book. Its {4} draw can be convoked away by tapping
/// creatures.
pub fn heirloom_epic() -> CardDefinition {
    CardDefinition {
        name: "Heirloom Epic",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            sorcery_speed: true,
            convoke: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eriette, the Beguiler — {1}{W}{U}{B} 4/4 lifelink. An Aura landing on a
/// small enough opposing permanent steals it for as long as it stays attached.
pub fn eriette_the_beguiler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AuraAttachedToAny, EventScope::YourControl)
                .with_filter(Predicate::AuraHostIsCheaperOpponentPermanent),
            effect: Effect::GainControlWhileTriggerAuraAttached,
        }],
        ..legend(
            "Eriette, the Beguiler",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            4,
            4,
        )
    }
}

/// Rottenmouth Viper — {5}{B} 6/6. Sacrificing on the way down makes it cheap;
/// each blight counter then squeezes every opponent.
pub fn rottenmouth_viper() -> CardDefinition {
    let squeeze = Effect::EachPlayerDoes {
        who: PlayerRef::EachOpponent,
        body: Box::new(Effect::MaySacrifice {
            description: "Sacrifice a nonland permanent to Rottenmouth Viper?".into(),
            filter: R::Nonland.and(R::ControlledByYou),
            count: Value::ONE,
            then: Box::new(Effect::Noop),
            else_: Some(Box::new(Effect::MayDiscard {
                description: "Discard a card instead?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                })),
            })),
        }),
    };
    let body = || {
        Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Blight,
                amount: Value::ONE,
            },
            Effect::Repeat {
                count: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Blight,
                },
                body: Box::new(squeeze.clone()),
            },
        ])
    };
    CardDefinition {
        name: "Rottenmouth Viper",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Snake],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        additional_cast_cost: vec![AdditionalCastCost::SacrificeAnyNumber { filter: R::Nonland }],
        self_cost_reduction_per_sacrificed: true,
        triggered_abilities: vec![etb(body()), on_attack(body())],
        ..Default::default()
    }
}

/// Portent of Calamity — {X}{U} sorcery. Reveal X, keep one card of each type,
/// and cast one for free if you kept four.
pub fn portent_of_calamity() -> CardDefinition {
    CardDefinition {
        name: "Portent of Calamity",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealTopExileOnePerCardType {
            count: Value::XFromCost,
            free_cast_at: 4,
        },
        ..Default::default()
    }
}

/// An enchantment token that cashes itself in for a scry-and-draw.
fn shard_token() -> TokenDefinition {
    TokenDefinition {
        name: "Shard".into(),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Niko, Light of Hope — {2}{W}{U} 3/4. Two Shards on entry, and a blink that
/// turns every Shard into a copy of the blinked creature.
pub fn niko_light_of_hope() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: shard_token(),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::BecomeCopyOfFor {
                    what: Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::HasName("Shard".into()),
                    },
                    source: Selector::Target(0),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                Effect::Exile {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::HasSupertype(Supertype::Legendary).negate()),
                    ),
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                            tapped: false,
                        },
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Niko, Light of Hope",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Wishing Well — {3}{U} artifact. Each coin counter raises the mana value it
/// can flash back out of your graveyard for free.
pub fn wishing_well() -> CardDefinition {
    CardDefinition {
        name: "Wishing Well",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Coin,
                    amount: Value::ONE,
                },
                Effect::Reflexive {
                    body: Box::new(Effect::CastWithoutPayingImmediate {
                        what: target_filtered(
                            R::InYourGraveyard
                                .and(
                                    R::HasCardType(CardType::Instant)
                                        .or(R::HasCardType(CardType::Sorcery)),
                                )
                                .and(R::ManaValueEqualsCountersOnSource(CounterType::Coin)),
                        ),
                        source_zone: crate::card::Zone::Graveyard,
                        exile_after: true,
                        copy: false,
                        reduce_generic: 0,
                                pay_own_cost: false,
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Valgavoth, Terror Eater — {6}{B}{B}{B} 9/9. Opponents' cards go to his
/// exile pile instead of their graveyards, and you can play them for life.
pub fn valgavoth_terror_eater() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Lifelink,
            Keyword::Ward(WardCost::SacrificeMatchingN(Box::new(R::Nonland), 3)),
        ],
        static_abilities: vec![
            StaticAbility {
                description: "If a card you didn't control would be put into an opponent's graveyard from anywhere, exile it instead.",
                effect: StaticEffect::ExileCardsBoundForGraveyard {
                    opponents_only: true,
                    own_only: false,
                    colors: None,
                    card_types: None,
                    void_counter: false,
                    stamp_source: true,
                },
            },
            StaticAbility {
                description: "During your turn, you may play cards exiled with this. If you cast a spell this way, pay life equal to its mana value rather than pay its mana cost.",
                effect: StaticEffect::PlayExiledWithSourceForLife,
            },
        ],
        ..legend(
            "Valgavoth, Terror Eater",
            cost(&[generic(6), b(), b(), b()]),
            vec![CreatureType::Elder, CreatureType::Demon],
            9,
            9,
        )
    }
}

/// Osteomancer Adept — {1}{B} 2/2 deathtouch. Tap it and your graveyard's
/// creatures become castable for a forage, arriving with a finality counter.
pub fn osteomancer_adept() -> CardDefinition {
    CardDefinition {
        name: "Osteomancer Adept",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantForageGraveyardCreatureCastsThisTurn,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Tale of Tamiyo — {2}{U} Saga. Three chapters of type-matched milling,
/// then a graveyard flashback of instants, sorceries and Tamiyos.
pub fn the_tale_of_tamiyo() -> CardDefinition {
    let mill = Effect::MillTwoRepeatSharing {
        who: Selector::You,
        axis: MillShareAxis::CardType,
        draw_on_repeat: true,
    };
    CardDefinition {
        name: "The Tale of Tamiyo",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, mill.clone()),
            (2, mill.clone()),
            (3, mill),
            (
                4,
                Effect::ApplyToTargets {
                    max_targets: 4,
                    min_targets: 0,
                    filter: R::InYourGraveyard.and(
                        R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .or(R::HasPlaneswalkerType(PlaneswalkerSubtype::Tamiyo)),
                    ),
                    effect: Box::new(Effect::CastWithoutPayingImmediate {
                        what: Selector::Target(0),
                        source_zone: crate::card::Zone::Graveyard,
                        exile_after: false,
                        copy: true,
                        reduce_generic: 0,
                        pay_own_cost: true,
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Kaito, Bane of Nightmares — {2}{U}{B} Kaito with ninjutsu; on your turn he
/// is a hexproof 3/4 Ninja while he still has loyalty.
pub fn kaito_bane_of_nightmares() -> CardDefinition {
    let live = Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::SourceHasCountersAtLeast { counter: CounterType::Loyalty, n: 1 },
    ]);
    CardDefinition {
        name: "Kaito, Bane of Nightmares",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Kaito],
            ..Default::default()
        },
        base_loyalty: 4,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), u(), b()]))],
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, as long as Kaito has one or more loyalty counters on him, he's a 3/4 Ninja creature and has hexproof.",
                effect: StaticEffect::SelfIsCreatureIf {
                    condition: live.clone(),
                    creature_types: vec![CreatureType::Ninja],
                },
            },
            StaticAbility {
                description: "…he's 3/4…",
                effect: StaticEffect::SetBasePtIf { condition: live.clone(), power: 3, toughness: 4 },
            },
            StaticAbility {
                description: "…and has hexproof.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Hexproof,
                    condition: live,
                },
            },
        ],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Kaito, Bane of Nightmares".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Ninjas you control get +1/+1.",
                        effect: StaticEffect::AnthemForFilter {
                            filter: R::Creature.and(R::HasCreatureType(CreatureType::Ninja)),
                            power: 1,
                            toughness: 1,
                            keywords: vec![],
                            opponents: false,
                            all_players: false,
                            only_your_turn: false,
                            scale_by_counters_on_self: None,
                        },
                    }],
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::OpponentsWhoLostLifeThisTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(R::Creature) },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Stun,
                        amount: Value::Const(2),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
