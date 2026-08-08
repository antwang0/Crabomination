//! Aetherdrift (DFT) Speed/Exhaust staples plus a few gap cards. Two new engine
//! pieces exercised here: `Value::PlayerSpeed` (Momentum Breaker's "gain life
//! equal to your speed") and the `EventKind::ExhaustAbilityActivated` trigger
//! (Adrenaline Jockey). Tests in `crabomination/src/tests/recent167.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, SpendRestriction, b, cost, g, generic, r, u, w, x};

/// The shared "Max speed — {N}, Exile this card from your graveyard: Draw a
/// card" ability the DFT Surveyor cycle prints.
fn max_speed_gy_draw(mana: u32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(mana)]),
        from_graveyard: true,
        exile_self_cost: true,
        condition: Some(Predicate::SpeedAtLeast {
            who: PlayerRef::You,
            speed: 4,
        }),
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        },
        ..Default::default()
    }
}

/// Leonin Surveyor — {1}{W} 2/2 Cat Scout. Start your engines! During your turn
/// it has first strike. Max speed — {3}, Exile it from your graveyard: draw.
pub fn leonin_surveyor() -> CardDefinition {
    CardDefinition {
        name: "Leonin Surveyor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::FirstStrike,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        activated_abilities: vec![max_speed_gy_draw(3)],
        ..Default::default()
    }
}

/// Loxodon Surveyor — {2}{G} 3/3 Elephant Scout. Start your engines! Max speed —
/// {3}, Exile it from your graveyard: draw.
pub fn loxodon_surveyor() -> CardDefinition {
    CardDefinition {
        name: "Loxodon Surveyor",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![max_speed_gy_draw(3)],
        ..Default::default()
    }
}

/// Mutant Surveyor — {2}{B} 2/3 Mutant Scout. Start your engines! {2}: +1/+1.
/// Max speed — {3}, Exile it from your graveyard: draw.
pub fn mutant_surveyor() -> CardDefinition {
    CardDefinition {
        name: "Mutant Surveyor",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            max_speed_gy_draw(3),
        ],
        ..Default::default()
    }
}

/// Ooze Patrol — {3}{G} 2/2 Ooze. ETB: mill two, then put a +1/+1 counter on it
/// for each artifact and/or creature card in your graveyard.
pub fn ooze_patrol() -> CardDefinition {
    CardDefinition {
        name: "Ooze Patrol",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::HasCardType(CardType::Artifact)
                        .or(R::HasCardType(CardType::Creature)),
                },
            },
        ]))],
        ..Default::default()
    }
}

/// Marketback Walker — {X}{X} 0/0 Construct artifact creature. Enters with X
/// +1/+1 counters. {4}: put a +1/+1 counter on it. Dies: draw a card for each
/// +1/+1 counter on it.
pub fn marketback_walker() -> CardDefinition {
    CardDefinition {
        name: "Marketback Walker",
        cost: cost(&[x(), x()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        }],
        ..Default::default()
    }
}

/// Momentum Breaker — {1}{B} Enchantment. Start your engines! ETB: each opponent
/// sacrifices a creature or Vehicle of their choice. {2}, Sacrifice this: gain
/// life equal to your speed. (The "who can't discards a card" rider is dropped.)
pub fn momentum_breaker() -> CardDefinition {
    CardDefinition {
        name: "Momentum Breaker",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::PlayerSpeed(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hour of Victory — {2}{B} Enchantment. Start your engines! ETB: create a 2/2
/// black Zombie. Max speed — {1}{B}, Sacrifice this: search your library for a
/// card, put it into your hand, then shuffle. Sorcery speed.
pub fn hour_of_victory() -> CardDefinition {
    CardDefinition {
        name: "Hour of Victory",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(TokenDefinition {
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
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            sorcery_speed: true,
            condition: Some(Predicate::SpeedAtLeast {
                who: PlayerRef::You,
                speed: 4,
            }),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Intimidation Tactics — {B} Sorcery. Target opponent reveals their hand; you
/// choose an artifact or creature card from it and exile it. Cycling {3}.
pub fn intimidation_tactics() -> CardDefinition {
    CardDefinition {
        name: "Intimidation Tactics",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        effect: Effect::ExileChosenFromHand {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::HasCardType(CardType::Artifact).or(R::HasCardType(CardType::Creature)),
            link_to_source: false,
            face_down: false,
        },
        ..Default::default()
    }
}

/// Adrenaline Jockey — {2}{R} 3/3 Minotaur Pilot. Whenever a player casts a
/// spell, if it's not their turn, this creature deals 4 damage to them. Whenever
/// you activate an exhaust ability, put a +1/+1 counter on this creature.
pub fn adrenaline_jockey() -> CardDefinition {
    CardDefinition {
        name: "Adrenaline Jockey",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Pilot],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
                effect: Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::Triggerer))),
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(4),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ExhaustAbilityActivated, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

// ── DFT Speed lands ─────────────────────────────────────────────────────────

/// Avishkar Raceway — Land. Start your engines! {T}: Add {C}. Max speed — {3},
/// {T}, Discard a card: Draw a card.
pub fn avishkar_raceway() -> CardDefinition {
    CardDefinition {
        name: "Avishkar Raceway",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                condition: Some(Predicate::SpeedAtLeast {
                    who: PlayerRef::You,
                    speed: 4,
                }),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Muraganda Raceway — Land. Start your engines! {T}: Add {C}. Max speed —
/// {T}: Add {C}{C}.
pub fn muraganda_raceway() -> CardDefinition {
    CardDefinition {
        name: "Muraganda Raceway",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::SpeedAtLeast {
                    who: PlayerRef::You,
                    speed: 4,
                }),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(2)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Night Market — Land. Enters tapped. As it enters, choose a color. {T}: Add
/// one mana of the chosen color. Cycling {3}.
pub fn night_market() -> CardDefinition {
    CardDefinition {
        name: "Night Market",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::ChosenColorOfSource,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── DFT Vehicles ────────────────────────────────────────────────────────────

/// Marshals' Pathcruiser — {3} Vehicle 6/5. ETB: search your library for a basic
/// land card and put it into your hand. Exhaust — {W}{U}{B}{R}{G}: becomes an
/// artifact creature; put two +1/+1 counters on it. Crew 5.
pub fn marshals_pathcruiser() -> CardDefinition {
    CardDefinition {
        name: "Marshals' Pathcruiser",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Crew(5)],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCardTypeIndefinitely {
                    what: Selector::This,
                    card_type: CardType::Creature,
                    until_eot: false,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Boommobile — {2}{R}{R} Vehicle 5/5. ETB: add four mana of any one color;
/// spend it only to activate abilities. Exhaust — {X}{2}{R}: deal X damage to any
/// target; put a +1/+1 counter on it. Crew 2.
pub fn boommobile() -> CardDefinition {
    CardDefinition {
        name: "Boommobile",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Restricted(
                Box::new(ManaPayload::AnyOneColor(Value::Const(4))),
                SpendRestriction::AbilitiesOnly,
            ),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), generic(2), r()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_any(),
                    amount: Value::XFromCost,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Howlsquad Heavy — {2}{R} 2/3 Goblin Mercenary. Start your engines! Other
/// Goblins you control have haste. Beginning of combat on your turn: create a
/// 1/1 red Goblin. Max speed — {T}: Add {R} for each Goblin you control.
/// (The token's "attacks this combat if able" rider is dropped.)
pub fn howlsquad_heavy() -> CardDefinition {
    let goblin = || TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Howlsquad Heavy",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "Other Goblins you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Goblin)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(goblin()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::SpeedAtLeast {
                who: PlayerRef::You,
                speed: 4,
            }),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Red,
                    Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Goblin).and(R::ControlledByYou),
                        )),
                        filter: R::Any,
                    },
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── DFT cycling + attack-trigger creatures ──────────────────────────────────

/// Boosted Sloop — {1}{U}{R} Vehicle 3/3 with menace. Whenever you attack, draw
/// a card, then discard a card. Crew 1.
pub fn boosted_sloop() -> CardDefinition {
    CardDefinition {
        name: "Boosted Sloop",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Howler's Heavy — {3}{U} 3/4 Seal Pirate. Cycling {1}{U}; when you cycle it,
/// target creature or Vehicle an opponent controls gets -3/-0 until end of turn.
pub fn howlers_heavy() -> CardDefinition {
    CardDefinition {
        name: "Howler's Heavy",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Seal, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                        .and(R::ControlledByOpponent),
                },
                power: Value::Const(-3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── DFT value creatures + spell ─────────────────────────────────────────────

/// Wreckage Wickerfolk — {1}{B} 1/3 Scarecrow artifact creature. Flying. ETB:
/// surveil 2.
pub fn wreckage_wickerfolk() -> CardDefinition {
    CardDefinition {
        name: "Wreckage Wickerfolk",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scarecrow],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Transit Mage — {2}{U} 2/2 Human Wizard. ETB: you may search your library for
/// an artifact card with mana value 4 or 5 and put it into your hand.
pub fn transit_mage() -> CardDefinition {
    CardDefinition {
        name: "Transit Mage",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Artifact)
                .and(R::ManaValueExactly(4).or(R::ManaValueExactly(5))),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Veteran Beastrider — {1}{G}{W} 3/4 Human Knight. At the beginning of your end
/// step, untap each creature you control. {2}{G}{W}: creatures you control get
/// +1/+1 until end of turn.
pub fn veteran_beastrider() -> CardDefinition {
    CardDefinition {
        name: "Veteran Beastrider",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Untap {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                up_to: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ticket Tortoise — {2} 3/1 Turtle artifact creature with defender. ETB: if an
/// opponent controls more lands than you, create a Treasure token.
pub fn ticket_tortoise() -> CardDefinition {
    CardDefinition {
        name: "Ticket Tortoise",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Turtle],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::OpponentControlsMoreLandsThanYou,
            then: Box::new(crate::effect::shortcut::mint_treasures(1)),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Haunt the Network — {3}{U}{B} Sorcery. Choose target opponent. Create two 1/1
/// colorless Thopter artifact creature tokens with flying. Then that player loses
/// X life and you gain X life, where X is the number of artifacts you control.
pub fn haunt_the_network() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Haunt the Network",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Box::new(thopter),
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::Target(0)),
                to: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::HasCardType(CardType::Artifact).and(R::ControlledByYou),
                ))),
            },
        ]),
        ..Default::default()
    }
}
