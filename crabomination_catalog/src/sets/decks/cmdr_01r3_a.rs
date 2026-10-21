//! Commander: the 2026-10-21 (`01R3xh4r`) one-gap pair closers. Tests in
//! `tests/recent_b/cmdr_01r3_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, BattleSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, MayPlayDuration, Prototype, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Zone,
};
use crate::effect::shortcut::{craft, magecraft, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LibraryPosition, LookExileGrant, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, g, generic, hybrid, r, u, w, x};

/// Arcane Proxy — {7} 4/3 artifact creature Wizard; prototype {1}{U}{U} 2/1
/// (CR 702.160). When it enters, if you cast it, exile target instant or
/// sorcery card with mana value <= its power from your graveyard, copy it, and
/// you may cast the copy without paying its mana cost (CR 707.12).
pub fn arcane_proxy() -> CardDefinition {
    let spell = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Arcane Proxy",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 4,
        toughness: 3,
        prototype: Some(Box::new(Prototype { cost: cost(&[generic(1), u(), u()]), power: 2, toughness: 1 })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(spell.and(R::InYourGraveyard).and(R::ManaValueAtMostSourcePower)),
                    to: ZoneDest::Exile,
                },
                Effect::CastWithoutPayingImmediate {
                    reduce_generic: 0,
                    pay_own_cost: false,
                    what: Selector::LastMoved,
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: true,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Worldpurge — {4}{W/U}{W/U}{W/U}{W/U} sorcery. Return all permanents to
/// their owners' hands; each player keeps up to seven cards in hand and
/// shuffles the rest into their library; each player loses all unspent mana.
pub fn worldpurge() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        name: "Worldpurge",
        cost: cost(&[generic(4), wu(), wu(), wu(), wu()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move { what: Selector::EachPermanent(R::Any), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::EachPlayerKeepsUpToFromHandShufflesRest { keep: 7 },
            Effect::LoseUnspentMana { who: PlayerRef::EachPlayer },
        ]),
        ..Default::default()
    }
}

/// Braided Net // Braided Quipu — {2}{U} artifact; enters with three net
/// counters. {T}, remove a net counter: tap another target nonland permanent;
/// its activated abilities can't be activated for as long as it remains tapped
/// (CR 611.2c, `Duration::WhileTapped`). Craft with an artifact {1}{U}.
/// Braided Quipu: {3}{U}, {T}: draw a card for each artifact you control, then
/// put it into its owner's library third from the top.
pub fn braided_net() -> CardDefinition {
    let quipu = CardDefinition {
        name: "Braided Quipu",
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::Blue],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::This)), pos: LibraryPosition::FromTop(2) },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    };
    let other = R::Permanent.and(R::Land.negate()).and(R::OtherThanSource);
    CardDefinition {
        name: "Braided Net",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Net, Value::Const(3))),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Net, 1)),
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(other) },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantActivateAbilities,
                        duration: Duration::WhileTapped,
                    },
                ]),
                ..Default::default()
            },
            craft(cost(&[generic(1), u()]), R::Artifact, 1),
        ],
        back_face: Some(Box::new(quipu)),
        ..Default::default()
    }
}

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

/// Duskrider Peregrine — {5}{W} 3/3 Bird; flying, protection from black,
/// suspend 3—{1}{W}.
pub fn duskrider_peregrine() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(Color::Black),
            Keyword::Suspend(3, cost(&[generic(1), w()])),
        ],
        ..creature("Duskrider Peregrine", cost(&[generic(5), w()]), vec![CreatureType::Bird], 3, 3)
    }
}

/// Deem Worthy — {4}{R} instant: 7 damage to target creature. Cycling
/// {3}{R}; when you cycle it, you may have it deal 2 damage to target creature
/// (CR 702.29c).
pub fn deem_worthy() -> CardDefinition {
    CardDefinition {
        name: "Deem Worthy",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(3), r()]))],
        effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(7) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Deal 2 damage to target creature?".into(),
                body: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) }),
            },
        }],
        ..Default::default()
    }
}

/// Dire Wolves — {2}{G} 2/2 Wolf; banding as long as you control a Plains.
pub fn dire_wolves() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has banding as long as you control a Plains.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Banding,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasLandType(LandType::Plains).and(R::ControlledByYou)),
                    n: Value::ONE,
                },
            },
        }],
        ..creature("Dire Wolves", cost(&[generic(2), g()]), vec![CreatureType::Wolf], 2, 2)
    }
}

/// Nature's Blessing — {2}{G}{W} enchantment. {G}{W}, discard a card: put a
/// +1/+1 counter on target creature, or it gains banding, first strike or
/// trample indefinitely.
pub fn natures_blessing() -> CardDefinition {
    let creature = || target_filtered(R::Creature);
    let gain = |keyword| Effect::GrantKeyword { what: creature(), keyword, duration: Duration::Permanent };
    CardDefinition {
        name: "Nature's Blessing",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), w()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter { what: creature(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                gain(Keyword::Banding),
                gain(Keyword::FirstStrike),
                gain(Keyword::Trample),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Disciple of Caelus Nin — {4}{W} 3/4 Human Wizard. When it enters, starting
/// with you, each player chooses up to five permanents they control; every
/// other permanent but this one phases out. Permanents can't phase in.
pub fn disciple_of_caelus_nin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::EachPlayerKeepsUpToPermanentsRestPhaseOut { keep: 5 },
        }],
        static_abilities: vec![StaticAbility {
            description: "Permanents can't phase in.",
            effect: StaticEffect::PermanentsCantPhaseIn,
        }],
        ..creature(
            "Disciple of Caelus Nin",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Geistflame Reservoir — {2}{R} artifact. Whenever you cast an instant or
/// sorcery spell, put a charge counter on it. {1}{R}, {T}, remove any number
/// of charge counters: that much damage to any target. {1}{R}, {T}: exile the
/// top card of your library; you may play it this turn.
pub fn geistflame_reservoir() -> CardDefinition {
    CardDefinition {
        name: "Geistflame Reservoir",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Charge,
            amount: Value::ONE,
        })],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                remove_counter_x: Some(CounterType::Charge),
                effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::XFromCost },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Invasion of Arcavios // Invocation of the Founders — {3}{U}{U} Siege,
/// defense 7. When it enters, search your library and/or graveyard for an
/// instant or sorcery card you own and put it into your hand (outside the game
/// holds nothing in Commander, CR 903.11). Back: whenever you cast an instant
/// or sorcery spell from your hand, you may copy it and choose new targets.
pub fn invasion_of_arcavios() -> CardDefinition {
    let spell = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    let founders = CardDefinition {
        name: "Invocation of the Founders",
        card_types: vec![CardType::Enchantment],
        color_indicator: vec![Color::Blue],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(spell()),
                Predicate::CastSpellFromHand,
            ])),
            effect: Effect::MayDo {
                description: "Copy that spell?".into(),
                body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Arcavios",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Library, Zone::Graveyard],
                filter: spell(),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        back_face: Some(Box::new(founders)),
        ..Default::default()
    }
}

/// Fungal Behemoth — {3}{G} */* Fungus; P/T = +1/+1 counters on creatures you
/// control. Suspend X—{X}{G}{G}, X can't be 0. Whenever a time counter is
/// removed from it while it's exiled, you may put a +1/+1 counter on target
/// creature.
pub fn fungal_behemoth() -> CardDefinition {
    let counters = || Value::CountersOn {
        what: Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
        kind: CounterType::PlusOnePlusOne,
    };
    CardDefinition {
        keywords: vec![Keyword::Suspend(0, cost(&[x(), g(), g()]))],
        static_abilities: vec![StaticAbility {
            description: "P/T equal to the +1/+1 counters on creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: counters(), toughness: counters() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource).while_suspended(),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on target creature?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature("Fungal Behemoth", cost(&[generic(3), g()]), vec![CreatureType::Fungus], 0, 0)
    }
}

/// Feral Encounter — {G}{G} sorcery. Look at the top five; you may exile a
/// creature card among them and cast it this turn; the rest go to the bottom
/// at random. At the beginning of the next combat this turn, target creature
/// you control deals damage equal to its power to up to one target creature
/// you don't control.
pub fn feral_encounter() -> CardDefinition {
    CardDefinition {
        name: "Feral Encounter",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LookTopExileOneMayPlay {
                count: Value::Const(5),
                who: PlayerRef::You,
                grant: LookExileGrant::CreatureMayThisTurn,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextCombatThisTurn,
                body: Box::new(Effect::OptionalTargets {
                    min: 1,
                    body: Box::new(Effect::DealDamageEqualToPower {
                        source: target_filtered(R::Creature.and(R::ControlledByYou)),
                        target: target_filtered(R::Creature.and(R::ControlledByYou.negate())),
                    }),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Vedalken Squirrel-Whacker — {3}{U} */* Vedalken Guest. As it enters, roll
/// a d6 twice: base power the first result, base toughness the second. If you
/// would roll one or more d6, you may exchange one result with its base power
/// or base toughness (CR 706.6).
pub fn vedalken_squirrel_whacker() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::RollDiceSetBasePt { sides: 6 }),
        static_abilities: vec![StaticAbility {
            description: "Exchange a d6 result with this creature's base power or toughness.",
            effect: StaticEffect::MayExchangeD6WithSelfBasePt,
        }],
        ..creature(
            "Vedalken Squirrel-Whacker",
            cost(&[generic(3), u()]),
            vec![CreatureType::Vedalken, CreatureType::Guest],
            0,
            0,
        )
    }
}

/// Soul Echo — {X}{W}{W} enchantment; enters with X echo counters. You don't
/// lose the game for having 0 or less life. At the beginning of your upkeep,
/// sacrifice it with no echo counters on it; otherwise target opponent may
/// choose that each 1 damage dealt to you until your next upkeep removes an
/// echo counter from it instead (CR 614.1a).
pub fn soul_echo() -> CardDefinition {
    let echoes = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Echo };
    CardDefinition {
        name: "Soul Echo",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Echo, Value::XFromCost)),
        static_abilities: vec![
            StaticAbility {
                description: "You don't lose the game for having 0 or less life.",
                effect: StaticEffect::ControllerDoesntLoseForZeroLife,
            },
            StaticAbility {
                description: "Damage to you removes echo counters instead while chosen.",
                effect: StaticEffect::ReplaceDamageToYouRemovingStampedCounters { kind: CounterType::Echo },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ValueAtMost(echoes, Value::Const(0)),
                then: Box::new(Effect::SacrificeSource),
                else_: Box::new(Effect::PlayerMayStampSourceTurn { who: target_filtered(R::OpponentPlayer) }),
            },
        }],
        ..Default::default()
    }
}

/// Mindlink Mech — {2}{U} 4/3 Vehicle, flying, crew 1. Whenever it becomes
/// crewed for the first time each turn, until end of turn it becomes a copy of
/// target nonlegendary creature that crewed it this turn, except it's 4/3, a
/// Vehicle artifact in addition to its other types, and has flying (CR 707.9b).
pub fn mindlink_mech() -> CardDefinition {
    let mut crewed = EventSpec::new(EventKind::CrewsOrSaddles { main_phase_only: false }, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource });
    crewed.once_per_turn = true;
    let crew = R::Creature.and(R::CrewedSourceThisTurn).and(R::HasSupertype(Supertype::Legendary).negate());
    CardDefinition {
        name: "Mindlink Mech",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: crewed,
            effect: Effect::Seq(vec![
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(crew),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                Effect::AmendCopy {
                    what: Selector::This,
                    keep_activated: vec![],
                    keep_triggered: vec![],
                    pt: Some((4, 3)),
                    keep_name: false,
                    legendary: false,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Artifact],
                    creature_types: vec![],
                },
                // Layer 4; indefinite is harmless, the Mech is a Vehicle anyway.
                Effect::SetArtifactSubtypes { what: Selector::This, subtypes: vec![ArtifactSubtype::Vehicle] },
            ]),
        }],
        ..Default::default()
    }
}
