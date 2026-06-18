//! Theros Beyond Death (THB) — 2020. Escape payoffs, devotion demigods,
//! and the constellation/enchantment-matters shell.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

/// Heliod's Intervention — {X}{W}{W} Instant. Choose one — destroy X target
/// artifacts and/or enchantments; or target player gains twice X life.
pub fn heliods_intervention() -> CardDefinition {
    CardDefinition {
        name: "Heliod's Intervention",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DestroyTargets {
                filter: SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            },
            Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
            },
        ]),
        ..Default::default()
    }
}

fn shark_token() -> TokenDefinition {
    TokenDefinition {
        name: "Shark".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shark], ..Default::default() },
        dynamic_pt: Some((Value::TriggerEventAmount, Value::TriggerEventAmount)),
        ..Default::default()
    }
}

/// Shark Typhoon — {5}{U} Enchantment. Noncreature cast → X/X flying Shark
/// (X = that spell's mana value). Cycling {X}{1}{U}; cycle → X/X Shark.
pub fn shark_typhoon() -> CardDefinition {
    let mint = |scope| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, scope).with_filter(Predicate::Not(Box::new(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature,
            },
        ))),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: shark_token(),
        },
    };
    CardDefinition {
        name: "Shark Typhoon",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Cycling(cost(&[x(), generic(1), u()]))],
        triggered_abilities: vec![
            mint(EventScope::YourControl),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: shark_token(),
                },
            },
        ],
        ..Default::default()
    }
}

/// Nyxbloom Ancient — {4}{G}{G}{G} Enchantment Creature — Elemental 5/5.
/// Trample. If you tap a permanent for mana, it produces three times as much.
pub fn nyxbloom_ancient() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Nyxbloom Ancient",
        cost: cost(&[generic(4), g(), g(), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "If you tap a permanent for mana, it produces three times as much",
            effect: crate::effect::StaticEffect::ManaProductionTripled,
        }],
        ..Default::default()
    }
}

/// Polukranos, Unchained — {2}{B}{G} Legendary Zombie Hydra 0/0. Enters with
/// six +1/+1 counters (twelve if it escaped); damage to it is prevented by
/// removing that many counters; {1}{B}{G}: fights another target creature.
/// Escape — {4}{B}{G}, exile six other cards.
pub fn polukranos_unchained() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Polukranos, Unchained",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Hydra],
            ..Default::default()
        },
        keywords: vec![Keyword::Escape(cost(&[generic(4), b(), g()]), 6)],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastFromEscape),
                then: Box::new(Value::Const(12)),
                else_: Box::new(Value::Const(6)),
            },
        )),
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to this while it has a +1/+1 counter, \
                          prevent it and remove that many counters",
            effect: crate::effect::StaticEffect::PreventDamageByRemovingCounters {
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), g()]),
            effect: Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(SelectionRequirement::Creature
                    .and(SelectionRequirement::OtherThanSource)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elspeth Conquers Death — {3}{W}{W} Saga. I: exile target MV≥3 opponent
/// permanent. II: opponents' noncreature spells cost {2} more until your
/// next turn. III: return a creature/planeswalker from your graveyard with
/// a +1/+1 or loyalty counter.
pub fn elspeth_conquers_death() -> CardDefinition {
    CardDefinition {
        name: "Elspeth Conquers Death",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::ControlledByOpponent)
                            .and(SelectionRequirement::ManaValueAtLeast(3)),
                    ),
                },
            ),
            (
                2,
                Effect::SpellTaxUntilYourNextTurn {
                    amount: 2,
                    filter: SelectionRequirement::Noncreature,
                },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachMatching {
                                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                                filter: SelectionRequirement::Creature
                                    .or(SelectionRequirement::Planeswalker),
                            }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::LastMoved,
                            filter: SelectionRequirement::Planeswalker,
                        },
                        then: Box::new(Effect::AddCounter {
                            what: Selector::LastMoved,
                            kind: CounterType::Loyalty,
                            amount: Value::ONE,
                        }),
                        else_: Box::new(Effect::AddCounter {
                            what: Selector::LastMoved,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Dream Trawler — {2}{W}{W}{U}{U} Sphinx 3/5. Flying, lifelink; draw → +1/+0;
/// attacks → draw; discard a card: gains hexproof until end of turn.
pub fn dream_trawler() -> CardDefinition {
    CardDefinition {
        name: "Dream Trawler",
        cost: cost(&[generic(2), w(), w(), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Arasta of the Endless Web — {2}{G}{G} Legendary Enchantment Creature 3/5.
/// Reach; opponent casts an instant/sorcery → 1/2 reach Spider token.
pub fn arasta_of_the_endless_web() -> CardDefinition {
    CardDefinition {
        name: "Arasta of the Endless Web",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                crate::effect::shortcut::cast_is_instant_or_sorcery(),
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Spider".into(),
                    power: 1,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    keywords: vec![Keyword::Reach],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spider],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Daxos, Blessed by the Sun — {W}{W} Legendary Enchantment Creature 2/*.
/// Toughness = devotion to white; another creature you control enters or
/// dies → gain 1 life.
pub fn daxos_blessed_by_the_sun() -> CardDefinition {
    // CreatureDied resolves a graveyard card, where a battlefield `Creature`
    // check would fail — OtherThanSource alone is the right filter for both
    // (the entering side gates on creature entries via the event kind too).
    let other_creature = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::OtherThanSource,
            },
        ),
        effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
    };
    CardDefinition {
        name: "Daxos, Blessed by the Sun",
        cost: cost(&[w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demigod], ..Default::default() },
        power: 2,
        dynamic_pt: Some(DynamicPt::DevotionToToughness { color: Color::White, base_p: 2 }),
        triggered_abilities: vec![
            other_creature(EventKind::EntersBattlefield),
            other_creature(EventKind::CreatureDied),
        ],
        ..Default::default()
    }
}

/// Tymaret Calls the Dead — {2}{B} Saga. I, II: mill three, then exile a
/// creature or enchantment from your graveyard for a 2/2 Zombie. III: gain
/// life and scry equal to your Zombie count.
pub fn tymaret_calls_the_dead() -> CardDefinition {
    let dig = || {
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachMatching {
                    zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                    filter: SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                }),
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachMatching {
                                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                                filter: SelectionRequirement::Creature
                                    .or(SelectionRequirement::Enchantment),
                            }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Exile,
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: TokenDefinition {
                            name: "Zombie".into(),
                            power: 2,
                            toughness: 2,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Black],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Zombie],
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    let zombies = || {
        Value::CountMatching {
            sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
            filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie),
        }
    };
    CardDefinition {
        name: "Tymaret Calls the Dead",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, dig()),
            (2, dig()),
            (
                3,
                Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: zombies() },
                    Effect::Scry { who: PlayerRef::You, amount: zombies() },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Thirst for Meaning — {2}{U} Instant. Draw three cards, then discard two
/// cards unless you discard an enchantment card.
pub fn thirst_for_meaning() -> CardDefinition {
    CardDefinition {
        name: "Thirst for Meaning",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::DiscardUnlessKind {
                who: PlayerRef::You,
                count: Value::Const(2),
                instead: SelectionRequirement::Enchantment,
            },
        ]),
        ..Default::default()
    }
}

/// Shatter the Sky — {2}{W} Sorcery. Each player with a power-4+ creature
/// draws a card; then destroy all creatures.
pub fn shatter_the_sky() -> CardDefinition {
    CardDefinition {
        name: "Shatter the Sky",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::If {
                    cond: Predicate::SelectorExists(Selector::ControlledBy {
                        who: PlayerRef::Triggerer,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::PowerAtLeast(4)),
                    }),
                    then: Box::new(Effect::Draw {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
            Effect::Destroy { what: Selector::EachPermanent(SelectionRequirement::Creature) },
        ]),
        ..Default::default()
    }
}

/// Alseid of Life's Bounty — {W} Enchantment Creature — Nymph 1/1. Lifelink;
/// {1}, Sacrifice: target creature or enchantment you control gains
/// protection from the color of your choice until end of turn.
pub fn alseid_of_lifes_bounty() -> CardDefinition {
    CardDefinition {
        name: "Alseid of Life's Bounty",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nymph], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::GrantProtectionFromChosenColor {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Enchantment)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mire Triton — {1}{B} Zombie Merfolk 2/1. Deathtouch; ETB mill two and
/// gain 2 life.
pub fn mire_triton() -> CardDefinition {
    CardDefinition {
        name: "Mire Triton",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Merfolk],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Aphemia, the Cacophony — {1}{B} Legendary Enchantment Creature 2/1.
/// Flying; end step: exile an enchantment card from your graveyard for a
/// 2/2 Zombie.
pub fn aphemia_the_cacophony() -> CardDefinition {
    CardDefinition {
        name: "Aphemia, the Cacophony",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Harpy], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachMatching {
                    zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                    filter: SelectionRequirement::Enchantment,
                }),
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachMatching {
                                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                                filter: SelectionRequirement::Enchantment,
                            }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Exile,
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: TokenDefinition {
                            name: "Zombie".into(),
                            power: 2,
                            toughness: 2,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Black],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Zombie],
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Phoenix of Ash — {1}{R}{R} Phoenix 2/2. Flying, haste; {2}{R}: +2/+0;
/// Escape — {2}{R}{R}, exile three other cards.
pub fn phoenix_of_ash() -> CardDefinition {
    CardDefinition {
        name: "Phoenix of Ash",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phoenix], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![
            Keyword::Flying,
            Keyword::Haste,
            Keyword::Escape(cost(&[generic(2), r(), r()]), 3),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Underworld Rage-Hound — {1}{R} Elemental Dog 3/1. Attacks each combat if
/// able; Escape — {3}{R}, exile three other cards; escapes with a +1/+1
/// counter.
pub fn underworld_rage_hound() -> CardDefinition {
    CardDefinition {
        name: "Underworld Rage-Hound",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::MustAttack, Keyword::Escape(cost(&[generic(3), r()]), 3)],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastFromEscape),
                then: Box::new(Value::ONE),
                else_: Box::new(Value::ZERO),
            },
        )),
        ..Default::default()
    }
}

/// Nessian Boar — {3}{G}{G} Boar 10/6. All creatures able to block it do so;
/// each creature that blocks it lets its controller draw a card.
pub fn nessian_boar() -> CardDefinition {
    CardDefinition {
        name: "Nessian Boar",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        power: 10,
        toughness: 6,
        keywords: vec![Keyword::AllMustBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::AnyPlayer)
                .with_filter(Predicate::TriggerBlocksSource),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Mystic Repeal — {G} Instant. Put target enchantment on the bottom of its
/// owner's library.
pub fn mystic_repeal() -> CardDefinition {
    CardDefinition {
        name: "Mystic Repeal",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Enchantment),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::Bottom,
            },
        },
        ..Default::default()
    }
}

/// Agonizing Remorse — {1}{B} Sorcery. Target opponent reveals their hand;
/// exile a nonland card from it. You lose 1 life. (The graveyard-pick
/// option collapses to the hand pick.)
pub fn agonizing_remorse() -> CardDefinition {
    CardDefinition {
        name: "Agonizing Remorse",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileChosenFromHand {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: SelectionRequirement::Nonland,
            },
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Eat to Extinction — {3}{B} Instant. Exile target creature or
/// planeswalker. Surveil 1.
pub fn eat_to_extinction() -> CardDefinition {
    CardDefinition {
        name: "Eat to Extinction",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Taranika, Akroan Veteran — {1}{W}{W} 3/3. Vigilance; attacks → untap
/// another target creature you control, it's base 4/4 + indestructible EOT.
pub fn taranika_akroan_veteran() -> CardDefinition {
    let tgt = || {
        target_filtered(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
        )
    };
    CardDefinition {
        name: "Taranika, Akroan Veteran",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Untap { what: tgt(), up_to: None },
                Effect::SetBasePT {
                    what: tgt(),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: tgt(),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Sweet Oblivion — {1}{U} Sorcery. Target player mills four. Escape —
/// {3}{U}, exile four other cards.
pub fn sweet_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Sweet Oblivion",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Escape(cost(&[generic(3), u()]), 4)],
        effect: Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Klothys's Design — {5}{G} Sorcery. Creatures you control get +X/+X until
/// end of turn, where X is your devotion to green.
pub fn klothyss_design() -> CardDefinition {
    CardDefinition {
        name: "Klothys's Design",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::DevotionTo(vec![Color::Green]),
            toughness: Value::DevotionTo(vec![Color::Green]),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Escape Protocol — {1}{U} Enchantment. Cycle a card → may pay {1} to
/// flicker target artifact or creature you control.
pub fn escape_protocol() -> CardDefinition {
    let tgt = || {
        target_filtered(
            SelectionRequirement::Artifact
                .or(SelectionRequirement::Creature)
                .and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Escape Protocol",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to flicker an artifact or creature?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: tgt() },
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Protean Thaumaturge — {1}{U} 1/1. Constellation — may become a copy of
/// another target creature. (The "except it has this ability" rider is
/// dropped — the copy is plain.)
pub fn protean_thaumaturge() -> CardDefinition {
    CardDefinition {
        name: "Protean Thaumaturge",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::MayDo {
                description: "Become a copy of another target creature?".into(),
                body: Box::new(Effect::BecomeCopyOf {
                    what: Selector::This,
                    source: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                    ),
                    extra_creature_types: Vec::new(),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Enigmatic Incarnation — {2}{G}{U} Enchantment. End step: may sacrifice
/// another enchantment to fetch a creature with MV = 1 + its MV onto the
/// battlefield.
pub fn enigmatic_incarnation() -> CardDefinition {
    CardDefinition {
        name: "Enigmatic Incarnation",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::Enchantment
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                )),
                then: Box::new(Effect::MayDo {
                    description: "Sacrifice an enchantment to fetch a creature?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::SacrificeAndRemember {
                            who: PlayerRef::You,
                            filter: SelectionRequirement::Enchantment
                                .and(SelectionRequirement::OtherThanSource),
                        },
                        Effect::Search {
                            who: PlayerRef::You,
                            filter: SelectionRequirement::Creature
                                .and(SelectionRequirement::ManaValueEqualsSacrificedPlus(1)),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Gallia of the Endless Dance — {R}{G} Legendary Satyr 2/2. Haste; other
/// Satyrs get +1/+1 and have haste; attack with 3+ creatures → may discard
/// at random to draw two.
pub fn gallia_of_the_endless_dance() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Gallia of the Endless Dance",
        cost: cost(&[r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Satyr], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        static_abilities: vec![
            StaticAbility {
                description: "Other Satyrs you control get +1/+1",
                effect: crate::effect::StaticEffect::PumpTeamIf {
                    condition: Predicate::True,
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Satyr)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Haste],
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::ValueAtLeast(
                    Value::CreaturesAttackedWithThisTurn(PlayerRef::You),
                    Value::Const(3),
                ),
            ),
            effect: Effect::MayDo {
                description: "Discard a card at random to draw two?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Kunoros, Hound of Athreos — {1}{W}{B} Legendary Dog 3/3. Vigilance,
/// menace, lifelink; creatures can't enter from graveyards; players can't
/// cast from graveyards.
pub fn kunoros_hound_of_athreos() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::mana::b as black;
    CardDefinition {
        name: "Kunoros, Hound of Athreos",
        cost: cost(&[generic(1), w(), black()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Menace, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Creature cards in graveyards can't enter the battlefield; \
                          players can't cast spells from graveyards",
            effect: crate::effect::StaticEffect::GraveyardLockdown,
        }],
        ..Default::default()
    }
}

/// Tectonic Giant — {2}{R}{R} Elemental Giant 3/4. Attacks or targeted by
/// an opponent's spell → 3 to each opponent, or impulse two with a pick.
pub fn tectonic_giant() -> CardDefinition {
    let modal = || {
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
            // "Choose one of them" collapses to both getting the may-play
            // grant (strictly better for the controller; rarely relevant).
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: true,
                uncast_penalty: None,
            },
        ])
    };
    CardDefinition {
        name: "Tectonic Giant",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Giant],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: modal(),
            },
            TriggeredAbility {
                event: {
                    let mut e =
                        EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource);
                    e.actor_is_opponent = true;
                    e
                },
                effect: modal(),
            },
        ],
        ..Default::default()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// THB batch — vanilla Nyxborn enchantment creatures, simple ETB/death/
// constellation/activated bodies on existing primitives.
// ════════════════════════════════════════════════════════════════════════════

/// Constellation trigger helper: "Whenever an enchantment you control enters,
/// `body`." (CR 702.xx — an `EntersBattlefield`/`YourControl` trigger filtered
/// to enchantment trigger-sources, the same shape Protean Thaumaturge uses.)
fn constellation(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Enchantment,
            }),
        effect: body,
    }
}

/// Shorthand for an Enchantment Creature `CardDefinition` skeleton.
fn nyxborn(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

/// Nyxborn Brute — {3}{R}{R} 7/3 Enchantment Creature — Cyclops (vanilla).
pub fn nyxborn_brute() -> CardDefinition {
    nyxborn("Nyxborn Brute", cost(&[generic(3), r(), r()]), vec![CreatureType::Cyclops], 7, 3)
}

/// Nyxborn Colossus — {3}{G}{G}{G} 6/7 Enchantment Creature — Giant (vanilla).
pub fn nyxborn_colossus() -> CardDefinition {
    nyxborn("Nyxborn Colossus", cost(&[generic(3), g(), g(), g()]), vec![CreatureType::Giant], 6, 7)
}

/// Nyxborn Courser — {1}{W}{W} 2/4 Enchantment Creature — Centaur Scout (vanilla).
pub fn nyxborn_courser() -> CardDefinition {
    nyxborn(
        "Nyxborn Courser",
        cost(&[generic(1), w(), w()]),
        vec![CreatureType::Centaur, CreatureType::Scout],
        2,
        4,
    )
}

/// Nyxborn Marauder — {2}{B}{B} 4/3 Enchantment Creature — Minotaur (vanilla).
pub fn nyxborn_marauder() -> CardDefinition {
    nyxborn("Nyxborn Marauder", cost(&[generic(2), b(), b()]), vec![CreatureType::Minotaur], 4, 3)
}

/// Nyxborn Seaguard — {2}{U}{U} 2/5 Enchantment Creature — Merfolk Soldier (vanilla).
pub fn nyxborn_seaguard() -> CardDefinition {
    nyxborn(
        "Nyxborn Seaguard",
        cost(&[generic(2), u(), u()]),
        vec![CreatureType::Merfolk, CreatureType::Soldier],
        2,
        5,
    )
}

/// Moss Viper — {G} 1/1 Snake with deathtouch.
pub fn moss_viper() -> CardDefinition {
    CardDefinition {
        name: "Moss Viper",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Discordant Piper — {1}{B} 2/1 Zombie Satyr. Dies → create a 0/1 white Goat.
pub fn discordant_piper() -> CardDefinition {
    CardDefinition {
        name: "Discordant Piper",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Satyr],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Goat".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goat],
                        ..Default::default()
                    },
                    power: 0,
                    toughness: 1,
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Grim Physician — {B} 1/1 Zombie. Dies → target creature an opponent
/// controls gets -1/-1 until end of turn.
pub fn grim_physician() -> CardDefinition {
    CardDefinition {
        name: "Grim Physician",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Careless Celebrant — {1}{R} 2/1 Satyr Shaman. Dies → 2 damage to target
/// creature or planeswalker an opponent controls.
pub fn careless_celebrant() -> CardDefinition {
    CardDefinition {
        name: "Careless Celebrant",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Rumbling Sentry — {3}{W}{W} 3/6 Giant. ETB scry 1.
pub fn rumbling_sentry() -> CardDefinition {
    CardDefinition {
        name: "Rumbling Sentry",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 3,
        toughness: 6,
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Elite Instructor — {2}{U} 2/2 Human Wizard. ETB: draw a card, then discard a card.
pub fn elite_instructor() -> CardDefinition {
    CardDefinition {
        name: "Elite Instructor",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]))],
        ..Default::default()
    }
}

/// Hyrax Tower Scout — {2}{G} 3/3 Human Scout. ETB: untap target creature.
pub fn hyrax_tower_scout() -> CardDefinition {
    CardDefinition {
        name: "Hyrax Tower Scout",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Untap {
            what: target_filtered(SelectionRequirement::Creature),
            up_to: None,
        })],
        ..Default::default()
    }
}

/// Eidolon of Philosophy — {U} 1/2 Enchantment Creature — Spirit.
/// {6}{U}, Sacrifice this creature: Draw three cards.
pub fn eidolon_of_philosophy() -> CardDefinition {
    CardDefinition {
        name: "Eidolon of Philosophy",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), u()]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ..ActivatedAbility::default()
        }],
        ..Default::default()
    }
}

/// Oread of Mountain's Blaze — {1}{R} 1/3 Enchantment Creature — Nymph.
/// {2}{R}, Discard a card: Draw a card.
pub fn oread_of_mountains_blaze() -> CardDefinition {
    CardDefinition {
        name: "Oread of Mountain's Blaze",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nymph], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..ActivatedAbility::default()
        }],
        ..Default::default()
    }
}

/// Lampad of Death's Vigil — {1}{B} 1/3 Enchantment Creature — Nymph.
/// {1}, Sacrifice a creature: Each opponent loses 1 life and you gain 1 life.
pub fn lampad_of_deaths_vigil() -> CardDefinition {
    CardDefinition {
        name: "Lampad of Death's Vigil",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nymph], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
            ..ActivatedAbility::default()
        }],
        ..Default::default()
    }
}

/// Captivating Unicorn — {4}{W} 4/4 Unicorn. Constellation — tap target
/// creature an opponent controls.
pub fn captivating_unicorn() -> CardDefinition {
    CardDefinition {
        name: "Captivating Unicorn",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Unicorn], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![constellation(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Pious Wayfarer — {W} 1/2 Human Scout. Constellation — target creature gets
/// +1/+1 until end of turn.
pub fn pious_wayfarer() -> CardDefinition {
    CardDefinition {
        name: "Pious Wayfarer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![constellation(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Sage of Mysteries — {U} 0/2 Human Wizard. Constellation — target player
/// mills two cards.
pub fn sage_of_mysteries() -> CardDefinition {
    CardDefinition {
        name: "Sage of Mysteries",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        triggered_abilities: vec![constellation(Effect::Mill {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Daybreak Chimera — {3}{W}{W} 3/3 flying Chimera. This spell costs {X} less
/// to cast, where X is your devotion to white (CR 700.5).
pub fn daybreak_chimera() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Daybreak Chimera",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Chimera], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is your devotion to white",
            effect: crate::effect::StaticEffect::SelfCostReducedByDevotion { colors: vec![Color::White] },
        }],
        ..Default::default()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// THB batch 2 — heroic / constellation / begin-combat / devotion bodies.
// ════════════════════════════════════════════════════════════════════════════

/// Hero of the Games — {2}{R} 3/2 Human Soldier. Heroic: team +1/+0 EOT.
pub fn hero_of_the_games() -> CardDefinition {
    CardDefinition {
        name: "Hero of the Games",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::heroic(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Eidolon of Inspiration — {1}{W}{W} 2/2 Enchantment Creature — Spirit. At
/// the beginning of combat on your turn, target creature you control gets
/// +2/+0 until end of turn.
pub fn eidolon_of_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Eidolon of Inspiration",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Favored of Iroas — {2}{W} 2/2 Human Soldier. Constellation — this creature
/// gains double strike until end of turn.
pub fn favored_of_iroas() -> CardDefinition {
    CardDefinition {
        name: "Favored of Iroas",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![constellation(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::DoubleStrike,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Glory Bearers — {3}{W} 3/4 Enchantment Creature — Human Cleric. Whenever
/// another creature you control attacks, it gets +0/+1 until end of turn.
pub fn glory_bearers() -> CardDefinition {
    CardDefinition {
        name: "Glory Bearers",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(0),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Pheres-Band Brawler — {4}{G}{G} 4/4 Centaur Warrior. ETB: it fights up to
/// one target creature you don't control.
pub fn pheres_band_brawler() -> CardDefinition {
    CardDefinition {
        name: "Pheres-Band Brawler",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Fight {
            attacker: Selector::This,
            defender: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Nylea's Huntmaster — {3}{G} 4/3 Centaur Shaman. ETB: target creature you
/// control gets +X/+0 until end of turn, where X is your devotion to green.
pub fn nyleas_huntmaster() -> CardDefinition {
    CardDefinition {
        name: "Nylea's Huntmaster",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::DevotionTo(vec![Color::Green]),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Reverent Hoplite — {4}{W} 1/2 Human Soldier. ETB: create a number of 1/1
/// white Human Soldier tokens equal to your devotion to white.
pub fn reverent_hoplite() -> CardDefinition {
    CardDefinition {
        name: "Reverent Hoplite",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::DevotionTo(vec![Color::White]),
            definition: TokenDefinition {
                name: "Human Soldier".into(),
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                    ..Default::default()
                },
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Rage-Scarred Berserker — {4}{B} 5/4 Minotaur Berserker. ETB: target
/// creature you control gets +1/+0 and gains indestructible until end of turn.
pub fn rage_scarred_berserker() -> CardDefinition {
    CardDefinition {
        name: "Rage-Scarred Berserker",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Berserker],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Leonin of the Lost Pride — {1}{W} 3/1 Cat Warrior. Dies: exile target card
/// from a graveyard. (Printed "opponent's graveyard"; rides the graveyard-card
/// target primitive, the controller picks which card.)
pub fn leonin_of_the_lost_pride() -> CardDefinition {
    CardDefinition {
        name: "Leonin of the Lost Pride",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
            },
        }],
        ..Default::default()
    }
}

/// Eutropia the Twice-Favored — {1}{G}{U} 2/2 legendary Human Wizard.
/// Constellation — put a +1/+1 counter on target creature; it gains flying EOT.
pub fn eutropia_the_twice_favored() -> CardDefinition {
    CardDefinition {
        name: "Eutropia the Twice-Favored",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![constellation(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Brine Giant — {6}{U} 5/6 Giant. Affinity for enchantments (costs {1} less
/// per enchantment you control, CR 702.41-style generic reduction).
pub fn brine_giant() -> CardDefinition {
    CardDefinition {
        name: "Brine Giant",
        cost: cost(&[generic(6), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 5,
        toughness: 6,
        affinity_filter: Some(
            SelectionRequirement::Enchantment.and(SelectionRequirement::ControlledByYou),
        ),
        ..Default::default()
    }
}

/// Loathsome Chimera — {2}{G} 4/1 Chimera. Escape—{4}{G}, exile three other
/// cards from your graveyard (CR 702.139).
pub fn loathsome_chimera() -> CardDefinition {
    CardDefinition {
        name: "Loathsome Chimera",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Chimera], ..Default::default() },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Escape(cost(&[generic(4), g()]), 3)],
        ..Default::default()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// THB batch 5 — Omen enchantments, escape bodies, devotion payoffs, blue tempo.
// ════════════════════════════════════════════════════════════════════════════

fn human_soldier_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// `{cost}, Sacrifice this enchantment: Scry 2.` — the shared Omen-cycle ability.
fn omen_sac_scry(mana: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        sac_cost: true,
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Omen of the Sun — {2}{W} Flash Enchantment. ETB: two 1/1 Human Soldiers +
/// gain 2. {2}{W}, Sacrifice: Scry 2.
pub fn omen_of_the_sun() -> CardDefinition {
    CardDefinition {
        name: "Omen of the Sun",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: human_soldier_token(),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        activated_abilities: vec![omen_sac_scry(cost(&[generic(2), w()]))],
        ..Default::default()
    }
}

/// Omen of the Forge — {1}{R} Flash Enchantment. ETB: 2 damage to any target.
/// {2}{R}, Sacrifice: Scry 2.
pub fn omen_of_the_forge() -> CardDefinition {
    CardDefinition {
        name: "Omen of the Forge",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::DealDamage { to: target_any(), amount: Value::Const(2) })],
        activated_abilities: vec![omen_sac_scry(cost(&[generic(2), r()]))],
        ..Default::default()
    }
}

/// Omen of the Hunt — {2}{G} Flash Enchantment. ETB: may fetch a basic land
/// tapped. {2}{G}, Sacrifice: Scry 2.
pub fn omen_of_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "Omen of the Hunt",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        activated_abilities: vec![omen_sac_scry(cost(&[generic(2), g()]))],
        ..Default::default()
    }
}

/// Mire's Grasp — {1}{B} Aura. Enchanted creature gets -3/-3.
pub fn mires_grasp() -> CardDefinition {
    CardDefinition {
        name: "Mire's Grasp",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: -3, toughness: -3, ..Default::default() }),
        ..Default::default()
    }
}

/// Mogis's Favor — {B} Aura with Escape—{2}{B}, exile two. Enchanted creature
/// gets +2/-1.
pub fn mogiss_favor() -> CardDefinition {
    CardDefinition {
        name: "Mogis's Favor",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Escape(cost(&[generic(2), b()]), 2)],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: 2, toughness: -1, ..Default::default() }),
        ..Default::default()
    }
}

/// Funeral Rites — {2}{B} Sorcery. Draw two, lose 2 life, then mill two.
pub fn funeral_rites() -> CardDefinition {
    CardDefinition {
        name: "Funeral Rites",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Soulreaper of Mogis — {2}{B} 2/3 Enchantment Creature. {2}{B}, Sacrifice a
/// creature: Draw a card.
pub fn soulreaper_of_mogis() -> CardDefinition {
    CardDefinition {
        name: "Soulreaper of Mogis",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Drag to the Underworld — {2}{B}{B} Instant. Costs {X} less, X = devotion to
/// black. Destroy target creature.
pub fn drag_to_the_underworld() -> CardDefinition {
    CardDefinition {
        name: "Drag to the Underworld",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![crate::card::StaticAbility {
            description: "This spell costs {X} less to cast, where X is your devotion to black",
            effect: crate::effect::StaticEffect::SelfCostReducedByDevotion { colors: vec![Color::Black] },
        }],
        effect: Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Deny the Divine — {2}{U} Instant. Counter target creature or enchantment
/// spell; exile it instead of binning it.
pub fn deny_the_divine() -> CardDefinition {
    CardDefinition {
        name: "Deny the Divine",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpellToZone {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Creature)
                    .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
            ),
            zone: crate::effect::CounteredSpellZone::Exile,
        },
        ..Default::default()
    }
}

/// Venomous Hierophant — {3}{B} 3/3 Deathtouch. ETB: mill three.
pub fn venomous_hierophant() -> CardDefinition {
    CardDefinition {
        name: "Venomous Hierophant",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gorgon, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) })],
        ..Default::default()
    }
}

/// Vexing Gull — {2}{U} 2/2 Flash Flying Bird.
pub fn vexing_gull() -> CardDefinition {
    CardDefinition {
        name: "Vexing Gull",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        ..Default::default()
    }
}

/// Riptide Turtle — {1}{U} 0/5 Flash Defender Turtle.
pub fn riptide_turtle() -> CardDefinition {
    CardDefinition {
        name: "Riptide Turtle",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Turtle], ..Default::default() },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Flash, Keyword::Defender],
        ..Default::default()
    }
}

/// Glimpse of Freedom — {1}{U} Instant with Escape—{2}{U}, exile five. Draw a
/// card.
pub fn glimpse_of_freedom() -> CardDefinition {
    CardDefinition {
        name: "Glimpse of Freedom",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Escape(cost(&[generic(2), u()]), 5)],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ..Default::default()
    }
}

/// Chain to Memory — {U} Instant. Target creature gets -4/-0 until end of turn.
/// Scry 2.
pub fn chain_to_memory() -> CardDefinition {
    CardDefinition {
        name: "Chain to Memory",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-4),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Whirlwind of Thought — {1}{U}{R}{W} Enchantment. Whenever you cast a
/// noncreature spell, draw a card.
pub fn whirlwind_of_thought() -> CardDefinition {
    CardDefinition {
        name: "Whirlwind of Thought",
        cost: cost(&[generic(1), u(), r(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }))),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Triton Waverider — {3}{U} 3/3 Merfolk Wizard. Constellation: gains flying
/// until end of turn.
pub fn triton_waverider() -> CardDefinition {
    CardDefinition {
        name: "Triton Waverider",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![constellation(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Aspect of Lamprey — {3}{B} Aura on a creature you control. ETB: target
/// opponent discards two. Enchanted creature has lifelink.
pub fn aspect_of_lamprey() -> CardDefinition {
    CardDefinition {
        name: "Aspect of Lamprey",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
        },
        equipped_bonus: Some(crate::card::EquipBonus { keywords: vec![Keyword::Lifelink], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
            random: false,
        })],
        ..Default::default()
    }
}

/// Underworld Charger — {2}{B} 3/3 Nightmare Horse that can't block. Escape—
/// {4}{B}, exile three; escapes with two +1/+1 counters.
pub fn underworld_charger() -> CardDefinition {
    CardDefinition {
        name: "Underworld Charger",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Horse],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::CantBlock, Keyword::Escape(cost(&[generic(4), b()]), 3)],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastFromEscape),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::ZERO),
            },
        )),
        ..Default::default()
    }
}

/// Pharika's Spawn — {3}{B} 3/4 Gorgon. Escape—{5}{B}, exile three; escapes
/// with two +1/+1 counters.
pub fn pharikas_spawn() -> CardDefinition {
    CardDefinition {
        name: "Pharika's Spawn",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gorgon], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Escape(cost(&[generic(5), b()]), 3)],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastFromEscape),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::ZERO),
            },
        )),
        ..Default::default()
    }
}

/// Tymaret, Chosen from Death — {B}{B} 2/* Legendary Demigod whose toughness
/// equals your devotion to black. {1}{B}: Exile target card from a graveyard
/// (the printed "up to two… gain 1 if a creature" is modeled as a single
/// target).
pub fn tymaret_chosen_from_death() -> CardDefinition {
    CardDefinition {
        name: "Tymaret, Chosen from Death",
        cost: cost(&[b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demigod], ..Default::default() },
        dynamic_pt: Some(DynamicPt::DevotionToToughness { color: Color::Black, base_p: 2 }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::InGraveyard),
                to: ZoneDest::Exile,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── THB batch 6 — devotion shrinks, escape, sacrifice payoffs, trample lord ───

/// Final Death — {4}{B} Instant. Exile target creature.
pub fn final_death() -> CardDefinition {
    CardDefinition {
        name: "Final Death",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Fruit of Tizerus — {B} Sorcery with Escape—{3}{B}, exile three. Target
/// player loses 2 life.
pub fn fruit_of_tizerus() -> CardDefinition {
    CardDefinition {
        name: "Fruit of Tizerus",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Escape(cost(&[generic(3), b()]), 3)],
        effect: Effect::LoseLife {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Skophos Warleader — {4}{R} 4/5 Minotaur Warrior. {R}, Sacrifice another
/// creature or an enchantment: this gets +1/+0 and gains menace until EOT.
pub fn skophos_warleader() -> CardDefinition {
    CardDefinition {
        name: "Skophos Warleader",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                1,
            )),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Threnody Singer — {1}{U} 1/3 Flash Flying Siren. ETB: target creature an
/// opponent controls gets -X/-0, X = your devotion to blue.
pub fn threnody_singer() -> CardDefinition {
    CardDefinition {
        name: "Threnody Singer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Siren], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Diff(Box::new(Value::ZERO), Box::new(Value::DevotionTo(vec![Color::Blue]))),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Blight-Breath Catoblepas — {4}{B}{B} 3/2 Beast. ETB: target creature an
/// opponent controls gets -X/-X, X = your devotion to black.
pub fn blight_breath_catoblepas() -> CardDefinition {
    let neg_devotion =
        Value::Diff(Box::new(Value::ZERO), Box::new(Value::DevotionTo(vec![Color::Black])));
    CardDefinition {
        name: "Blight-Breath Catoblepas",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: neg_devotion.clone(),
            toughness: neg_devotion,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Nylea's Forerunner — {4}{G} 5/3 Beast. Trample; other creatures you control
/// have trample.
pub fn nyleas_forerunner() -> CardDefinition {
    CardDefinition {
        name: "Nylea's Forerunner",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Other creatures you control have trample",
            effect: crate::effect::StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

// ── THB batch 7 — devotion lifegain, escape, omen recursion, combat payoffs ───

/// Setessan Petitioner — {1}{G}{G} 2/2 Human Druid. ETB: gain life equal to
/// your devotion to green.
pub fn setessan_petitioner() -> CardDefinition {
    CardDefinition {
        name: "Setessan Petitioner",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::DevotionTo(vec![Color::Green]),
        })],
        ..Default::default()
    }
}

/// Voracious Typhon — {2}{G}{G} 4/4 Snake Beast. Escape—{5}{G}{G}, exile four;
/// escapes with three +1/+1 counters.
pub fn voracious_typhon() -> CardDefinition {
    CardDefinition {
        name: "Voracious Typhon",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Escape(cost(&[generic(5), g(), g()]), 4)],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastFromEscape),
                then: Box::new(Value::Const(3)),
                else_: Box::new(Value::ZERO),
            },
        )),
        ..Default::default()
    }
}

/// Omen of the Dead — {B} Flash Enchantment. ETB: return target creature card
/// from your graveyard to your hand. {2}{B}, Sacrifice: Scry 2.
pub fn omen_of_the_dead() -> CardDefinition {
    CardDefinition {
        name: "Omen of the Dead",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![omen_sac_scry(cost(&[generic(2), b()]))],
        ..Default::default()
    }
}

/// Nessian Hornbeetle — {1}{G} 2/2 Insect. At the beginning of combat on your
/// turn, if you control another creature with power 4+, put a +1/+1 counter on
/// it.
pub fn nessian_hornbeetle() -> CardDefinition {
    CardDefinition {
        name: "Nessian Hornbeetle",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::PowerAtLeast(4))
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Phalanx Tactics — {1}{W} Instant. Target creature you control gets +2/+1;
/// each other creature you control gets +1/+1 until end of turn.
///
/// Modeled as "each creature you control gets +1/+1, then the target gets an
/// additional +1/+0" — nets +2/+1 on the target and +1/+1 on the rest.
pub fn phalanx_tactics() -> CardDefinition {
    CardDefinition {
        name: "Phalanx Tactics",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── THB batch 8 — white/red removal, escape auras, sacrifice burn ─────────────

/// Revoke Existence — {1}{W} Sorcery. Exile target artifact or enchantment.
pub fn revoke_existence() -> CardDefinition {
    CardDefinition {
        name: "Revoke Existence",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Sentinel's Eyes — {W} Aura with Escape—{W}, exile two. Enchanted creature
/// gets +1/+1 and has vigilance.
pub fn sentinels_eyes() -> CardDefinition {
    CardDefinition {
        name: "Sentinel's Eyes",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Escape(cost(&[w()]), 2)],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Indomitable Will — {1}{W} Flash Aura. Enchanted creature gets +1/+2.
pub fn indomitable_will() -> CardDefinition {
    CardDefinition {
        name: "Indomitable Will",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        ..Default::default()
    }
}

/// Triumphant Surge — {3}{W} Instant. Destroy target creature with power 4 or
/// greater. You gain 3 life.
pub fn triumphant_surge() -> CardDefinition {
    CardDefinition {
        name: "Triumphant Surge",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                ),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Final Flare — {2}{R} Instant. Additional cost: sacrifice a creature or an
/// enchantment. Deals 5 damage to target creature.
pub fn final_flare() -> CardDefinition {
    CardDefinition {
        name: "Final Flare",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
            count: 1,
        }],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}

/// Iroas's Blessing — {3}{R} Aura on a creature you control. ETB: 4 damage to
/// a creature or planeswalker an opponent controls. Enchanted creature gets
/// +1/+1.
pub fn iroass_blessing() -> CardDefinition {
    CardDefinition {
        name: "Iroas's Blessing",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
        },
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::Const(4),
        })],
        ..Default::default()
    }
}

/// Dreadful Apathy — {2}{W} Aura. Enchanted creature can't attack or block.
/// {2}{W}: Exile enchanted creature.
pub fn dreadful_apathy() -> CardDefinition {
    CardDefinition {
        name: "Dreadful Apathy",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Move {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: ZoneDest::Exile,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sea God's Scorn — {3}{U} Instant. Costs {1} less for each enchantment you
/// control. Return up to three target creatures to their owners' hands.
pub fn sea_gods_scorn() -> CardDefinition {
    CardDefinition {
        name: "Sea God's Scorn",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(SelectionRequirement::Enchantment.and(SelectionRequirement::ControlledByYou)),
        effect: Effect::ApplyToTargets {
            max_targets: 3,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        },
        ..Default::default()
    }
}

/// Wrap in Flames — {2}{R} Sorcery. Deals 1 damage to each of up to three
/// target creatures. Those creatures can't block this turn.
pub fn wrap_in_flames() -> CardDefinition {
    CardDefinition {
        name: "Wrap in Flames",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 3,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
        ..Default::default()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// THB extra batch (modern_decks rebase): cards not in the constellation batch.
// ════════════════════════════════════════════════════════════════════════════

/// Setessan Skirmisher — {1}{G} 2/1 Human Warrior. Constellation — whenever
/// an enchantment you control enters, this creature gets +1/+1 until end of turn.
pub fn setessan_skirmisher() -> CardDefinition {
    CardDefinition {
        name: "Setessan Skirmisher",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![constellation(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Gift of Strength — {1}{G} Instant. Target creature gets +3/+3 and gains
/// reach until end of turn.
pub fn gift_of_strength() -> CardDefinition {
    CardDefinition {
        name: "Gift of Strength",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Karametra's Blessing — {W} Instant. Target creature gets +2/+2 until end
/// of turn. If it's an enchanted creature or enchantment creature, it also
/// gains hexproof and indestructible until end of turn.
pub fn karametras_blessing() -> CardDefinition {
    CardDefinition {
        name: "Karametra's Blessing",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::IsEnchanted
                        .or(SelectionRequirement::Enchantment),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Underworld Fires — {1}{R} Sorcery. 1 damage to each creature and each
/// planeswalker. If a permanent dealt damage this way would die this turn,
/// exile it instead.
pub fn underworld_fires() -> CardDefinition {
    let each = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
        )
    };
    CardDefinition {
        name: "Underworld Fires",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: each() },
            Effect::ForEach {
                selector: each(),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(1),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Satyr's Cunning — {R} Sorcery. Create a 1/1 red Satyr with "can't block".
/// Escape — {2}{R}, exile two other cards.
pub fn satyrs_cunning() -> CardDefinition {
    CardDefinition {
        name: "Satyr's Cunning",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Escape(cost(&[generic(2), r()]), 2)],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Satyr".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Red],
                keywords: vec![Keyword::CantBlock],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Satyr],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Traveler's Amulet — {1} Artifact. {1}, Sacrifice this: search your library
/// for a basic land card, reveal it, put it into your hand, then shuffle.
pub fn travelers_amulet() -> CardDefinition {
    CardDefinition {
        name: "Traveler's Amulet",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Escape Velocity — {R} Aura. Enchant creature; enchanted creature gets
/// +1/+0 and has haste. Escape — {1}{R}, exile two other cards.
pub fn escape_velocity() -> CardDefinition {
    CardDefinition {
        name: "Escape Velocity",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Escape(cost(&[generic(1), r()]), 2)],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 0,
            keywords: vec![Keyword::Haste],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Setessan Training — {1}{G} Aura. Enchant creature; ETB draw a card;
/// enchanted creature gets +1/+0 and has trample.
pub fn setessan_training() -> CardDefinition {
    CardDefinition {
        name: "Setessan Training",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        },
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 0,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Staggering Insight — {W}{U} Aura. Enchant creature; enchanted creature
/// gets +1/+1, has lifelink, and "Whenever this creature deals combat damage
/// to a player, draw a card."
pub fn staggering_insight() -> CardDefinition {
    CardDefinition {
        name: "Staggering Insight",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Lifelink],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// THB fill batch — lands, spells, and creatures on existing primitives.
// ════════════════════════════════════════════════════════════════════════════

/// Scry-tapland helper: no basic land types, enters tapped, scry 1, taps for
/// either of two colors (the Theros "Temple" cycle).
fn temple(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![super::tap_add(a), super::tap_add(b)],
        triggered_abilities: vec![super::etb_tap_then_scry_one()],
        ..Default::default()
    }
}

/// Temple of Abandon — Land. Enters tapped, scry 1, {T}: Add {R} or {G}.
pub fn temple_of_abandon() -> CardDefinition {
    temple("Temple of Abandon", Color::Red, Color::Green)
}
/// Temple of Deceit — Land. Enters tapped, scry 1, {T}: Add {U} or {B}.
pub fn temple_of_deceit() -> CardDefinition {
    temple("Temple of Deceit", Color::Blue, Color::Black)
}
/// Temple of Enlightenment — Land. Enters tapped, scry 1, {T}: Add {W} or {U}.
pub fn temple_of_enlightenment() -> CardDefinition {
    temple("Temple of Enlightenment", Color::White, Color::Blue)
}
/// Temple of Malice — Land. Enters tapped, scry 1, {T}: Add {B} or {R}.
pub fn temple_of_malice() -> CardDefinition {
    temple("Temple of Malice", Color::Black, Color::Red)
}
/// Temple of Plenty — Land. Enters tapped, scry 1, {T}: Add {G} or {W}.
pub fn temple_of_plenty() -> CardDefinition {
    temple("Temple of Plenty", Color::Green, Color::White)
}

/// Fateful End — {2}{R} Instant. Deal 3 damage to any target, then scry 1.
pub fn fateful_end() -> CardDefinition {
    CardDefinition {
        name: "Fateful End",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Memory Drain — {2}{U}{U} Instant. Counter target spell, then scry 2.
pub fn memory_drain() -> CardDefinition {
    CardDefinition {
        name: "Memory Drain",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(SelectionRequirement::Any) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Scavenging Harpy — {2}{B} 2/1 Harpy. Flying. ETB: exile target card from
/// an opponent's graveyard.
pub fn scavenging_harpy() -> CardDefinition {
    CardDefinition {
        name: "Scavenging Harpy",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Harpy], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Exile {
            what: target_filtered(SelectionRequirement::InOpponentGraveyard),
        })],
        ..Default::default()
    }
}

/// Sphinx Mindbreaker — {5}{U}{U} 6/6 Sphinx. Flying. ETB: each opponent
/// mills ten cards.
pub fn sphinx_mindbreaker() -> CardDefinition {
    CardDefinition {
        name: "Sphinx Mindbreaker",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(10),
        })],
        ..Default::default()
    }
}

/// Thaumaturge's Familiar — {3} 1/3 artifact Bird. Flying. ETB: scry 1.
pub fn thaumaturges_familiar() -> CardDefinition {
    CardDefinition {
        name: "Thaumaturge's Familiar",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Mindwrack Harpy — {3}{B} 3/2 Enchantment Creature — Harpy. Flying. At the
/// beginning of combat on your turn, each player mills three cards.
pub fn mindwrack_harpy() -> CardDefinition {
    CardDefinition {
        name: "Mindwrack Harpy",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Harpy], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) },
        }],
        ..Default::default()
    }
}

/// Demon of Loathing — {5}{B}{B} 7/7 Demon. Flying, trample. Combat damage to
/// a player → that player sacrifices a creature of their choice.
pub fn demon_of_loathing() -> CardDefinition {
    CardDefinition {
        name: "Demon of Loathing",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                count: Value::ONE,
                filter: SelectionRequirement::Creature,
            },
        }],
        ..Default::default()
    }
}

/// Victory's Envoy — {3}{W}{W} 3/3 Human Cleric. At the beginning of your
/// upkeep, put a +1/+1 counter on each other creature you control.
pub fn victorys_envoy() -> CardDefinition {
    CardDefinition {
        name: "Victory's Envoy",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Witness of Tomorrows — {4}{U} 3/4 Enchantment Creature — Sphinx. Flying.
/// {3}{U}: Scry 1.
pub fn witness_of_tomorrows() -> CardDefinition {
    CardDefinition {
        name: "Witness of Tomorrows",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Towering-Wave Mystic — {1}{U} 2/1 Merfolk Wizard. Whenever this creature
/// deals damage, target player mills that many cards.
pub fn towering_wave_mystic() -> CardDefinition {
    CardDefinition {
        name: "Towering-Wave Mystic",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        // "Whenever this creature deals damage" — modeled on the two combat-
        // damage events (engine has no non-combat damage source on this body).
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::TriggerEventAmount,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToCreature,
                    EventScope::SelfSource,
                ),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..Default::default()
    }
}

/// Transcendent Envoy — {1}{W} 1/2 Enchantment Creature — Griffin. Flying.
/// Aura spells you cast cost {1} less to cast.
pub fn transcendent_envoy() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Transcendent Envoy",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Aura spells you cast cost {1} less to cast",
            effect: crate::effect::StaticEffect::CostReduction {
                filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Pharika's Libation — {2}{B} Instant. Choose one — target opponent
/// sacrifices a creature of their choice; or sacrifices an enchantment.
pub fn pharikas_libation() -> CardDefinition {
    CardDefinition {
        name: "Pharika's Libation",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: SelectionRequirement::Creature,
            },
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: SelectionRequirement::Enchantment,
            },
        ]),
        ..Default::default()
    }
}

/// Return to Nature — {1}{G} Instant. Choose one — destroy target artifact;
/// destroy target enchantment; or exile target card from a graveyard.
pub fn return_to_nature() -> CardDefinition {
    CardDefinition {
        name: "Return to Nature",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Enchantment) },
            Effect::Exile { what: target_filtered(SelectionRequirement::InGraveyard) },
        ]),
        ..Default::default()
    }
}

/// Portent of Betrayal — {3}{R} Sorcery. Gain control of target creature
/// until end of turn, untap it, it gains haste; then scry 1.
pub fn portent_of_betrayal() -> CardDefinition {
    CardDefinition {
        name: "Portent of Betrayal",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}


/// Renata, Called to the Hunt — {2}{G}{G} */3 Demigod. Power = devotion to
/// green. Other creatures you cast enter with an extra +1/+1 counter.
pub fn renata_called_to_the_hunt() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Renata, Called to the Hunt",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demigod], ..Default::default() },
        dynamic_pt: Some(DynamicPt::DevotionTo { color: Color::Green, base_t: 3 }),
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control enters with an additional +1/+1 counter.",
            effect: crate::effect::StaticEffect::ExtraEtbCountersForCreatureCasts {
                kind: CounterType::PlusOnePlusOne,
                value: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Nyx Lotus — {4} Legendary Artifact. Enters tapped. {T}: Choose a color,
/// add mana of that color equal to your devotion to it.
pub fn nyx_lotus() -> CardDefinition {
    CardDefinition {
        name: "Nyx Lotus",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![super::etb_tap()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::DevotionOfChosenColor,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flicker of Fate — {1}{W} Instant. Exile target creature or enchantment,
/// then return it to the battlefield under its owner's control.
pub fn flicker_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Flicker of Fate",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
            },
        ]),
        ..Default::default()
    }
}

// ── THB auras / equipment / constellation bodies ─────────────────────────────

/// Aura helper: enchant-creature `CardDefinition` that attaches on resolution.
fn creature_aura(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

/// Aspect of Manticore — {2}{R} Flash Aura. ETB: enchanted creature gains
/// first strike until end of turn. Enchanted creature gets +2/+0.
pub fn aspect_of_manticore() -> CardDefinition {
    use crate::card::EquipBonus;
    let mut c = creature_aura("Aspect of Manticore", cost(&[generic(2), r()]));
    c.keywords = vec![Keyword::Flash];
    c.triggered_abilities = vec![etb(Effect::GrantKeyword {
        what: Selector::AttachedTo(Box::new(Selector::This)),
        keyword: Keyword::FirstStrike,
        duration: Duration::EndOfTurn,
    })];
    c.equipped_bonus = Some(EquipBonus { power: 2, toughness: 0, ..Default::default() });
    c
}

/// Commanding Presence — {3}{W} Aura. Enchanted creature gets +2/+2, has first
/// strike, and "Combat damage to a player → make a 1/1 white Human Soldier."
pub fn commanding_presence() -> CardDefinition {
    use crate::card::EquipBonus;
    let mut c = creature_aura("Commanding Presence", cost(&[generic(3), w()]));
    c.equipped_bonus = Some(EquipBonus {
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: human_soldier_token(),
            },
        }],
        ..Default::default()
    });
    c
}

/// Hydra's Growth — {2}{G} Aura. ETB: +1/+1 counter on enchanted creature.
/// Your upkeep: double the +1/+1 counters on enchanted creature.
pub fn hydras_growth() -> CardDefinition {
    let mut c = creature_aura("Hydra's Growth", cost(&[generic(2), g()]));
    c.triggered_abilities = vec![
        etb(Effect::AddCounter {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        }),
        TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::DoubleCountersOnEach {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::PlusOnePlusOne,
            },
        },
    ];
    c
}

/// Warbriar Blessing — {1}{G} Aura (enchant creature you control). ETB:
/// enchanted creature fights up to one target creature you don't control.
/// Enchanted creature gets +0/+2.
pub fn warbriar_blessing() -> CardDefinition {
    use crate::card::EquipBonus;
    let mut c = creature_aura("Warbriar Blessing", cost(&[generic(1), g()]));
    c.effect = Effect::Attach {
        what: Selector::This,
        to: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
    };
    c.triggered_abilities = vec![etb(Effect::Fight {
        attacker: Selector::AttachedTo(Box::new(Selector::This)),
        defender: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
        ),
    })];
    c.equipped_bonus = Some(EquipBonus { power: 0, toughness: 2, ..Default::default() });
    c
}

/// Starlit Mantle — {1}{U} Flash Aura (enchant creature you control). ETB:
/// enchanted creature gains hexproof until end of turn. It gets +1/+1.
pub fn starlit_mantle() -> CardDefinition {
    use crate::card::EquipBonus;
    let mut c = creature_aura("Starlit Mantle", cost(&[generic(1), u()]));
    c.keywords = vec![Keyword::Flash];
    c.effect = Effect::Attach {
        what: Selector::This,
        to: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
    };
    c.triggered_abilities = vec![etb(Effect::GrantKeyword {
        what: Selector::AttachedTo(Box::new(Selector::This)),
        keyword: Keyword::Hexproof,
        duration: Duration::EndOfTurn,
    })];
    c.equipped_bonus = Some(EquipBonus { power: 1, toughness: 1, ..Default::default() });
    c
}

/// Mantle of the Wolf — {3}{G} Aura. Enchanted creature gets +4/+4. When this
/// Aura is put into a graveyard from the battlefield, make two 2/2 Wolves.
pub fn mantle_of_the_wolf() -> CardDefinition {
    use crate::card::EquipBonus;
    let mut c = creature_aura("Mantle of the Wolf", cost(&[generic(3), g()]));
    c.equipped_bonus = Some(EquipBonus { power: 4, toughness: 4, ..Default::default() });
    c.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Wolf".into(),
                power: 2,
                toughness: 2,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Wolf],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    }];
    c
}

/// Bronze Sword — {1} Equipment. Equipped creature gets +2/+0. Equip {3}.
pub fn bronze_sword() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Bronze Sword",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 0, ..Default::default() }),
        ..Default::default()
    }
}

/// Wings of Hubris — {2} Equipment. Equipped creature has flying. Sacrifice:
/// equipped creature can't be blocked this turn. Equip {1}.
pub fn wings_of_hubris() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Wings of Hubris",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nexus Wardens — {2}{G} 1/4 Satyr Archer. Reach. Constellation — gain 2 life.
pub fn nexus_wardens() -> CardDefinition {
    CardDefinition {
        name: "Nexus Wardens",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![constellation(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Nessian Wanderer — {1}{G} 1/3 Satyr Scout. Constellation — look at the top
/// three cards; you may put a land among them into your hand, rest on bottom.
pub fn nessian_wanderer() -> CardDefinition {
    CardDefinition {
        name: "Nessian Wanderer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![constellation(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: false,
            pick_filter: Some(SelectionRequirement::Land),
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Nyx Herald — {2}{G} 2/3 Enchantment Creature — Centaur Shaman. Begin combat
/// on your turn: target enchanted/enchantment creature you control gets +1/+1
/// and gains trample until end of turn.
pub fn nyx_herald() -> CardDefinition {
    CardDefinition {
        name: "Nyx Herald",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::IsEnchanted.or(SelectionRequirement::Enchantment)),
                    ),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Nadir Kraken — {1}{U}{U} 2/3 Kraken. Whenever you draw a card, you may pay
/// {1}. If you do, put a +1/+1 counter on this and make a 1/1 blue Tentacle.
pub fn nadir_kraken() -> CardDefinition {
    CardDefinition {
        name: "Nadir Kraken",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kraken], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to grow Nadir Kraken and make a Tentacle?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: TokenDefinition {
                            name: "Tentacle".into(),
                            power: 1,
                            toughness: 1,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Blue],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Tentacle],
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Sunmane Pegasus — {3}{W} 2/3 Pegasus. Flying. {1}{W}: gains vigilance and
/// lifelink until end of turn.
pub fn sunmane_pegasus() -> CardDefinition {
    CardDefinition {
        name: "Sunmane Pegasus",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pegasus], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skola Grovedancer — {1}{G} 2/2 Enchantment Creature — Satyr Druid. A land
/// card put into your graveyard → gain 1 life. {2}{G}: mill a card.
pub fn skola_grovedancer() -> CardDefinition {
    CardDefinition {
        name: "Skola Grovedancer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPutIntoGraveyard, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Mill { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── THB combat-restriction / loyalty-tax / utility-land bodies ───────────────

/// Labyrinth of Skophos — Land. {T}: Add {C}. {4}, {T}: Remove target
/// attacking or blocking creature from combat.
pub fn labyrinth_of_skophos() -> CardDefinition {
    CardDefinition {
        name: "Labyrinth of Skophos",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::RemoveFromCombat {
                    what: target_filtered(
                        SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Unknown Shores — Land. {T}: Add {C}. {1}, {T}: Add one mana of any color.
pub fn unknown_shores() -> CardDefinition {
    CardDefinition {
        name: "Unknown Shores",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Temple Thief — {1}{B} 2/2 Human Rogue. Can't be blocked by enchanted
/// creatures or enchantment creatures.
pub fn temple_thief() -> CardDefinition {
    CardDefinition {
        name: "Temple Thief",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(
            SelectionRequirement::IsEnchanted.or(SelectionRequirement::Enchantment),
        ))],
        ..Default::default()
    }
}

/// Serpent of Yawning Depths — {4}{U}{U} 6/6 Enchantment Creature — Serpent.
/// Krakens, Leviathans, Octopuses, and Serpents you control can't be blocked
/// except by Krakens, Leviathans, Octopuses, and Serpents.
pub fn serpent_of_yawning_depths() -> CardDefinition {
    use crate::card::StaticAbility;
    let sea = || {
        SelectionRequirement::HasCreatureType(CreatureType::Kraken)
            .or(SelectionRequirement::HasCreatureType(CreatureType::Leviathan))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Octopus))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Serpent))
    };
    CardDefinition {
        name: "Serpent of Yawning Depths",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 6,
        toughness: 6,
        static_abilities: vec![StaticAbility {
            description: "Sea creatures you control can't be blocked except by sea creatures.",
            effect: crate::effect::StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    sea().and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(sea())),
            },
        }],
        ..Default::default()
    }
}

/// Eidolon of Obstruction — {1}{W} 2/1 Enchantment Creature — Spirit. First
/// strike. Loyalty abilities of planeswalkers your opponents control cost {1}
/// more to activate.
pub fn eidolon_of_obstruction() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Eidolon of Obstruction",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Opponents' planeswalker loyalty abilities cost {1} more to activate.",
            effect: crate::effect::StaticEffect::OpponentLoyaltyActivationTax { amount: 1 },
        }],
        ..Default::default()
    }
}

// ── THB heroic / sacrifice-matters / aristocrat bodies ───────────────────────

fn satyr_cant_block_token() -> TokenDefinition {
    TokenDefinition {
        name: "Satyr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        keywords: vec![Keyword::CantBlock],
        subtypes: Subtypes { creature_types: vec![CreatureType::Satyr], ..Default::default() },
        ..Default::default()
    }
}

/// Heroic team-pump: "Whenever you cast a spell that targets this creature,
/// creatures you control get +1/+0 until end of turn." (the THB Hero cycle).
fn heroic_team_pump() -> TriggeredAbility {
    crate::effect::shortcut::heroic(Effect::PumpPT {
        what: Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        power: Value::ONE,
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    })
}

/// Hero of the Winds — {3}{W} 1/4 Human Soldier. Flying. Heroic: team +1/+0.
pub fn hero_of_the_winds() -> CardDefinition {
    CardDefinition {
        name: "Hero of the Winds",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![heroic_team_pump()],
        ..Default::default()
    }
}

/// Hero of the Nyxborn — {1}{R}{W} 2/2 Enchantment Creature — Human Soldier.
/// ETB: make a 1/1 Human Soldier. Heroic: team +1/+0.
pub fn hero_of_the_nyxborn() -> CardDefinition {
    CardDefinition {
        name: "Hero of the Nyxborn",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: human_soldier_token(),
            }),
            heroic_team_pump(),
        ],
        ..Default::default()
    }
}

/// Heroes of the Revel — {4}{R} 4/4 Satyr Soldier. ETB: make a 1/1 Satyr that
/// can't block. Heroic: team +1/+0.
pub fn heroes_of_the_revel() -> CardDefinition {
    CardDefinition {
        name: "Heroes of the Revel",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: satyr_cant_block_token(),
            }),
            heroic_team_pump(),
        ],
        ..Default::default()
    }
}

/// Irreverent Revelers — {2}{R} 2/2 Satyr. ETB: choose — destroy target
/// artifact; or this creature gains haste until end of turn.
pub fn irreverent_revelers() -> CardDefinition {
    CardDefinition {
        name: "Irreverent Revelers",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Satyr], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Stampede Rider — {2}{R} 2/3 Satyr. Trample. At the beginning of each
/// combat, if you control a creature with power 4+, it gets +1/+1 until EOT.
pub fn stampede_rider() -> CardDefinition {
    CardDefinition {
        name: "Stampede Rider",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Satyr], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::SelectorExists(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::PowerAtLeast(4)),
            ))),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Treeshaker Chimera — {5}{G}{G} 8/5 Chimera. All creatures able to block it
/// do so (Lure). When it dies, draw three cards.
pub fn treeshaker_chimera() -> CardDefinition {
    CardDefinition {
        name: "Treeshaker Chimera",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Chimera], ..Default::default() },
        power: 8,
        toughness: 5,
        keywords: vec![Keyword::AllMustBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        }],
        ..Default::default()
    }
}

/// Blood Aspirant — {1}{R} 1/1 Satyr Berserker. Sacrifice a permanent →
/// +1/+1 counter. {1}{R}, {T}, Sacrifice a creature or enchantment: deal 1 to
/// target creature; it can't block this turn.
pub fn blood_aspirant() -> CardDefinition {
    CardDefinition {
        name: "Blood Aspirant",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Berserker],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                1,
            )),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Creature),
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Slaughter-Priest of Mogis — {B}{R} 2/2 Minotaur Shaman. Sacrifice a
/// permanent → +2/+0 until end of turn. {2}, Sacrifice another creature or an
/// enchantment: first strike until end of turn.
pub fn slaughter_priest_of_mogis() -> CardDefinition {
    CardDefinition {
        name: "Slaughter-Priest of Mogis",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                1,
            )),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Underworld Sentinel — {3}{B}{B} 4/5 Skeleton Soldier. Attacks → exile a
/// creature card from your graveyard (linked). When it dies, put all cards
/// exiled with it onto the battlefield.
pub fn underworld_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Underworld Sentinel",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::ExileWithSource {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                    ),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
        ..Default::default()
    }
}

// ── THB intervention / enchantment-recursion / saga batch ────────────────────

/// Erebos's Intervention — {X}{B} Instant. Choose one — target creature gets
/// -X/-X until end of turn and you gain X life; or exile up to twice X target
/// cards from graveyards.
pub fn erebos_s_intervention() -> CardDefinition {
    let neg_x = Value::Diff(Box::new(Value::ZERO), Box::new(Value::XFromCost));
    CardDefinition {
        name: "Erebos's Intervention",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: neg_x.clone(),
                    toughness: neg_x,
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
            ]),
            Effect::ExileUpToNFromGraveyards {
                count: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
            },
        ]),
        ..Default::default()
    }
}

/// Chainweb Aracnir — {G} 1/2 Spider. Reach. ETB: deal damage equal to its
/// power to target creature with flying an opponent controls. Escape—{3}{G}{G},
/// exile four; escapes with three +1/+1 counters.
pub fn chainweb_aracnir() -> CardDefinition {
    CardDefinition {
        name: "Chainweb Aracnir",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Reach, Keyword::Escape(cost(&[generic(3), g(), g()]), 4)],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying))
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastFromEscape),
                then: Box::new(Value::Const(3)),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..Default::default()
    }
}

fn pegasus_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pegasus".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pegasus], ..Default::default() },
        ..Default::default()
    }
}

/// Archon of Sun's Grace — {2}{W}{W} 3/4 Archon. Flying, lifelink. Pegasus you
/// control have lifelink. Constellation — make a 2/2 white Pegasus with flying.
pub fn archon_of_suns_grace() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Archon of Sun's Grace",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Archon], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Pegasus creatures you control have lifelink.",
            effect: crate::effect::StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Pegasus)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Lifelink,
            },
        }],
        triggered_abilities: vec![constellation(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: pegasus_token(),
        })],
        ..Default::default()
    }
}

/// Archon of Falling Stars — {4}{W}{W} 4/4 Archon. Flying. When it dies, you
/// may return target enchantment card from your graveyard to the battlefield.
pub fn archon_of_falling_stars() -> CardDefinition {
    CardDefinition {
        name: "Archon of Falling Stars",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Archon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return an enchantment from your graveyard to the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Enchantment
                            .and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Elspeth's Nightmare — {2}{B} Saga. I: destroy target opponent creature with
/// power ≤ 2. II: target opponent discards a chosen noncreature, nonland card.
/// III: exile target opponent's graveyard.
pub fn elspeths_nightmare() -> CardDefinition {
    CardDefinition {
        name: "Elspeth's Nightmare",
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
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent)
                            .and(SelectionRequirement::PowerAtMost(2)),
                    ),
                },
            ),
            (
                2,
                Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::Target(0)),
                    count: Value::ONE,
                    filter: SelectionRequirement::Noncreature.and(SelectionRequirement::Nonland),
                },
            ),
            (3, Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0) }),
        ],
        ..Default::default()
    }
}

/// Alirios, Enraptured — {2}{U} 2/3 Legendary Human. Enters tapped; ETB make a
/// 3/2 blue Reflection. (The "doesn't untap while you control a Reflection"
/// drawback is approximated — it untaps normally.)
pub fn alirios_enraptured() -> CardDefinition {
    CardDefinition {
        name: "Alirios, Enraptured",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::Tap { what: Selector::This }),
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Reflection".into(),
                    power: 3,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Reflection],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
        ],
        ..Default::default()
    }
}

// ── THB "first spell each opponent's turn" payoffs ───────────────────────────

/// "Whenever you cast your first spell during each opponent's turn, `body`."
/// (CR 603.2 — a `SpellCast`/`YourControl` trigger gated on it not being your
/// turn and being your first spell that turn; the Wavebreak Hippocamp shape.)
fn first_spell_each_opponents_turn(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::All(vec![
                Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::ONE },
            ]),
        ),
        effect: body,
    }
}

/// Arena Trickster — {3}{R} 3/3 Human Shaman. First spell each opponent's turn
/// → put a +1/+1 counter on this creature.
pub fn arena_trickster() -> CardDefinition {
    CardDefinition {
        name: "Arena Trickster",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![first_spell_each_opponents_turn(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Dreamstalker Manticore — {2}{R} 4/2 Enchantment Creature — Manticore. First
/// spell each opponent's turn → deal 1 damage to any target.
pub fn dreamstalker_manticore() -> CardDefinition {
    CardDefinition {
        name: "Dreamstalker Manticore",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Manticore], ..Default::default() },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![first_spell_each_opponents_turn(Effect::DealDamage {
            to: target_any(),
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Mischievous Chimera — {U}{R} 2/2 Enchantment Creature — Chimera. Flying.
/// First spell each opponent's turn → deal 1 to each opponent, then scry 1.
pub fn mischievous_chimera() -> CardDefinition {
    CardDefinition {
        name: "Mischievous Chimera",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Chimera], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![first_spell_each_opponents_turn(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Stinging Lionfish — {1}{U} 2/1 Enchantment Creature — Fish. First spell each
/// opponent's turn → you may tap or untap target nonland permanent.
pub fn stinging_lionfish() -> CardDefinition {
    CardDefinition {
        name: "Stinging Lionfish",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![first_spell_each_opponents_turn(Effect::MayDo {
            description: "Tap or untap target nonland permanent?".into(),
            body: Box::new(Effect::ChooseMode(vec![
                Effect::Tap { what: target_filtered(SelectionRequirement::Nonland) },
                Effect::Untap { what: target_filtered(SelectionRequirement::Nonland), up_to: None },
            ])),
        })],
        ..Default::default()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// THB modern_decks batch — creatures, auras, artifacts on existing primitives.
// ════════════════════════════════════════════════════════════════════════════

/// Terror of Mount Velus — {5}{R}{R} 5/5 Dragon. Flying, double strike; ETB →
/// creatures you control gain double strike until end of turn.
pub fn terror_of_mount_velus() -> CardDefinition {
    CardDefinition {
        name: "Terror of Mount Velus",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Thundering Chariot — {4} Artifact — Vehicle 3/3. First strike, trample,
/// haste; Crew 1.
pub fn thundering_chariot() -> CardDefinition {
    CardDefinition {
        name: "Thundering Chariot",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Trample, Keyword::Haste, Keyword::Crew(1)],
        ..Default::default()
    }
}

/// Wolfwillow Haven — {1}{G} Aura. Enchant land; enchanted land tapped for mana
/// adds an additional {G}. {4}{G}, Sacrifice this Aura: make a 2/2 Wolf (only
/// during your turn).
pub fn wolfwillow_haven() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{ExtraManaKind, StaticEffect};
    CardDefinition {
        name: "Wolfwillow Haven",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Land },
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land tapped for mana adds an additional {G}.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: true,
                filter: SelectionRequirement::Any,
                extra: ExtraManaKind::Fixed(Color::Green),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            sac_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Wolf".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Wolf],
                        ..Default::default()
                    },
                    power: 2,
                    toughness: 2,
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mirror Shield — {2} Equipment. Equipped creature gets +0/+2 and has
/// hexproof. Equip {2}. (The deathtouch-blocker destroy clause is omitted.)
pub fn mirror_shield() -> CardDefinition {
    CardDefinition {
        name: "Mirror Shield",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 0,
            toughness: 2,
            keywords: vec![Keyword::Hexproof],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Shimmerwing Chimera — {3}{U} 3/2 Enchantment Creature — Chimera. Flying; at
/// the beginning of your upkeep, return up to one other target enchantment you
/// control to its owner's hand.
pub fn shimmerwing_chimera() -> CardDefinition {
    CardDefinition {
        name: "Shimmerwing Chimera",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Chimera], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Return another enchantment you control to hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Enchantment
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Thryx, the Sudden Storm — {3}{U}{U} 4/5 Elemental Giant. Flash, flying;
/// spells you cast with mana value 5+ cost {1} less and can't be countered.
pub fn thryx_the_sudden_storm() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Thryx, the Sudden Storm",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Giant],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Spells you cast with mana value 5 or greater cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::ManaValueAtLeast(5),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Spells you cast with mana value 5 or greater can't be countered.",
                effect: StaticEffect::SpellsUncounterable {
                    filter: SelectionRequirement::ManaValueAtLeast(5),
                },
            },
        ],
        ..Default::default()
    }
}


/// Sleep of the Dead — {U} Sorcery. Tap target creature; it doesn't untap
/// during its controller's next untap step. Escape—{2}{U}, exile three cards.
pub fn sleep_of_the_dead() -> CardDefinition {
    CardDefinition {
        name: "Sleep of the Dead",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Escape(cost(&[generic(2), u()]), 3)],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Inevitable End — {2}{B} Aura. Enchant creature; enchanted creature has
/// "At the beginning of your upkeep, sacrifice a creature."
pub fn inevitable_end() -> CardDefinition {
    CardDefinition {
        name: "Inevitable End",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: SelectionRequirement::Creature,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Impending Doom — {2}{R} Aura. Enchant creature; enchanted creature gets
/// +3/+3 and attacks each combat if able. When it dies, this Aura deals 3
/// damage to that creature's controller.
pub fn impending_doom() -> CardDefinition {
    CardDefinition {
        name: "Impending Doom",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::MustAttack],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::You),
                    amount: Value::Const(3),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Naiad of Hidden Coves — {2}{U} 2/3 Enchantment Creature — Nymph. During
/// turns other than yours, spells you cast cost {1} less to cast.
pub fn naiad_of_hidden_coves() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Naiad of Hidden Coves",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nymph], ..Default::default() },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "During turns other than yours, spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReductionDuringOpponentsTurn {
                filter: SelectionRequirement::Any,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Grasping Giant — {5}{W} 5/7 Giant. Vigilance; whenever it becomes blocked by
/// a creature, exile that creature until Grasping Giant leaves the battlefield.
pub fn grasping_giant() -> CardDefinition {
    CardDefinition {
        name: "Grasping Giant",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: Selector::BlockingCreatures,
                return_to: crate::card::ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

/// Sunlit Hoplite — {1}{W} 2/1 Human Soldier. Has first strike during your
/// turn; gets +1/+0 while you control an Elspeth planeswalker.
pub fn sunlit_hoplite() -> CardDefinition {
    use crate::card::{PlaneswalkerSubtype, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Sunlit Hoplite",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, this creature has first strike.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::IsTurnOf(PlayerRef::You),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::FirstStrike],
                },
            },
            StaticAbility {
                description: "Gets +1/+0 while you control an Elspeth planeswalker.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            SelectionRequirement::Planeswalker
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::HasPlaneswalkerType(PlaneswalkerSubtype::Elspeth)),
                        ),
                        n: Value::ONE,
                    },
                    power: 1,
                    toughness: 0,
                    keywords: vec![],
                },
            },
        ],
        ..Default::default()
    }
}

/// Swimmer in Nightmares — {2}{U} 1/4 Nightmare Merfolk. +3/+0 while ten or
/// more cards are in a single graveyard; can't be blocked while you control an
/// Ashiok planeswalker.
pub fn swimmer_in_nightmares() -> CardDefinition {
    use crate::card::{PlaneswalkerSubtype, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Swimmer in Nightmares",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Merfolk],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Gets +3/+0 while ten or more cards are in a single graveyard.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::ValueAtLeast(Value::MaxGraveyardSize, Value::Const(10)),
                    power: 3,
                    toughness: 0,
                    keywords: vec![],
                },
            },
            StaticAbility {
                description: "Can't be blocked while you control an Ashiok planeswalker.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            SelectionRequirement::Planeswalker
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::HasPlaneswalkerType(PlaneswalkerSubtype::Ashiok)),
                        ),
                        n: Value::ONE,
                    },
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Unblockable],
                },
            },
        ],
        ..Default::default()
    }
}

/// Gold token (CR 111.10) — artifact, "Sacrifice this token: Add one mana of
/// any color." (Treasure without the {T}.)
fn gold_token() -> TokenDefinition {
    TokenDefinition {
        name: "Gold".into(),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The First Iroan Games — {2}{G} Saga. I: 1/1 Human Soldier. II: three +1/+1
/// counters on a creature you control. III: if you control a power-4+ creature,
/// draw two. IV: make a Gold token.
pub fn the_first_iroan_games() -> CardDefinition {
    CardDefinition {
        name: "The First Iroan Games",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: human_soldier_token() }),
            (
                2,
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
            ),
            (
                3,
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::PowerAtLeast(4)),
                        ),
                        n: Value::ONE,
                    },
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
            ),
            (4, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: gold_token() }),
        ],
        ..Default::default()
    }
}

/// The Binding of the Titans — {1}{G} Saga. I: each player mills three. II:
/// exile up to two target cards from graveyards. III: return a creature or land
/// card from your graveyard to your hand. (The per-creature life gain on II is
/// omitted.)
pub fn the_binding_of_the_titans() -> CardDefinition {
    CardDefinition {
        name: "The Binding of the Titans",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) }),
            (2, Effect::ExileUpToNFromGraveyards { count: Value::Const(2) }),
            (
                3,
                Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                            filter: SelectionRequirement::Creature.or(SelectionRequirement::Land),
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Kiora Bests the Sea God — {5}{U}{U} Saga. I: 8/8 hexproof Kraken. II: tap
/// each nonland permanent your opponents control; it stays tapped through their
/// next untap. III: gain control of a permanent an opponent controls, untapped.
pub fn kiora_bests_the_sea_god() -> CardDefinition {
    let kraken = TokenDefinition {
        name: "Kraken".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Hexproof],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kraken], ..Default::default() },
        power: 8,
        toughness: 8,
        ..Default::default()
    };
    let opp_nonland = || {
        Selector::EachPermanent(
            SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
        )
    };
    CardDefinition {
        name: "Kiora Bests the Sea God",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: kraken }),
            (
                2,
                Effect::Seq(vec![
                    Effect::Tap { what: opp_nonland() },
                    Effect::AddCounter { what: opp_nonland(), kind: CounterType::Stun, amount: Value::ONE },
                ]),
            ),
            (
                3,
                Effect::GainControl {
                    what: target_filtered(
                        SelectionRequirement::Permanent.and(SelectionRequirement::ControlledByOpponent),
                    ),
                    to: Some(PlayerRef::You),
                    duration: Duration::Permanent,
                },
            ),
        ],
        ..Default::default()
    }
}

/// The Akroan War — {3}{R} Saga. I: gain control of a creature while this Saga
/// remains. II: until your next turn, creatures your opponents control attack
/// each combat if able. III: each tapped creature deals damage to itself equal
/// to its power.
pub fn the_akroan_war() -> CardDefinition {
    CardDefinition {
        name: "The Akroan War",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::GainControlWhileSourceRemains {
                    what: target_filtered(SelectionRequirement::Creature),
                },
            ),
            (
                2,
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                    ),
                    keyword: Keyword::MustAttack,
                    duration: Duration::UntilNextTurn,
                },
            ),
            (
                3,
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
                    ),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Thassa's Intervention — {X}{U}{U} Instant. Choose one — look at the top X
/// cards of your library, put up to two into your hand and the rest on the
/// bottom in a random order; or counter target spell unless its controller pays
/// twice X.
pub fn thassas_intervention() -> CardDefinition {
    CardDefinition {
        name: "Thassa's Intervention",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::XFromCost,
                rest_to_graveyard: false,
                pick_filter: None,
                take: Some(Value::Const(2)),
                to_battlefield: false,
            },
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: crate::mana::ManaCost::default(),
                exile: false,
                extra_generic: Some(Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost))),
            },
        ]),
        ..Default::default()
    }
}

/// Relentless Pursuit — {2}{G} Sorcery. Reveal the top four cards; put a
/// creature and/or land from among them into your hand, the rest into your
/// graveyard. (Modeled as "take up to two creature/land cards".)
pub fn relentless_pursuit() -> CardDefinition {
    CardDefinition {
        name: "Relentless Pursuit",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: true,
            pick_filter: Some(SelectionRequirement::Creature.or(SelectionRequirement::Land)),
            take: Some(Value::Const(2)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Furious Rise — {2}{R} Enchantment. At the beginning of your end step, if you
/// control a creature with power 4 or greater, exile the top card of your
/// library; you may play it while it remains exiled.
pub fn furious_rise() -> CardDefinition {
    CardDefinition {
        name: "Furious Rise",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::WhileExiled,
                    pay_any_color: false,
                    uncast_penalty: None,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}


/// Nightmare Shepherd — {2}{B}{B} 4/4 Demon. Flying. Whenever another nontoken
/// creature you control dies, you may exile it; if you do, make a token that's
/// a copy of it except it's a 1/1 Nightmare.
pub fn nightmare_shepherd() -> CardDefinition {
    CardDefinition {
        name: "Nightmare Shepherd",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::NotToken.and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::MayDo {
                description: "Exile it to make a 1/1 Nightmare copy?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::TriggerSource,
                        extra_creature_types: vec![CreatureType::Nightmare],
                        override_pt: Some((1, 1)),
                        non_legendary: false,
                    },
                    Effect::Exile { what: Selector::TriggerSource },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Rise to Glory — {3}{W}{B} Sorcery. Choose one or both — return target
/// creature card from your graveyard to the battlefield; and/or return target
/// Aura card from your graveyard to the battlefield.
pub fn rise_to_glory() -> CardDefinition {
    CardDefinition {
        name: "Rise to Glory",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                            .and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ],
        },
        ..Default::default()
    }
}

/// Lagonna-Band Storyteller — {3}{W} 3/4 Centaur Advisor. ETB: you may put a
/// target enchantment card from your graveyard on top of your library; if you
/// do, gain life equal to its mana value.
pub fn lagonna_band_storyteller() -> CardDefinition {
    CardDefinition {
        name: "Lagonna-Band Storyteller",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Put an enchantment from your graveyard on top, gain its mana value?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                },
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Enchantment.and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Purphoros's Intervention — {X}{R} Sorcery. Choose one — create an X/1 red
/// Elemental with trample and haste, sacrificed at the next end step; or deal
/// twice X damage to target creature or planeswalker.
pub fn purphoross_intervention() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        keywords: vec![Keyword::Trample, Keyword::Haste],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        dynamic_pt: Some((Value::XFromCost, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::AnyPlayer,
            ),
            effect: Effect::SacrificeSource,
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Purphoros's Intervention",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: elemental },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
            },
        ]),
        ..Default::default()
    }
}

/// Dalakos, Crafter of Wonders — {1}{U}{R} 2/4 legendary Merfolk Artificer.
/// {T}: Add {C}{C}, spend only on artifacts. Equipped creatures you control
/// have flying and haste.
pub fn dalakos_crafter_of_wonders() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{ManaPayload, StaticEffect};
    CardDefinition {
        name: "Dalakos, Crafter of Wonders",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(2))),
                    crate::mana::SpendRestriction::ArtifactOnly,
                ),
            },
            ..Default::default()
        }],
        // Per-creature `GrantPumpSelfIf` so the IsEquipped condition is judged
        // with each creature's own battlefield context (a bare `GrantKeyword`
        // selector evaluates IsEquipped without attachment state and misses).
        static_abilities: vec![StaticAbility {
            description: "Equipped creatures you control have flying and haste.",
            effect: StaticEffect::GrantPumpSelfIf {
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::IsEquipped,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying, Keyword::Haste],
            },
        }],
        ..Default::default()
    }
}

/// The Triumph of Anax — {2}{R} Saga. I, II, III: target creature gains trample
/// and +X/+0 until end of turn, where X is the number of lore counters on this
/// Saga. IV: a creature you control fights one you don't.
pub fn the_triumph_of_anax() -> CardDefinition {
    let pump = || {
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Lore,
                },
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    CardDefinition {
        name: "The Triumph of Anax",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, pump()),
            (2, pump()),
            (3, pump()),
            (
                4,
                Effect::Fight {
                    attacker: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    },
                    defender: Selector::TargetFiltered {
                        slot: 1,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    },
                },
            ),
        ],
        ..Default::default()
    }
}

/// Warden of the Chained — {1}{R}{G} 4/4 Minotaur Warrior. Trample; can't attack
/// unless you control another creature with power 4 or greater.
pub fn warden_of_the_chained() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Warden of the Chained",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Can't attack unless you control another creature with power 4 or greater.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource)
                            .and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                })),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::CantAttack],
            },
        }],
        ..Default::default()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// THB planeswalkers, gods, and the aura-matters tail (modern_decks finish pass)
// ════════════════════════════════════════════════════════════════════════════

/// Elspeth, Sun's Nemesis — {2}{W}{W} Elspeth planeswalker, 5 loyalty.
/// −1: up to two creatures you control get +2/+1. −2: make two 1/1 Soldiers.
/// −3: gain 5 life. Escape—{4}{W}{W}, exile four other graveyard cards.
pub fn elspeth_suns_nemesis() -> CardDefinition {
    CardDefinition {
        name: "Elspeth, Sun's Nemesis",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth],
            ..Default::default()
        },
        keywords: vec![Keyword::Escape(cost(&[generic(4), w(), w()]), 4)],
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                    effect: Box::new(Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::Const(2),
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: soldier_token(),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// 1/1 white Human Soldier token.
fn soldier_token() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Gravebreaker Lamia — {4}{B} 4/4 Snake Lamia with Lifelink. ETB: search
/// your library for a card and put it into your graveyard. Spells you cast
/// from your graveyard cost {1} less.
pub fn gravebreaker_lamia() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Gravebreaker Lamia",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Lamia],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Graveyard,
        })],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast from your graveyard cost {1} less to cast.",
            effect: StaticEffect::GraveyardCastCostReduction { amount: 1 },
        }],
        ..Default::default()
    }
}

/// Calix, Destiny's Hand — {2}{G}{W} Calix planeswalker, 4 loyalty.
/// +1: dig four for an enchantment to hand, rest to bottom. −3: exile a
/// creature/enchantment you don't control until Calix leaves. −7: return all
/// enchantment cards from your graveyard to the battlefield.
pub fn calix_destinys_hand() -> CardDefinition {
    use crate::card::ExileReturnZone;
    use crate::effect::ZoneRef;
    CardDefinition {
        name: "Calix, Destiny's Hand",
        cost: cost(&[generic(2), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Calix],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    rest_to_graveyard: false,
                    pick_filter: Some(SelectionRequirement::Enchantment),
                    take: None,
                    to_battlefield: false,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::ExileUntilSourceLeaves {
                    what: target_filtered(
                        (SelectionRequirement::Creature.or(SelectionRequirement::Enchantment))
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    return_to: ExileReturnZone::Battlefield,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Move {
                    what: Selector::EachMatching {
                        zone: ZoneRef::Graveyard(PlayerRef::You),
                        filter: SelectionRequirement::Enchantment,
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// 2/3 blue-and-black Nightmare token whose attack/block raids each
/// opponent's library for two cards.
fn ashiok_nightmare_token() -> TokenDefinition {
    let raid = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::ExileTopOfLibrary {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
            link_to_source: false,
            face_down: false,
        },
    };
    TokenDefinition {
        name: "Nightmare".into(),
        power: 2,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare],
            ..Default::default()
        },
        triggered_abilities: vec![raid(EventKind::Attacks), raid(EventKind::Blocks)],
        ..Default::default()
    }
}

/// Ashiok, Nightmare Muse — {3}{U}{B} Ashiok planeswalker, 5 loyalty.
/// +1: make a 2/3 Nightmare that mills opponents on attack/block. −3: bounce
/// a nonland permanent, then its owner exiles a card from hand. −7: cast up
/// to three opponent-owned cards from exile for free this turn.
pub fn ashiok_nightmare_muse() -> CardDefinition {
    CardDefinition {
        name: "Ashiok, Nightmare Muse",
        cost: cost(&[generic(3), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ashiok],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: ashiok_nightmare_token(),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Nonland),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    },
                    Effect::ExileFromHand {
                        who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CastUpToNFromOpponentsExile { count: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── THB batch (modern_decks): missing commons/uncommons on existing primitives ──

/// Skophos Maze-Warden — {3}{R} 3/4 Minotaur Warrior. {1}: this gets +1/-1
/// until end of turn. (The Labyrinth-of-Skophos fight rider is dropped — it
/// keys off a specific named land's targeted ability.)
pub fn skophos_maze_warden() -> CardDefinition {
    CardDefinition {
        name: "Skophos Maze-Warden",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Incendiary Oracle — {1}{R} 2/2 Human Shaman. {1}{R}: +1/+0 until end of
/// turn. Creatures it damages that would die are exiled instead.
pub fn incendiary_oracle() -> CardDefinition {
    CardDefinition {
        name: "Incendiary Oracle",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        damage_exiles_if_dies: true,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shoal Kraken — {4}{U} 3/5 Kraken. Constellation — whenever an enchantment
/// you control enters, you may draw a card, then discard a card.
pub fn shoal_kraken() -> CardDefinition {
    CardDefinition {
        name: "Shoal Kraken",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kraken], ..Default::default() },
        power: 3,
        toughness: 5,
        triggered_abilities: vec![constellation(Effect::MayDo {
            description: "draw a card, then discard a card".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ])),
        })],
        ..Default::default()
    }
}

/// Ilysian Caryatid — {1}{G} 1/1 Plant. {T}: Add one mana of any color; add
/// two mana of one color instead if you control a power-4-or-greater creature.
pub fn ilysian_caryatid() -> CardDefinition {
    CardDefinition {
        name: "Ilysian Caryatid",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::Const(2)),
                }),
                else_: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elspeth, Undaunted Hero — {2}{W}{W}{W} Elspeth planeswalker, 5 loyalty.
/// +2: +1/+1 counter on up to two target creatures. −2: search library/
/// graveyard for Sunlit Hoplite onto the battlefield. −8: until end of turn
/// your creatures gain flying and get +X/+X, X = your devotion to white.
pub fn elspeth_undaunted_hero() -> CardDefinition {
    let mine = || {
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)
    };
    CardDefinition {
        name: "Elspeth, Undaunted Hero",
        cost: cost(&[generic(2), w(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    filter: SelectionRequirement::Creature,
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasName("Sunlit Hoplite".into()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::EachPermanent(mine()),
                        power: Value::DevotionTo(vec![Color::White]),
                        toughness: Value::DevotionTo(vec![Color::White]),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(mine()),
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Elspeth's Devotee — {2}{W}{W} 3/3 Human Soldier. ETB: you may search your
/// library for Elspeth, Undaunted Hero and put it into your hand. (The
/// graveyard half is dropped — Search only walks the library.)
pub fn elspeths_devotee() -> CardDefinition {
    CardDefinition {
        name: "Elspeth's Devotee",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasName("Elspeth, Undaunted Hero".into()),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Ashiok's Forerunner — {3}{U}{B} 3/3 Human Wizard with Flash. ETB: you may
/// search your library for Ashiok, Sculptor of Fears and put it into your hand.
pub fn ashioks_forerunner() -> CardDefinition {
    CardDefinition {
        name: "Ashiok's Forerunner",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasName("Ashiok, Sculptor of Fears".into()),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Enemy of Enlightenment — {5}{B} 5/5 Demon with Flying. Gets -1/-1 for each
/// card in your opponents' hands. At your upkeep, each player discards a card.
pub fn enemy_of_enlightenment() -> CardDefinition {
    CardDefinition {
        name: "Enemy of Enlightenment",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::BaseMinusOpponentsHandTotal { base_p: 5, base_t: 5 }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Ashiok, Sculptor of Fears — {4}{U}{B} Ashiok planeswalker, 4 loyalty.
/// +2: draw a card; each player mills two. −5: reanimate a creature card from
/// a graveyard under your control. −11: gain control of all creatures your
/// opponents control. (Single-target-opponent clause widened to all opponents
/// — identical in two-player.)
pub fn ashiok_sculptor_of_fears() -> CardDefinition {
    CardDefinition {
        name: "Ashiok, Sculptor of Fears",
        cost: cost(&[generic(4), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ashiok],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Mill {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(2),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -11,
                effect: Effect::GainControl {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    to: Some(PlayerRef::You),
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ichthyomorphosis — {2}{U} Aura. Enchant creature. Enchanted creature loses
/// all abilities and is a blue Fish with base power and toughness 0/1.
pub fn ichthyomorphosis() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Ichthyomorphosis",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((0, 1)),
            set_creature_types: Some(vec![CreatureType::Fish]),
            set_colors: Some(vec![Color::Blue]),
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// One with the Stars — {3}{U} Aura. Enchant creature or enchantment.
/// Enchanted permanent is an enchantment and loses all other card types. (It
/// keeps its abilities, so it's no longer a creature.)
pub fn one_with_the_stars() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "One with the Stars",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
            ),
        },
        equipped_bonus: Some(EquipBonus {
            set_card_types: Some(vec![CardType::Enchantment]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Heliod's Punishment — {1}{W} Aura. Enchant creature. Enchanted creature
/// can't attack or block and loses all abilities. (The four task-counter
/// self-removal timer is dropped — the lock is modeled as permanent, the
/// printed play pattern of neutralizing the creature.)
pub fn heliods_punishment() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Heliod's Punishment",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            remove_abilities: true,
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Nylea's Intervention — {X}{G}{G} Sorcery. Choose one — search your library
/// for up to X land cards and put them into your hand; or deal twice X damage
/// to each creature with flying.
pub fn nyleas_intervention() -> CardDefinition {
    CardDefinition {
        name: "Nylea's Intervention",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::XFromCost,
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
                amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
            },
        ]),
        ..Default::default()
    }
}

/// Deathbellow War Cry — {5}{R}{R}{R} Sorcery. Search your library for up to
/// four Minotaur creature cards and put them onto the battlefield, then
/// shuffle. (The "different names" rider is not enforced.)
pub fn deathbellow_war_cry() -> CardDefinition {
    CardDefinition {
        name: "Deathbellow War Cry",
        cost: cost(&[generic(5), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::HasCreatureType(CreatureType::Minotaur)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            count: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Callaphe, Beloved of the Sea — {1}{U}{U} Legendary Enchantment Creature —
/// Demigod, */3. Power equals your devotion to blue. (The "your permanents
/// tax opponents' targeted spells {1} more" static is dropped — `extra_cost_
/// for_spell` can't yet read a cast's chosen target; tracked in TODO.md.)
pub fn callaphe_beloved_of_the_sea() -> CardDefinition {
    CardDefinition {
        name: "Callaphe, Beloved of the Sea",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demigod], ..Default::default() },
        power: 0,
        toughness: 3,
        dynamic_pt: Some(DynamicPt::DevotionTo { color: Color::Blue, base_t: 3 }),
        ..Default::default()
    }
}

/// Siona, Captain of the Pyleas — {1}{G}{W} Legendary 2/2 Human Soldier. ETB:
/// look at the top seven cards, you may put an Aura into your hand, the rest
/// on the bottom. (The "Aura attaches → make a Soldier" static is dropped —
/// the engine has no aura-attach event yet; tracked in TODO.md.)
pub fn siona_captain_of_the_pyleas() -> CardDefinition {
    CardDefinition {
        name: "Siona, Captain of the Pyleas",
        cost: cost(&[generic(1), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(7),
            rest_to_graveyard: false,
            pick_filter: Some(SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Flummoxed Cyclops — {3}{R} 4/4 Cyclops with Reach. While two or more
/// creatures your opponents control are attacking, this creature can't block.
/// (Modeled as a static active during the attack rather than a combat-scoped
/// grant — functionally equivalent within the combat.)
pub fn flummoxed_cyclops() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Flummoxed Cyclops",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cyclops], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Can't block while two or more creatures your opponents control are attacking.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent)
                            .and(SelectionRequirement::IsAttacking),
                    ),
                    n: Value::Const(2),
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::CantBlock],
            },
        }],
        ..Default::default()
    }
}

/// Altar of the Pantheon — {3} Artifact. Your devotion to each color (and
/// combination) is increased by one. {T}: Add one mana of any color; if you
/// control a God, Demigod, or legendary enchantment, gain 1 life.
pub fn altar_of_the_pantheon() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{ManaPayload, StaticEffect};
    let payoff = SelectionRequirement::HasCreatureType(CreatureType::God)
        .or(SelectionRequirement::HasCreatureType(CreatureType::Demigod))
        .or(SelectionRequirement::HasSupertype(Supertype::Legendary)
            .and(SelectionRequirement::Enchantment));
    CardDefinition {
        name: "Altar of the Pantheon",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Your devotion to each color and each combination of colors is increased by one.",
            effect: StaticEffect::DevotionBonus,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(payoff),
                        n: Value::Const(1),
                    },
                    then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(1) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hateful Eidolon — {B} 1/2 Enchantment Creature — Spirit. Lifelink.
/// Whenever an enchanted creature dies, draw a card for each Aura you
/// controlled that was attached to it.
pub fn hateful_eidolon() -> CardDefinition {
    CardDefinition {
        name: "Hateful Eidolon",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::AurasYouControlledOnDyingSubject, Value::Const(1)),
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::AurasYouControlledOnDyingSubject },
        }],
        ..Default::default()
    }
}

/// Dawn Evangel — {2}{W} 2/3 Enchantment Creature — Human Cleric. Whenever a
/// creature dies, if an Aura you controlled was attached to it, return target
/// creature card with mana value 2 or less from your graveyard to your hand.
pub fn dawn_evangel() -> CardDefinition {
    CardDefinition {
        name: "Dawn Evangel",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::AurasYouControlledOnDyingSubject, Value::Const(1)),
            ),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(2))
                        .and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Minion's Return — {2}{B} Enchantment — Aura. Flash. Enchant creature.
/// When enchanted creature dies, return that card to the battlefield under
/// your control.
pub fn minions_return() -> CardDefinition {
    CardDefinition {
        name: "Minion's Return",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..Default::default()
    }
}

/// Inspire Awe — {3}{G} Instant. Prevent all combat damage this turn except
/// damage dealt by enchanted creatures and enchantment creatures. Scry 2.
pub fn inspire_awe() -> CardDefinition {
    CardDefinition {
        name: "Inspire Awe",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventCombatDamageExceptDealtBy {
                except: SelectionRequirement::Creature.and(
                    SelectionRequirement::IsEnchanted.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Ironscale Hydra — {3}{G}{G} 5/5 Hydra. If a creature would deal combat
/// damage to this creature, prevent that damage and put a +1/+1 counter on it.
pub fn ironscale_hydra() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Ironscale Hydra",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hydra], ..Default::default() },
        power: 5,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "If a creature would deal combat damage to this creature, prevent that damage and put a +1/+1 counter on this creature.",
            effect: StaticEffect::PreventCombatDamageToSelfAndGrow,
        }],
        ..Default::default()
    }
}
