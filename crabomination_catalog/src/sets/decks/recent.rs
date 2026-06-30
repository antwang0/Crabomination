//! Recent-set staples (MH3 / BLB / DSK / OTJ / FDN / DFT / TDM …) that fill
//! gaps in the Modern-playable pool. Each card has at least one functionality
//! test in `crabomination/src/tests/recent.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, Predicate, Selector, SelectionRequirement, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, recover, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, colorless, cost, g, generic, r, u, w};

// === Innistrad: Midnight Hunt — Coven (control 3+ creatures with different
// powers). `Predicate::CovenActive { who }`. ===

/// Sigarda, Champion of Light — {1}{G}{W}{W} 4/4 Legendary Angel. Flying,
/// trample. Humans you control get +1/+1. Coven — whenever it attacks, if
/// coven is active, look at the top five cards, reveal a Human creature among
/// them to your hand, rest on the bottom in random order.
pub fn sigarda_champion_of_light() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    let humans = Selector::EachPermanent(
        SelectionRequirement::HasCreatureType(CreatureType::Human)
            .and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Sigarda, Champion of Light",
        cost: cost(&[generic(1), g(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Humans you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: humans, power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::CovenActive { who: PlayerRef::You }),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(5),
                rest_to_graveyard: false,
                pick_filter: Some(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Human)),
                ),
                take: Some(Value::Const(1)),
                to_battlefield: false,
            },
        }],
        ..Default::default()
    }
}

/// Dawnhart Mentor — {2}{G} 0/4 Human Warlock. ETB create a 1/1 white Human.
/// Coven — {5}{G}: target creature you control gets +3/+3 and gains trample
/// until end of turn. Activate only with coven active.
pub fn dawnhart_mentor() -> CardDefinition {
    use crate::card::{ActivatedAbility, TokenDefinition};
    use crate::mana::Color;
    let human = TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Dawnhart Mentor",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: human,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            condition: Some(Predicate::CovenActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sungold Sentinel — {1}{W} 3/2 Human Soldier. ETB or attack: exile up to one
/// target card from a graveyard. Coven — {1}{W}: it gains hexproof and can't
/// be blocked this turn. Activate only with coven active. (The "from a chosen
/// color" sub-clause is approximated as blanket hexproof + unblockable.)
pub fn sungold_sentinel() -> CardDefinition {
    use crate::card::ActivatedAbility;
    // "up to one" is honored by the target being optional at resolution.
    let exile_gy = Effect::Move {
        what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::InGraveyard },
        to: ZoneDest::Exile,
    };
    CardDefinition {
        name: "Sungold Sentinel",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            etb(exile_gy.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: exile_gy,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            condition: Some(Predicate::CovenActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bladebrand — {1}{B} Instant. Target creature gains deathtouch until end of
/// turn. Draw a card.
pub fn bladebrand() -> CardDefinition {
    CardDefinition {
        name: "Bladebrand",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Halana and Alena, Partners — {2}{R}{G} 2/3 Legendary Human Ranger. First
/// strike, reach. At the beginning of combat on your turn, put X +1/+1
/// counters on another target creature you control, where X is this creature's
/// power; that creature gains haste until end of turn.
pub fn halana_and_alena() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Halana and Alena, Partners",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Welcoming Vampire — {2}{W} 2/3 Vampire. Flying. Whenever one or more other
/// creatures you control with power 2 or less enter, draw a card. Once each turn.
pub fn welcoming_vampire() -> CardDefinition {
    CardDefinition {
        name: "Welcoming Vampire",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
                })
                .once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Cruel Witness — {2}{U}{U} 3/3 Bird Horror. Flying. Whenever you cast a
/// noncreature spell, surveil 1.
pub fn cruel_witness() -> CardDefinition {
    CardDefinition {
        name: "Cruel Witness",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Gryff Rider — {2}{W} 2/1 Human Soldier. Flying, Training.
pub fn gryff_rider() -> CardDefinition {
    use crate::effect::shortcut::training;
    CardDefinition {
        name: "Gryff Rider",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![training()],
        ..Default::default()
    }
}

/// Apprentice Sharpshooter — {2}{G} 1/4 Human Archer. Reach, Training.
pub fn apprentice_sharpshooter() -> CardDefinition {
    use crate::effect::shortcut::training;
    CardDefinition {
        name: "Apprentice Sharpshooter",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![training()],
        ..Default::default()
    }
}

/// Sporeback Wolf — {1}{G} 2/2 Wolf. During your turn, it gets +0/+2.
pub fn sporeback_wolf() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Sporeback Wolf",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "During your turn, Sporeback Wolf gets +0/+2.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 0,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Dawnhart Wardens — {1}{G}{W} 3/3 Human Warrior. Vigilance. Coven — at the
/// beginning of combat on your turn, if you control three or more creatures
/// with different powers, creatures you control get +1/+0 until end of turn.
pub fn dawnhart_wardens() -> CardDefinition {
    CardDefinition {
        name: "Dawnhart Wardens",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::CovenActive { who: PlayerRef::You }),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Brimstone Trebuchet — {2}{R} 1/3 Goblin. Defender, reach. {T}: deals 1
/// damage to each opponent. Whenever a Knight you control enters, untap it.
pub fn brimstone_trebuchet() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Brimstone Trebuchet",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender, Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Knight),
                }),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..Default::default()
    }
}

/// Whispering Wizard — {3}{U} 3/2 Human Wizard. Whenever you cast a noncreature
/// spell, create a 1/1 white flying Spirit. Once each turn.
pub fn whispering_wizard() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Whispering Wizard",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature))
                .once_per_turn(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: spirit },
        }],
        ..Default::default()
    }
}

/// Patrician Geist — {2}{U} 2/2 Spirit Knight. Flying. Other Spirits you
/// control get +1/+1. Spells you cast from your graveyard cost {1} less.
pub fn patrician_geist() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Patrician Geist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Other Spirits you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Spells you cast from your graveyard cost {1} less.",
                effect: StaticEffect::GraveyardCastCostReduction { amount: 1 },
            },
        ],
        ..Default::default()
    }
}

/// Predator's Howl — {3}{G} Instant. Create a 2/2 green Wolf. Morbid — create
/// three instead if a creature died this turn.
pub fn predators_howl() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let wolf = || TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Predator's Howl",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) },
            then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: wolf() }),
            else_: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: wolf() }),
        },
        ..Default::default()
    }
}

/// Ardenvale Tactician // Dizzying Swoop — {1}{W}{W} 2/3 Human Knight, Flying.
/// Adventure: Dizzying Swoop {1}{W} Instant — tap up to two target creatures.
pub fn ardenvale_tactician() -> CardDefinition {
    use crate::card::Adventure;
    CardDefinition {
        name: "Ardenvale Tactician",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(Adventure {
            name: "Dizzying Swoop",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            },
        })),
        ..Default::default()
    }
}

/// Bloodcrazed Socialite — {3}{B} 3/3 Vampire. Menace. ETB create a Blood
/// token. Attacks → may sacrifice a Blood; if you do, it gets +2/+2.
pub fn bloodcrazed_socialite() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Bloodcrazed Socialite",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice a Blood token? (Bloodcrazed Socialite gets +2/+2)".into(),
                    filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                    count: Value::Const(1),
                    then: Box::new(Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Intrepid Adversary — {1}{W} 3/1 Human Scout. Lifelink. ETB pay {1}{W} any
/// number of times for that many valor counters; creatures you control get
/// +1/+1 for each valor counter on it. (The any-number ETB payment is modeled
/// as Multikicker — paid at cast time — which is functionally identical here.)
pub fn intrepid_adversary() -> CardDefinition {
    use crate::card::{CounterType, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Intrepid Adversary",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Lifelink, Keyword::Multikicker(cost(&[generic(1), w()]))],
        enters_with_counters: Some((CounterType::Valor, Value::TimesKicked)),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1 for each valor counter on this creature.",
            effect: StaticEffect::PumpPTPerCounterOnSource {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::Valor,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Bloodthirsty Adversary — {1}{R} 2/2 Vampire. Haste. ETB pay {2}{R} any
/// number of times for that many +1/+1 counters (modeled as Multikicker). The
/// "exile up to that many I/S of MV ≤3 from your graveyard and copy them" value
/// rider is deferred (TODO.md).
pub fn bloodthirsty_adversary() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Bloodthirsty Adversary",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Multikicker(cost(&[generic(2), r()]))],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::TimesKicked)),
        ..Default::default()
    }
}

/// Diregraf Scavenger — {3}{B} 2/3 Zombie Bear. Deathtouch. ETB exile up to
/// one target card from a graveyard; if a creature card was exiled this way,
/// each opponent loses 2 life and you gain 2.
pub fn diregraf_scavenger() -> CardDefinition {
    CardDefinition {
        name: "Diregraf Scavenger",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Exile {
                what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::InGraveyard },
            },
            Effect::If {
                cond: Predicate::EntityMatchesAny {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::Creature,
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Gut, True Soul Zealot — {2}{R} 2/2 Legendary Goblin Shaman. Whenever you
/// attack, you may sacrifice another creature or artifact; if you do, create a
/// 4/1 black Skeleton with menace, tapped and attacking. (Choose-a-Background
/// commander clause is cosmetic and omitted.)
pub fn gut_true_soul_zealot() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let skeleton = TokenDefinition {
        name: "Skeleton".into(),
        power: 4,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton],
            ..Default::default()
        },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    };
    CardDefinition {
        name: "Gut, True Soul Zealot",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature or artifact? (create a 4/1 Skeleton, tapped and attacking)".into(),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::OtherThanSource),
                count: Value::Const(1),
                then: Box::new(Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: skeleton,
                    cleanup: crate::effect::AttackingTokenCleanup::None,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Eccentric Farmer — {2}{G} 2/3 Human Peasant. ETB mill three, then you may
/// return a land card from your graveyard to your hand.
pub fn eccentric_farmer() -> CardDefinition {
    CardDefinition {
        name: "Eccentric Farmer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::MayDo {
                description: "Return a land card from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Graveyard,
                            filter: SelectionRequirement::Land,
                        }),
                        count: Box::new(Value::Const(1)),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        ]))],
        ..Default::default()
    }
}

/// Briarbridge Tracker — {2}{G} 2/3 Human Scout. Vigilance. ETB investigate;
/// gets +2/+0 as long as you control a token.
pub fn briarbridge_tracker() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::shortcut::investigate;
    CardDefinition {
        name: "Briarbridge Tracker",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(investigate(1))],
        static_abilities: vec![StaticAbility {
            description: "As long as you control a token, this creature gets +2/+0.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::IsToken.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Markov Waltzer — {2}{R}{W} 1/3 Vampire. Flying, haste. At the beginning of
/// combat on your turn, up to two target creatures you control each get +1/+0.
pub fn markov_waltzer() -> CardDefinition {
    CardDefinition {
        name: "Markov Waltzer",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Heron-Blessed Geist — {4}{W} 3/3 Spirit. Flying. {3}{W}, exile from your
/// graveyard: make two 1/1 white flying Spirits. Sorcery-speed, only if you
/// control an enchantment.
pub fn heron_blessed_geist() -> CardDefinition {
    use crate::card::{ActivatedAbility, CardType as CT, TokenDefinition};
    use crate::mana::Color;
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Heron-Blessed Geist",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::HasCardType(CT::Enchantment)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: spirit,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vampire Socialite — {B}{R} 2/2 Vampire Noble. Menace. ETB, if an opponent
/// lost life this turn, put a +1/+1 counter on each other Vampire you control.
/// (The "enters with an extra counter while an opponent lost life" static is
/// approximated by the ETB; tracked in TODO.md.)
pub fn vampire_socialite() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Vampire Socialite",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Vampire)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Sigarda's Imprisonment — {2}{W} Aura. Enchanted creature can't attack or
/// block. {4}{W}: exile enchanted creature and create a Blood token.
pub fn sigardas_imprisonment() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Sigarda's Imprisonment",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    to: ZoneDest::Exile,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::blood_token(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vampire Spawn — {2}{B} 2/3 Vampire. ETB each opponent loses 2 life and you
/// gain 2.
pub fn vampire_spawn() -> CardDefinition {
    use crate::effect::shortcut::etb_drain;
    CardDefinition {
        name: "Vampire Spawn",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Wedding Security — {3}{B}{B} 4/4 Vampire Soldier. Attacks → may sacrifice a
/// Blood; if you do, put a +1/+1 counter on it and draw a card.
pub fn wedding_security() -> CardDefinition {
    use crate::card::{ArtifactSubtype, CounterType};
    CardDefinition {
        name: "Wedding Security",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a Blood token? (+1/+1 counter and draw)".into(),
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                count: Value::Const(1),
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Falcon Abomination — {2}{U} 2/2 Zombie Bird. Flying. ETB create a 2/2 black
/// Zombie with decayed.
pub fn falcon_abomination() -> CardDefinition {
    CardDefinition {
        name: "Falcon Abomination",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: decayed_zombie_token(),
        })],
        ..Default::default()
    }
}

/// Militia Rallier — {2}{W} 3/3 Human Soldier. Can't attack alone. Attacks →
/// untap target creature.
pub fn militia_rallier() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Militia Rallier",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::CantAttackAlone],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Untap { what: target_filtered(SelectionRequirement::Creature), up_to: None },
        }],
        ..Default::default()
    }
}

/// Bleed Dry — {2}{B}{B} Instant. Target creature gets -13/-13; if it would die
/// this turn, exile it instead.
pub fn bleed_dry() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Bleed Dry",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        // Install the exile replacement before the shrink so the SBA death is
        // caught (CR 614).
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: target_filtered(SelectionRequirement::Creature) },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(-13),
                toughness: Value::Const(-13),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Flame-Blessed Bolt — {R} Instant. Deals 2 damage to target creature or
/// planeswalker; if it would die this turn, exile it instead.
pub fn flame_blessed_bolt() -> CardDefinition {
    use crate::card::CardType as CT;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Flame-Blessed Bolt",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::HasCardType(CT::Planeswalker)),
                ),
            },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Ancestral Anger — {R} Sorcery. Target creature gains trample and gets +X/+0,
/// where X is 1 plus the number of cards named Ancestral Anger in your
/// graveyard. Draw a card.
pub fn ancestral_anger() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Ancestral Anger",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Sum(vec![
                    Value::Const(1),
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: SelectionRequirement::HasName("Ancestral Anger".into()),
                    },
                ]),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Famished Foragers — {3}{R} 4/3 Vampire. ETB, if an opponent lost life this
/// turn, add {R}{R}{R}. {2}{R}, discard a card: draw a card.
pub fn famished_foragers() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Famished Foragers",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(3)),
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pointed Discussion — {2}{B} Sorcery. Draw two cards, lose 2 life, then
/// create a Blood token.
pub fn pointed_discussion() -> CardDefinition {
    CardDefinition {
        name: "Pointed Discussion",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Bloodtithe Collector — {4}{B} 3/4 Vampire Noble. Flying. ETB, if an opponent
/// lost life this turn, each opponent discards a card.
pub fn bloodtithe_collector() -> CardDefinition {
    CardDefinition {
        name: "Bloodtithe Collector",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Dawnhart Disciple — {1}{G} 2/2 Human Warlock. Whenever another Human you
/// control enters, this creature gets +1/+1 until end of turn.
pub fn dawnhart_disciple() -> CardDefinition {
    CardDefinition {
        name: "Dawnhart Disciple",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Human),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Bramble Armor — {1}{G} Equipment. ETB attach to a creature you control.
/// Equipped creature gets +2/+1. Equip {4}.
pub fn bramble_armor() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Bramble Armor",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
        })],
        ..Default::default()
    }
}

/// Repository Skaab — {3}{U} 3/3 Zombie. Exploit; when it exploits a creature,
/// return target instant or sorcery card from your graveyard to your hand.
pub fn repository_skaab() -> CardDefinition {
    use crate::effect::shortcut::{exploit, target_filtered};
    let return_is = Effect::Move {
        what: target_filtered(
            SelectionRequirement::InGraveyard.and(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            ),
        ),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Repository Skaab",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![exploit(return_is)],
        ..Default::default()
    }
}

/// Fleshtaker — {W}{B} 2/2 Human Assassin. Whenever you sacrifice another
/// creature, gain 1 life and scry 1. {1}, sacrifice another creature: +2/+2.
pub fn fleshtaker() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Fleshtaker",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::OtherThanSource,
                }),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blessed Defiance — {W} Instant. Target creature you control gets +2/+0 and
/// gains lifelink; when it dies this turn, create a 1/1 white flying Spirit.
pub fn blessed_defiance() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::effect::shortcut::target_filtered;
    use crate::mana::Color;
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Blessed Defiance",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                slot: 0,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: spirit,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Gavony Trapper — {W} 0/2 Human Soldier. {2}, {T}: tap target creature.
pub fn gavony_trapper() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Gavony Trapper",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sure Strike — {1}{R} Instant. Target creature gets +3/+0 and gains first
/// strike until end of turn.
pub fn sure_strike() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Sure Strike",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Lunar Frenzy — {X}{R} Instant. Target creature you control gets +X/+0 and
/// gains first strike and trample until end of turn.
pub fn lunar_frenzy() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lunar Frenzy",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::XFromCost,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Dawnhart Rejuvenator — {3}{G} 2/4 Human Warlock. ETB gain 3 life. {T}: add
/// one mana of any color.
pub fn dawnhart_rejuvenator() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Dawnhart Rejuvenator",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spore Crawler — {2}{G} 3/2 Fungus. When it dies, draw a card.
pub fn spore_crawler() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Spore Crawler",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

/// Snarling Wolf — {G} 1/1 Wolf. {1}{G}: gets +2/+2 until end of turn. Once
/// each turn.
pub fn snarling_wolf() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Snarling Wolf",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wolfkin Bond — {4}{G} Aura. Enchant creature. ETB create a 2/2 green Wolf.
/// Enchanted creature gets +2/+2.
pub fn wolfkin_bond() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, TokenDefinition};
    use crate::effect::shortcut::target_filtered;
    use crate::mana::Color;
    let wolf = TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Wolfkin Bond",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: 2, toughness: 2, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: wolf,
        })],
        ..Default::default()
    }
}

/// Vampires' Vengeance — {2}{R} Instant. Deals 2 damage to each non-Vampire
/// creature. Create a Blood token.
pub fn vampires_vengeance() -> CardDefinition {
    CardDefinition {
        name: "Vampires' Vengeance",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::Not(Box::new(
                        SelectionRequirement::HasCreatureType(CreatureType::Vampire),
                    ))),
                ),
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(2) }),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Defenestrate — {2}{B} Instant. Destroy target creature without flying.
pub fn defenestrate() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Defenestrate",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Not(Box::new(
                    SelectionRequirement::HasKeyword(Keyword::Flying),
                ))),
            ),
        },
        ..Default::default()
    }
}

/// Stitched Assistant — {2}{U} 3/2 Zombie. Exploit; when it exploits a creature,
/// scry 1, then draw a card.
pub fn stitched_assistant() -> CardDefinition {
    use crate::effect::shortcut::exploit;
    CardDefinition {
        name: "Stitched Assistant",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![exploit(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}


/// Burn the Accursed — {4}{R} Instant. Deals 5 damage to target creature and 2
/// to its controller; if it would die this turn, exile it instead.
pub fn burn_the_accursed() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Burn the Accursed",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: target_filtered(SelectionRequirement::Creature) },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Fortify — {2}{W} Instant. Choose one — creatures you control get +2/+0, or
/// +0/+2, until end of turn.
pub fn fortify() -> CardDefinition {
    let team = Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Fortify",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT { what: team.clone(), power: Value::Const(2), toughness: Value::Const(0), duration: Duration::EndOfTurn },
            Effect::PumpPT { what: team, power: Value::Const(0), toughness: Value::Const(2), duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Lambholt Harrier — {1}{R} 2/2 Wolf. {3}{R}: target creature can't block this
/// turn.
pub fn lambholt_harrier() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lambholt Harrier",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crash the Ramparts — {2}{G} Instant. Target creature gets +3/+3 and gains
/// trample until end of turn.
pub fn crash_the_ramparts() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Crash the Ramparts",
        cost: cost(&[generic(2), g()]),
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
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Markov Purifier — {1}{W}{B} 2/3 Vampire Cleric. Lifelink. At your end step,
/// if you gained life this turn, you may pay {2} to draw a card.
pub fn markov_purifier() -> CardDefinition {
    CardDefinition {
        name: "Markov Purifier",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::LifeGainedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(1),
            }),
            effect: Effect::MayPay {
                description: "Pay {2} to draw a card?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Twins of Maurer Estate — {4}{B} 3/5 Vampire. Madness {2}{B}.
pub fn twins_of_maurer_estate() -> CardDefinition {
    CardDefinition {
        name: "Twins of Maurer Estate",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Madness(cost(&[generic(2), b()]))],
        ..Default::default()
    }
}

/// Estwald Shieldbasher — {3}{W} 4/2 Human Soldier. Attacks → may pay {1} to
/// gain indestructible until end of turn.
pub fn estwald_shieldbasher() -> CardDefinition {
    CardDefinition {
        name: "Estwald Shieldbasher",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1} for indestructible until end of turn?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Stensia Banquet — {2}{R} Sorcery. Deals damage to target opponent or
/// planeswalker equal to the number of Vampires you control. Draw a card.
pub fn stensia_banquet() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Stensia Banquet",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::OpponentPlayer.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(SelectionRequirement::Any)),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Vampire)
                        .and(SelectionRequirement::ControlledByYou),
                },
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Sheltering Boughs — {2}{G} Aura. Enchant creature. ETB draw a card.
/// Enchanted creature gets +1/+3.
pub fn sheltering_boughs() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Sheltering Boughs",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(SelectionRequirement::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 3, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

/// Vampire's Kiss — {1}{B} Sorcery. Target player loses 2 life and you gain 2
/// life. Create two Blood tokens.
pub fn vampires_kiss() -> CardDefinition {
    CardDefinition {
        name: "Vampire's Kiss",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: crabomination_base::tokens::blood_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Alchemist's Gift — {B} Instant. Target creature gets +1/+1 and gains your
/// choice of deathtouch or lifelink until end of turn.
pub fn alchemists_gift() -> CardDefinition {
    CardDefinition {
        name: "Alchemist's Gift",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::ChooseMode(vec![
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Dawnhart Geist — {1}{W} 1/3 Spirit Warlock. Whenever you cast an enchantment
/// spell, you gain 2 life.
pub fn dawnhart_geist() -> CardDefinition {
    CardDefinition {
        name: "Dawnhart Geist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(SelectionRequirement::HasCardType(CardType::Enchantment)),
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Bramble Wurm — {6}{G} 7/6 Wurm. Reach, trample. ETB gain 5 life. {2}{G},
/// Exile this card from your graveyard: You gain 5 life.
pub fn bramble_wurm() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Bramble Wurm",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(5) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Parish-Blade Trainee — {1}{W} 1/2 Human Soldier. Training. When it dies, put
/// its counters on target creature you control.
pub fn parish_blade_trainee() -> CardDefinition {
    use crate::effect::shortcut::{on_dies, training};
    CardDefinition {
        name: "Parish-Blade Trainee",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![
            training(),
            on_dies(Effect::MoveAllCounters {
                from: Selector::This,
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            }),
        ],
        ..Default::default()
    }
}

/// Olivia, Crimson Bride — {4}{B}{R} 3/4 Legendary Vampire Noble. Flying,
/// haste. Whenever Olivia attacks, return target creature card from a graveyard
/// to the battlefield tapped and attacking. (The legendary-Vampire exile rider
/// is omitted.)
pub fn olivia_crimson_bride() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Olivia, Crimson Bride",
        cost: cost(&[generic(4), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::JoinCombatAttacking { what: Selector::LastMoved },
        ]))],
        ..Default::default()
    }
}

/// Covetous Castaway // Ghostly Castigator — {1}{U} 1/3 Human. When it dies,
/// mill three. Disturb {3}{U}{U} into a 3/4 flying Spirit whose ETB may shuffle
/// up to three target cards from your graveyard into your library.
pub fn covetous_castaway() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    let castigator = CardDefinition {
        name: "Ghostly Castigator",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Shuffle up to three target cards from your graveyard into your library".into(),
            body: Box::new(Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    }),
                    count: Box::new(Value::Const(3)),
                },
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: crate::effect::LibraryPosition::Shuffled,
                },
            }),
        })],
        ..Default::default()
    };
    CardDefinition {
        name: "Covetous Castaway",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Disturb(cost(&[generic(3), u(), u()]))],
        triggered_abilities: vec![on_dies(Effect::Mill { who: Selector::You, amount: Value::Const(3) })],
        back_face: Some(Box::new(castigator)),
        ..Default::default()
    }
}

/// Geistwave — {1}{U} Instant. Return target nonland permanent to its owner's
/// hand. If you controlled that permanent, draw a card.
pub fn geistwave() -> CardDefinition {
    let bounce = || Effect::Move {
        what: target_filtered(SelectionRequirement::Nonland),
        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
    };
    CardDefinition {
        name: "Geistwave",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::ControlledByYou,
            },
            then: Box::new(Effect::Seq(vec![
                bounce(),
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
            else_: Box::new(bounce()),
        },
        ..Default::default()
    }
}

/// Adamant Will — {1}{W} Instant. Target creature gets +2/+2 and gains
/// indestructible until end of turn.
pub fn adamant_will() -> CardDefinition {
    CardDefinition {
        name: "Adamant Will",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Bladestitched Skaab — {U}{B} 2/3 Zombie Soldier. Other Zombies you control
/// get +1/+0.
pub fn bladestitched_skaab() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Bladestitched Skaab",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Other Zombies you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Zombie)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Angelic Quartermaster — {3}{W}{W} 3/3 Angel Soldier. Flying. ETB put a
/// +1/+1 counter on each of up to two other target creatures.
pub fn angelic_quartermaster() -> CardDefinition {
    CardDefinition {
        name: "Angelic Quartermaster",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::SupportCounters {
            max_targets: 2,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
        })],
        ..Default::default()
    }
}

/// Slogurk, the Overslime — {1}{G}{U} 3/3 Ooze. Trample. Whenever a land card
/// is put into your graveyard, put a +1/+1 counter on it. Remove three +1/+1
/// counters: return it to its owner's hand. When it leaves the battlefield,
/// return up to three target land cards from your graveyard to your hand.
pub fn slogurk_the_overslime() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    // "up to three target land cards from your graveyard" — auto-pulled.
    let return_lands = Effect::Move {
        what: Selector::Take {
            inner: Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::Land,
            }),
            count: Box::new(Value::Const(3)),
        },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Slogurk, the Overslime",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPutIntoGraveyard, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CreatureLeavesBattlefieldNotDying,
                    EventScope::SelfSource,
                ),
                effect: return_lands.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: return_lands,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 3)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Saryth, the Viper's Fang — {2}{G}{G} 3/4 Human Warlock. Other tapped
/// creatures you control have deathtouch; other untapped creatures you control
/// have hexproof. {1}, {T}: Untap another target creature or land you control.
pub fn saryth_the_vipers_fang() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility, StaticEffect};
    let others = |req: SelectionRequirement| {
        Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource)
                .and(req),
        )
    };
    CardDefinition {
        name: "Saryth, the Viper's Fang",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Other tapped creatures you control have deathtouch.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others(SelectionRequirement::Tapped),
                    keyword: Keyword::Deathtouch,
                },
            },
            StaticAbility {
                description: "Other untapped creatures you control have hexproof.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others(SelectionRequirement::Untapped),
                    keyword: Keyword::Hexproof,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Untap {
                what: target_filtered(
                    SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::OtherThanSource)
                        .and(
                            SelectionRequirement::Creature.or(SelectionRequirement::HasCardType(CardType::Land)),
                        ),
                ),
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reckless Stormseeker // Storm-Charged Slasher — {2}{R} 2/3 Human Werewolf,
/// Daybound. At the beginning of combat on your turn, target creature you
/// control gets +1/+0 and gains haste. Back: 3/4 Werewolf, Nightbound, +2/+0
/// trample + haste instead.
pub fn reckless_stormseeker() -> CardDefinition {
    let begin_combat = |power: i32, kws: Vec<Keyword>| {
        let mut seq = vec![Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(power),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        }];
        for kw in kws {
            seq.push(Effect::GrantKeyword { what: Selector::Target(0), keyword: kw, duration: Duration::EndOfTurn });
        }
        TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(seq),
        }
    };
    let slasher = CardDefinition {
        name: "Storm-Charged Slasher",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Nightbound],
        triggered_abilities: vec![begin_combat(2, vec![Keyword::Trample, Keyword::Haste])],
        ..Default::default()
    };
    CardDefinition {
        name: "Reckless Stormseeker",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Werewolf],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Daybound],
        triggered_abilities: vec![begin_combat(1, vec![Keyword::Haste])],
        back_face: Some(Box::new(slasher)),
        ..Default::default()
    }
}

/// Tovolar's Huntmaster // Tovolar's Packleader — {4}{G}{G} 6/6 Human Werewolf,
/// Daybound. ETB create two 2/2 green Wolves. Back: 7/7 Werewolf, Nightbound,
/// enters-or-attacks two Wolves + {2}{G}{G}: another Wolf/Werewolf you control
/// fights a creature you don't control.
pub fn tovolars_huntmaster() -> CardDefinition {
    use crate::card::{ActivatedAbility, TokenDefinition};
    use crate::mana::Color;
    let wolf = || TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    };
    let two_wolves = || Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: wolf() };
    let packleader = CardDefinition {
        name: "Tovolar's Packleader",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Nightbound],
        triggered_abilities: vec![
            etb(two_wolves()),
            crate::effect::shortcut::on_attack(two_wolves()),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            effect: Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::OtherThanSource)
                        .and(
                            SelectionRequirement::HasCreatureType(CreatureType::Wolf)
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Werewolf)),
                        ),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Tovolar's Huntmaster",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Werewolf],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Daybound],
        triggered_abilities: vec![etb(two_wolves())],
        back_face: Some(Box::new(packleader)),
        ..Default::default()
    }
}

/// Dreadhound — {4}{B}{B} 6/6 Demon Dog. ETB mill three. Whenever a creature
/// dies or a creature card is put into a graveyard from a library, each
/// opponent loses 1 life.
pub fn dreadhound() -> CardDefinition {
    let drain_each_opp = || Effect::LoseLife {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Dreadhound",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon, CreatureType::Dog],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
                effect: drain_each_opp(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardMilled, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    },
                ),
                effect: drain_each_opp(),
            },
        ],
        ..Default::default()
    }
}

/// Mask of Avacyn — {2} Equipment. Equipped creature gets +1/+2 and has
/// hexproof. Equip {3}.
pub fn mask_of_avacyn() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Mask of Avacyn",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 2,
            keywords: vec![Keyword::Hexproof],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Stormchaser Drake — {1}{U} 2/1 Drake. Flying. Whenever this becomes the
/// target of a spell you control, draw a card.
pub fn stormchaser_drake() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser Drake",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drake], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Falkenrath Pit Fighter — {R} 2/1 Vampire Warrior. {1}{R}, discard a card,
/// sacrifice a Vampire: draw two cards. Activate only if an opponent lost life
/// this turn.
pub fn falkenrath_pit_fighter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Falkenrath Pit Fighter",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            discard_cost: Some((SelectionRequirement::Any, 1)),
            sac_other_filter: Some((
                SelectionRequirement::HasCreatureType(CreatureType::Vampire),
                1,
            )),
            condition: Some(Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent }),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hungry Ridgewolf — {1}{R} 2/2 Wolf. As long as you control another Wolf or
/// Werewolf, it gets +1/+0 and has trample.
pub fn hungry_ridgewolf() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Hungry Ridgewolf",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "While you control another Wolf or Werewolf: +1/+0 and trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Wolf)
                            .or(SelectionRequirement::HasCreatureType(CreatureType::Werewolf))
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..Default::default()
    }
}

/// Skaab Wrangler — {1}{U} 2/1 Human Wizard. Tap three untapped creatures you
/// control: Tap target creature.
pub fn skaab_wrangler() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Skaab Wrangler",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((SelectionRequirement::Creature, 3)),
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blood Petal Celebrant — {1}{R} 2/1 Vampire. First strike while attacking.
/// When it dies, create a Blood token.
pub fn blood_petal_celebrant() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Blood Petal Celebrant",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Has first strike as long as it's attacking.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::IsAttacking,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        })],
        ..Default::default()
    }
}

/// Questing Beast — {2}{G}{G} 4/4 Legendary Beast. Vigilance, deathtouch,
/// haste; can't be blocked by creatures with power 2 or less; combat damage
/// dealt by creatures you control can't be prevented. (The planeswalker-redirect
/// rider is omitted — planeswalkers aren't attack targets yet.)
pub fn questing_beast() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Questing Beast",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Deathtouch,
            Keyword::Haste,
            Keyword::CantBeBlockedByPowerAtMost(2),
        ],
        static_abilities: vec![StaticAbility {
            description: "Combat damage that would be dealt by creatures you control can't be prevented.",
            effect: StaticEffect::ControllerCreaturesCombatDamageCantBePrevented,
        }],
        ..Default::default()
    }
}

/// Cackling Slasher — {3}{B} 3/3 Human Assassin. Deathtouch; enters with a
/// +1/+1 counter if a creature died this turn.
pub fn cackling_slasher() -> CardDefinition {
    CardDefinition {
        name: "Cackling Slasher",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Vaultborn Tyrant — {5}{G}{G} 6/6 Dinosaur. Trample. Whenever this or
/// another creature you control with power 4+ enters, gain 3 life and draw.
/// When it dies (if not a token), create a token copy of it.
pub fn vaultborn_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Vaultborn Tyrant",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(
                        Value::PowerOf(Box::new(Selector::TriggerSource)),
                        Value::Const(4),
                    )),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::NotToken,
                    }),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    // The token "is an artifact in addition to its other types".
                    extra_card_types: vec![CardType::Artifact],
                    override_pt: None,
                    non_legendary: false,
                    legendary: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Star Charter — {3}{W} 3/1 Bat Cleric. Flying. At your end step, if you
/// gained or lost life this turn, look at the top four cards; you may reveal a
/// creature card with power 3 or less and put it into your hand.
pub fn star_charter() -> CardDefinition {
    CardDefinition {
        name: "Star Charter",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::Any(vec![
                Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(1) },
                Predicate::PlayerLostLifeThisTurn { who: PlayerRef::You },
            ])),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: false,
                pick_filter: Some(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
                ),
                take: None,
                to_battlefield: false,
            },
        }],
        ..Default::default()
    }
}

/// Dour Port-Mage — {1}{U} 1/3 Frog Wizard. Whenever one or more other
/// creatures you control leave the battlefield without dying, draw a card
/// (modeled per-creature). {1}{U}, {T}: Return another target creature you
/// control to its owner's hand.
pub fn dour_port_mage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Dour Port-Mage",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureLeavesBattlefieldNotDying,
                EventScope::AnotherOfYours,
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Three Tree Scribe — {1}{G} 2/3 Frog Druid. Whenever this or another creature
/// you control leaves the battlefield without dying, put a +1/+1 counter on
/// target creature you control. (The self-leave half is approximate — the
/// "another creature you control" case is the one that fires.)
pub fn three_tree_scribe() -> CardDefinition {
    CardDefinition {
        name: "Three Tree Scribe",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureLeavesBattlefieldNotDying,
                EventScope::YourControl,
            ),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Krydle of Baldur's Gate — {U}{B} 1/3 Legendary Human Elf Rogue. Whenever it
/// deals combat damage to a player, that player loses 1 life and mills a card,
/// then you gain 1 life and scry 1. (The attack pay-{2} unblockable rider is
/// omitted.)
pub fn krydle_of_baldurs_gate() -> CardDefinition {
    CardDefinition {
        name: "Krydle of Baldur's Gate",
        cost: cost(&[u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Elf, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(1) },
                Effect::Mill { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(1) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Wary Watchdog — {1}{G} 3/1 Dog. When it enters or dies, surveil 1.
pub fn wary_watchdog() -> CardDefinition {
    CardDefinition {
        name: "Wary Watchdog",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }),
            on_dies(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }),
        ],
        ..Default::default()
    }
}

/// Hunted Bonebrute — {2}{B} 6/2 Skeleton Beast. Menace; when it enters, target
/// opponent creates two 1/1 white Dog tokens; when it dies, each opponent loses
/// 3 life. Disguise {1}{B}.
pub fn hunted_bonebrute() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Hunted Bonebrute",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 2,
        keywords: vec![Keyword::Menace, Keyword::Disguise(cost(&[generic(1), b()]))],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::EachOpponent,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Dog".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Dog],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            on_dies(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            }),
        ],
        ..Default::default()
    }
}

/// Trumpeting Herd — {2}{G}{G} Sorcery. Create a 3/3 green Elephant token.
/// Rebound.
pub fn trumpeting_herd() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Trumpeting Herd",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Rebound],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Elephant".into(),
                power: 3,
                toughness: 3,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elephant],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Festergloom — {2}{B} Sorcery. Nonblack creatures get -1/-1 until end of turn.
pub fn festergloom() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Festergloom",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(Color::Black).negate()),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Intrepid Rabbit — {2}{W} 3/2 Rabbit Soldier. Offspring {1}. When it enters,
/// target creature you control gets +1/+1 until end of turn.
pub fn intrepid_rabbit() -> CardDefinition {
    CardDefinition {
        name: "Intrepid Rabbit",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Offspring(cost(&[generic(1)]))],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Marauding Brinefang — {5}{U}{U} 6/7 Dinosaur. Ward {3}. Islandcycling {2}.
pub fn marauding_brinefang() -> CardDefinition {
    use crate::card::{LandType, WardCost};
    CardDefinition {
        name: "Marauding Brinefang",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 7,
        keywords: vec![
            Keyword::Ward(WardCost::Mana(cost(&[generic(3)]))),
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                SelectionRequirement::HasLandType(LandType::Island),
            ))),
        ],
        ..Default::default()
    }
}

/// Crystal Barricade — {1}{W} 0/4 Wall. Defender; you have hexproof. (The
/// prevent-noncombat-damage-to-your-other-creatures rider is omitted.)
pub fn crystal_barricade() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Crystal Barricade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "You have hexproof.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        ..Default::default()
    }
}

/// Persistent Marshstalker — {1}{B} 3/1 Rat Berserker. Gets +1/+0 for each
/// other Rat you control. (Its threshold attack-recursion is omitted.)
pub fn persistent_marshstalker() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Persistent Marshstalker",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each other Rat you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Rat)
                    .and(SelectionRequirement::OtherThanSource),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Druid of the Spade — {2}{G} 2/3 Rabbit Druid. As long as you control a
/// token, it gets +2/+0 and has trample.
pub fn druid_of_the_spade() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Druid of the Spade",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "As long as you control a token, this creature gets +2/+0 and has trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::IsToken.and(SelectionRequirement::ControlledByYou),
                )),
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..Default::default()
    }
}

/// Nightbird's Clutches — {1}{R} Sorcery. Up to two target creatures can't
/// block this turn. Flashback {3}{R}.
pub fn nightbirds_clutches() -> CardDefinition {
    CardDefinition {
        name: "Nightbird's Clutches",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Get Out — {U}{U} Instant. Choose one — counter target creature or enchantment
/// spell; or return one or two target creatures and/or enchantments you own to
/// your hand.
pub fn get_out() -> CardDefinition {
    CardDefinition {
        name: "Get Out",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                    ),
                },
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: (SelectionRequirement::Creature.or(SelectionRequirement::Enchantment))
                    .and(SelectionRequirement::ControlledByYou),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Helpful Hunter — {1}{W} 1/1 Cat. When it enters, draw a card.
pub fn helpful_hunter() -> CardDefinition {
    CardDefinition {
        name: "Helpful Hunter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

/// Sunshower Druid — {G} 0/2 Frog Druid. When it enters, put a +1/+1 counter on
/// target creature and you gain 1 life.
pub fn sunshower_druid() -> CardDefinition {
    CardDefinition {
        name: "Sunshower Druid",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Treetop Snarespinner — {3}{G} 1/4 Spider. Reach, deathtouch. {2}{G}: Put a
/// +1/+1 counter on target creature you control. Activate only as a sorcery.
pub fn treetop_snarespinner() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Treetop Snarespinner",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Coruscation Mage — {1}{R} 2/2 Otter Wizard. Offspring {2}. Whenever you cast
/// a noncreature spell, this deals 1 damage to each opponent.
pub fn coruscation_mage() -> CardDefinition {
    CardDefinition {
        name: "Coruscation Mage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Brimstone Vandal — {2}{R} 2/3 Devil. Menace. If it's neither day nor night,
/// it becomes day as this enters. Whenever day becomes night or night becomes
/// day, it deals 1 damage to each opponent.
pub fn brimstone_vandal() -> CardDefinition {
    CardDefinition {
        name: "Brimstone Vandal",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::Not(Box::new(Predicate::Any(vec![
                    Predicate::IsDay,
                    Predicate::IsNight,
                ]))),
                then: Box::new(Effect::BecomeDay),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DayNightChanged, EventScope::AnyPlayer),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Cemetery Gatekeeper — {1}{R} 2/1 Vampire. First strike. ETB exile a card
/// from a graveyard. Whenever a player plays a land or casts a spell that
/// shares a card type with the exiled card, it deals 2 damage to that player.
pub fn cemetery_gatekeeper() -> CardDefinition {
    CardDefinition {
        name: "Cemetery Gatekeeper",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            etb(Effect::ExileTaggedWithSource {
                what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::InGraveyard },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                    .with_filter(Predicate::SharesCardTypeWithExiledBySource),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::AnyPlayer)
                    .with_filter(Predicate::SharesCardTypeWithExiledBySource),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Cemetery Protector — {2}{W}{W} 3/4 Human Soldier. Flash. ETB exile a card
/// from a graveyard. Whenever you play a land or cast a spell that shares a
/// card type with the exiled card, create a 1/1 white Human creature token.
pub fn cemetery_protector() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let human = TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    };
    let token_effect = Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: human,
    };
    CardDefinition {
        name: "Cemetery Protector",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::ExileTaggedWithSource {
                what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::InGraveyard },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::SharesCardTypeWithExiledBySource),
                effect: token_effect.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl)
                    .with_filter(Predicate::SharesCardTypeWithExiledBySource),
                effect: token_effect,
            },
        ],
        ..Default::default()
    }
}

/// Thornplate Intimidator — {3}{B} 4/3 Rat Rogue. Offspring {3}. When it
/// enters, each opponent loses 3 life unless they sacrifice a nonland permanent
/// or discard a card. (Modeled as each opponent; printed targets one.)
pub fn thornplate_intimidator() -> CardDefinition {
    CardDefinition {
        name: "Thornplate Intimidator",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Offspring(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Punisher {
            chooser: Selector::Player(PlayerRef::EachOpponent),
            options: vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::You),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Nonland,
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(1),
                    random: false,
                },
            ],
            otherwise: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            }),
        })],
        ..Default::default()
    }
}

/// Repeating Barrage — {1}{R}{R} Sorcery. Deals 3 damage to any target. Raid —
/// {3}{R}{R}: Return this from your graveyard to your hand if you attacked.
pub fn repeating_barrage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::{deal, target_any};
    CardDefinition {
        name: "Repeating Barrage",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: deal(3, target_any()),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r(), r()]),
            from_graveyard: true,
            condition: Some(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fountainport Bell — {1} Artifact. When it enters, you may search your library
/// for a basic land and put it on top. {1}, Sacrifice: Draw a card.
pub fn fountainport_bell() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Fountainport Bell",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Overprotect — {1}{G} Instant. Target creature you control gets +3/+3 and
/// gains trample, hexproof, and indestructible until end of turn.
pub fn overprotect() -> CardDefinition {
    let grant = |kw| Effect::GrantKeyword { what: Selector::Target(0), keyword: kw, duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Overprotect",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            grant(Keyword::Trample),
            grant(Keyword::Hexproof),
            grant(Keyword::Indestructible),
        ]),
        ..Default::default()
    }
}

/// Plumecreed Escort — {1}{U} 2/1 Bird Scout. Flash, flying. When it enters,
/// target creature you control gains hexproof until end of turn.
pub fn plumecreed_escort() -> CardDefinition {
    CardDefinition {
        name: "Plumecreed Escort",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Banishing Slash — {W}{W} Sorcery. Destroy up to one target artifact,
/// enchantment, or tapped creature. Then if you control an artifact and an
/// enchantment, create a 2/2 white Samurai with vigilance.
pub fn banishing_slash() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Banishing Slash",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Creature.and(SelectionRequirement::Tapped)),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            Effect::If {
                cond: Predicate::All(vec![
                    Predicate::SelectorExists(Selector::EachPermanent(
                        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    )),
                    Predicate::SelectorExists(Selector::EachPermanent(
                        SelectionRequirement::Enchantment.and(SelectionRequirement::ControlledByYou),
                    )),
                ]),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Samurai".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Samurai],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Vigilance],
                        ..Default::default()
                    },
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Marsh Hulk — {4}{B}{B} 4/6 Zombie Ogre. Megamorph {6}{B}.
pub fn marsh_hulk() -> CardDefinition {
    CardDefinition {
        name: "Marsh Hulk",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Ogre],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Megamorph(cost(&[generic(6), b()]))],
        ..Default::default()
    }
}

/// Brave-Kin Duo — {W} 1/1 Rabbit Mouse. {1}, {T}: Target creature gets +1/+1
/// until end of turn. Activate only as a sorcery.
pub fn brave_kin_duo() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Brave-Kin Duo",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Mouse],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lightshield Parry — {W} Instant. Target creature gets +2/+2 until end of
/// turn. Cycling {2}.
pub fn lightshield_parry() -> CardDefinition {
    CardDefinition {
        name: "Lightshield Parry",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Hard-Hitting Question — {G} Sorcery. Target creature you control deals damage
/// equal to its power to target creature or planeswalker you don't control.
pub fn hard_hitting_question() -> CardDefinition {
    CardDefinition {
        name: "Hard-Hitting Question",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker)
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            })),
        },
        ..Default::default()
    }
}

/// Refurbished Familiar — {3}{B} 2/1 Zombie Rat. Affinity for artifacts, flying.
/// When it enters, each opponent discards a card. (The draw-per-opponent-who-
/// can't rider is omitted.)
pub fn refurbished_familiar() -> CardDefinition {
    CardDefinition {
        name: "Refurbished Familiar",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Rat],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(SelectionRequirement::Artifact),
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
    }
}

/// Galvanic Discharge — {R} Instant. Choose target creature or planeswalker.
/// You get {E}{E}{E}, then you may pay any amount of {E}; deal that much damage.
pub fn galvanic_discharge() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Discharge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddEnergy(Value::Const(3)),
            Effect::PayAnyEnergyDealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// This Town Ain't Big Enough — {4}{U} Instant. Costs {3} less if it targets a
/// permanent you control. Return up to two target nonland permanents to hand.
pub fn this_town_aint_big_enough() -> CardDefinition {
    CardDefinition {
        name: "This Town Ain't Big Enough",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::ControlledByYou, 3)),
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Nonland,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        },
        ..Default::default()
    }
}

/// Hardened Scales — {G} enchantment. If one or more +1/+1 counters would be
/// put on a creature you control, that many plus one are put on it instead.
pub fn hardened_scales() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Hardened Scales",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature \
                          you control, that many plus one +1/+1 counters are put \
                          on it instead.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        ..Default::default()
    }
}

/// Highspire Bell-Ringer — {2}{U} 1/4 Djinn Monk. Flying; the second spell you
/// cast each turn costs {1} less.
pub fn highspire_bell_ringer() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Highspire Bell-Ringer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "The second spell you cast each turn costs {1} less to cast.",
            effect: StaticEffect::CostReductionNthSpell {
                filter: SelectionRequirement::Any,
                nth: 2,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Emberheart Challenger — {1}{R} 2/2 Mouse Warrior. Haste, prowess; Valiant
/// — the first time it becomes the target of your spell/ability each turn,
/// exile the top card of your library; you may play it this turn.
pub fn emberheart_challenger() -> CardDefinition {
    CardDefinition {
        name: "Emberheart Challenger",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            crate::effect::shortcut::prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl)
                    .once_per_turn(),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Eldrazi Linebreaker — {1}{C}{R} 3/3 Eldrazi. Devoid, trample. At the
/// beginning of combat on your turn, target creature you control gains haste
/// and gets +X/+0 until end of turn, where X is the number of Eldrazi you
/// control.
pub fn eldrazi_linebreaker() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Linebreaker",
        cost: cost(&[generic(1), colorless(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Devoid, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::ControlledByYou,
                        )),
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Eldrazi),
                    },
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// No More Lies — {W}{U} Instant. Counter target spell unless its controller
/// pays {3}. If countered this way, exile it instead.
pub fn no_more_lies() -> CardDefinition {
    CardDefinition {
        name: "No More Lies",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(3)]),
            exile: true,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Unstoppable Slasher — {2}{B} 2/3 Zombie Assassin. Deathtouch; when it deals
/// combat damage to a player, they lose half their life, rounded up. When it
/// dies, return it tapped with two stun counters under its owner's control.
pub fn unstoppable_slasher() -> CardDefinition {
    CardDefinition {
        name: "Unstoppable Slasher",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LoseHalfLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    rounded_up: true,
                },
            },
            on_dies(Effect::ReturnSelfTappedWithCounters {
                kind: CounterType::Stun,
                amount: 2,
            }),
        ],
        ..Default::default()
    }
}

/// Enduring Curiosity — {2}{U}{U} 4/3 Cat Glimmer enchantment creature. Flash;
/// whenever a creature you control deals combat damage to a player, draw a
/// card. When it dies, return it to the battlefield as an enchantment.
pub fn enduring_curiosity() -> CardDefinition {
    CardDefinition {
        name: "Enduring Curiosity",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Glimmer],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
            on_dies(Effect::ReturnSelfAsEnchantment),
        ],
        ..Default::default()
    }
}

/// Galvanic Relay — {2}{R} Sorcery. Exile the top card of your library; you
/// may play it during your next turn. Storm.
pub fn galvanic_relay() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Relay",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(1),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        },
        ..Default::default()
    }
}

/// The Necrobloom — {1}{W}{B}{G} 2/7 Legendary Plant. Landfall — whenever a
/// land you control enters, create a 0/1 green Plant token; if you control 7+
/// lands with different names, a 2/2 Zombie instead. (The "lands in your
/// graveyard have dredge 2" static is omitted.)
pub fn the_necrobloom() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let plant = TokenDefinition {
        name: "Plant".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "The Necrobloom",
        cost: cost(&[generic(1), w(), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 2,
        toughness: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: plant },
        }],
        ..Default::default()
    }
}

/// Tyvar's Stand — {X}{G} Instant. Target creature you control gets +X/+X and
/// gains hexproof and indestructible until end of turn.
pub fn tyvars_stand() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Tyvar's Stand",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
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
        ]),
        ..Default::default()
    }
}

/// Gird for Battle — {W} Sorcery. Put a +1/+1 counter on each of up to two
/// target creatures.
pub fn gird_for_battle() -> CardDefinition {
    CardDefinition {
        name: "Gird for Battle",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
        ..Default::default()
    }
}

/// Stock Up — {2}{U} Sorcery. Look at the top five cards of your library, put
/// two into your hand and the rest on the bottom.
pub fn stock_up() -> CardDefinition {
    CardDefinition {
        name: "Stock Up",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: None,
            take: Some(Value::Const(2)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Shelter — {1}{W} Instant. Target creature you control gains protection from
/// the color of your choice until end of turn. Draw a card.
pub fn shelter() -> CardDefinition {
    CardDefinition {
        name: "Shelter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantProtectionFromChosenColor {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Pick Your Poison — {G} Sorcery. Choose one — each opponent sacrifices an
/// artifact / an enchantment / a creature with flying, their choice.
pub fn pick_your_poison() -> CardDefinition {
    let edict = |filter: SelectionRequirement| Effect::Sacrifice {
        who: Selector::Player(PlayerRef::EachOpponent),
        count: Value::Const(1),
        filter,
    };
    CardDefinition {
        name: "Pick Your Poison",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            edict(SelectionRequirement::Artifact),
            edict(SelectionRequirement::Enchantment),
            edict(SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying))),
        ]),
        ..Default::default()
    }
}

/// Tail Swipe — {G} Instant. Target creature you control fights target creature
/// you don't control; if cast in your main phase, yours gets +1/+1 until end of
/// turn first.
pub fn tail_swipe() -> CardDefinition {
    use crate::effect::Predicate;
    use crate::game::TurnStep;
    let attacker = Selector::TargetFiltered {
        slot: 0,
        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    };
    CardDefinition {
        name: "Tail Swipe",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::Any(vec![
                        Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                        Predicate::CurrentStepIs(TurnStep::PostCombatMain),
                    ]),
                ]),
                then: Box::new(Effect::PumpPT {
                    what: attacker.clone(),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker,
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Lightning Axe — {R} Instant. (As an additional cost, discard a card.)
/// Deals 5 damage to target creature. (The "or pay {5}" alternative is
/// omitted — the discard is taken at resolution, Deadly-Dispute style.)
pub fn lightning_axe() -> CardDefinition {
    CardDefinition {
        name: "Lightning Axe",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(5),
            },
        ]),
        ..Default::default()
    }
}

/// Stormsplitter — {3}{R} 1/4 Otter Wizard. Haste. Whenever you cast an instant
/// or sorcery spell, create a token copy of this creature; exile it at the
/// beginning of the next end step.
pub fn stormsplitter() -> CardDefinition {
    use crate::effect::DelayedTriggerKind;
    CardDefinition {
        name: "Stormsplitter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    non_legendary: false,
                    legendary: false,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Unburden — {1}{B}{B} Sorcery. Target player discards two cards. Cycling {2}.
pub fn unburden() -> CardDefinition {
    CardDefinition {
        name: "Unburden",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(2),
            random: false,
        },
        ..Default::default()
    }
}

/// Goblin Anarchomancer — {R}{G} 2/2 Goblin Shaman. Each red or green spell you
/// cast costs {1} less to cast.
pub fn goblin_anarchomancer() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    use crate::mana::Color;
    CardDefinition {
        name: "Goblin Anarchomancer",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Red or green spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasColor(Color::Red)
                    .or(SelectionRequirement::HasColor(Color::Green)),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Beza, the Bounding Spring — {2}{W}{W} 4/5 Legendary Elemental Elk. ETB: a
/// Treasure if an opponent has more lands; gain 4 if an opponent has more life;
/// two 1/1 Fish if an opponent has more creatures; draw if an opponent has more
/// cards in hand.
pub fn beza_the_bounding_spring() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let if_then = |cond: Predicate, then: Effect| Effect::If {
        cond,
        then: Box::new(then),
        else_: Box::new(Effect::Noop),
    };
    let fish = TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Beza, the Bounding Spring",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Elk],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            if_then(
                Predicate::OpponentControlsMoreLandsThanYou,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            ),
            if_then(
                Predicate::AnOpponentHasMoreLife,
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ),
            if_then(
                Predicate::AnOpponentControlsMoreCreatures,
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: fish },
            ),
            if_then(
                Predicate::AnOpponentHasMoreCardsInHand,
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ),
        ]))],
        ..Default::default()
    }
}

/// Optimistic Scavenger — {W} 1/1 Human Scout. Eerie — whenever an enchantment
/// you control enters or you fully unlock a Room, put a +1/+1 counter on
/// target creature.
pub fn optimistic_scavenger() -> CardDefinition {
    CardDefinition {
        name: "Optimistic Scavenger",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: crate::effect::shortcut::eerie(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        }),
        ..Default::default()
    }
}

/// Frilled Sandwalla — {G} 1/1 Lizard. {1}{G}: +2/+2 until end of turn,
/// once each turn.
pub fn frilled_sandwalla() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Frilled Sandwalla",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Lizard], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spell Stutter — {1}{U} Instant. Counter target spell unless its controller
/// pays {2} plus {1} for each Faerie you control.
pub fn spell_stutter() -> CardDefinition {
    CardDefinition {
        name: "Spell Stutter",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
            exile: false,
            extra_generic: Some(Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Faerie),
            }),
        },
        ..Default::default()
    }
}

/// Spectral Interference — {1}{U} Instant. Counter target artifact or creature
/// spell unless its controller pays {4}.
pub fn spectral_interference() -> CardDefinition {
    CardDefinition {
        name: "Spectral Interference",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::HasCardType(CardType::Creature)),
                ),
            ),
            mana_cost: cost(&[generic(4)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Refute — {1}{U}{U} Instant. Counter target spell. Draw a card, then discard.
pub fn refute() -> CardDefinition {
    CardDefinition {
        name: "Refute",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Skullcap Snail — {1}{B} 1/1 Fungus Snail. ETB: target opponent exiles a
/// card from their hand.
pub fn skullcap_snail() -> CardDefinition {
    CardDefinition {
        name: "Skullcap Snail",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Snail],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ExileFromHand {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Aspirant's Ascent — {U} Instant. Target creature gets +1/+3 and gains flying
/// and toxic 1 until end of turn.
pub fn aspirants_ascent() -> CardDefinition {
    CardDefinition {
        name: "Aspirant's Ascent",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Toxic(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Take the Fall — {U} Instant. Target creature gets -1/-0 (or -4/-0 if you
/// control an outlaw) until end of turn. Draw a card.
pub fn take_the_fall() -> CardDefinition {
    let outlaw = SelectionRequirement::Creature
        .and(SelectionRequirement::ControlledByYou)
        .and(
            SelectionRequirement::HasCreatureType(CreatureType::Assassin)
                .or(SelectionRequirement::HasCreatureType(CreatureType::Mercenary))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Pirate))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Rogue))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Warlock)),
        );
    CardDefinition {
        name: "Take the Fall",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(outlaw),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-3),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Hopeful Vigil — {1}{W} Enchantment. ETB: create a 2/2 white Knight with
/// vigilance. When it leaves the battlefield, scry 2. {2}{W}: sacrifice it.
pub fn hopeful_vigil() -> CardDefinition {
    use crate::card::{ActivatedAbility, TokenDefinition};
    use crate::mana::Color;
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Hopeful Vigil",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: knight }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sac_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hopeless Nightmare — {B} Enchantment. ETB: each opponent discards a card and
/// loses 2 life. When it leaves the battlefield, scry 2. {2}{B}: sacrifice it.
pub fn hopeless_nightmare() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Hopeless Nightmare",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hangar Scrounger — {2}{R} 2/1 Dwarf Pilot. Backup 1. Whenever it becomes
/// tapped, you may discard a card; if you do, draw a card. (The backup-grant
/// of the loot ability to the backed-up creature is omitted.)
pub fn hangar_scrounger() -> CardDefinition {
    let loot = TriggeredAbility {
        event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
        effect: Effect::MayDo {
            description: "discard a card, then draw a card".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
        },
    };
    CardDefinition {
        name: "Hangar Scrounger",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::backup(1, vec![]), loot],
        ..Default::default()
    }
}

/// Bristlebud Farmer — {2}{G}{G} 5/5 Plant Druid. Trample. ETB: create two
/// Food tokens. (The attack "sac a Food → mill three, grab a permanent" rider
/// is omitted.)
pub fn bristlebud_farmer() -> CardDefinition {
    CardDefinition {
        name: "Bristlebud Farmer",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: crabomination_base::tokens::food_token(),
        })],
        ..Default::default()
    }
}

/// Outcaster Greenblade — {2}{G} 1/2 Human Mercenary. ETB: search your library
/// for a basic land or Desert card and put it into your hand. Gets +1/+1 for
/// each Desert you control.
pub fn outcaster_greenblade() -> CardDefinition {
    use crate::card::{DynamicPt, LandType};
    CardDefinition {
        name: "Outcaster Greenblade",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        dynamic_pt: Some(DynamicPt::BasePlusLandsOfTypeControlled {
            land_type: LandType::Desert,
            base_p: 1,
            base_t: 2,
        }),
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand
                .or(SelectionRequirement::HasLandType(LandType::Desert)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Mizzium Skin — {U} Instant. Target creature you control gets +0/+1 and gains
/// hexproof until end of turn. Overload {3}{U}: each creature you control
/// instead.
pub fn mizzium_skin() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Mizzium Skin",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(0),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u()]),
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(0),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Demand Answers — {1}{R} Instant. (As an additional cost, discard a card —
/// the "sacrifice an artifact" alternative is omitted.) Draw two cards.
pub fn demand_answers() -> CardDefinition {
    CardDefinition {
        name: "Demand Answers",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Boltwave — {R} Sorcery. Deals 3 damage to each opponent.
pub fn boltwave() -> CardDefinition {
    CardDefinition {
        name: "Boltwave",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Strike It Rich — {R} Sorcery. Create a Treasure token. Flashback {2}{R}.
pub fn strike_it_rich() -> CardDefinition {
    CardDefinition {
        name: "Strike It Rich",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crate::game::effects::treasure_token(),
        },
        ..Default::default()
    }
}

/// Brotherhood's End — {1}{R}{R} Sorcery. Choose one — 3 damage to each
/// creature and planeswalker; or destroy all artifacts with mana value 3 or
/// less.
pub fn brotherhoods_end() -> CardDefinition {
    CardDefinition {
        name: "Brotherhood's End",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::Destroy {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Boon-Bringer Valkyrie — {3}{W}{W} 4/4 Angel Warrior. Flying, first strike,
/// lifelink. Backup 1 (grants those abilities to the backed-up creature).
pub fn boon_bringer_valkyrie() -> CardDefinition {
    CardDefinition {
        name: "Boon-Bringer Valkyrie",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        triggered_abilities: vec![crate::effect::shortcut::backup(
            1,
            vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        )],
        ..Default::default()
    }
}

/// Inti, Seneschal of the Sun — {1}{R} 2/2 Legendary Human Knight. Whenever you
/// attack, you may discard a card to put a +1/+1 counter on target attacking
/// creature and give it trample. Whenever you discard a card, exile the top of
/// your library; you may play it until your next end step.
pub fn inti_seneschal_of_the_sun() -> CardDefinition {
    CardDefinition {
        name: "Inti, Seneschal of the Sun",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Discard a card to grow a target attacking creature".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                        // CR 603.7 — the counter's target is chosen on the
                        // reflexive "when you do", after the discard, not up front.
                        Effect::Reflexive {
                            body: Box::new(Effect::Seq(vec![
                                Effect::AddCounter {
                                    what: target_filtered(SelectionRequirement::IsAttacking),
                                    kind: CounterType::PlusOnePlusOne,
                                    amount: Value::Const(1),
                                },
                                Effect::GrantKeyword {
                                    what: Selector::Target(0),
                                    keyword: Keyword::Trample,
                                    duration: Duration::EndOfTurn,
                                },
                            ])),
                        },
                    ])),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Warren Soultrader — {2}{B} 3/3 Zombie Goblin Wizard. Pay 1 life, Sacrifice
/// another creature: Create a Treasure token.
pub fn warren_soultrader() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Warren Soultrader",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hostile Investigator — {3}{B} 4/3 Ogre Rogue Detective. When it enters,
/// target opponent discards a card. Whenever one or more players discard one or
/// more cards, investigate (once each turn).
pub fn hostile_investigator() -> CardDefinition {
    CardDefinition {
        name: "Hostile Investigator",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rogue, CreatureType::Detective],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::AnyPlayer)
                    .once_per_turn(),
                effect: crate::effect::shortcut::investigate(1),
            },
        ],
        ..Default::default()
    }
}

/// Marshal of Zhalfir — {W}{U} 2/2 Human Knight. Other Knights you control get
/// +1/+1. {W}{U}, {T}: Tap another target creature.
pub fn marshal_of_zhalfir() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Marshal of Zhalfir",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Knights you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Knight)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pawpatch Recruit — {G} 2/1 Rabbit Warrior with trample. Offspring {2}.
/// Whenever a creature you control becomes the target of a spell or ability an
/// opponent controls, put a +1/+1 counter on target creature you control other
/// than that creature.
pub fn pawpatch_recruit() -> CardDefinition {
    CardDefinition {
        name: "Pawpatch Recruit",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![
            // Offspring (CR 702.166): if its cost was paid, mint a 1/1 copy.
            etb(Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    non_legendary: false,
                    legendary: false,
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::BecameTarget,
                    EventScope::YourPermanentTargetedByOpponent,
                ),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Helping Hand — {W} Sorcery. Return target creature card with mana value 3 or
/// less from your graveyard to the battlefield tapped.
pub fn helping_hand() -> CardDefinition {
    CardDefinition {
        name: "Helping Hand",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3))
                    .and(SelectionRequirement::InYourGraveyard),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Diversion Unit — {1}{U} 2/1 Robot artifact creature with flying. {U},
/// Sacrifice this creature: Counter target instant or sorcery spell unless its
/// controller pays {3}.
pub fn diversion_unit() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Diversion Unit",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                ),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Final Vengeance — {B} Sorcery. As an additional cost, sacrifice a creature
/// or enchantment. Exile target creature.
pub fn final_vengeance() -> CardDefinition {
    CardDefinition {
        name: "Final Vengeance",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: (SelectionRequirement::Creature.or(SelectionRequirement::Enchantment))
                .and(SelectionRequirement::ControlledByYou),
            count: 1,
        }],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Roughshod Mentor — {5}{G} 5/4 Giant Warrior. Green creatures you control
/// have trample.
pub fn roughshod_mentor() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    use crate::mana::Color;
    CardDefinition {
        name: "Roughshod Mentor",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Green creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasColor(Color::Green)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

/// Innocuous Rat — {1}{B} 1/1 Rat. When it dies, manifest dread.
pub fn innocuous_rat() -> CardDefinition {
    CardDefinition {
        name: "Innocuous Rat",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::ManifestDread { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Quaketusk Boar — {3}{R}{R} 5/5 Elemental Boar with reach, trample, haste.
pub fn quaketusk_boar() -> CardDefinition {
    CardDefinition {
        name: "Quaketusk Boar",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Boar],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Haste],
        ..Default::default()
    }
}

/// Veteran Guardmouse — {3}{R/W} 3/4 Mouse Soldier. Valiant — the first time it
/// becomes the target of a spell or ability you control each turn, it gets
/// +1/+0 and gains first strike until end of turn, then scry 1.
pub fn veteran_guardmouse() -> CardDefinition {
    use crate::mana::hybrid;
    use crate::mana::Color::{Red, White};
    CardDefinition {
        name: "Veteran Guardmouse",
        cost: cost(&[generic(3), hybrid(Red, White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Polliwallop — {3}{G} Instant. Affinity for Frogs. Target creature you
/// control deals damage equal to twice its power to target creature you don't
/// control. (Damage is dealt by the spell rather than the creature.)
pub fn polliwallop() -> CardDefinition {
    CardDefinition {
        name: "Polliwallop",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Frog)
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            amount: Value::Times(
                Box::new(Value::PowerOf(Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                }))),
                Box::new(Value::Const(2)),
            ),
        },
        ..Default::default()
    }
}

/// Coiling Rebirth — {3}{B}{B} Sorcery. Gift a card. Return target creature card
/// from your graveyard to the battlefield. If the gift was promised and that
/// creature isn't legendary, also create a 1/1 token copy of it.
pub fn coiling_rebirth() -> CardDefinition {
    use crate::card::Gift;
    let reanimate = Effect::Move {
        what: Selector::TargetFiltered {
            slot: 0,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
        },
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        name: "Coiling Rebirth",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: reanimate.clone(),
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
                reanimate,
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::Target(0),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    non_legendary: true,
                    legendary: false,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Pearl of Wisdom — {2}{U} Sorcery. Costs {1} less if you control an Otter.
/// Draw two cards.
pub fn pearl_of_wisdom() -> CardDefinition {
    CardDefinition {
        name: "Pearl of Wisdom",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_control: vec![(
            SelectionRequirement::HasCreatureType(CreatureType::Otter)
                .and(SelectionRequirement::ControlledByYou),
            1,
        )],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Geistlight Snare — {2}{U} Instant. Costs {1} less if you control a Spirit
/// and {1} less if you control an enchantment. Counter target spell unless its
/// controller pays {3}.
pub fn geistlight_snare() -> CardDefinition {
    CardDefinition {
        name: "Geistlight Snare",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_control: vec![
            (
                SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                    .and(SelectionRequirement::ControlledByYou),
                1,
            ),
            (
                SelectionRequirement::HasCardType(CardType::Enchantment)
                    .and(SelectionRequirement::ControlledByYou),
                1,
            ),
        ],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(3)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Ride's End — {4}{W} Instant. Costs {3} less to cast if it targets a tapped
/// permanent. Exile target creature or Vehicle.
pub fn rides_end() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Ride's End",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::Tapped, 3)),
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            ),
        },
        ..Default::default()
    }
}

/// Nurturing Pixie — {W} 1/1 Faerie Rogue with flying. When it enters, return
/// up to one target non-Faerie, nonland permanent you control to its owner's
/// hand; if one was returned, put a +1/+1 counter on this creature.
pub fn nurturing_pixie() -> CardDefinition {
    CardDefinition {
        name: "Nurturing Pixie",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Bounce a nonland permanent you control to grow the Pixie".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Permanent
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::Nonland)
                            .and(SelectionRequirement::Not(Box::new(
                                SelectionRequirement::HasCreatureType(CreatureType::Faerie),
                            ))),
                    },
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Stab — {B} Instant. Target creature gets -2/-2 until end of turn.
pub fn stab() -> CardDefinition {
    CardDefinition {
        name: "Stab",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Slumbering Keepguard — {W} 1/1 Human Knight. Whenever an enchantment you
/// control enters, scry 1. {2}{W}: This creature gets +1/+1 until end of turn
/// for each enchantment you control.
pub fn slumbering_keepguard() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let enchant_count = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::Enchantment,
    };
    CardDefinition {
        name: "Slumbering Keepguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
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
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: enchant_count.clone(),
                toughness: enchant_count,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ruby, Daring Tracker — {R}{G} 1/2 Legendary Human Scout with haste. Whenever
/// Ruby attacks while you control a creature with power 4 or greater, it gets
/// +2/+2 until end of turn. {T}: Add {R} or {G}.
pub fn ruby_daring_tracker() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Ruby, Daring Tracker",
        cost: cost(&[r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtLeast(4)),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Red),
            crate::sets::tap_add(Color::Green),
        ],
        ..Default::default()
    }
}


/// Anoint with Affliction — {1}{B} Instant. Exile target creature with mana
/// value 3 or less. (The Corrupted "any creature if its controller has 3+
/// poison" rider is dropped; the base mode caps the target at MV 3.)
pub fn anoint_with_affliction() -> CardDefinition {
    CardDefinition {
        name: "Anoint with Affliction",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Wing It — {1}{W} Instant. Target creature gets +2/+2 until end of turn, gets
/// a flying counter, then scry 1.
pub fn wing_it() -> CardDefinition {
    CardDefinition {
        name: "Wing It",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Cackling Prowler — {3}{G} 4/3 Hyena Rogue. Ward {2}. Morbid — at the
/// beginning of your end step, if a creature died this turn, put a +1/+1
/// counter on it.
pub fn cackling_prowler() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Cackling Prowler",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hyena, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Glimmerlight — {2} Equipment. When it enters, create a 1/1 white Glimmer
/// enchantment creature token. Equipped creature gets +1/+1. Equip {1}.
pub fn glimmerlight() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, TokenDefinition};
    use crate::mana::Color;
    CardDefinition {
        name: "Glimmerlight",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Glimmer".into(),
                card_types: vec![CardType::Enchantment, CardType::Creature],
                colors: vec![Color::White],
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Demonic Ruckus — {1}{R} Aura. Enchanted creature gets +1/+1 and has menace
/// and trample. When this Aura is put into a graveyard from the battlefield,
/// draw a card. Plot {R}.
pub fn demonic_ruckus() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Demonic Ruckus",
        cost: cost(&[generic(1), r()]),
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
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Menace, Keyword::Trample],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        plot_cost: Some(cost(&[r()])),
        ..Default::default()
    }
}

/// Hugs, Grisly Guardian — {X}{R}{R}{G}{G} 5/5 Legendary Badger Warrior with
/// trample. When it enters, exile the top X cards of your library; you may play
/// them until your next end step. You may play an additional land each turn.
pub fn hugs_grisly_guardian() -> CardDefinition {
    use crate::effect::StaticEffect;
    use crate::mana::x;
    CardDefinition {
        name: "Hugs, Grisly Guardian",
        cost: cost(&[x(), r(), r(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![crate::card::StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::XFromCost,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Gloomfang Mauler — {5}{B}{B} 5/5 Nightmare. Swampcycling {2}. Backup 2.
pub fn gloomfang_mauler() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Gloomfang Mauler",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Swamp)],
        triggered_abilities: vec![crate::effect::shortcut::backup(2, vec![])],
        ..Default::default()
    }
}

/// Audacity — {G} Aura. Enchanted creature gets +2/+0 and has trample. When
/// this Aura is put into a graveyard from the battlefield, draw a card.
pub fn audacity() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Audacity",
        cost: cost(&[g()]),
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
            power: 2,
            toughness: 0,
            keywords: vec![Keyword::Trample],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Felonious Rage — {R} Instant. Target creature you control gets +2/+0 and
/// gains haste until end of turn. When that creature dies this turn, create a
/// 2/2 white and blue Detective creature token.
pub fn felonious_rage() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Felonious Rage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                slot: 0,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Detective".into(),
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White, Color::Blue],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Detective],
                            ..Default::default()
                        },
                        power: 2,
                        toughness: 2,
                        ..Default::default()
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Razorkin Hordecaller — {4}{R} 4/4 Human Clown Berserker with haste. Whenever
/// you attack, create a 1/1 red Gremlin creature token.
pub fn razorkin_hordecaller() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Razorkin Hordecaller",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Clown, CreatureType::Berserker],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::on_you_attack(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Gremlin".into(),
                card_types: vec![CardType::Creature],
                colors: vec![Color::Red],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Gremlin],
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

/// Goldvein Pick — {2} Equipment. Equipped creature gets +1/+1 and, whenever it
/// deals combat damage to a player, creates a Treasure token. Equip {1}.
pub fn goldvein_pick() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Goldvein Pick",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![],
            scale: None,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Tarkir: Dragonstorm + recent-set batch (claude/modern_decks) ─────────────

/// Boulderborn Dragon — {5} Artifact Dragon 3/3. Flying, vigilance; attacks →
/// surveil 1.
pub fn boulderborn_dragon() -> CardDefinition {
    CardDefinition {
        name: "Boulderborn Dragon",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Scales of Shale — {2}{B} Instant. Affinity for Lizards. Target creature gets
/// +2/+0 and gains lifelink and indestructible until end of turn.
pub fn scales_of_shale() -> CardDefinition {
    CardDefinition {
        name: "Scales of Shale",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Lizard)
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Sunset Strikemaster — {1}{R} 3/1 Human Monk. {T}: Add {R}. {2}{R}, {T},
/// Sacrifice this: it deals 6 damage to target creature with flying.
pub fn sunset_strikemaster() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Sunset Strikemaster",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                    ),
                    amount: Value::Const(6),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wardens of the Cycle — {1}{B}{G}{G} 3/4 Elf Warlock. Morbid — at your end
/// step, if a creature died this turn, gain 2 life, or draw a card and lose 1.
pub fn wardens_of_the_cycle() -> CardDefinition {
    CardDefinition {
        name: "Wardens of the Cycle",
        cost: cost(&[generic(1), b(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) },
                then: Box::new(Effect::ChooseMode(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
                    ]),
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Roiling Dragonstorm — {1}{U} Enchantment. ETB: draw two, then discard a
/// card. When a Dragon you control enters, return this to its owner's hand.
pub fn roiling_dragonstorm() -> CardDefinition {
    CardDefinition {
        name: "Roiling Dragonstorm",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// Stormcatch Mentor — {U}{R} 1/1 Otter Wizard. Haste, prowess; instant and
/// sorcery spells you cast cost {1} less.
pub fn stormcatch_mentor() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Stormcatch Mentor",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Gurmag Drowner — {3}{U} 2/4 Snake Wizard. Exploit; when it exploits a
/// creature, look at the top four cards, put one into your hand, the rest on
/// the bottom.
pub fn gurmag_drowner() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Drowner",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![crate::effect::shortcut::exploit(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Temur Battlecrier — {G}{U}{R} 4/3 Orc Ranger. Spells you cast cost {1} less
/// for each creature you control with power 4 or greater. (The "during your
/// turn" gate is approximated as always-on.)
pub fn temur_battlecrier() -> CardDefinition {
    CardDefinition {
        name: "Temur Battlecrier",
        cost: cost(&[g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Ranger],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        affinity_filter: Some(
            SelectionRequirement::Creature
                .and(SelectionRequirement::PowerAtMost(3).negate())
                .and(SelectionRequirement::ControlledByYou),
        ),
        ..Default::default()
    }
}

/// Nullpriest of Oblivion — {1}{B} 2/1 Vampire Cleric. Kicker {3}{B}. Lifelink,
/// menace. ETB, if kicked: return target creature card from your graveyard to
/// the battlefield.
pub fn nullpriest_of_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Nullpriest of Oblivion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink, Keyword::Menace, Keyword::Kicker(cost(&[generic(3), b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Ureni, the Song Unending — {5}{G}{U}{R} 10/10 Spirit Dragon. Flying,
/// protection from white and from black. ETB: deal X damage divided as you
/// choose among any number of target creatures/planeswalkers opponents control,
/// where X is the number of lands you control.
pub fn ureni_the_song_unending() -> CardDefinition {
    use crate::card::CardType as CT;
    CardDefinition {
        name: "Ureni, the Song Unending",
        cost: cost(&[generic(5), g(), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Dragon],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(crate::mana::Color::White),
            Keyword::Protection(crate::mana::Color::Black),
        ],
        triggered_abilities: vec![etb(Effect::DealDamageDivided {
            total: Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            ))),
            filter: (SelectionRequirement::Creature
                .or(SelectionRequirement::HasCardType(CT::Planeswalker)))
            .and(SelectionRequirement::ControlledByOpponent),
            max_targets: 10,
        })],
        ..Default::default()
    }
}

/// Elspeth, Storm Slayer — {3}{W}{W} Legendary Planeswalker. Tokens you create
/// are doubled. +1: make a 1/1 Soldier. 0: +1/+1 on each creature you control,
/// they gain flying until your next turn. −3: destroy target creature an
/// opponent controls with mana value 3 or greater.
pub fn elspeth_storm_slayer() -> CardDefinition {
    use crate::card::{
        LoyaltyAbility, PlaneswalkerSubtype, StaticAbility, StaticEffect, TokenDefinition,
    };
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Elspeth, Storm Slayer",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth],
            ..Default::default()
        },
        base_loyalty: 5,
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, twice that many are created instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: soldier },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        keyword: Keyword::Flying,
                        duration: Duration::UntilNextTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent)
                            .and(SelectionRequirement::ManaValueAtLeast(3)),
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Betor, Kin to All — {2}{W}{B}{G} 5/7 Spirit Dragon. Flying. At your end step:
/// if your creatures' total toughness ≥10 draw a card; then ≥20 untap each
/// creature you control; then ≥40 each opponent loses half their life, rounded
/// up.
pub fn betor_kin_to_all() -> CardDefinition {
    CardDefinition {
        name: "Betor, Kin to All",
        cost: cost(&[generic(2), w(), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(10)),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(20)),
                    then: Box::new(Effect::Untap {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        up_to: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(40)),
                    then: Box::new(Effect::LoseHalfLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        rounded_up: true,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Mistmoon Griffin — {3}{W} 2/2 Griffin. Flying. When it dies, return the top
/// creature card of your graveyard to the battlefield.
pub fn mistmoon_griffin() -> CardDefinition {
    CardDefinition {
        name: "Mistmoon Griffin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::ReturnTopCreatureFromGraveyard {
            who: PlayerRef::You,
        })],
        ..Default::default()
    }
}

/// Dalek Squadron — {2}{B} 3/3 Artifact Dalek. Menace, myriad.
pub fn dalek_squadron() -> CardDefinition {
    CardDefinition {
        name: "Dalek Squadron",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dalek], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Myriad,
        }],
        ..Default::default()
    }
}

/// Perennation — {3}{W}{B}{G} Sorcery. Return target permanent card from your
/// graveyard to the battlefield with a hexproof counter and an indestructible
/// counter on it.
pub fn perennation() -> CardDefinition {
    CardDefinition {
        name: "Perennation",
        cost: cost(&[generic(3), w(), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Permanent
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Karakyk Guardian — {3}{G}{U}{R} 6/5 Dragon. Flying, vigilance, trample.
/// (Its conditional hexproof-while-it-hasn't-dealt-damage rider is omitted —
/// no lifetime damage-dealt tracking yet.)
pub fn karakyk_guardian() -> CardDefinition {
    CardDefinition {
        name: "Karakyk Guardian",
        cost: cost(&[generic(3), g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample],
        ..Default::default()
    }
}

/// Sarkhan, Soul Aflame — {1}{U}{R} 2/4 Human Shaman. Dragon spells you cast
/// cost {1} less. Whenever a Dragon you control enters, you may have Sarkhan
/// become a copy of it until end of turn. (The copy keeps the Dragon's name —
/// the printed "name stays Sarkhan" override is approximated.)
pub fn sarkhan_soul_aflame() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Sarkhan, Soul Aflame",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Dragon spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                }),
            effect: Effect::MayDo {
                description: "have Sarkhan become a copy of that Dragon until end of turn".into(),
                body: Box::new(Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::TriggerSource,
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Recent-set batch 2 (claude/modern_decks) ─────────────────────────────────

/// Skirmish Rhino — {W}{B}{G} 3/4 Rhino. Trample. ETB: each opponent loses 2
/// life and you gain 2 life.
pub fn skirmish_rhino() -> CardDefinition {
    CardDefinition {
        name: "Skirmish Rhino",
        cost: cost(&[w(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rhino], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Rabid Gnaw — {1}{R} Instant. Target creature you control gets +1/+0 until end
/// of turn, then deals damage equal to its power to target creature you don't
/// control.
pub fn rabid_gnaw() -> CardDefinition {
    CardDefinition {
        name: "Rabid Gnaw",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Reckless Lackey — {R} 1/2 Goblin Pirate. First strike, haste. {2}{R},
/// Sacrifice this: draw a card and create a Treasure token.
pub fn reckless_lackey() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Reckless Lackey",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lunar Convocation — {W}{B} Enchantment. At your end step: if you gained life
/// this turn, each opponent loses 1 life; if you also lost life this turn,
/// create a 1/1 black Bat with flying.
pub fn lunar_convocation() -> CardDefinition {
    use crate::card::TokenDefinition;
    let bat = TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Lunar Convocation",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::LifeGainedThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    },
                    then: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::All(vec![
                        Predicate::LifeGainedThisTurnAtLeast {
                            who: PlayerRef::You,
                            at_least: Value::Const(1),
                        },
                        Predicate::PlayerLostLifeThisTurn { who: PlayerRef::You },
                    ]),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: bat,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dazzling Denial — {1}{U} Instant. Counter target spell unless its controller
/// pays {2} — or {4} instead if you control a Bird.
pub fn dazzling_denial() -> CardDefinition {
    CardDefinition {
        name: "Dazzling Denial",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Bird)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[generic(4)]),
                exile: false,
                extra_generic: None,
            }),
            else_: Box::new(Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            }),
        },
        ..Default::default()
    }
}

/// Cori Mountain Monastery — Land. Enters tapped unless you control a Plains or
/// an Island. {T}: Add {R}. {3}{R}, {T}: exile the top card of your library;
/// you may play it until the end of your next turn.
pub fn cori_mountain_monastery() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Cori Mountain Monastery",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![land_tapped_unless(LandType::Plains, LandType::Island)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), r()]),
                tap_cost: true,
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mistrise Village — Land. Enters tapped unless you control a Mountain or a
/// Forest. {T}: Add {U}. {U}, {T}: your spells can't be countered this turn.
/// (The printed "next spell" scope is approximated as all your spells.)
pub fn mistrise_village() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Mistrise Village",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![land_tapped_unless(LandType::Mountain, LandType::Forest)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(crate::mana::Color::Blue, Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                effect: Effect::GrantSpellsUncounterableThisTurn { who: Selector::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// "Enters tapped unless you control a land of `type_a` or `type_b`" — the
/// check-land ETB conditional reused by recent dual-color utility lands.
fn land_tapped_unless(type_a: crate::card::LandType, type_b: crate::card::LandType) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::HasLandType(type_a)
                        .or(SelectionRequirement::HasLandType(type_b))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::Tap { what: Selector::This }),
        },
    }
}

/// Bloodletter of Aclazotz — {1}{B}{B}{B} 2/4 Vampire Demon. Flying. If an
/// opponent would lose life during your turn, they lose twice that much instead.
pub fn bloodletter_of_aclazotz() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Bloodletter of Aclazotz",
        cost: cost(&[generic(1), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Demon],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would lose life during your turn, they lose twice that much life instead.",
            effect: StaticEffect::OpponentLifeLossDoubledDuringYourTurn,
        }],
        ..Default::default()
    }
}

/// Touch the Spirit Realm — {2}{W} Enchantment. ETB: exile up to one target
/// artifact or creature until this leaves. (The Channel discard-mode is omitted.)
pub fn touch_the_spirit_realm() -> CardDefinition {
    use crate::card::{CardType as CT, ExileReturnZone};
    CardDefinition {
        name: "Touch the Spirit Realm",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::HasCardType(CT::Artifact)),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Sonar Strike — {1}{W} Instant. Deals 4 damage to target attacking, blocking,
/// or tapped creature; gain 3 life if you control a Bat.
pub fn sonar_strike() -> CardDefinition {
    CardDefinition {
        name: "Sonar Strike",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::IsAttacking
                            .or(SelectionRequirement::IsBlocking)
                            .or(SelectionRequirement::Tapped),
                    ),
                ),
                amount: Value::Const(4),
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Bat)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Aerie Auxiliary — {3}{W} 3/3 Bird Soldier. Flying. ETB: support 2 (put a
/// +1/+1 counter on each of up to two other target creatures).
pub fn aerie_auxiliary() -> CardDefinition {
    CardDefinition {
        name: "Aerie Auxiliary",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(crate::effect::shortcut::support(2))],
        ..Default::default()
    }
}

/// Loran's Escape — {W} Instant. Target artifact or creature gains hexproof and
/// indestructible until end of turn. Scry 1.
pub fn lorans_escape() -> CardDefinition {
    use crate::card::CardType as CT;
    CardDefinition {
        name: "Loran's Escape",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::HasCardType(CT::Artifact)),
                ),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Dauntless Veteran — {1}{W}{W} 2/2 Human Soldier. Whenever it attacks,
/// creatures you control get +1/+1 until end of turn.
pub fn dauntless_veteran() -> CardDefinition {
    CardDefinition {
        name: "Dauntless Veteran",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Spectral Denial — {X}{U} Instant. Costs {1} less for each creature you
/// control with power 4 or greater. Counter target spell unless its controller
/// pays {X}.
pub fn spectral_denial() -> CardDefinition {
    CardDefinition {
        name: "Spectral Denial",
        cost: cost(&[crate::mana::x(), u()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::Creature
                .and(SelectionRequirement::PowerAtMost(3).negate())
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::CounterUnlessPaid {
            what: crate::effect::shortcut::target(),
            mana_cost: crate::mana::ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::XFromCost),
        },
        ..Default::default()
    }
}

/// Glistener Seer — {U} 0/3 Phyrexian Advisor. Enters with three oil counters.
/// {T}, Remove an oil counter: scry 1.
pub fn glistener_seer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Glistener Seer",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Advisor],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        enters_with_counters: Some((CounterType::Oil, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vengeful Bloodwitch — {1}{B} 1/1 Vampire Warlock. Whenever this or another
/// creature you control dies, an opponent loses 1 life and you gain 1.
pub fn vengeful_bloodwitch() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Bloodwitch",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Hulking Raptor — {2}{G}{G} 5/3 Dinosaur. Ward {2}. At your first main phase,
/// add {G}{G}.
pub fn hulking_raptor() -> CardDefinition {
    use crate::card::WardCost;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Hulking Raptor",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::PreCombatMain),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(crate::mana::Color::Green, Value::Const(2)),
            },
        }],
        ..Default::default()
    }
}

// ── DFT "Start your engines!" / speed (CR 702.179) ──────────────────────────

/// Nesting Bot — {W} 1/1 Robot artifact creature. Start your engines! When it
/// dies, create a 1/1 colorless Servo artifact creature token. Max speed — it
/// gets +1/+0.
pub fn nesting_bot() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, TokenDefinition};
    CardDefinition {
        name: "Nesting Bot",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Servo".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Artifact, CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Servo], ..Default::default() },
                ..Default::default()
            },
        })],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Nesting Bot gets +1/+0.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Burnout Bashtronaut — {R} 1/1 Goblin Warrior. Menace, Start your engines!
/// {2}: it gets +1/+0 until end of turn. Max speed — it has double strike.
pub fn burnout_bashtronaut() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Burnout Bashtronaut",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Burnout Bashtronaut has double strike.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::DoubleStrike],
            },
        }],
        ..Default::default()
    }
}

/// Swiftwing Assailant — {3}{W} 3/3 Bird Warrior. Flying, Start your engines!
/// Max speed — it gets +0/+1 and has vigilance.
pub fn swiftwing_assailant() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Swiftwing Assailant",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Swiftwing Assailant gets +0/+1 and has vigilance.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 1,
                keywords: vec![Keyword::Vigilance],
            },
        }],
        ..Default::default()
    }
}

/// Risen Necroregent — {4}{B} 5/4 Zombie Cat Knight. Start your engines! Max
/// speed — at the beginning of your end step, create a 2/2 black Zombie token.
pub fn risen_necroregent() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Risen Necroregent",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cat, CreatureType::Knight],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Embalmed Ascendant — {1}{W}{B} 1/2 Zombie. Start your engines! When it
/// enters, create a 2/2 black Zombie token. Max speed — whenever a creature you
/// control dies, each opponent loses 1 life and you gain 1 life.
pub fn embalmed_ascendant() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Embalmed Ascendant",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
                    ..Default::default()
                },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Walking Sarcophagus — {2} 2/1 Zombie Cat artifact creature. Start your
/// engines! Max speed — it gets +1/+2.
pub fn walking_sarcophagus() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Walking Sarcophagus",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Walking Sarcophagus gets +1/+2.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 1,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Streaking Oilgorger — {4}{B} 3/3 Vampire. Flying, haste, Start your engines!
/// Max speed — it has lifelink.
pub fn streaking_oilgorger() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Streaking Oilgorger",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Streaking Oilgorger has lifelink.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Goblin Surveyor — {2}{R} 3/2 Goblin Scout. Trample, Start your engines! Max
/// speed — {3}, Exile this card from your graveyard: Draw a card.
pub fn goblin_surveyor() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Goblin Surveyor",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            from_graveyard: true,
            exile_self_cost: true,
            condition: Some(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gastal Thrillseeker — {B}{R} 2/3 Lizard Berserker. Start your engines! When
/// it enters, deal 1 damage to each opponent (printed "target opponent",
/// 1v1-faithful) and you gain 1 life. Max speed — it has deathtouch and haste.
pub fn gastal_thrillseeker() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Gastal Thrillseeker",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Gastal Thrillseeker has deathtouch and haste.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Deathtouch, Keyword::Haste],
            },
        }],
        ..Default::default()
    }
}

// ── Recover (Coldsnap, CR 702.58) ───────────────────────────────────────────

/// Grim Harvest — {1}{B} Instant. Return target creature card from your
/// graveyard to your hand. Recover {2}{B}. (The main effect's graveyard zone
/// filter is dropped per the Disentomb convention.)
pub fn grim_harvest() -> CardDefinition {
    CardDefinition {
        name: "Grim Harvest",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        triggered_abilities: vec![recover(cost(&[generic(2), b()]))],
        ..Default::default()
    }
}

/// Sun's Bounty — {1}{W} Instant. You gain 4 life. Recover {1}{W}.
pub fn suns_bounty() -> CardDefinition {
    CardDefinition {
        name: "Sun's Bounty",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        triggered_abilities: vec![recover(cost(&[generic(1), w()]))],
        ..Default::default()
    }
}

/// Icefall — {2}{R}{R} Sorcery. Destroy target artifact or land. Recover {R}{R}.
pub fn icefall() -> CardDefinition {
    CardDefinition {
        name: "Icefall",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Land),
            ),
        },
        triggered_abilities: vec![recover(cost(&[r(), r()]))],
        ..Default::default()
    }
}

/// Resize — {1}{G} Instant. Target creature gets +3/+3 until end of turn.
/// Recover {1}{G}.
pub fn resize() -> CardDefinition {
    CardDefinition {
        name: "Resize",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(3),
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![recover(cost(&[generic(1), g()]))],
        ..Default::default()
    }
}

// ── Ripple (Coldsnap, CR 702.20) ────────────────────────────────────────────
// `shortcut::ripple(n)` wires the "when you cast this spell" trigger that
// reveals the top N, free-casts same-named copies, and bottoms the rest.

/// Surging Flame — {1}{R} Sorcery. Ripple 4. Deals 2 damage to any target.
pub fn surging_flame() -> CardDefinition {
    use crate::effect::shortcut::{deal, ripple, target_any};
    CardDefinition {
        name: "Surging Flame",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: deal(2, target_any()),
        triggered_abilities: vec![ripple(4)],
        ..Default::default()
    }
}

/// Surging Dementia — {1}{B} Sorcery. Ripple 4. Each player discards a card.
pub fn surging_dementia() -> CardDefinition {
    use crate::effect::shortcut::ripple;
    CardDefinition {
        name: "Surging Dementia",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(1),
            random: false,
        },
        triggered_abilities: vec![ripple(4)],
        ..Default::default()
    }
}

/// Surging Might — {G} Instant. Ripple 4. Target creature gets +1/+1 and gains
/// trample until end of turn.
pub fn surging_might() -> CardDefinition {
    use crate::effect::shortcut::{ripple, target_filtered};
    CardDefinition {
        name: "Surging Might",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![ripple(4)],
        ..Default::default()
    }
}

/// Surging Sentinels — {3}{W} 3/1 Spirit. Ripple 4. (Its "gains protection from
/// black when you cast a white spell" rider is omitted.)
pub fn surging_sentinels() -> CardDefinition {
    use crate::effect::shortcut::ripple;
    CardDefinition {
        name: "Surging Sentinels",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![ripple(4)],
        ..Default::default()
    }
}

/// Surging Æther — {2}{U} Instant. Ripple 4. Return target creature to its
/// owner's hand. (Printed "target spell or permanent"; modeled as a creature.)
pub fn surging_aether() -> CardDefinition {
    use crate::effect::shortcut::{ripple, target_filtered};
    CardDefinition {
        name: "Surging Æther",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(target_filtered(
                SelectionRequirement::Creature,
            )))),
        },
        triggered_abilities: vec![ripple(4)],
        ..Default::default()
    }
}

// ── Simple staples (existing primitives) ─────────────────────────────────────

/// Moment of Craving — {1}{B} Instant. Target creature gets -2/-2; you gain 2.
pub fn moment_of_craving() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Moment of Craving",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Kindled Fury — {R} Instant. Target creature gets +1/+0 and gains first
/// strike until end of turn.
pub fn kindled_fury() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Kindled Fury",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Brute Strength — {1}{R} Instant. Target creature gets +3/+1 and gains
/// trample until end of turn.
pub fn brute_strength() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Brute Strength",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Gather Courage — {G} Instant. Convoke. Target creature gets +2/+2 until end
/// of turn.
pub fn gather_courage() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Gather Courage",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Bond Beetle — {G} 0/1 Insect. When it enters, put a +1/+1 counter on target
/// creature.
pub fn bond_beetle() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Bond Beetle",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Fleeting Distraction — {U} Instant. Target creature gets -1/-0 until end of
/// turn. Draw a card.
pub fn fleeting_distraction() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Fleeting Distraction",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Mistral Charge — {1}{U} Instant. Target creature gets +1/+1 and gains flying
/// until end of turn.
pub fn mistral_charge() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Mistral Charge",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Run Amok — {2}{R} Instant. Target attacking creature gets +3/+3 and gains
/// trample until end of turn. (The "attacking" target restriction is relaxed
/// to any creature.)
pub fn run_amok() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Run Amok",
        cost: cost(&[generic(2), r()]),
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
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Unearth (Shards of Alara, CR 702.84) ────────────────────────────────────
// `shortcut::unearth(cost)` builds the sorcery-speed graveyard ability that
// returns the card with haste and schedules an end-step exile.

/// Viscera Dragger — {4}{B} 3/2 Zombie Warrior. Unearth {1}{B}.
pub fn viscera_dragger() -> CardDefinition {
    use crate::effect::shortcut::unearth;
    CardDefinition {
        name: "Viscera Dragger",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![unearth(cost(&[generic(1), b()]))],
        ..Default::default()
    }
}

/// Skeletal Kathari — {3}{B} 2/1 Bird Skeleton. Flying. Unearth {2}{B}.
pub fn skeletal_kathari() -> CardDefinition {
    use crate::effect::shortcut::unearth;
    CardDefinition {
        name: "Skeletal Kathari",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![unearth(cost(&[generic(2), b()]))],
        ..Default::default()
    }
}

/// Rotting Rats — {1}{B} 1/1 Zombie Rat. ETB each player discards a card.
/// Unearth {1}{B}.
pub fn rotting_rats() -> CardDefinition {
    use crate::effect::shortcut::unearth;
    CardDefinition {
        name: "Rotting Rats",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(1),
                random: false,
            },
        }],
        activated_abilities: vec![unearth(cost(&[generic(1), b()]))],
        ..Default::default()
    }
}

/// Fledgling Mawcor — {3}{U} 2/2 Beast. Flying. {T}: deal 1 damage to any
/// target. Unearth {5}{U}.
pub fn fledgling_mawcor() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::{deal, target_any, unearth};
    CardDefinition {
        name: "Fledgling Mawcor",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: deal(1, target_any()), ..Default::default() },
            unearth(cost(&[generic(5), u()])),
        ],
        ..Default::default()
    }
}

// ── More recent-set staples ─────────────────────────────────────────────────

/// Bloodthirsty Conqueror — {3}{B}{B} 5/5 Vampire Knight. Flying, deathtouch.
/// Whenever an opponent loses life, you gain that much life.
pub fn bloodthirsty_conqueror() -> CardDefinition {
    CardDefinition {
        name: "Bloodthirsty Conqueror",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..Default::default()
    }
}

/// Razorkin Needlehead — {R}{R} 2/2 Human Assassin. First strike during your
/// turn. Whenever an opponent draws a card, it deals 1 damage to them.
pub fn razorkin_needlehead() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Razorkin Needlehead",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Razorkin Needlehead has first strike during your turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Savor — {1}{B} Instant. Target creature gets -2/-2 until end of turn; create
/// a Food token.
pub fn savor() -> CardDefinition {
    CardDefinition {
        name: "Savor",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::food_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Spinewoods Armadillo — {4}{G}{G} 7/7 Armadillo. Reach, ward {3}. {1}{G},
/// Discard this card: Search your library for a basic land or Desert card, put
/// it into your hand, then shuffle. You gain 3 life.
pub fn spinewoods_armadillo() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType, WardCost};
    CardDefinition {
        name: "Spinewoods Armadillo",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Armadillo], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Reach, Keyword::Ward(WardCost::generic(3))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand
                        .or(SelectionRequirement::HasLandType(LandType::Desert)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Screaming Nemesis — {2}{R} 3/3 Spirit. Haste. Whenever it's dealt damage, it
/// deals that much damage to any target; if a player is dealt damage this way,
/// they can't gain life for the rest of the game.
pub fn screaming_nemesis() -> CardDefinition {
    CardDefinition {
        name: "Screaming Nemesis",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Any),
                    amount: Value::TriggerEventAmount,
                },
                // No-op unless the target was a player (CR 119.7 rest-of-game lock).
                Effect::LifeGainLockGame { who: Selector::Target(0) },
            ]),
        }],
        ..Default::default()
    }
}

/// Goblin Boarders — {2}{R} 3/2 Goblin Pirate. Raid — enters with a +1/+1
/// counter if you attacked this turn.
pub fn goblin_boarders() -> CardDefinition {
    CardDefinition {
        name: "Goblin Boarders",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Cogwork Wrestler — {U} 1/2 Gnome artifact creature. Flash. When it enters,
/// target creature an opponent controls gets -2/-0 until end of turn.
pub fn cogwork_wrestler() -> CardDefinition {
    CardDefinition {
        name: "Cogwork Wrestler",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Crocodile of the Crossing — {3}{G} 5/4 Crocodile. Haste. When it enters, put
/// a -1/-1 counter on target creature you control.
pub fn crocodile_of_the_crossing() -> CardDefinition {
    CardDefinition {
        name: "Crocodile of the Crossing",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Crocodile], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::MinusOneMinusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Topiary Stomper — {1}{G}{G} 4/4 Plant Dinosaur. Vigilance. Can't attack or
/// block unless you control seven or more lands. When it enters, search your
/// library for a basic land and put it onto the battlefield tapped.
pub fn topiary_stomper() -> CardDefinition {
    CardDefinition {
        name: "Topiary Stomper",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::CantAttackOrBlockUnlessYouControlCount {
                filter: Box::new(SelectionRequirement::Land),
                min: 7,
                attack_only: false,
                block_only: false,
            },
        ],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        ..Default::default()
    }
}

/// Cache Grab — {1}{G} Instant. Mill four, then you may put a permanent card
/// milled this way into your hand. If you control a Squirrel, create a Food.
/// (The "returned a Squirrel this way" half of the Food trigger is approximated
/// to controlling one.)
pub fn cache_grab() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Cache Grab",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::MillThenToHand {
                amount: Value::Const(4),
                filter: SelectionRequirement::PermanentCard,
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Squirrel),
                }),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::food_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Lumbering Worldwagon — {2}{G} Vehicle `*`/4. Power = lands you control.
/// Whenever it enters or attacks, you may fetch a basic land tapped. Crew 4.
pub fn lumbering_worldwagon() -> CardDefinition {
    use crate::card::{ArtifactSubtype, DynamicPt};
    let fetch = || Effect::MayDo {
        description: "Search for a basic land, put it onto the battlefield tapped".into(),
        body: Box::new(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        }),
    };
    CardDefinition {
        name: "Lumbering Worldwagon",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 0,
        toughness: 4,
        dynamic_pt: Some(DynamicPt::LandsControlledPower { base_p: 0, base_t: 4 }),
        keywords: vec![Keyword::Crew(4)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: fetch(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: fetch(),
            },
        ],
        ..Default::default()
    }
}

/// Bakersbane Duo — {1}{G} 2/2 Squirrel Raccoon. When it enters, create a Food
/// token. Whenever you expend 4, it gets +1/+1 until end of turn.
pub fn bakersbane_duo() -> CardDefinition {
    CardDefinition {
        name: "Bakersbane Duo",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Raccoon],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::food_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                    .with_filter(Predicate::ExpendReached(4)),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Spire Mangler — {2}{B} 2/1 Insect. Flash, flying. When it enters, target
/// creature you control with flying gets +2/+0 until end of turn.
pub fn spire_mangler() -> CardDefinition {
    CardDefinition {
        name: "Spire Mangler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Palace Familiar — {1}{U} 1/1 Bird. Flying; when it dies, draw a card.
pub fn palace_familiar() -> CardDefinition {
    CardDefinition {
        name: "Palace Familiar",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Symbiotic Elf — {3}{G} 2/2 Elf. When it dies, create two 1/1 green Insect
/// creature tokens.
pub fn symbiotic_elf() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let insect = TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Symbiotic Elf",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: insect,
        })],
        ..Default::default()
    }
}

/// Bear's Companion — {2}{G}{U}{R} 2/2 Human Warrior. When it enters, create a
/// 4/4 green Bear creature token.
pub fn bears_companion() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let bear = TokenDefinition {
        name: "Bear".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Bear's Companion",
        cost: cost(&[generic(2), g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: bear,
        })],
        ..Default::default()
    }
}

/// Grasping Thrull — {3}{W}{B} 3/3 Thrull. Flying; when it enters, it deals 2
/// damage to each opponent and you gain 2 life.
pub fn grasping_thrull() -> CardDefinition {
    CardDefinition {
        name: "Grasping Thrull",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thrull], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Hero of Precinct One — {1}{W} 2/2 Human Warrior. Whenever you cast a
/// multicolored spell, create a 1/1 white Human creature token.
pub fn hero_of_precinct_one() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let human = TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Hero of Precinct One",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Multicolored,
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: human,
            },
        }],
        ..Default::default()
    }
}

/// Havoc Devils — {2}{R}{R} 4/3 Devil with trample.
pub fn havoc_devils() -> CardDefinition {
    CardDefinition {
        name: "Havoc Devils",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Hollow Dogs — {4}{B} 3/3 Phyrexian Zombie Dog. Whenever it attacks, it gets
/// +2/+0 until end of turn.
pub fn hollow_dogs() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Hollow Dogs",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Argothian Enchantress — {1}{G} 0/1 Human Druid. Shroud; whenever you cast an
/// enchantment spell, draw a card.
pub fn argothian_enchantress() -> CardDefinition {
    CardDefinition {
        name: "Argothian Enchantress",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Shroud],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Enchantment),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Patrol Hound — {1}{W} 2/2 Dog. "Discard a card: This creature gains first
/// strike until end of turn."
pub fn patrol_hound() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Patrol Hound",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((SelectionRequirement::Any, 1)),
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

/// Canyon Wildcat — {1}{R} 2/1 Cat with mountainwalk.
pub fn canyon_wildcat() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Canyon Wildcat",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..Default::default()
    }
}

/// Squirrelanoids — {B} 1/1 Squirrel Mutant with deathtouch.
pub fn squirrelanoids() -> CardDefinition {
    CardDefinition {
        name: "Squirrelanoids",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Mutant],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Vile Deacon — {2}{B}{B} 2/2 Human Cleric. Whenever it attacks, it gets +X/+X
/// until end of turn, where X is the number of Clerics on the battlefield.
pub fn vile_deacon() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    let clerics = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::Creature)),
        filter: SelectionRequirement::HasCreatureType(CreatureType::Cleric),
    };
    CardDefinition {
        name: "Vile Deacon",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: clerics.clone(),
            toughness: clerics,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Mischievous Mystic — {1}{U} 2/1 Human Wizard. Flying; whenever you draw your
/// second card each turn, create a 1/1 blue Faerie creature token with flying.
pub fn mischievous_mystic() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let faerie = TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Mischievous Mystic",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::ValueEquals(
                    Value::CardsDrawnThisTurn(PlayerRef::You),
                    Value::Const(2),
                ))
                // CR 603.3d — fire once even when a multi-card draw leaves the
                // running count at 2 for several CardDrawn events at once.
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: faerie,
            },
        }],
        ..Default::default()
    }
}

/// Dawn's Light Archer — {2}{G} 4/2 Elf Archer with flash and reach.
pub fn dawns_light_archer() -> CardDefinition {
    CardDefinition {
        name: "Dawn's Light Archer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Archer],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Reach],
        ..Default::default()
    }
}

/// Plumeveil — {W/U}{W/U}{W/U} 4/4 Elemental with flash, defender, and flying.
pub fn plumeveil() -> CardDefinition {
    use crate::mana::{hybrid, Color};
    CardDefinition {
        name: "Plumeveil",
        cost: cost(&[
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flash, Keyword::Defender, Keyword::Flying],
        ..Default::default()
    }
}

/// Rooftop Assassin — {3}{B} 2/2 Vampire Assassin. Flash, flying, lifelink. When
/// it enters, destroy target creature an opponent controls that was dealt
/// damage this turn.
pub fn rooftop_assassin() -> CardDefinition {
    CardDefinition {
        name: "Rooftop Assassin",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent)
                    .and(SelectionRequirement::DealtDamageThisTurn),
            ),
        })],
        ..Default::default()
    }
}

/// Spellgorger Barbarian — {3}{R} 3/1 Human Nightmare Barbarian. When it enters,
/// discard a card at random. When it leaves the battlefield, draw a card.
pub fn spellgorger_barbarian() -> CardDefinition {
    CardDefinition {
        name: "Spellgorger Barbarian",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Nightmare,
                CreatureType::Barbarian,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: true,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Bog Gnarr — {4}{G} 2/2 Beast. Whenever a player casts a black spell, it gets
/// +2/+2 until end of turn.
pub fn bog_gnarr() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Bog Gnarr",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasColor(Color::Black),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Elf Replica — {3} 2/2 artifact Elf. "{1}{G}, Sacrifice this creature:
/// Destroy target enchantment."
pub fn elf_replica() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Elf Replica",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::HasCardType(CardType::Enchantment)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seismic Mage — {3}{R} 1/1 Human Spellshaper. "{2}{R}, {T}, Discard a card:
/// Destroy target land."
pub fn seismic_mage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Seismic Mage",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Spellshaper],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::Land),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Etched Oracle — {4} 0/0 artifact Wizard. Sunburst; "{1}, Remove four +1/+1
/// counters from this creature: Target player draws three cards."
pub fn etched_oracle() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Etched Oracle",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 4)),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skyreach Manta — {5} 0/0 artifact Fish. Sunburst; flying.
pub fn skyreach_manta() -> CardDefinition {
    CardDefinition {
        name: "Skyreach Manta",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        ..Default::default()
    }
}

/// Phyrexian Digester — {3} 2/1 artifact Phyrexian Construct with infect.
pub fn phyrexian_digester() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Digester",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Infect],
        ..Default::default()
    }
}

/// Blackcleave Goblin — {3}{B} 2/1 Phyrexian Goblin Zombie with haste and infect.
pub fn blackcleave_goblin() -> CardDefinition {
    CardDefinition {
        name: "Blackcleave Goblin",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin, CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Infect],
        ..Default::default()
    }
}

/// Essence Depleter — {2}{B} 2/3 Eldrazi Drone. Devoid; "{1}{C}: Target opponent
/// loses 1 life and you gain 1 life."
pub fn essence_depleter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Essence Depleter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), colorless(1)]),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stormclaw Rager — {1}{B}{R} 2/2 Ogre Warrior. "{1}, Sacrifice another creature
/// or artifact: Put a +1/+1 counter on this creature and draw a card. Activate
/// only as a sorcery."
pub fn stormclaw_rager() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Stormclaw Rager",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                1,
            )),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wave Elemental — {2}{U}{U} 2/3 Elemental. "{U}, {T}, Sacrifice this creature:
/// Tap up to three target creatures without flying."
pub fn wave_elemental() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Wave Elemental",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::TapUpToValue {
                count: Value::Const(3),
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::Not(Box::new(SelectionRequirement::HasKeyword(
                        Keyword::Flying,
                    )))),
                skip_untap: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shipwreck Moray — {3}{U} 0/5 Fish. When it enters, you get four energy. "Pay
/// {E}: This creature gets +2/-2 until end of turn."
pub fn shipwreck_moray() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Shipwreck Moray",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        power: 0,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(4)))],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 1,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Argothian Sprite — {1}{G} 2/2 Faerie. Can't be blocked by artifact creatures;
/// "{7}: Put two +1/+1 counters on this creature."
pub fn argothian_sprite() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Argothian Sprite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(SelectionRequirement::HasCardType(
            CardType::Artifact,
        )))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nadier's Nightblade — {2}{B} 1/3 Elf Warrior. Whenever a token you control
/// leaves the battlefield, each opponent loses 1 life and you gain 1 life.
pub fn nadiers_nightblade() -> CardDefinition {
    CardDefinition {
        name: "Nadier's Nightblade",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::IsToken,
                }),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Gnarlroot Pallbearer — {4}{G}{G} 5/5 Treefolk Druid. Trample; when it enters,
/// target creature gets +X/+X until end of turn, where X is the number of
/// creature cards in your graveyard.
pub fn gnarlroot_pallbearer() -> CardDefinition {
    let gy_creatures = Value::CardsInGraveyardMatching {
        who: PlayerRef::You,
        filter: SelectionRequirement::Creature,
    };
    CardDefinition {
        name: "Gnarlroot Pallbearer",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: gy_creatures.clone(),
            toughness: gy_creatures,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Illusionary Servant — {1}{U}{U} 3/4 Illusion. Flying; when it becomes the
/// target of a spell or ability, sacrifice it.
pub fn illusionary_servant() -> CardDefinition {
    CardDefinition {
        name: "Illusionary Servant",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Illusion], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Graveyard },
        }],
        ..Default::default()
    }
}

/// Bounding Wolf — {2}{G} 3/2 Wolf with flash and reach.
pub fn bounding_wolf() -> CardDefinition {
    CardDefinition {
        name: "Bounding Wolf",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Reach],
        ..Default::default()
    }
}

/// Goblin Sky Raider — {2}{R} 1/2 Goblin Warrior with flying.
pub fn goblin_sky_raider() -> CardDefinition {
    CardDefinition {
        name: "Goblin Sky Raider",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Glowing Anemone — {3}{U} 1/3 Jellyfish Beast. When it enters, you may return
/// target land to its owner's hand.
pub fn glowing_anemone() -> CardDefinition {
    CardDefinition {
        name: "Glowing Anemone",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return target land to its owner's hand".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(SelectionRequirement::Land),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..Default::default()
    }
}

/// Contraband Kingpin — {U}{B} 1/4 Aetherborn Rogue. Lifelink; whenever an
/// artifact you control enters, scry 1.
pub fn contraband_kingpin() -> CardDefinition {
    CardDefinition {
        name: "Contraband Kingpin",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Aetherborn, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                },
            ),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Kingpin's Enforcers — {2}{B} 2/3 Human Villain. Lifelink; "{2}{B}, Sacrifice
/// an artifact or creature: Draw a card."
pub fn kingpins_enforcers() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Kingpin's Enforcers",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Villain],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                1,
            )),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goldmaw Champion — {2}{W} 2/3 Dwarf Warrior. Boast — {1}{W}: Tap target
/// creature.
pub fn goldmaw_champion() -> CardDefinition {
    use crate::effect::shortcut::boast;
    CardDefinition {
        name: "Goldmaw Champion",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![boast(
            cost(&[generic(1), w()]),
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
        )],
        ..Default::default()
    }
}

/// Gold Myr — {2} 1/1 artifact Myr. "{T}: Add {W}."
pub fn gold_myr() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Gold Myr",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![crate::sets::tap_add(Color::White)],
        ..Default::default()
    }
}

/// Drumhunter — {3}{G} 2/2 Human Druid Warrior. At the beginning of your end
/// step, if you control a creature with power 5 or greater, you may draw a card.
/// "{T}: Add {C}."
pub fn drumhunter() -> CardDefinition {
    CardDefinition {
        name: "Drumhunter",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(5)),
            })),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..Default::default()
    }
}


/// Roc of Kher Ridges — {3}{R} 3/3 Bird with flying.
pub fn roc_of_kher_ridges() -> CardDefinition {
    CardDefinition {
        name: "Roc of Kher Ridges",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Minotaur Aggressor — {6}{R} 6/2 Minotaur Berserker with first strike and haste.
pub fn minotaur_aggressor() -> CardDefinition {
    CardDefinition {
        name: "Minotaur Aggressor",
        cost: cost(&[generic(6), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Berserker],
            ..Default::default()
        },
        power: 6,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        ..Default::default()
    }
}

/// Malakir Familiar — {2}{B} 2/1 Bat. Flying, deathtouch; whenever you gain
/// life, it gets +1/+1 until end of turn.
pub fn malakir_familiar() -> CardDefinition {
    CardDefinition {
        name: "Malakir Familiar",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Mercurial Geists — {2}{U}{R} 1/3 Spirit. Flying; whenever you cast an instant
/// or sorcery spell, it gets +3/+0 until end of turn.
pub fn mercurial_geists() -> CardDefinition {
    CardDefinition {
        name: "Mercurial Geists",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Engine Rat — {B} 1/1 Zombie Rat. Deathtouch; "{5}{B}: Each opponent loses 2
/// life."
pub fn engine_rat() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Engine Rat",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gavony Silversmith — {3}{W} 2/3 Human Soldier. When it enters, put a +1/+1
/// counter on each of up to two target creatures.
pub fn gavony_silversmith() -> CardDefinition {
    CardDefinition {
        name: "Gavony Silversmith",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Reputable Merchant — {2/W}{2/B}{2/G} 2/2 Human Citizen. When it enters or
/// dies, put a +1/+1 counter on target creature you control.
pub fn reputable_merchant() -> CardDefinition {
    use crate::mana::{mono_hybrid, Color};
    let counter = || Effect::AddCounter {
        what: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Reputable Merchant",
        cost: cost(&[
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
            mono_hybrid(2, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(counter()), on_dies(counter())],
        ..Default::default()
    }
}

/// Withering Torment — {2}{B} Instant. Destroy target creature or enchantment;
/// you lose 2 life.
pub fn withering_torment() -> CardDefinition {
    CardDefinition {
        name: "Withering Torment",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Voltage Surge — {R} Instant. You may sacrifice an artifact as an additional
/// cost; deals 2 damage to target creature or planeswalker, or 4 if you did.
/// (The additional cost is taken at resolution — a `MayDo` sacrifice.)
pub fn voltage_surge() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Voltage Surge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::MayDo {
                description: "Sacrifice an artifact for extra damage".into(),
                body: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Artifact,
                }),
            },
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(4),
                }),
                else_: Box::new(Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Corpse Appraiser — {U}{B}{R} 3/3 Vampire Rogue. ETB exile a creature card
/// from a graveyard, then dig three (one to hand, the rest to your graveyard).
/// (The "only if a card was exiled" gate is approximated — the dig is
/// unconditional.)
pub fn corpse_appraiser() -> CardDefinition {
    CardDefinition {
        name: "Corpse Appraiser",
        cost: cost(&[u(), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Exile,
            },
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: true,
                pick_filter: None,
                take: None,
                to_battlefield: false,
            },
        ]))],
        ..Default::default()
    }
}

/// The Wandering Rescuer — {3}{W}{W} 3/4 Legendary Human Samurai Noble. Flash,
/// Convoke, Double strike. Other tapped creatures you control have hexproof.
pub fn the_wandering_rescuer() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "The Wandering Rescuer",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flash, Keyword::Convoke, Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "Other tapped creatures you control have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::Tapped)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Hexproof,
            },
        }],
        ..Default::default()
    }
}

/// Light Up the Night — {X}{R} Sorcery. Deals X damage to any target — X+1 if
/// that target is a creature or planeswalker. (Flashback via remove-loyalty is
/// omitted — no loyalty-removal alt-cost primitive.)
pub fn light_up_the_night() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Light Up the Night",
        cost: cost(&[crate::mana::x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::XFromCost,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Tyrant's Scorn — {U}{B} Instant. Destroy a creature with mana value 3 or
/// less; or return target creature to its owner's hand.
pub fn tyrants_scorn() -> CardDefinition {
    CardDefinition {
        name: "Tyrant's Scorn",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
        ..Default::default()
    }
}

/// Fang of Shigeki — {G} 1/1 Snake Ninja Enchantment Creature with deathtouch.
pub fn fang_of_shigeki() -> CardDefinition {
    CardDefinition {
        name: "Fang of Shigeki",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Ninja],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}




/// Lifecraft Cavalry — {4}{G} 4/4 Elf Warrior with trample. Revolt — enters
/// with two +1/+1 counters if a permanent left the battlefield under your
/// control this turn.
pub fn lifecraft_cavalry() -> CardDefinition {
    CardDefinition {
        name: "Lifecraft Cavalry",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::revolt_etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Workshop Warchief — {3}{G}{G} 5/3 Rhino Warrior with trample. ETB gain 3
/// life; dies → make a 4/4 green Rhino Warrior. Blitz {4}{G}{G}.
pub fn workshop_warchief() -> CardDefinition {
    use crate::card::TokenDefinition;
    let rhino = TokenDefinition {
        name: "Rhino".into(), power: 4, toughness: 4,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Green],
        keywords: vec![Keyword::Trample],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Workshop Warchief",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(crate::effect::shortcut::blitz(cost(&[generic(4), g(), g()]))),
        triggered_abilities: vec![
            etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            on_dies(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: rhino }),
        ],
        ..Default::default()
    }
}

/// Prosperous Innkeeper — {1}{G} 1/1 Halfling Citizen. ETB create a Treasure.
/// Whenever another creature you control enters, gain 1 life.
pub fn prosperous_innkeeper() -> CardDefinition {
    CardDefinition {
        name: "Prosperous Innkeeper",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Halfling, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Jadar, Ghoulcaller of Nephalia — {1}{B} 1/1 Legendary Human Wizard. At your
/// end step, if you control no creature with decayed, create a 2/2 black Zombie
/// with decayed.
pub fn jadar_ghoulcaller_of_nephalia() -> CardDefinition {
    let zombie = crate::card::TokenDefinition {
        name: "Zombie".into(), power: 2, toughness: 2,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Black],
        keywords: vec![Keyword::Decayed],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Jadar, Ghoulcaller of Nephalia",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasKeyword(Keyword::Decayed),
                },
            )))),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: zombie },
        }],
        ..Default::default()
    }
}

/// The Goose Mother — {X}{G}{U} 2/2 Legendary Bird Hydra. Flying. Enters with X
/// +1/+1 counters and makes half X Food (rounded up). Attack: you may sacrifice
/// a Food to draw a card.
pub fn the_goose_mother() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "The Goose Mother",
        cost: cost(&[crate::mana::x(), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Hydra],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::HalvedRoundUp(Box::new(Value::XFromCost)),
                definition: crabomination_base::tokens::food_token(),
            }),
            crate::effect::shortcut::on_attack(Effect::MayDo {
                description: "Sacrifice a Food to draw a card".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Food),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
            }),
        ],
        ..Default::default()
    }
}

/// Archangel of Wrath — {2}{W}{W} 3/4 Angel. Flying, lifelink. Kicker (multi,
/// approximated from the printed {B} and/or {R}): when it enters, deals 2 damage
/// to any target for each time it was kicked (up to twice).
pub fn archangel_of_wrath() -> CardDefinition {
    CardDefinition {
        name: "Archangel of Wrath",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Multikicker(cost(&[r()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(Value::TimesKicked, Value::Const(1))),
                effect: Effect::DealDamage { to: target_filtered(SelectionRequirement::Any), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(Value::TimesKicked, Value::Const(2))),
                effect: Effect::DealDamage { to: target_filtered(SelectionRequirement::Any), amount: Value::Const(2) },
            },
        ],
        ..Default::default()
    }
}

/// Ascendant Packleader — {G} 2/1 Wolf. Enters with a +1/+1 counter if you
/// control a permanent with mana value 4+; gains a counter when you cast a
/// spell with mana value 4 or greater.
pub fn ascendant_packleader() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Ascendant Packleader",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::ManaValueAtLeast(4),
                }),
                then: Box::new(Effect::AddCounter {
                    what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(SelectionRequirement::ManaValueAtLeast(4))),
                effect: Effect::AddCounter {
                    what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Persistent Specimen — {B} 1/1 Skeleton. {2}{B}: Return this from your
/// graveyard to the battlefield tapped.
pub fn persistent_specimen() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Persistent Specimen",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wedding Invitation — {2} Artifact. ETB draw a card. {T}, Sacrifice: target
/// creature can't be blocked this turn; if it's a Vampire it also gains lifelink.
pub fn wedding_invitation() -> CardDefinition {
    use crate::card::{ActivatedAbility, CreatureType};
    use crate::effect::Predicate;
    CardDefinition {
        name: "Wedding Invitation",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Vampire),
                    },
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Unlucky Witness — {R} 1/1 Human Citizen. When it dies, exile the top two
/// cards of your library; until your next end step, you may play one of them.
pub fn unlucky_witness() -> CardDefinition {
    CardDefinition {
        name: "Unlucky Witness",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Citizen], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Squee, Dubious Monarch — {2}{R} 2/2 Legendary Goblin Noble. Haste. Attacks →
/// make a tapped, attacking 1/1 Goblin. Escape {3}{R}, exile four other cards.
pub fn squee_dubious_monarch() -> CardDefinition {
    let goblin = crate::card::TokenDefinition {
        name: "Goblin".into(), power: 1, toughness: 1,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Squee, Dubious Monarch",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Noble], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Escape(cost(&[generic(3), r()]), 4)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: goblin,
                cleanup: crate::effect::AttackingTokenCleanup::None,
            },
        }],
        ..Default::default()
    }
}

/// A 2/2 black Zombie with decayed (shared by several Innistrad makers).
fn decayed_zombie_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Zombie".into(), power: 2, toughness: 2,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Black],
        keywords: vec![Keyword::Decayed],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

/// A plain 2/2 black Zombie token.
fn black_zombie_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Zombie".into(), power: 2, toughness: 2,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}



/// Headless Rider — {2}{B} 3/1 Zombie. Whenever this or another nontoken Zombie
/// you control dies, create a 2/2 black Zombie.
pub fn headless_rider() -> CardDefinition {
    let make = || Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: black_zombie_token() };
    CardDefinition {
        name: "Headless Rider",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            on_dies(make()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie)
                            .and(SelectionRequirement::NotToken),
                    }),
                effect: make(),
            },
        ],
        ..Default::default()
    }
}

/// Diregraf Horde — {4}{B} 3/4 Zombie. ETB make two 2/2 decayed Zombies and
/// exile up to two cards from graveyards.
pub fn diregraf_horde() -> CardDefinition {
    CardDefinition {
        name: "Diregraf Horde",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: decayed_zombie_token() },
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    },
                    Value::Const(2),
                ),
                to: ZoneDest::Exile,
            },
        ]))],
        ..Default::default()
    }
}

/// The Meathook Massacre — {X}{B}{B} Legendary Enchantment. ETB each creature
/// gets -X/-X EOT. Your creature dies → each opponent loses 1; an opponent's
/// creature dies → you gain 1.
pub fn the_meathook_massacre() -> CardDefinition {
    CardDefinition {
        name: "The Meathook Massacre",
        cost: cost(&[crate::mana::x(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                    toughness: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                    duration: Duration::EndOfTurn,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Cobblebrute — {3}{R} 5/2 Elemental (vanilla beater).
pub fn cobblebrute() -> CardDefinition {
    CardDefinition {
        name: "Cobblebrute",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 5,
        toughness: 2,
        ..Default::default()
    }
}

/// Reckoner's Bargain — {1}{B} Instant. Sacrifice an artifact or creature (taken
/// at resolution); gain life equal to its mana value and draw two cards.
pub fn reckoners_bargain() -> CardDefinition {
    CardDefinition {
        name: "Reckoner's Bargain",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            },
            Effect::GainLife { who: Selector::You, amount: Value::SacrificedManaValue },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Phyrexian Missionary — {1}{W} 2/3 Phyrexian Human Cleric. Lifelink. Kicker
/// {1}{B}; if kicked, ETB return a creature card from your graveyard to hand.
pub fn phyrexian_missionary() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Phyrexian Missionary",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink, Keyword::Kicker(cost(&[generic(1), b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Soul Transfer — {1}{B}{B} Sorcery. Exile target creature or planeswalker; or
/// return a creature or planeswalker card from your graveyard to your hand. (The
/// "choose both if you control an artifact and an enchantment" rider is omitted.)
pub fn soul_transfer() -> CardDefinition {
    CardDefinition {
        name: "Soul Transfer",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                to: ZoneDest::Exile,
            },
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

// === Innistrad: Midnight Hunt / Crimson Vow — second batch ===

/// Lier, Disciple of the Drowned — {3}{U}{U} 3/4 Human Wizard. Spells can't be
/// countered; each instant and sorcery card in your graveyard has flashback
/// equal to its mana cost (`StaticEffect::GraveyardInstantsSorceriesHaveFlashback`).
pub fn lier_disciple_of_the_drowned() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, Supertype};
    CardDefinition {
        name: "Lier, Disciple of the Drowned",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Spells can't be countered.",
                effect: StaticEffect::SpellsUncounterable { filter: SelectionRequirement::Any },
            },
            StaticAbility {
                description: "Each instant and sorcery card in your graveyard has flashback equal to its mana cost.",
                effect: StaticEffect::GraveyardInstantsSorceriesHaveFlashback,
            },
        ],
        ..Default::default()
    }
}

/// Markov Crusader — {4}{B} 4/3 Vampire Knight. Lifelink; has haste as long as
/// you control another Vampire.
pub fn markov_crusader() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Markov Crusader",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "This creature has haste as long as you control another Vampire.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Vampire)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
            },
        }],
        ..Default::default()
    }
}

/// Stensia Masquerade — {2}{R} Enchantment. Attacking creatures you control
/// have first strike; whenever a Vampire you control deals combat damage to a
/// player, put a +1/+1 counter on it. Madness {2}{R}.
pub fn stensia_masquerade() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Stensia Masquerade",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Madness(cost(&[generic(2), r()]))],
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control have first strike.",
            effect: StaticEffect::GrantKeywordToAttackers { keyword: Keyword::FirstStrike },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Vampire),
                }),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Cradle of the Accursed — Desert land. {T}: Add {C}. {3}, {T}, Sacrifice it:
/// create a 2/2 black Zombie. Activate only as a sorcery.
pub fn cradle_of_the_accursed() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType, TokenDefinition};
    CardDefinition {
        name: "Cradle of the Accursed",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Zombie".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![crate::mana::Color::Black],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Zombie],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kessig Wolfrider — {R} 1/2 Human Knight. Menace. {2}{R}, {T}, exile three
/// cards from your graveyard: create a 3/2 red Wolf.
pub fn kessig_wolfrider() -> CardDefinition {
    use crate::card::{ActivatedAbility, TokenDefinition};
    CardDefinition {
        name: "Kessig Wolfrider",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            exile_other_filter: Some((SelectionRequirement::Any, 3)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Wolf".into(),
                    power: 3,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Wolf],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Storm Skreelix — {3}{U}{R} 2/4 Drake Horror. Flying. Instant and sorcery
/// spells you cast cost {1} less; whenever you cast an instant or sorcery
/// spell, this creature gets +2/+0 until end of turn.
pub fn storm_skreelix() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Storm Skreelix",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                amount: 1,
            },
        }],
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Bloodvial Purveyor — {2}{B}{B} 5/6 Vampire. Flying, trample. Whenever an
/// opponent casts a spell, that player creates a Blood token. Whenever it
/// attacks, it gets +1/+0 until end of turn for each Blood token the defending
/// player controls (read as Blood an opponent controls).
pub fn bloodvial_purveyor() -> CardDefinition {
    CardDefinition {
        name: "Bloodvial Purveyor",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
                effect: Effect::CreateToken {
                    who: PlayerRef::Triggerer,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::blood_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::CountOf(Box::new(Selector::EachPermanent(
                        SelectionRequirement::HasArtifactSubtype(
                            crate::card::ArtifactSubtype::Blood,
                        )
                        .and(SelectionRequirement::ControlledByOpponent),
                    ))),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Croaking Counterpart — {1}{G}{U} Sorcery. Create a token that's a copy of
/// target creature, except it's a 1/1 Frog. Flashback {3}{G}{U}. (The copy's
/// recolor to green is approximated — it keeps the original's colors plus the
/// Frog type and 1/1 body.)
pub fn croaking_counterpart() -> CardDefinition {
    CardDefinition {
        name: "Croaking Counterpart",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), g(), u()]))],
        effect: Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::Const(1),
            source: target_filtered(SelectionRequirement::Creature),
            extra_creature_types: vec![CreatureType::Frog],
            extra_card_types: vec![],
            override_pt: Some((1, 1)),
            non_legendary: true,
            legendary: false,
        },
        ..Default::default()
    }
}

/// Voldaren Estate — Land. {T}: Add {C}. {T}, Pay 1 life: Add one mana of any
/// color. {5}, {T}: Create a Blood token. (The "only for Vampire spells" spend
/// restriction and per-Vampire cost reduction are approximated away.)
pub fn voldaren_estate() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Voldaren Estate",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::blood_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Sigarda's Vanguard — {4}{W} 3/3 Angel. Flash, flying. Whenever it enters or
/// attacks, up to three target creatures gain double strike until end of turn.
/// (The "any number of creatures with different powers" clause is approximated
/// as up to three targets.)
pub fn sigardas_vanguard() -> CardDefinition {
    let grant = || Effect::ApplyToTargets {
        max_targets: 3,
        filter: SelectionRequirement::Creature,
        effect: Box::new(Effect::GrantKeyword {
            what: Selector::Target(0),
            keyword: Keyword::DoubleStrike,
            duration: Duration::EndOfTurn,
        }),
    };
    CardDefinition {
        name: "Sigarda's Vanguard",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![
            etb(grant()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: grant(),
            },
        ],
        ..Default::default()
    }
}

/// Diregraf Colossus — {2}{B} 2/2 Zombie Giant. Enters with a +1/+1 counter for
/// each Zombie card in your graveyard. Whenever you cast a Zombie spell, create
/// a tapped 2/2 black Zombie.
pub fn diregraf_colossus() -> CardDefinition {
    let mut tapped_zombie = black_zombie_token();
    tapped_zombie.tapped = true;
    CardDefinition {
        name: "Diregraf Colossus",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Giant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountMatching {
                sel: Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                }),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie),
            },
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(SelectionRequirement::HasCreatureType(
                    CreatureType::Zombie,
                )),
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: tapped_zombie,
            },
        }],
        ..Default::default()
    }
}

/// Wilhelt, the Rotcleaver — {2}{U}{B} 3/3 Zombie Warrior. When another Zombie
/// you control without decayed dies, create a 2/2 black Zombie with decayed. At
/// your end step, you may sacrifice a Zombie to draw a card.
pub fn wilhelt_the_rotcleaver() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Wilhelt, the Rotcleaver",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie)
                            .and(SelectionRequirement::OtherThanSource)
                            .and(SelectionRequirement::HasKeyword(Keyword::Decayed).negate()),
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: decayed_zombie_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice a Zombie to draw a card?".into(),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie),
                    count: Value::Const(1),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// A 1/1 white Spirit token with flying.
fn white_spirit_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Spirit".into(), power: 1, toughness: 1,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    }
}

/// Millicent, Restless Revenant — {5}{W}{U} 4/4 Spirit Soldier. Affinity for
/// Spirits, flying. Whenever Millicent or another nontoken Spirit you control
/// dies or deals combat damage to a player, create a 1/1 white flying Spirit.
pub fn millicent_restless_revenant() -> CardDefinition {
    use crate::card::Supertype;
    let spirit_filter = SelectionRequirement::HasCreatureType(CreatureType::Spirit)
        .and(SelectionRequirement::IsToken.negate());
    let make_spirit = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: white_spirit_token(),
    };
    CardDefinition {
        name: "Millicent, Restless Revenant",
        cost: cost(&[generic(5), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                .and(SelectionRequirement::ControlledByYou),
        ),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: spirit_filter.clone(),
                    }),
                effect: make_spirit(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: spirit_filter,
                    }),
                effect: make_spirit(),
            },
        ],
        ..Default::default()
    }
}

/// Ollenbock Escort — {W} 1/1 Human Cleric. Vigilance. Sacrifice it: target
/// creature you control with a +1/+1 counter on it gains lifelink and
/// indestructible until end of turn.
pub fn ollenbock_escort() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ollenbock Escort",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                    },
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sigarda, Font of Blessings — {2}{G}{W} 4/4 Angel. Flying. Other permanents
/// you control have hexproof. Play with the top card of your library revealed;
/// you may cast Angel and Human spells from the top of your library.
pub fn sigarda_font_of_blessings() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, Supertype};
    CardDefinition {
        name: "Sigarda, Font of Blessings",
        cost: cost(&[generic(2), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Other permanents you control have hexproof.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::ControlledByYou
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    keyword: Keyword::Hexproof,
                },
            },
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast Angel and Human spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop {
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Angel)
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Human)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Sungold Barrage — {2}{W} Instant. Destroy target creature with toughness 4
/// or greater.
pub fn sungold_barrage() -> CardDefinition {
    CardDefinition {
        name: "Sungold Barrage",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtLeast(4)),
            ),
        },
        ..Default::default()
    }
}

/// Ghoulcaller Gisa — {3}{B}{B} 3/4 Human Wizard. {B}, {T}, Sacrifice another
/// creature: create X 2/2 black Zombies, where X is the sacrificed creature's
/// power.
pub fn ghoulcaller_gisa() -> CardDefinition {
    use crate::card::{ActivatedAbility, Supertype};
    CardDefinition {
        name: "Ghoulcaller Gisa",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::SacrificedPower,
                    definition: black_zombie_token(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ghoulish Procession — {1}{B} Enchantment. Whenever one or more nontoken
/// creatures die, create a 2/2 black Zombie with decayed. Once each turn.
pub fn ghoulish_procession() -> CardDefinition {
    CardDefinition {
        name: "Ghoulish Procession",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::IsToken.negate(),
                })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: decayed_zombie_token(),
            },
        }],
        ..Default::default()
    }
}

/// Necroduality — {3}{U} Enchantment. Whenever a nontoken Zombie you control
/// enters, create a token that's a copy of that creature.
pub fn necroduality() -> CardDefinition {
    CardDefinition {
        name: "Necroduality",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie)
                        .and(SelectionRequirement::IsToken.negate()),
                }),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(1),
                source: Selector::TriggerSource,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                non_legendary: false,
                legendary: false,
            },
        }],
        ..Default::default()
    }
}

/// Falkenrath Forebear — {2}{B} 3/1 Vampire. Flying; can't block. Whenever it
/// deals combat damage to a player, create a Blood token. {B}, Sacrifice two
/// Blood tokens: return this card from your graveyard to the battlefield.
pub fn falkenrath_forebear() -> CardDefinition {
    use crate::card::{ActivatedAbility, ArtifactSubtype};
    CardDefinition {
        name: "Falkenrath Forebear",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            from_graveyard: true,
            sac_other_filter: Some((
                SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                2,
            )),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Geralf, Visionary Stitcher — {2}{U} 1/4 Human Wizard. Zombies you control
/// have flying. {U}, {T}, Sacrifice another nontoken creature: create an X/X
/// blue Zombie, where X is the sacrificed creature's toughness.
pub fn geralf_visionary_stitcher() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility, StaticEffect, Supertype, TokenDefinition};
    CardDefinition {
        name: "Geralf, Visionary Stitcher",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Zombies you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Zombie)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Flying,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::IsToken.negate())
                        .and(SelectionRequirement::OtherThanSource),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Zombie".into(),
                        card_types: vec![CardType::Creature],
                        colors: vec![crate::mana::Color::Blue],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Zombie],
                            ..Default::default()
                        },
                        dynamic_pt: Some((Value::SacrificedToughness, Value::SacrificedToughness)),
                        ..Default::default()
                    },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wickerwing Effigy — {3} 1/4 Artifact Creature — Scarecrow. Defender. Play
/// with the top card of your library revealed; you may cast creature spells
/// from the top of your library. (The "cast creature becomes a 1/1 flying Bird"
/// rider is omitted.)
pub fn wickerwing_effigy() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Wickerwing Effigy",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scarecrow], ..Default::default() },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast creature spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: SelectionRequirement::Creature },
            },
        ],
        ..Default::default()
    }
}

/// Massive Might — {G} Instant. Target creature gets +2/+2 and gains trample
/// until end of turn.
pub fn massive_might() -> CardDefinition {
    CardDefinition {
        name: "Massive Might",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Mossbeard Ancient — {5}{G}{G} 7/7 Treefolk. Trample. ETB gain 5 life.
pub fn mossbeard_ancient() -> CardDefinition {
    CardDefinition {
        name: "Mossbeard Ancient",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(5) })],
        ..Default::default()
    }
}

/// Shadowbeast Sighting — {3}{G} Sorcery. Create a 4/4 green Beast. Flashback
/// {6}{G}.
pub fn shadowbeast_sighting() -> CardDefinition {
    CardDefinition {
        name: "Shadowbeast Sighting",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(6), g()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crate::card::TokenDefinition {
                name: "Beast".into(),
                power: 4,
                toughness: 4,
                card_types: vec![CardType::Creature],
                colors: vec![crate::mana::Color::Green],
                subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Sawblade Slinger — {3}{G} 4/3 Human Archer. ETB choose one — destroy target
/// artifact an opponent controls; or it fights target creature an opponent
/// controls. (The fight's Zombie sub-restriction is widened to any creature.)
pub fn sawblade_slinger() -> CardDefinition {
    CardDefinition {
        name: "Sawblade Slinger",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
            Effect::Fight {
                attacker: Selector::This,
                defender: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]))],
        ..Default::default()
    }
}

/// Gisa, Glorious Resurrector — {2}{B}{B} 4/4 Human Wizard. If a creature an
/// opponent controls would die, exile it instead. At your upkeep, put all
/// creature cards exiled with Gisa onto the battlefield under your control with
/// decayed.
pub fn gisa_glorious_resurrector() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, Supertype};
    CardDefinition {
        name: "Gisa, Glorious Resurrector",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead.",
            effect: StaticEffect::ExileDyingOpponentCreatures { when_you_do: None },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::ReturnExiledBySourceToBattlefield { decayed: true },
        }],
        ..Default::default()
    }
}

/// Mounted Dreadknight — {4}{R} 5/4 Vampire Knight. Trample. Enters with a
/// +1/+1 counter if an opponent lost life this turn.
pub fn mounted_dreadknight() -> CardDefinition {
    CardDefinition {
        name: "Mounted Dreadknight",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Path to the Festival — {2}{G} Sorcery. Search for a basic land onto the
/// battlefield tapped, then scry 1 if you control three or more basic land
/// types. Flashback {4}{G}.
pub fn path_to_the_festival() -> CardDefinition {
    CardDefinition {
        name: "Path to the Festival",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), g()]))],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::DomainCount(PlayerRef::You), Value::Const(3)),
                then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Thraben Exorcism — {1}{W} Instant. Exile target Spirit or enchantment. (The
/// "creature with disturb" sub-filter is approximated away.)
pub fn thraben_exorcism() -> CardDefinition {
    CardDefinition {
        name: "Thraben Exorcism",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                    .or(SelectionRequirement::Enchantment),
            ),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Falkenrath Celebrants — {4}{R} 4/4 Vampire. Menace. ETB create two Blood
/// tokens.
pub fn falkenrath_celebrants() -> CardDefinition {
    CardDefinition {
        name: "Falkenrath Celebrants",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: crabomination_base::tokens::blood_token(),
        })],
        ..Default::default()
    }
}

/// Slaughter Specialist — {1}{B} 3/3 Vampire Warrior. ETB each opponent creates
/// a 1/1 white Human. Whenever a creature an opponent controls dies, put a
/// +1/+1 counter on this creature.
pub fn slaughter_specialist() -> CardDefinition {
    CardDefinition {
        name: "Slaughter Specialist",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::EachOpponent,
                count: Value::Const(1),
                definition: crate::card::TokenDefinition {
                    name: "Human".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Human],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Unhallowed Phalanx — {4}{B} 1/13 Zombie Soldier that enters tapped. A
/// defensive wall.
pub fn unhallowed_phalanx() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Unhallowed Phalanx",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 13,
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..Default::default()
    }
}

/// Moldgraf Millipede — {4}{G} 2/2 Insect Horror. ETB mill three, then put a
/// +1/+1 counter on it for each creature card in your graveyard.
pub fn moldgraf_millipede() -> CardDefinition {
    CardDefinition {
        name: "Moldgraf Millipede",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountMatching {
                    sel: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    }),
                    filter: SelectionRequirement::Creature,
                },
            },
        ]))],
        ..Default::default()
    }
}

/// Overcharged Amalgam — {2}{U}{U} 3/3 Zombie Horror. Flash, flying. Exploit;
/// when it exploits a creature, counter target spell. (The "or activated /
/// triggered ability" sub-mode is approximated to spells.)
pub fn overcharged_amalgam() -> CardDefinition {
    use crate::effect::shortcut::exploit;
    CardDefinition {
        name: "Overcharged Amalgam",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![exploit(Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        })],
        ..Default::default()
    }
}

/// Hobbling Zombie — {2}{B} 2/2 Zombie. Deathtouch. When it dies, create a 2/2
/// black Zombie with decayed.
pub fn hobbling_zombie() -> CardDefinition {
    CardDefinition {
        name: "Hobbling Zombie",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: decayed_zombie_token(),
        })],
        ..Default::default()
    }
}

/// Selhoff Entomber — {1}{U} 1/3 Zombie. {T}, Discard a creature card: draw a
/// card.
pub fn selhoff_entomber() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Selhoff Entomber",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Falkenrath Perforator — {1}{R} 2/1 Vampire. Whenever it attacks, it deals 1
/// damage to the defending player.
pub fn falkenrath_perforator() -> CardDefinition {
    CardDefinition {
        name: "Falkenrath Perforator",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Crawl from the Cellar — {B} Sorcery. Return target creature card from your
/// graveyard to your hand. Flashback {3}{B}. (The optional Zombie counter rider
/// is omitted.)
pub fn crawl_from_the_cellar() -> CardDefinition {
    CardDefinition {
        name: "Crawl from the Cellar",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), b()]))],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Foul Play — {1}{B} Sorcery. Destroy target creature with power 2 or less,
/// then investigate.
pub fn foul_play() -> CardDefinition {
    use crate::effect::shortcut::investigate;
    CardDefinition {
        name: "Foul Play",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
                ),
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Rotten Reunion — {B} Sorcery. Exile up to one target card from a graveyard,
/// then create a 2/2 black Zombie with decayed. Flashback {1}{B}.
pub fn rotten_reunion() -> CardDefinition {
    CardDefinition {
        name: "Rotten Reunion",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(1), b()]))],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::InGraveyard },
                to: ZoneDest::Exile,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: decayed_zombie_token(),
            },
        ]),
        ..Default::default()
    }
}

// === Counter-synergy + Phyrexian-infect + artifact-aristocrat batch
// (claude/modern_decks). Each rides an existing primitive; the only new
// engine piece is `StaticEffect::ExtraCounterAllKinds` (Winding Constrictor). ===

/// Winding Constrictor — {B}{G} 2/3 Snake. If one or more counters would be put
/// on an artifact or creature you control, that many plus one are put on it
/// instead. (The "counters you'd get" player-counter clause is approximated.)
pub fn winding_constrictor() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Winding Constrictor",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "If one or more counters would be put on an artifact or \
                          creature you control, that many plus one are put on it \
                          instead.",
            effect: StaticEffect::ExtraCounterAllKinds,
        }],
        ..Default::default()
    }
}

/// Conclave Mentor — {G}{W} 2/2 Centaur Cleric. +1/+1 counters on your creatures
/// get the Hardened-Scales bonus; when it dies, gain life equal to its power.
pub fn conclave_mentor() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Conclave Mentor",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature \
                          you control, that many plus one are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        triggered_abilities: vec![on_dies(Effect::GainLife {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Branching Evolution — {2}{G} Enchantment. Double +1/+1 counters placed on
/// your creatures (CR 614.16 counter-doubling).
pub fn branching_evolution() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Branching Evolution",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature \
                          you control, twice that many are put on it instead.",
            effect: StaticEffect::DoubleCounters,
        }],
        ..Default::default()
    }
}

/// Blight Mamba — {1}{G} 1/1 Phyrexian Snake. Infect; {1}{G}: Regenerate.
pub fn blight_mamba() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Blight Mamba",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Snake],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Infect],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ichorclaw Myr — {2} Artifact Creature — Phyrexian Myr 1/1. Infect; when it
/// becomes blocked, +2/+2 until end of turn.
pub fn ichorclaw_myr() -> CardDefinition {
    CardDefinition {
        name: "Ichorclaw Myr",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Myr],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Necropede — {2} Artifact Creature — Phyrexian Insect 1/1. Infect; when it
/// dies, you may put a -1/-1 counter on target creature.
pub fn necropede() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Necropede",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "put a -1/-1 counter on target creature".into(),
            body: Box::new(Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Fuel for the Cause — {2}{U}{U} Instant. Counter target spell, then
/// proliferate.
pub fn fuel_for_the_cause() -> CardDefinition {
    CardDefinition {
        name: "Fuel for the Cause",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Contagion Engine — {6} Artifact. ETB put a -1/-1 counter on each creature an
/// opponent controls (the "target player" prompt collapses to each opponent);
/// {4}, {T}: Proliferate twice.
pub fn contagion_engine() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Contagion Engine",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(1),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::Seq(vec![Effect::Proliferate, Effect::Proliferate]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kami of False Hope — {W} 1/1 Spirit. Sacrifice it: prevent all combat damage
/// this turn.
pub fn kami_of_false_hope() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Kami of False Hope",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PreventAllCombatDamageThisTurn,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Renegade Rallier — {1}{G}{W} 3/2 Human Warrior. Revolt — ETB, if a permanent
/// left under your control this turn, return target permanent card with mana
/// value 2 or less from your graveyard to the battlefield.
pub fn renegade_rallier() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Renegade Rallier",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::RevoltActive { who: PlayerRef::You },
            then: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::InGraveyard
                        .and(SelectionRequirement::PermanentCard)
                        .and(SelectionRequirement::ManaValueAtMost(2)),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Pitiless Plunderer — {3}{B} 1/4 Human Pirate. Whenever another creature you
/// control dies, create a Treasure token.
pub fn pitiless_plunderer() -> CardDefinition {
    use crate::effect::shortcut::{mint_treasures, on_other_dies};
    CardDefinition {
        name: "Pitiless Plunderer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![on_other_dies(mint_treasures(1))],
        ..Default::default()
    }
}

/// Falkenrath Aristocrat — {2}{B}{R} 4/1 Vampire Noble. Flying, haste. Sacrifice
/// a creature: this gains indestructible until end of turn. (The "+1/+1 if the
/// sacrificed creature was a Human" rider is approximated away.)
pub fn falkenrath_aristocrat() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Falkenrath Aristocrat",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Terrarion — {1} Artifact, enters tapped. {2}, {T}, Sacrifice this: Add two
/// mana in any combination of colors. When it's sacrificed, draw a card.
pub fn terrarion() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility, StaticEffect};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Terrarion",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(2)) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Implement of Combustion — {1} Artifact. {R}, Sacrifice this: 1 damage to any
/// target. When it's sacrificed, draw a card.
pub fn implement_of_combustion() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Implement of Combustion",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Reckless Fireweaver — {1}{R} 1/3 Human Artificer. Whenever an artifact you
/// control enters, this deals 1 damage to each opponent.
pub fn reckless_fireweaver() -> CardDefinition {
    CardDefinition {
        name: "Reckless Fireweaver",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Disciple of the Vault — {B} 1/1 Human Cleric. Whenever an artifact is
/// sacrificed, you may have target opponent lose 1. (The "put into a graveyard"
/// destroy case is approximated to the sacrifice path.)
pub fn disciple_of_the_vault() -> CardDefinition {
    CardDefinition {
        name: "Disciple of the Vault",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::MayDo {
                description: "target opponent loses 1 life".into(),
                body: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Marionette Master — {4}{B}{B} 1/3 Human Artificer. Fabricate 3; whenever an
/// artifact you control is sacrificed, target opponent loses life equal to that
/// artifact's mana value. (Destroy case approximated to the sacrifice path.)
pub fn marionette_master() -> CardDefinition {
    use crate::effect::shortcut::fabricate;
    let mut abilities = vec![fabricate(3)];
    abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Artifact,
            }),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
        },
    });
    CardDefinition {
        name: "Marionette Master",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: abilities,
        ..Default::default()
    }
}

/// Glassdust Hulk — {3}{W}{U} 3/4 Golem. Whenever another artifact you control
/// enters, this gets +1/+1 and can't be blocked this turn. Cycling {W/U}.
pub fn glassdust_hulk() -> CardDefinition {
    use crate::mana::{hybrid, Color};
    CardDefinition {
        name: "Glassdust Hulk",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[hybrid(Color::White, Color::Blue)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

// === Second modern_decks batch: control / midrange staples. ===

/// Logic Knot — {X}{U}{U} Instant. Delve. Counter target spell unless its
/// controller pays {X}.
pub fn logic_knot() -> CardDefinition {
    CardDefinition {
        name: "Logic Knot",
        cost: cost(&[crate::mana::x(), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Delve],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[]),
            exile: false,
            extra_generic: Some(Value::XFromCost),
        },
        ..Default::default()
    }
}

/// Beanstalk Giant — {6}{G} Giant whose power and toughness each equal the
/// number of lands you control. Adventure — Fertile Footsteps {2}{G}: search
/// your library for a basic land, put it onto the battlefield, then shuffle.
pub fn beanstalk_giant() -> CardDefinition {
    use crate::card::{Adventure, DynamicPt};
    CardDefinition {
        name: "Beanstalk Giant",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        dynamic_pt: Some(DynamicPt::LandsControlled { base: 0 }),
        adventure: Some(Box::new(Adventure {
            name: "Fertile Footsteps",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        })),
        ..Default::default()
    }
}

/// Ambush Viper — {1}{G} 2/1 Snake. Flash, deathtouch.
pub fn ambush_viper() -> CardDefinition {
    CardDefinition {
        name: "Ambush Viper",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Etherium Sculptor — {1}{U} 1/2 Vedalken Artificer. Artifact spells you cast
/// cost {1} less.
pub fn etherium_sculptor() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Etherium Sculptor",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: SelectionRequirement::Artifact, amount: 1 },
        }],
        ..Default::default()
    }
}

/// Toolcraft Exemplar — {W} 1/1 Dwarf Artificer. At combat on your turn, if you
/// control an artifact it gets +2/+1; with three or more artifacts it also
/// gains first strike.
pub fn toolcraft_exemplar() -> CardDefinition {
    let artifacts = Value::count(Selector::EachPermanent(
        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
    ));
    CardDefinition {
        name: "Toolcraft Exemplar",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(artifacts.clone(), Value::Const(1)),
                then: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(artifacts, Value::Const(3)),
                        then: Box::new(Effect::GrantKeyword {
                            what: Selector::This,
                            keyword: Keyword::FirstStrike,
                            duration: Duration::EndOfTurn,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Vampire Gourmand — {1}{B} 2/2 Vampire. Attacks → you may sacrifice another
/// creature; if you do, draw a card and it can't be blocked this turn.
pub fn vampire_gourmand() -> CardDefinition {
    CardDefinition {
        name: "Vampire Gourmand",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature? (draw + unblockable)".into(),
                filter: SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                count: Value::Const(1),
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Recruitment Officer — {W} 2/1 Human Soldier. {3}{W}: look at the top four
/// cards, you may reveal a creature card with mana value 3 or less to hand,
/// rest on the bottom.
pub fn recruitment_officer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Recruitment Officer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: false,
                pick_filter: Some(
                    SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
                take: Some(Value::Const(1)),
                to_battlefield: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Squee, Goblin Nabob — {2}{R} 1/1 Legendary Goblin. At your upkeep, you may
/// return it from your graveyard to your hand.
pub fn squee_goblin_nabob() -> CardDefinition {
    CardDefinition {
        name: "Squee, Goblin Nabob",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayDo {
                description: "Return Squee from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Hazoret the Fervent — {3}{R} 5/4 Legendary God. Indestructible, haste. Can't
/// attack or block unless you have one or fewer cards in hand. {2}{R}, Discard a
/// card: deal 2 damage to each opponent.
pub fn hazoret_the_fervent() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Hazoret the Fervent",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::God], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![
            Keyword::Indestructible,
            Keyword::Haste,
            Keyword::CantAttackOrBlockUnlessHandSizeAtMost(1),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Patchwork Beastie — {G} 3/3 Artifact Creature — Beast. Delirium — can't
/// attack or block unless there are four or more card types in your graveyard.
/// At your upkeep, you may mill a card.
pub fn patchwork_beastie() -> CardDefinition {
    CardDefinition {
        name: "Patchwork Beastie",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::CantAttackOrBlockUnlessDelirium],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayDo {
                description: "Mill a card?".into(),
                body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

// === Third modern_decks batch: aggro + equipment + adventure staples. ===

/// Fervent Champion — {R} 1/1 Human Knight. First strike, haste. Whenever it
/// attacks, another target Knight you control gets +1/+0. (The equip-cost
/// rider is dropped — the engine has no self-targeted equip discount.)
pub fn fervent_champion() -> CardDefinition {
    CardDefinition {
        name: "Fervent Champion",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Knight)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Porcelain Legionnaire — {2}{W/P} 3/1 Phyrexian Soldier with first strike.
pub fn porcelain_legionnaire() -> CardDefinition {
    use crate::mana::{phyrexian, Color};
    CardDefinition {
        name: "Porcelain Legionnaire",
        cost: cost(&[generic(2), phyrexian(Color::White)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Short Sword — {1} Equipment. Equipped creature gets +1/+1. Equip {1}.
pub fn short_sword() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Short Sword",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Axebane Beast — {3}{G} 3/4 Beast.
pub fn axebane_beast() -> CardDefinition {
    CardDefinition {
        name: "Axebane Beast",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 4,
        ..Default::default()
    }
}

/// Yavimaya Sapherd — {2}{G} 2/2 Fungus. ETB create a 1/1 green Saproling.
pub fn yavimaya_sapherd() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::effect::shortcut::etb;
    use crate::mana::Color;
    let saproling = TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Yavimaya Sapherd",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: saproling,
        })],
        ..Default::default()
    }
}

/// Faerie Guidemother — {W} 1/1 Faerie with flying. Adventure — Gift of the Fae
/// {1}{W}: target creature gets +2/+1 and gains flying until end of turn.
pub fn faerie_guidemother() -> CardDefinition {
    use crate::card::Adventure;
    CardDefinition {
        name: "Faerie Guidemother",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(Adventure {
            name: "Gift of the Fae",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// All That Glitters — {1}{W} Aura. Enchanted creature gets +1/+1 for each
/// artifact and/or enchantment you control.
pub fn all_that_glitters() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus, EquipScale};
    CardDefinition {
        name: "All That Glitters",
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
            scale: Some(EquipScale {
                filter: SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .and(SelectionRequirement::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
                count_self_counters: None,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Scorching Dragonfire — {1}{R} Instant. Deal 3 damage to target creature or
/// planeswalker; if it would die this turn, exile it instead.
pub fn scorching_dragonfire() -> CardDefinition {
    CardDefinition {
        name: "Scorching Dragonfire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // Install the death replacement before the damage so a lethal hit
            // exiles rather than buries (mirrors Anger of the Gods).
            Effect::ExileIfWouldDieThisTurn {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Slaying Fire — {2}{R} Instant. Deal 3 damage to any target. (Adamant's
/// "4 instead if three red was spent" is dropped — no per-color spend tracking.)
pub fn slaying_fire() -> CardDefinition {
    CardDefinition {
        name: "Slaying Fire",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Searing Barrage — {4}{R} Instant. Deal 5 damage to target creature.
/// (Adamant's controller-burn rider is dropped — no per-color spend tracking.)
pub fn searing_barrage() -> CardDefinition {
    CardDefinition {
        name: "Searing Barrage",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}

// === Fourth modern_decks batch: small beaters. ===

/// Brazen Wolves — {2}{R} 2/3 Wolf. Whenever it attacks, it gets +2/+0.
pub fn brazen_wolves() -> CardDefinition {
    CardDefinition {
        name: "Brazen Wolves",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Glory Seeker — {1}{W} 2/2 Human Soldier.
pub fn glory_seeker() -> CardDefinition {
    CardDefinition {
        name: "Glory Seeker",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Pheres-Band Tromper — {3}{G} 3/3 Centaur Warrior. Inspired — whenever it
/// becomes untapped, put a +1/+1 counter on it.
pub fn pheres_band_tromper() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Pheres-Band Tromper",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// === Fifth modern_decks batch: combat tricks + utility bodies. ===

/// Dwarven Berserker — {1}{R} 1/1 Dwarf Berserker. Whenever it becomes blocked,
/// it gets +3/+0 and gains trample.
pub fn dwarven_berserker() -> CardDefinition {
    CardDefinition {
        name: "Dwarven Berserker",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Berserker],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Elvish Hexhunter — {G/W} 1/1 Elf Shaman. {G/W}, {T}, Sacrifice this:
/// destroy target enchantment.
pub fn elvish_hexhunter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::{hybrid, Color};
    CardDefinition {
        name: "Elvish Hexhunter",
        cost: cost(&[hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[hybrid(Color::Green, Color::White)]),
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::Enchantment),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Felidar Savior — {3}{W} 2/3 Cat Beast with lifelink. ETB put a +1/+1 counter
/// on each of up to two other target creatures you control.
pub fn felidar_savior() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Felidar Savior",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}
