//! Theros Beyond Death (THB) — 2020. Escape payoffs, devotion demigods,
//! and the constellation/enchantment-matters shell.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, Selector,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

fn req(r: SelectionRequirement) -> SelectionRequirement {
    r
}

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
                defender: target_filtered(req(SelectionRequirement::Creature)
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
                        req(SelectionRequirement::Permanent)
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
                        filter: req(SelectionRequirement::Creature)
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
                    req(SelectionRequirement::Creature)
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
