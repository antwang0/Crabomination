//! Commander: 2026-10-17 (`3chree`) closers, eighth batch — one-deck closers
//! from the EDHREC long tail. Tests in `tests/recent_b/cmdr_oct17_x.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{
    choose_some_then, chosen_one, drain, draw, each_opponent, etb, on_attack, on_other_dies,
    target_filtered,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use crabomination_base::tokens::{food_token, treasure_token};
use std::sync::Arc;

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
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}


/// Ant-Man's Army — {2}{G} 3/2 Insect. Enters: create a Food or a Treasure.
pub fn ant_mans_army() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(food_token()),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(treasure_token()),
            },
        ]))],
        ..creature(
            "Ant-Man's Army",
            cost(&[generic(2), g()]),
            vec![CreatureType::Insect],
            3,
            2,
        )
    }
}

/// Bob, Reluctant HYDRA Agent — {1}{B} legendary 2/2 Human Coward Villain.
/// Attacks alone: return him to his owner's hand; if you do, drain 2.
pub fn bob_reluctant_hydra_agent() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsAttackingAlone,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::LastMoved),
                    then: Box::new(drain(2)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature(
            "Bob, Reluctant HYDRA Agent",
            cost(&[generic(1), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Coward,
                CreatureType::Villain,
            ],
            2,
            2,
        )
    }
}

/// Stolen Stark Tech — {1}{B} Equipment, flash. Enters: attach to target
/// creature you control; it gains indestructible until end of turn.
/// Equipped creature gets +1/+0. Equip {1}.
pub fn stolen_stark_tech() -> CardDefinition {
    CardDefinition {
        name: "Stolen Stark Tech",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// The Death of Gwen Stacy — {2}{B} Saga. I: destroy target creature. II: each
/// player loses 3 unless they discard a card. III: exile any number of target
/// players' graveyards (eight player slots: a pod's maximum).
pub fn the_death_of_gwen_stacy() -> CardDefinition {
    CardDefinition {
        name: "The Death of Gwen Stacy",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Destroy {
                    what: target_filtered(R::Creature),
                },
            ),
            (
                2,
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachPlayer),
                    body: Box::new(Effect::UnlessPlayerPays {
                        who: PlayerRef::Triggerer,
                        cost: WardCost::Discard(1),
                        then: Box::new(Effect::LoseLife {
                            who: Selector::Player(PlayerRef::Triggerer),
                            amount: Value::Const(3),
                        }),
                        if_paid: None,
                    }),
                },
            ),
            (
                3,
                Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Player,
                    effect: Box::new(Effect::ExilePlayerGraveyard {
                        who: PlayerRef::Target(0),
                        filter: None,
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Roxxon Brutes — {4}{B} 4/4 Human Berserker Villain, menace. Your second
/// draw each turn: a +1/+1 counter on target creature. Basic landcycling {2}.
pub fn roxxon_brutes() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Menace,
            Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand))),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Roxxon Brutes",
            cost(&[generic(4), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Berserker,
                CreatureType::Villain,
            ],
            4,
            4,
        )
    }
}

/// Whiplash, Vengeful Engineer — {B} legendary 2/2 Human Artificer Villain;
/// enters tapped. Attacks, if equipped: drain X, X = Equipment attached.
pub fn whiplash_vengeful_engineer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Whiplash enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SourceIsEquipped),
            effect: Effect::Drain {
                from: each_opponent(),
                to: Selector::You,
                amount: Value::CountMatching {
                    sel: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                },
            },
        }],
        ..creature(
            "Whiplash, Vengeful Engineer",
            cost(&[b()]),
            vec![
                CreatureType::Human,
                CreatureType::Artificer,
                CreatureType::Villain,
            ],
            2,
            2,
        )
    }
}

/// Winter Soldier, Icy Assassin — {W}{B} legendary 2/2, vigilance, menace;
/// +2/+0 per attached Equipment. {3}{W}{B} from graveyard: return with a
/// finality counter, then you may attach an Equipment you control to him.
pub fn winter_soldier_icy_assassin() -> CardDefinition {
    let equipment = R::HasArtifactSubtype(ArtifactSubtype::Equipment);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Winter Soldier gets +2/+0 for each Equipment attached to him.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CountMatching {
                    sel: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
                    filter: equipment.clone(),
                },
                per_power: 2,
                per_toughness: 0,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), b()]),
            from_graveyard: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Finality,
                    amount: Value::ONE,
                },
                choose_some_then(
                    Selector::EachPermanent(equipment.and(R::ControlledByYou)),
                    PlayerRef::You,
                    Value::ONE,
                    true,
                    Effect::Attach {
                        what: chosen_one(),
                        to: Selector::This,
                    },
                ),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Winter Soldier, Icy Assassin",
            cost(&[w(), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Assassin,
                CreatureType::Villain,
            ],
            2,
            2,
        )
    }
}

/// Ms. Marvel, Elastic Ally — {2}{G/W} legendary 2/2, reach. Enters: target
/// creature +2/+0 EOT. Once each turn, your creature with power above its base
/// power dealing combat damage to a player draws a card.
pub fn ms_marvel_elastic_ally() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            etb(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                )
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::PowerGreaterThanBasePower,
                })
                .once_per_turn(),
                effect: draw(1),
            },
        ],
        ..creature(
            "Ms. Marvel, Elastic Ally",
            cost(&[generic(2), hybrid(Color::Green, Color::White)]),
            vec![
                CreatureType::Mutant,
                CreatureType::Inhuman,
                CreatureType::Hero,
            ],
            2,
            2,
        )
    }
}

/// Bereaved Survivor // Dauntless Avenger — {2}{W} 2/1 Human Peasant; another
/// creature of yours dying transforms it. Back (3/2 Human Soldier): attacks →
/// return target creature card with MV ≤ 2 from your graveyard tapped and attacking.
pub fn bereaved_survivor() -> CardDefinition {
    let avenger = CardDefinition {
        color_indicator: vec![Color::White],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::InYourGraveyard)
                        .and(R::ManaValueAtMost(2)),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            Effect::JoinCombatAttacking {
                what: Selector::LastMoved,
            },
        ]))],
        ..creature(
            "Dauntless Avenger",
            ManaCost::default(),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            2,
        )
    };
    CardDefinition {
        triggered_abilities: vec![on_other_dies(Effect::Transform {
            what: Selector::This,
        })],
        back_face: Some(Box::new(avenger)),
        ..creature(
            "Bereaved Survivor",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Peasant],
            2,
            1,
        )
    }
}

/// Skinshifter — {1}{G} 1/1 Human Shaman. {G}, once each turn, choose one until
/// end of turn: 4/4 trample Rhino; 2/2 flying Bird; 0/8 Plant.
pub fn skinshifter() -> CardDefinition {
    let become_ = |ty: CreatureType, p: i32, t: i32, kw: Option<Keyword>| {
        let mut seq = vec![
            Effect::BecomeCreatureType {
                what: Selector::This,
                creature_types: vec![ty],
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(p),
                toughness: Value::Const(t),
                duration: Duration::EndOfTurn,
            },
        ];
        if let Some(keyword) = kw {
            seq.push(Effect::GrantKeyword {
                what: Selector::This,
                keyword,
                duration: Duration::EndOfTurn,
            });
        }
        Effect::Seq(seq)
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            once_per_turn: true,
            effect: Effect::ChooseMode(vec![
                become_(CreatureType::Rhino, 4, 4, Some(Keyword::Trample)),
                become_(CreatureType::Bird, 2, 2, Some(Keyword::Flying)),
                become_(CreatureType::Plant, 0, 8, None),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Skinshifter",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Belltoll Dragon — {5}{U} 3/3 Dragon, flying, hexproof, megamorph {5}{U}{U}.
/// Turned face up: a +1/+1 counter on each other Dragon creature you control.
pub fn belltoll_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Hexproof,
            Keyword::Megamorph(cost(&[generic(5), u(), u()])),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Dragon))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Belltoll Dragon",
            cost(&[generic(5), u()]),
            vec![CreatureType::Dragon],
            3,
            3,
        )
    }
}

/// Robot Domination — {3}{B} Enchantment — Plan. One or more creature cards to
/// your graveyard: draw, lose 1, a plan counter. Third plan counter: sacrifice
/// it and create three 2/2 colorless Robot Villain artifact creatures.
pub fn robot_domination() -> CardDefinition {
    let robot = Arc::new(TokenDefinition {
        name: "Robot Villain".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Villain],
            ..Default::default()
        },
        ..Default::default()
    });
    CardDefinition {
        name: "Robot Domination",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Plan],
            ..Default::default()
        },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    })
                    .once_per_batch(),
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Plan,
                        amount: Value::ONE,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::Plan),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::SourceHasCountersAtLeast {
                    counter: CounterType::Plan,
                    n: 3,
                }),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(3),
                        definition: robot,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Armor Wars — {2}{U}{R} Saga. I: you may draw a card per artifact you
/// control; if you do, each opponent draws one. II: artifact spells cost {1}
/// less this turn. III: X damage to target opponent, X = greatest artifact MV.
pub fn armor_wars() -> CardDefinition {
    let your_artifacts = || Selector::EachPermanent(R::Artifact.and(R::ControlledByYou));
    CardDefinition {
        name: "Armor Wars",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::MayDo {
                    description:
                        "Draw a card for each artifact you control (each opponent draws one)?"
                            .into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw {
                            who: Selector::You,
                            amount: Value::count(your_artifacts()),
                        },
                        Effect::Draw {
                            who: each_opponent(),
                            amount: Value::ONE,
                        },
                    ])),
                },
            ),
            (
                2,
                Effect::SpellsCostLessThisTurn {
                    filter: R::Artifact,
                    amount: 1,
                },
            ),
            (
                3,
                Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer),
                    amount: Value::HighestManaValueAmong(Box::new(your_artifacts())),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Portal Manipulator — {2}{W/U}{W/U} 3/2 flash Human Wizard. When it enters
/// during the declare attackers step, attacks on you move to a player of your
/// choice (both picks made on resolution; every attack you could redirect
/// moves).
pub fn portal_manipulator() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    let mut redirect = crate::effect::shortcut::etb(Effect::AttackersNowAttackChosenPlayer);
    // "During the declare attackers step" is an intervening if (CR 603.4).
    redirect.event =
        redirect.event.with_filter(Predicate::CurrentStepIs(crate::game::types::TurnStep::DeclareAttackers));
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![redirect],
        ..creature(
            "Portal Manipulator",
            cost(&[generic(2), wu(), wu()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            2,
        )
    }
}
