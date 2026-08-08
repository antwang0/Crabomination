//! Outlaws of Thunder Junction's remaining build-around legends plus
//! Assimilation Aegis. Tests in `tests/recent_b/otj_gaps2.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    ExileReturnZone, Keyword, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, flurry, target_any};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, ZoneDest,
};
use crate::mana::{Color, cost, g, generic, r, u, w};

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

/// Assimilation Aegis — {1}{W}{U} Equipment. It banishes a creature on entry,
/// then turns whatever it equips into a copy of that card.
pub fn assimilation_aegis() -> CardDefinition {
    CardDefinition {
        name: "Assimilation Aegis",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                    return_to: ExileReturnZone::Battlefield,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameAttached, EventScope::SelfSource),
                effect: Effect::BecomeCopyOfFor {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    source: Selector::CardExiledWithSource,
                    duration: Duration::WhileSourceAttached,
                    non_legendary: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Breeches, the Blastmaker — {1}{U}{R} 3/3 menace Goblin Pirate. Feed him an
/// artifact on your second spell and flip: heads copies it, tails burns.
pub fn breeches_the_blastmaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![flurry(Effect::MaySacrifice {
            description: "Sacrifice an artifact to flip for Breeches?".into(),
            filter: R::Artifact.and(R::ControlledByYou),
            count: Value::ONE,
            then: Box::new(Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::CopySpellMayChooseTargets {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                }),
                on_tails: Box::new(Effect::DealDamage {
                    to: target_any(),
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                }),
            }),
            else_: None,
        })],
        ..legend(
            "Breeches, the Blastmaker",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Goblin, CreatureType::Pirate],
            3,
            3,
        )
    }
}

/// Calamity, Galloping Inferno — {4}{R}{R} 4/6 haste Horse Mount. Attacking
/// saddled, it mints two temporary attacking copies of its saddlers.
pub fn calamity_galloping_inferno() -> CardDefinition {
    let copy_a_saddler = || {
        Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::MatchingAmong {
                    inner: Box::new(Selector::CreaturesThatSaddledSource),
                    filter: R::Not(Box::new(R::HasSupertype(Supertype::Legendary))),
                },
                enters_tapped: true,
                extra_creature_types: Vec::new(),
                extra_card_types: Vec::new(),
                override_pt: None,
                override_colors: None,
                non_legendary: false,
                legendary: false,
                extra_keywords: Vec::new(),
            },
            Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
            Effect::SacrificeLastCreatedTokensAtNextEndStep,
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Saddle(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SourceSaddled),
            effect: Effect::Seq(vec![copy_a_saddler(), copy_a_saddler()]),
        }],
        ..legend(
            "Calamity, Galloping Inferno",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Horse, CreatureType::Mount],
            4,
            6,
        )
    }
}

/// Kellan, the Kid — {G}{W}{U} 3/3 flying lifelink. Every cast from outside
/// your hand pays out a free permanent, or a land drop if you pass.
pub fn kellan_the_kid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::SpellNotCastFromHand,
                }),
            effect: Effect::MayCastPermanentFromHandFree {
                max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                else_: Box::new(Effect::MayDo {
                    description: "Put a land from your hand onto the battlefield?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::ChosenCardInHand(R::Land),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                }),
            },
        }],
        ..legend(
            "Kellan, the Kid",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Faerie, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Lilah, Undefeated Slickshot — {1}{U}{R} 3/3 prowess Rogue. Your gold
/// instants and sorceries plot themselves instead of dying to the graveyard.
pub fn lilah_undefeated_slickshot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Multicolored
                        .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                        .and(R::Not(Box::new(R::SpellNotCastFromHand))),
                },
            ),
            effect: Effect::PlotSpellOnResolve { what: Selector::TriggerSource },
        }],
        ..legend(
            "Lilah, Undefeated Slickshot",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Riku of Many Paths — {G}{U}{R} 3/3 Wizard. Every modal spell pays out one
/// pick per mode you chose.
pub fn riku_of_many_paths() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(
                    Value::ModesChosenOf(Box::new(Selector::TriggerSource)),
                    Value::ONE,
                )),
            effect: Effect::ChooseUpToN {
                max: Box::new(Value::ModesChosenOf(Box::new(Selector::TriggerSource))),
                modes: vec![
                    Effect::Seq(vec![
                        Effect::ExileTopOfLibrary {
                            who: Selector::You,
                            amount: Value::ONE,
                            link_to_source: false,
                            face_down: false,
                        },
                        Effect::GrantMayPlay {
                            what: Selector::LastMoved,
                            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                            to_owner: false,
                            exile_after: false,
                            pay_own_cost: true,
                            any_color: false,
                        },
                    ]),
                    Effect::Seq(vec![
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::GrantKeyword {
                            what: Selector::This,
                            keyword: Keyword::Trample,
                            duration: Duration::EndOfTurn,
                        },
                    ]),
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Box::new(bird),
                    },
                ],
            },
        }],
        ..legend(
            "Riku of Many Paths",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Taii Wakeen, Perfect Shot — {R}{W} 2/3 Mercenary. Exact-lethal pings draw
/// a card, and {X} turns every noncombat burn spell up by X.
pub fn taii_wakeen_perfect_shot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::YourSourceDealtNoncombatDamageEqualToToughness,
                EventScope::AnyPlayer,
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: crate::mana::ManaCost::new(vec![crate::mana::x()]),
            tap_cost: true,
            effect: Effect::YourNoncombatDamageBonusThisTurn { amount: Value::XFromCost },
            ..Default::default()
        }],
        ..legend(
            "Taii Wakeen, Perfect Shot",
            cost(&[r(), w()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            2,
            3,
        )
    }
}

/// The Gitrog, Ravenous Ride — {3}{B}{G} 6/5 trample haste Frog Horror Mount.
/// Connect, eat a saddler, and turn its power into cards and lands.
pub fn the_gitrog_ravenous_ride() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Saddle(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a creature that saddled The Gitrog?".into(),
                filter: R::Creature.and(R::SaddledSourceThisTurn),
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::SacrificedPower,
                    },
                    Effect::PutLandsFromHandOntoBattlefieldTapped {
                        count: Value::SacrificedPower,
                    },
                ])),
                else_: None,
            },
        }],
        ..legend(
            "The Gitrog, Ravenous Ride",
            cost(&[generic(3), crate::mana::b(), g()]),
            vec![CreatureType::Frog, CreatureType::Horror, CreatureType::Mount],
            6,
            5,
        )
    }
}
