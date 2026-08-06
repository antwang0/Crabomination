//! Born of the Gods (BNG) — the Fated cycle, tribute, bestow, and the
//! remaining rares. Tests in `classic_sets/bng`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EquipScale,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{heroic, target_filtered, tribute};
use crate::effect::{LookPick, Duration, Effect, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: ct,
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// "… If it's your turn, scry 2." — the Fated cycle's shared rider.
fn fated(name: &'static str, mana: ManaCost, body: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            body,
            Effect::If {
                cond: Predicate::IsTurnOf(PlayerRef::You),
                then: Box::new(Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// A bestow creature that is an enchantment creature card in every zone.
fn bestow_creature(
    name: &'static str,
    mana: ManaCost,
    bestow_cost: ManaCost,
    pt: (i32, i32),
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
    bonus: EquipBonus,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(bestow_cost),
        equipped_bonus: Some(bonus),
        ..creature(name, mana, pt.0, pt.1, ct, kw)
    }
}

// ── The Fated cycle ──────────────────────────────────────────────────────────

/// Fated Conflagration — {1}{R}{R}{R}. 5 damage to a creature or planeswalker.
pub fn fated_conflagration() -> CardDefinition {
    fated(
        "Fated Conflagration",
        cost(&[generic(1), r(), r(), r()]),
        Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Planeswalker)),
            amount: Value::Const(5),
        },
    )
}

/// Fated Infatuation — {U}{U}{U}. Token copy of a creature you control.
pub fn fated_infatuation() -> CardDefinition {
    fated(
        "Fated Infatuation",
        cost(&[u(), u(), u()]),
        Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::ONE,
            source: target_filtered(R::Creature.and(R::ControlledByYou)),
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            override_colors: None,
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
            extra_keywords: vec![],
        },
    )
}

/// Fated Intervention — {2}{G}{G}{G}. Two 3/3 Centaur enchantment creatures.
pub fn fated_intervention() -> CardDefinition {
    fated(
        "Fated Intervention",
        cost(&[generic(2), g(), g(), g()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Centaur".into(),
                power: 3,
                toughness: 3,
                colors: vec![Color::Green],
                card_types: vec![CardType::Enchantment, CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Centaur],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    )
}

/// Fated Retribution — {4}{W}{W}{W}. Destroy all creatures and planeswalkers.
pub fn fated_retribution() -> CardDefinition {
    fated(
        "Fated Retribution",
        cost(&[generic(4), w(), w(), w()]),
        Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.or(R::Planeswalker)),
        },
    )
}

/// Fated Return — {4}{B}{B}{B}. Reanimate a creature card from any graveyard;
/// it gains indestructible.
pub fn fated_return() -> CardDefinition {
    fated(
        "Fated Return",
        cost(&[generic(4), b(), b(), b()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::Permanent,
            },
        ]),
    )
}

// ── Tribute ──────────────────────────────────────────────────────────────────

/// Ornitharch — {3}{W}{W} 3/3 flier. Tribute 2, else two 1/1 flying Birds.
pub fn ornitharch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![tribute(
            2,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Bird".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::White],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Bird],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
            },
        )],
        ..creature(
            "Ornitharch",
            cost(&[generic(3), w(), w()]),
            3,
            3,
            vec![CreatureType::Archon],
            vec![Keyword::Flying],
        )
    }
}

/// Shrike Harpy — {3}{B}{B} 2/2 flier. Tribute 2, else an opponent sacrifices
/// a creature.
pub fn shrike_harpy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![tribute(
            2,
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Creature,
            },
        )],
        ..creature(
            "Shrike Harpy",
            cost(&[generic(3), b(), b()]),
            2,
            2,
            vec![CreatureType::Harpy],
            vec![Keyword::Flying],
        )
    }
}

/// Siren of the Fanged Coast — {3}{U}{U} 1/1 flier. Tribute 3, else steal a
/// creature.
pub fn siren_of_the_fanged_coast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![tribute(
            3,
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::Permanent,
            },
        )],
        ..creature(
            "Siren of the Fanged Coast",
            cost(&[generic(3), u(), u()]),
            1,
            1,
            vec![CreatureType::Siren],
            vec![Keyword::Flying],
        )
    }
}

/// Flame-Wreathed Phoenix — {2}{R}{R} 3/3 flier. Tribute 2, else it gains
/// haste and "when this dies, return it to its owner's hand."
pub fn flame_wreathed_phoenix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![tribute(
            2,
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Haste,
                    duration: Duration::Permanent,
                },
                Effect::GrantTriggeredAbility {
                    what: Selector::This,
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                        effect: Effect::Move {
                            what: Selector::This,
                            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                        },
                    }),
                    duration: Duration::Permanent,
                },
            ]),
        )],
        ..creature(
            "Flame-Wreathed Phoenix",
            cost(&[generic(2), r(), r()]),
            3,
            3,
            vec![CreatureType::Phoenix],
            vec![Keyword::Flying],
        )
    }
}

// ── Bestow ───────────────────────────────────────────────────────────────────

/// Chromanticore — {W}{U}{B}{R}{G} 4/4 with the five-keyword suite; bestow
/// {2}{W}{U}{B}{R}{G} grants +4/+4 and all five.
pub fn chromanticore() -> CardDefinition {
    let suite = vec![
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::Vigilance,
        Keyword::Trample,
        Keyword::Lifelink,
    ];
    bestow_creature(
        "Chromanticore",
        cost(&[w(), u(), b(), r(), g()]),
        cost(&[generic(2), w(), u(), b(), r(), g()]),
        (4, 4),
        vec![CreatureType::Manticore],
        suite.clone(),
        EquipBonus {
            power: 4,
            toughness: 4,
            keywords: suite,
            ..Default::default()
        },
    )
}

/// Ghostblade Eidolon — {2}{W} 1/1 double strike; bestow {5}{W}.
pub fn ghostblade_eidolon() -> CardDefinition {
    bestow_creature(
        "Ghostblade Eidolon",
        cost(&[generic(2), w()]),
        cost(&[generic(5), w()]),
        (1, 1),
        vec![CreatureType::Spirit],
        vec![Keyword::DoubleStrike],
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::DoubleStrike],
            ..Default::default()
        },
    )
}

/// Flitterstep Eidolon — {1}{U} 1/1 unblockable; bestow {5}{U}.
pub fn flitterstep_eidolon() -> CardDefinition {
    bestow_creature(
        "Flitterstep Eidolon",
        cost(&[generic(1), u()]),
        cost(&[generic(5), u()]),
        (1, 1),
        vec![CreatureType::Spirit],
        vec![Keyword::Unblockable],
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Unblockable],
            ..Default::default()
        },
    )
}

/// Herald of Torment — {1}{B}{B} 3/3 flier that costs you 1 life each upkeep;
/// bestow {3}{B}{B} grants +3/+3 and flying.
pub fn herald_of_torment() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crabomination_base::turn_step::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::LoseLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..bestow_creature(
            "Herald of Torment",
            cost(&[generic(1), b(), b()]),
            cost(&[generic(3), b(), b()]),
            (3, 3),
            vec![CreatureType::Demon],
            vec![Keyword::Flying],
            EquipBonus {
                power: 3,
                toughness: 3,
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        )
    }
}

/// Spiteful Returned — {1}{B} 1/1; whenever it or the enchanted creature
/// attacks, the defending player loses 2 life. Bestow {3}{B}.
pub fn spiteful_returned() -> CardDefinition {
    let drain = || TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            amount: Value::Const(2),
        },
    };
    CardDefinition {
        triggered_abilities: vec![drain()],
        ..bestow_creature(
            "Spiteful Returned",
            cost(&[generic(1), b()]),
            cost(&[generic(3), b()]),
            (1, 1),
            vec![CreatureType::Zombie],
            vec![],
            EquipBonus {
                power: 1,
                toughness: 1,
                triggered_abilities: vec![drain()],
                ..Default::default()
            },
        )
    }
}

/// Noble Quarry — {2}{G} 1/1 lure; bestow {5}{G} passes the lure along.
pub fn noble_quarry() -> CardDefinition {
    bestow_creature(
        "Noble Quarry",
        cost(&[generic(2), g()]),
        cost(&[generic(5), g()]),
        (1, 1),
        vec![CreatureType::Unicorn],
        vec![Keyword::AllMustBlock],
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::AllMustBlock],
            ..Default::default()
        },
    )
}

/// Everflame Eidolon — {1}{R} 1/1; {R} pumps it, or the enchanted creature
/// while it's an Aura. Bestow {2}{R}.
pub fn everflame_eidolon() -> CardDefinition {
    let pump = |what: Selector| Effect::PumpPT {
        what,
        power: Value::ONE,
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsBestowed,
                },
                then: Box::new(pump(Selector::AttachedTo(Box::new(Selector::This)))),
                else_: Box::new(pump(Selector::This)),
            },
            ..Default::default()
        }],
        ..bestow_creature(
            "Everflame Eidolon",
            cost(&[generic(1), r()]),
            cost(&[generic(2), r()]),
            (1, 1),
            vec![CreatureType::Spirit],
            vec![],
            EquipBonus {
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        )
    }
}

/// Eidolon of Countless Battles — {1}{W}{W} 0/0; it and the enchanted creature
/// each get +1/+1 per creature and per Aura you control. Bestow {2}{W}{W}.
pub fn eidolon_of_countless_battles() -> CardDefinition {
    let count = R::Creature.or(R::HasEnchantmentSubtype(
        crate::card::EnchantmentSubtype::Aura,
    ));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each creature you control and +1/+1 for each Aura you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: count.clone(),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..bestow_creature(
            "Eidolon of Countless Battles",
            cost(&[generic(1), w(), w()]),
            cost(&[generic(2), w(), w()]),
            (0, 0),
            vec![CreatureType::Spirit],
            vec![],
            EquipBonus {
                scale: Some(EquipScale {
                    filter: count,
                    per_power: 1,
                    per_toughness: 1,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

// ── Heroic and the remaining rares ───────────────────────────────────────────

/// Hero of Iroas — {1}{W} 2/2. Aura spells cost {1} less; heroic adds a
/// +1/+1 counter.
pub fn hero_of_iroas() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Aura spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura),
                amount: 1,
            },
        }],
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Hero of Iroas",
            cost(&[generic(1), w()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Hero of Leina Tower — {G} 1/1. Heroic: you may pay {X} for X +1/+1
/// counters.
pub fn hero_of_leina_tower() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::MayPayX {
            description: "Pay {X} for X +1/+1 counters".into(),
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            }),
        })],
        ..creature(
            "Hero of Leina Tower",
            cost(&[g()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Meletis Astronomer — {1}{U} 1/3. Heroic: dig three for an enchantment.
pub fn meletis_astronomer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(3),
            pick_filter: Some(R::Enchantment),
            optional: true,
    ..Default::default()
})))],
        ..creature(
            "Meletis Astronomer",
            cost(&[generic(1), u()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Akroan Conscriptor — {4}{R} 3/2. Heroic: threaten another target creature.
pub fn akroan_conscriptor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Akroan Conscriptor",
            cost(&[generic(4), r()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Satyr Firedancer — {1}{R} 1/1. Whenever an instant or sorcery you control
/// deals damage to a player, it deals that much to target creature that player
/// controls.
pub fn satyr_firedancer() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::YourInstantOrSorceryDealtDamageToPlayer,
                EventScope::YourControl,
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Satyr Firedancer",
            cost(&[generic(1), r()]),
            1,
            1,
            vec![CreatureType::Satyr],
            vec![],
        )
    }
}

/// Pillar of War — {3} 3/3 Golem with defender that can attack while enchanted.
pub fn pillar_of_war() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is enchanted, it can attack as though it didn't have defender.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsEnchanted,
                },
            },
        }],
        ..creature(
            "Pillar of War",
            cost(&[generic(3)]),
            3,
            3,
            vec![CreatureType::Golem],
            vec![Keyword::Defender],
        )
    }
}

/// Ragemonger — {1}{B}{R} 2/3. Minotaur spells you cast cost {B}{R} less.
pub fn ragemonger() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Minotaur spells you cast cost {B}{R} less to cast.",
            effect: StaticEffect::ColoredCostReduction {
                filter: R::HasCreatureType(CreatureType::Minotaur),
                less: cost(&[b(), r()]),
            },
        }],
        ..creature(
            "Ragemonger",
            cost(&[generic(1), b(), r()]),
            2,
            3,
            vec![CreatureType::Minotaur, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Acolyte's Reward — {1}{W} Instant. Prevent the next X damage to a creature,
/// X = your devotion to white, and reflect the prevented damage at any target.
pub fn acolytes_reward() -> CardDefinition {
    CardDefinition {
        name: "Acolyte's Reward",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::RedirectNextDamage {
            target: target_filtered(R::Creature),
            to: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
            },
            amount: Value::DevotionTo(vec![Color::White]),
        },
        ..Default::default()
    }
}

/// Champion of Stray Souls — {4}{B}{B} 4/4. Sacrifice X creatures to reanimate
/// X creature cards from your graveyard (picked at resolution rather than
/// targeted); {5}{B}{B} from the graveyard puts it back on top.
pub fn champion_of_stray_souls() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b(), b()]),
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                sac_other_x: true,
                effect: Effect::MoveChosen {
                    from: Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                        filter: R::Creature,
                    },
                    filter: None,
                    count: Value::XFromCost,
                    up_to: false,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5), b(), b()]),
                from_graveyard: true,
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: crate::effect::LibraryPosition::Top,
                    },
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Champion of Stray Souls",
            cost(&[generic(4), b(), b()]),
            4,
            4,
            vec![CreatureType::Skeleton, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Felhide Spiritbinder — {3}{R} 3/4. Inspired: pay {1}{R} for a hasty
/// enchantment token copy of another creature, exiled at the next end step.
pub fn felhide_spiritbinder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1}{R} for a token copy".into(),
                mana_cost: cost(&[generic(1), r()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: target_filtered(R::Creature.and(R::OtherThanSource)),
                        extra_creature_types: vec![],
                        extra_card_types: vec![CardType::Enchantment],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![Keyword::Haste],
                    },
                    Effect::ExileLastCreatedTokensAtNextEndStep,
                ])),
                else_: None,
            },
        }],
        ..creature(
            "Felhide Spiritbinder",
            cost(&[generic(3), r()]),
            3,
            4,
            vec![CreatureType::Minotaur, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Vortex Elemental — {U} 0/1. {U} shuffles it and its blockers away;
/// {3}{U}{U} forces a creature to block it.
pub fn vortex_elemental() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                        to: ZoneDest::Library {
                            who: PlayerRef::OwnerOfMoved,
                            pos: crate::effect::LibraryPosition::Shuffled,
                        },
                    },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Library {
                            who: PlayerRef::OwnerOfMoved,
                            pos: crate::effect::LibraryPosition::Shuffled,
                        },
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u(), u()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::MustBlock,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Vortex Elemental",
            cost(&[u()]),
            0,
            1,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

/// Floodtide Serpent — {4}{U} 4/4 that can't attack unless you bounce an
/// enchantment you control.
pub fn floodtide_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AttackCostBounce(Box::new(
            R::Enchantment.and(R::ControlledByYou),
        ))],
        ..creature(
            "Floodtide Serpent",
            cost(&[generic(4), u()]),
            4,
            4,
            vec![CreatureType::Serpent],
            vec![],
        )
    }
}

/// Arbiter of the Ideal — {4}{U}{U} 4/5 flier. Inspired: manifest-style
/// cheat — reveal the top card and, if it's an artifact, creature, or land,
/// you may put it onto the battlefield as an enchantment with a manifestation
/// counter.
pub fn arbiter_of_the_ideal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
            effect: Effect::RevealTopMayPutOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Artifact.or(R::Creature).or(R::Land),
                counter: Some(CounterType::Manifestation),
                extra_types: vec![CardType::Enchantment],
            },
        }],
        ..creature(
            "Arbiter of the Ideal",
            cost(&[generic(4), u(), u()]),
            4,
            5,
            vec![CreatureType::Sphinx],
            vec![Keyword::Flying],
        )
    }
}

// ── The last four ────────────────────────────────────────────────────────────

/// Kiora, the Crashing Wave — {2}{G}{U} planeswalker, loyalty 2.
pub fn kiora_the_crashing_wave() -> CardDefinition {
    use crate::card::{LoyaltyAbility, Supertype};
    CardDefinition {
        name: "Kiora, the Crashing Wave",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Kiora],
            ..Default::default()
        },
        base_loyalty: 2,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::PreventDamageToAndByUntilYourNextTurn {
                    target: target_filtered(R::Permanent.and(R::ControlledByOpponent)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::GrantExtraLandPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Kiora, the Crashing Wave".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::StepBegins(crabomination_base::turn_step::TurnStep::End),
                            EventScope::YourControl,
                        ),
                        effect: Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: TokenDefinition {
                                name: "Kraken".into(),
                                power: 9,
                                toughness: 9,
                                colors: vec![Color::Blue],
                                card_types: vec![CardType::Creature],
                                subtypes: Subtypes {
                                    creature_types: vec![CreatureType::Kraken],
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
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

/// Mindreaver — {U}{U} 2/1. Heroic exiles the top three of a player's library
/// with it; {U}{U}, Sacrifice: counter a spell sharing a name with them.
pub fn mindreaver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::ExileTopOfLibrary {
            who: Selector::Player(PlayerRef::Target(1)),
            amount: Value::Const(3),
            link_to_source: true,
            face_down: false,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            sac_cost: true,
            effect: Effect::CounterSpellIfNameExiledWithSource {
                what: target_filtered(R::IsSpellOnStack),
            },
            ..Default::default()
        }],
        ..creature(
            "Mindreaver",
            cost(&[u(), u()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Perplexing Chimera — {4}{U} 3/3. Whenever an opponent casts a spell, you may
/// exchange control of this creature and that spell.
pub fn perplexing_chimera() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Exchange control of Perplexing Chimera and that spell".into(),
                body: Box::new(Effect::ExchangeControlWithTriggeringSpell {
                    what: Selector::This,
                }),
            },
        }],
        ..creature(
            "Perplexing Chimera",
            cost(&[generic(4), u()]),
            3,
            3,
            vec![CreatureType::Chimera],
            vec![],
        )
    }
}

/// Whims of the Fates — {5}{R} Sorcery. Each player splits their permanents
/// into three piles and sacrifices one pile at random.
pub fn whims_of_the_fates() -> CardDefinition {
    CardDefinition {
        name: "Whims of the Fates",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerSplitsAndSacrificesRandomPile { piles: 3 },
        ..Default::default()
    }
}
