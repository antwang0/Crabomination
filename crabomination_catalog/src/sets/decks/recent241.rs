//! MKM (Murders at Karlov Manor) gap batch — Detectives, Disguise creatures,
//! surveil/investigate value, and graveyard payoffs. Tests in
//! `tests/recent_b/recent241.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, investigate};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
    ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaSymbol};

/// Trigger for "whenever you draw your second card each turn".
fn on_second_draw(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
            .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::You, n: 2 })
            .once_per_turn(),
        effect,
    }
}

/// Sanitation Automaton — {2} Artifact Creature — Construct 2/1. ETB: surveil 1.
pub fn sanitation_automaton() -> CardDefinition {
    CardDefinition {
        name: "Sanitation Automaton",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Snarling Gorehound — {B} Dog 1/1, menace. Whenever another creature you
/// control with power 2 or less enters, surveil 1.
pub fn snarling_gorehound() -> CardDefinition {
    CardDefinition {
        name: "Snarling Gorehound",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtMost(2)),
                }),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Loxodon Eavesdropper — {3}{G} Elephant Detective 3/3. ETB: investigate.
/// Whenever you draw your second card each turn, +1/+1 and vigilance until EOT.
pub fn loxodon_eavesdropper() -> CardDefinition {
    CardDefinition {
        name: "Loxodon Eavesdropper",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(investigate(1)),
            on_second_draw(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ])),
        ],
        ..Default::default()
    }
}

/// Jaded Analyst — {1}{U} Human Detective 3/2, defender. Whenever you draw your
/// second card each turn, it loses defender and gains vigilance until EOT.
pub fn jaded_analyst() -> CardDefinition {
    CardDefinition {
        name: "Jaded Analyst",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![on_second_draw(Effect::Seq(vec![
            Effect::LoseKeywordThisTurn { what: Selector::This, keyword: Keyword::Defender },
            Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
        ]))],
        ..Default::default()
    }
}

/// Innocent Bystander — {1}{R} Goblin Citizen 2/1. Whenever it's dealt 3 or more
/// damage, investigate.
pub fn innocent_bystander() -> CardDefinition {
    CardDefinition {
        name: "Innocent Bystander",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource)
                .with_filter(Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(3))),
            effect: investigate(1),
        }],
        ..Default::default()
    }
}

/// Rot Farm Mortipede — {3}{B} Insect 3/4. Whenever one or more creature cards
/// leave your graveyard, it gets +1/+0 and gains menace and lifelink until EOT.
pub fn rot_farm_mortipede() -> CardDefinition {
    CardDefinition {
        name: "Rot Farm Mortipede",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Menace, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            ]),
        }],
        ..Default::default()
    }
}

/// Dog Walker — {R}{W} Human Citizen 3/1, vigilance. Disguise {R/W}{R/W}. When
/// turned face up, create two tapped 1/1 white Dog tokens.
pub fn dog_walker() -> CardDefinition {
    let rw = ManaSymbol::Hybrid(Color::Red, Color::White);
    CardDefinition {
        name: "Dog Walker",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Vigilance, Keyword::Disguise(cost(&[rw, rw]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: crate::card::TokenDefinition {
                    name: "Dog".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
                    tapped: true,
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Forum Familiar — {W} Cat 1/1. Disguise {1}{W}. When turned face up, return
/// another target permanent you control to its owner's hand and put a +1/+1
/// counter on this creature.
pub fn forum_familiar() -> CardDefinition {
    CardDefinition {
        name: "Forum Familiar",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Disguise(cost(&[generic(1), w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::ControlledByYou.and(R::OtherThanSource),
                    },
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Slice from the Shadows — {X}{B} Instant. Can't be countered. Target creature
/// gets -X/-X until end of turn.
pub fn slice_from_the_shadows() -> CardDefinition {
    CardDefinition {
        name: "Slice from the Shadows",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::PumpPT {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            power: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
            toughness: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Cerebral Confiscation — {2}{B} Sorcery. Choose one — target opponent discards
/// two cards; or reveal their hand and you choose a nonland card to discard.
pub fn cerebral_confiscation() -> CardDefinition {
    CardDefinition {
        name: "Cerebral Confiscation",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2), random: false },
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Nonland,
            },
        ]),
        ..Default::default()
    }
}

/// Caught Red-Handed — {4}{R} Instant. Can't be countered. Gain control of
/// target creature until end of turn, untap it, it gains haste, and suspect it.
pub fn caught_red_handed() -> CardDefinition {
    CardDefinition {
        name: "Caught Red-Handed",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::Suspect { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

/// A "you may draw a card, then discard a card" loot payoff.
fn may_loot() -> Effect {
    Effect::MayDo {
        description: "draw a card, then discard a card".into(),
        body: Box::new(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ])),
    }
}

/// Mistway Spy — {U} Merfolk Detective 1/1, flying. Disguise {1}{U}. When turned
/// face up, until end of turn, whenever a creature you control deals combat
/// damage to a player, investigate.
pub fn mistway_spy() -> CardDefinition {
    CardDefinition {
        name: "Mistway Spy",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Disguise(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::CreaturesYouControlDealingCombatDamageThisTurn {
                body: Box::new(investigate(1)),
            },
        }],
        ..Default::default()
    }
}

/// Glint Weaver — {5}{G}{G} Spider 3/3, reach. ETB: distribute three +1/+1
/// counters among up to three target creatures, then gain life equal to the
/// greatest toughness among creatures you control.
pub fn glint_weaver() -> CardDefinition {
    CardDefinition {
        name: "Glint Weaver",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DistributeCounters {
                total: Value::Const(3),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 3,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::GreatestToughnessYouControl)),
            },
        ]))],
        ..Default::default()
    }
}

/// Exit Specialist — {1}{U} Human Detective 2/1. Can't be blocked by power 3+.
/// Disguise {1}{U}. When turned face up, return another target creature to hand.
pub fn exit_specialist() -> CardDefinition {
    CardDefinition {
        name: "Exit Specialist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(3), Keyword::Disguise(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::OtherThanSource) },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..Default::default()
    }
}

/// Projektor Inspector — {2}{U} Human Detective 3/2. Whenever this or another
/// Detective you control enters, and whenever a Detective you control is turned
/// face up, you may draw then discard.
pub fn projektor_inspector() -> CardDefinition {
    CardDefinition {
        name: "Projektor Inspector",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Detective),
                    }),
                effect: may_loot(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Detective),
                    }),
                effect: may_loot(),
            },
        ],
        ..Default::default()
    }
}

/// Hotshot Investigators — {5}{U} Vedalken Detective 4/4. ETB: return up to one
/// other target creature to its owner's hand; if you controlled it, investigate.
pub fn hotshot_investigators() -> CardDefinition {
    CardDefinition {
        name: "Hotshot Investigators",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Detective],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::ControlledByYou,
                    },
                    then: Box::new(investigate(1)),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Move {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::OtherThanSource) },
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Frantic Scapegoat — {R} Goat 1/1, haste. ETB: suspect it. (The "move the
/// suspicion to another creature you control" upkeep-shuffle rider is dropped.)
pub fn frantic_scapegoat() -> CardDefinition {
    CardDefinition {
        name: "Frantic Scapegoat",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goat], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Suspect { what: Selector::This })],
        ..Default::default()
    }
}

/// Sanguine Savior — {1}{W}{B} Vampire Cleric 2/1, flying, lifelink. Disguise
/// {W/B}{W/B}. When turned face up, another target creature you control gains
/// lifelink until end of turn.
pub fn sanguine_savior() -> CardDefinition {
    let wb = ManaSymbol::Hybrid(Color::White, Color::Black);
    CardDefinition {
        name: "Sanguine Savior",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Disguise(cost(&[wb, wb]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                },
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
