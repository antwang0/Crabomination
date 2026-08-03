//! Outlaws of Thunder Junction staples (non-Spree): crime payoffs, Mounts,
//! Plot bodies, and utility spells. All ride existing engine primitives. Tests
//! in `tests/recent66.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, LandType, Predicate, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w};

fn attack_while_saddled(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
            .with_filter(Predicate::SourceSaddled),
        effect,
    }
}

fn mount(types: Vec<CreatureType>) -> Subtypes {
    let mut creature_types = types;
    creature_types.push(CreatureType::Mount);
    Subtypes {
        creature_types,
        ..Default::default()
    }
}

/// Vengeful Townsfolk — {2}{W} 3/3 Human Citizen. Whenever one or more other
/// creatures you control die, put a +1/+1 counter on this creature.
pub fn vengeful_townsfolk() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Townsfolk",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        // "one or more other creatures you control die" is a batch trigger;
        // `once_per_turn` approximates the once-per-death-event cap (a lone
        // self-death is a no-op counter on the departing card, so no filter).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).once_per_turn(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Loan Shark — {3}{U} 3/4 Shark Rogue. ETB: if you've cast two or more spells
/// this turn, draw a card. Plot {3}{U}.
pub fn loan_shark() -> CardDefinition {
    CardDefinition {
        name: "Loan Shark",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shark, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellsCastThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(2),
            },
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        plot_cost: Some(cost(&[generic(3), u()])),
        ..Default::default()
    }
}

/// Servant of the Stinger — {1}{B} 1/3 Human Warlock. Deathtouch. Whenever it
/// deals combat damage to a player, if you've committed a crime this turn, you
/// may sacrifice it to search your library for a card, put it into your hand.
pub fn servant_of_the_stinger() -> CardDefinition {
    CardDefinition {
        name: "Servant of the Stinger",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::CommittedCrimeThisTurn {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::MayDo {
                    description: "Sacrifice Servant of the Stinger to search your library?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::Search {
                            who: PlayerRef::You,
                            filter: R::Any,
                            to: ZoneDest::Hand(PlayerRef::You),
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Rattleback Apothecary — {2}{B} 3/2 Gorgon Warlock. Deathtouch. Whenever you
/// commit a crime (once each turn), target creature you control gains your
/// choice of menace or lifelink until end of turn.
pub fn rattleback_apothecary() -> CardDefinition {
    CardDefinition {
        name: "Rattleback Apothecary",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gorgon, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::ChooseMode(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Wrangler of the Damned — {3}{W}{U} 1/4 Human Soldier. Flash. At the beginning
/// of your end step, if you haven't cast a spell this turn, create a 2/2 white
/// Spirit with flying. (Approximated: any spell cast, not only from hand.)
pub fn wrangler_of_the_damned() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Wrangler of the Damned",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SpellsCastThisTurnEquals {
                who: PlayerRef::You,
                count: Value::ZERO,
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Spirit".into(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spirit],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Bounding Felidar — {5}{W} 4/7 Cat Beast Mount. Saddle 2. Attacks while
/// saddled → put a +1/+1 counter on each other creature you control; gain 1
/// life for each of those creatures.
pub fn bounding_felidar() -> CardDefinition {
    let others =
        || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        name: "Bounding Felidar",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: mount(vec![CreatureType::Cat, CreatureType::Beast]),
        power: 4,
        toughness: 7,
        keywords: vec![Keyword::Saddle(2)],
        triggered_abilities: vec![attack_while_saddled(Effect::Seq(vec![
            Effect::AddCounter {
                what: others(),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::count(others()),
            },
        ]))],
        ..Default::default()
    }
}

/// Trained Arynx — {1}{W} 3/1 Cat Beast Mount. Saddle 2. Attacks while saddled
/// → gains first strike until end of turn and scry 1.
pub fn trained_arynx() -> CardDefinition {
    CardDefinition {
        name: "Trained Arynx",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: mount(vec![CreatureType::Cat, CreatureType::Beast]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Saddle(2)],
        triggered_abilities: vec![attack_while_saddled(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Rambling Possum — {2}{G} 3/3 Possum Mount. Saddle 1. Attacks while saddled →
/// gets +1/+2 until end of turn. (The optional return-the-saddlers rider is
/// omitted.)
pub fn rambling_possum() -> CardDefinition {
    CardDefinition {
        name: "Rambling Possum",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: mount(vec![CreatureType::Possum]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Saddle(1)],
        triggered_abilities: vec![attack_while_saddled(Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Quick Draw — {R} Instant. Target creature you control gets +1/+1 and gains
/// first strike until end of turn. Creatures target opponent controls lose
/// first strike and double strike until end of turn.
pub fn quick_draw() -> CardDefinition {
    CardDefinition {
        name: "Quick Draw",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::ControlledBy {
                    who: PlayerRef::Target(1),
                    filter: R::Creature,
                },
                keyword: Keyword::FirstStrike,
            },
            Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::ControlledBy {
                    who: PlayerRef::Target(1),
                    filter: R::Creature,
                },
                keyword: Keyword::DoubleStrike,
            },
        ]),
        ..Default::default()
    }
}

/// Desert's Due — {1}{B} Instant. Target creature gets -2/-2 until end of turn,
/// and an additional -1/-1 for each Desert you control.
pub fn deserts_due() -> CardDefinition {
    let deserts = Value::count(Selector::EachPermanent(
        R::HasLandType(LandType::Desert).and(R::ControlledByYou),
    ));
    let debuff = Value::Diff(Box::new(Value::Const(-2)), Box::new(deserts));
    CardDefinition {
        name: "Desert's Due",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: debuff.clone(),
            toughness: debuff,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Prickly Pair — {2}{R} 2/2 Plant Mercenary. ETB create a 1/1 red Mercenary
/// with "{T}: Target creature you control gets +1/+0 until end of turn.
/// Activate only as a sorcery."
pub fn prickly_pair() -> CardDefinition {
    CardDefinition {
        name: "Prickly Pair",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: mercenary_token(),
        })],
        ..Default::default()
    }
}

/// The Weatherseed Treaty — {2}{G} Saga with Read Ahead (CR 702.155 / 714).
/// I: search your library for a basic land, put it onto the battlefield tapped.
/// II: create a 1/1 green Saproling. III: Domain — target creature you control
/// gets +X/+X and gains trample until end of turn, where X is the number of
/// basic land types among lands you control.
pub fn the_weatherseed_treaty() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "The Weatherseed Treaty",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
            ..Default::default()
        },
        read_ahead: true,
        saga_chapters: vec![
            (
                1,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
            ),
            (
                2,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
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
                    },
                },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        power: Value::DomainCount(PlayerRef::You),
                        toughness: Value::DomainCount(PlayerRef::You),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// The 1/1 red Mercenary token minted by Prickly Pair.
fn mercenary_token() -> TokenDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::Color;
    TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
