//! +1/+1 counters matters — Golgari / Selesnya / Simic / Abzan value: enter- and
//! cast-triggered counter payoffs, counter-doubling, "counter-bearers have
//! [keyword]" anthems, and green ramp/removal. Tests in `tests/recent54.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, evolve, outlast, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef};
use crate::mana::{b, cost, g, generic, hybrid, u, w, Color};

/// Anthem granting `keyword` to your creatures that carry a +1/+1 counter.
fn counter_bearers_have(keyword: Keyword, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                R::Creature
                    .and(R::ControlledByYou)
                    .and(R::WithCounter(CounterType::PlusOnePlusOne)),
            ),
            keyword,
        },
    }
}

fn plus_one(what: Selector, amount: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount }
}

/// Good-Fortune Unicorn — {1}{G}{W} 2/2 Unicorn. Whenever another creature you
/// control enters, put a +1/+1 counter on that creature.
pub fn good_fortune_unicorn() -> CardDefinition {
    CardDefinition {
        name: "Good-Fortune Unicorn",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Unicorn], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: plus_one(Selector::TriggerSource, Value::ONE),
        }],
        ..Default::default()
    }
}

/// Ivy Lane Denizen — {3}{G} 2/3 Elf Warrior. Whenever another green creature
/// you control enters, put a +1/+1 counter on target creature.
pub fn ivy_lane_denizen() -> CardDefinition {
    CardDefinition {
        name: "Ivy Lane Denizen",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasColor(Color::Green)),
                }),
            effect: plus_one(target_filtered(R::Creature), Value::ONE),
        }],
        ..Default::default()
    }
}

/// Managorger Hydra — {2}{G} 1/1 Hydra with trample. Whenever a player casts a
/// spell, put a +1/+1 counter on this creature.
pub fn managorger_hydra() -> CardDefinition {
    CardDefinition {
        name: "Managorger Hydra",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hydra], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: plus_one(Selector::This, Value::ONE),
        }],
        ..Default::default()
    }
}

/// Herd Baloth — {3}{G}{G} 4/4 Beast. Whenever one or more +1/+1 counters are
/// put on this creature, you may create a 4/4 green Beast creature token.
pub fn herd_baloth() -> CardDefinition {
    let beast = TokenDefinition {
        name: "Beast".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Herd Baloth",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::SelfSource,
            ),
            effect: Effect::MayDo {
                description: "Create a 4/4 green Beast creature token.".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: beast,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Duskshell Crawler — {1}{G} 0/3 Insect. ETB put a +1/+1 counter on target
/// creature. Each creature you control with a +1/+1 counter on it has trample.
pub fn duskshell_crawler() -> CardDefinition {
    CardDefinition {
        name: "Duskshell Crawler",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![etb(plus_one(target_filtered(R::Creature), Value::ONE))],
        static_abilities: vec![counter_bearers_have(
            Keyword::Trample,
            "Each creature you control with a +1/+1 counter on it has trample.",
        )],
        ..Default::default()
    }
}

/// Longshot Squad — {3}{G} 3/3 Dog Archer. Outlast {1}{G}. Each creature you
/// control with a +1/+1 counter on it has reach.
pub fn longshot_squad() -> CardDefinition {
    CardDefinition {
        name: "Longshot Squad",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Archer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![outlast(cost(&[generic(1), g()]))],
        static_abilities: vec![counter_bearers_have(
            Keyword::Reach,
            "Each creature you control with a +1/+1 counter on it has reach.",
        )],
        ..Default::default()
    }
}

/// Kami of Whispered Hopes — {2}{G} 1/1 Spirit. If one or more +1/+1 counters
/// would be put on a permanent you control, that many plus one are instead.
/// {T}: Add X mana of any one color, where X is this creature's power (its
/// counters raise its power).
pub fn kami_of_whispered_hopes() -> CardDefinition {
    CardDefinition {
        name: "Kami of Whispered Hopes",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a permanent you \
                          control, that many plus one are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Old Gnawbone — {5}{G}{G} 7/7 Dragon with flying. Whenever a creature you
/// control deals combat damage to a player, create that many Treasure tokens.
pub fn old_gnawbone() -> CardDefinition {
    use crabomination_base::tokens::treasure_token;
    CardDefinition {
        name: "Old Gnawbone",
        cost: cost(&[generic(5), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Ulvenwald Tracker — {G} 1/1 Human Shaman. {1}{G}, {T}: Target creature you
/// control fights another target creature.
pub fn ulvenwald_tracker() -> CardDefinition {
    CardDefinition {
        name: "Ulvenwald Tracker",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            effect: Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nissa, Voice of Zendikar — {1}{G}{G} Legendary Planeswalker — Nissa. 3
/// loyalty. +1 make a 0/1 Plant; -2 +1/+1 counter on each creature you control;
/// -7 gain X life and draw X cards, X = lands you control.
pub fn nissa_voice_of_zendikar() -> CardDefinition {
    let plant = TokenDefinition {
        name: "Plant".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        ..Default::default()
    };
    let lands = || Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou)));
    CardDefinition {
        name: "Nissa, Voice of Zendikar",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nissa],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: plant,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: plus_one(
                    Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    Value::ONE,
                ),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: lands() },
                    Effect::Draw { who: Selector::You, amount: lands() },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Gyre Sage — {1}{G} 1/2 Elf Druid with Evolve. {T}: Add {G} for each +1/+1
/// counter on this creature.
pub fn gyre_sage() -> CardDefinition {
    CardDefinition {
        name: "Gyre Sage",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![evolve()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elusive Krasis — {1}{G}{U} 0/4 Fish Mutant. Can't be blocked. Evolve.
pub fn elusive_krasis() -> CardDefinition {
    CardDefinition {
        name: "Elusive Krasis",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish, CreatureType::Mutant],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Corpsejack Menace — {2}{B}{G} 4/4 Fungus. If one or more +1/+1 counters
/// would be put on a creature you control, twice that many are put on it.
pub fn corpsejack_menace() -> CardDefinition {
    CardDefinition {
        name: "Corpsejack Menace",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus], ..Default::default() },
        power: 4,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature you \
                          control, twice that many are put on it instead.",
            effect: StaticEffect::DoublePlusOneCounters,
        }],
        ..Default::default()
    }
}

/// Prime Speaker Zegana — {2}{G}{G}{U}{U} 1/1 Legendary Merfolk Wizard. Enters
/// with X +1/+1 counters, X = greatest power among other creatures you control.
/// ETB draw cards equal to its power.
pub fn prime_speaker_zegana() -> CardDefinition {
    CardDefinition {
        name: "Prime Speaker Zegana",
        cost: cost(&[generic(2), g(), g(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(
                R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            ))),
        )),
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Cold-Eyed Selkie — {1}{G/U}{G/U} 1/1 Merfolk Rogue with islandwalk. Whenever
/// it deals combat damage to a player, you may draw that many cards.
pub fn cold_eyed_selkie() -> CardDefinition {
    CardDefinition {
        name: "Cold-Eyed Selkie",
        cost: crate::mana::ManaCost::new(vec![
            crate::mana::ManaSymbol::Generic(1),
            hybrid(Color::Green, Color::Blue),
            hybrid(Color::Green, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Draw that many cards.".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount }),
            },
        }],
        ..Default::default()
    }
}

/// Bioshift — {G/U} Instant. Move any number of +1/+1 counters from target
/// creature onto another target creature. (Any-number modeled as all.)
pub fn bioshift() -> CardDefinition {
    CardDefinition {
        name: "Bioshift",
        cost: crate::mana::ManaCost::new(vec![hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Instant],
        effect: Effect::MoveAllCounters {
            from: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
        },
        ..Default::default()
    }
}

/// Woodland Champion — {1}{G} 2/2 Elf Scout. Whenever one or more tokens you
/// control enter, put that many +1/+1 counters on this creature. (Modeled as
/// one counter per entering token.)
pub fn woodland_champion() -> CardDefinition {
    CardDefinition {
        name: "Woodland Champion",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsToken.and(R::OtherThanSource),
                }),
            effect: plus_one(Selector::This, Value::ONE),
        }],
        ..Default::default()
    }
}

/// Feat of Resistance — {1}{W} Instant. Put a +1/+1 counter on target creature
/// you control; it gains protection from the color of your choice until EOT.
pub fn feat_of_resistance() -> CardDefinition {
    CardDefinition {
        name: "Feat of Resistance",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            plus_one(target_filtered(R::Creature.and(R::ControlledByYou)), Value::ONE),
            Effect::GrantProtectionFromChosenColor {
                what: Selector::Target(0),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Travel Preparations — {1}{G} Sorcery. Put a +1/+1 counter on each of up to
/// two target creatures. Flashback {1}{W}.
pub fn travel_preparations() -> CardDefinition {
    CardDefinition {
        name: "Travel Preparations",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(1), w()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: R::Creature,
            effect: Box::new(plus_one(Selector::Target(0), Value::ONE)),
        },
        ..Default::default()
    }
}

/// Master Biomancer — {2}{G}{U} 2/4 Elf Wizard. Each other creature you control
/// enters with a number of additional +1/+1 counters equal to this creature's
/// power. (The "as a Mutant" type rider is omitted.)
pub fn master_biomancer() -> CardDefinition {
    CardDefinition {
        name: "Master Biomancer",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control enters with a number of additional \
                          +1/+1 counters on it equal to this creature's power.",
            effect: StaticEffect::OtherCreaturesEnterWithCountersEqualToSourcePower {
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        ..Default::default()
    }
}
