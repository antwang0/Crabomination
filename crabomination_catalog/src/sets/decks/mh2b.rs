//! Modern Horizons 2 sweep, batch 1 — Squirrel/token package (Chatterfang,
//! Academy Manufactor), suspend artifacts, graveyard tutors, Thrasta.
//! Tests in `tests/mh2b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement, Selector, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{draw, etb, eternalize, evolve, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, StaticEffect, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};

use SelectionRequirement as R;

fn squirrel_token() -> TokenDefinition {
    TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Thrasta, Tempest's Roar — {10}{G}{G} 7/7. {3} less per other spell cast
/// this turn; trample, haste, trample over planeswalkers; hexproof the turn
/// it enters.
pub fn thrasta_tempests_roar() -> CardDefinition {
    CardDefinition {
        name: "Thrasta, Tempest's Roar",
        cost: cost(&[generic(10), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![
            Keyword::Trample,
            Keyword::Haste,
            Keyword::TrampleOverPlaneswalkers,
        ],
        static_abilities: vec![
            StaticAbility {
                description: "This spell costs {3} less to cast for each other spell cast this turn.",
                effect: StaticEffect::SelfCostReducedPerSpellCastThisTurn { per: 3 },
            },
            StaticAbility {
                description: "Thrasta has hexproof as long as it entered the battlefield this turn.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Hexproof,
                    condition: Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::EnteredThisTurn,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Academy Manufactor — {3} 1/3. A Clue, Food, or Treasure mint becomes one
/// of each.
pub fn academy_manufactor() -> CardDefinition {
    CardDefinition {
        name: "Academy Manufactor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::AssemblyWorker],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "If you would create a Clue, Food, or Treasure token, instead create one of each.",
            effect: StaticEffect::ClueFoodTreasureMintsOneOfEach,
        }],
        ..Default::default()
    }
}

/// Chatterfang, Squirrel General — {2}{G} 3/3. Forestwalk; token mints add
/// that many Squirrels; {B}, sac X Squirrels: target creature gets +X/-X.
pub fn chatterfang_squirrel_general() -> CardDefinition {
    CardDefinition {
        name: "Chatterfang, Squirrel General",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, those tokens plus that many 1/1 green Squirrel creature tokens are created instead.",
            effect: StaticEffect::TokenCreationAddsTokenPerToken {
                definition: squirrel_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Squirrel), 0)),
            sac_other_x: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::XFromCost,
                toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chatterstorm — {1}{G} Sorcery. Storm; create a 1/1 Squirrel.
pub fn chatterstorm() -> CardDefinition {
    CardDefinition {
        name: "Chatterstorm",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: squirrel_token(),
        },
        ..Default::default()
    }
}

/// Ravenous Squirrel — {B/G} 1/1. Sacrificed artifact/creature → +1/+1
/// counter; {1}{B}{G}, sac an artifact or creature: gain 1 and draw.
pub fn ravenous_squirrel() -> CardDefinition {
    CardDefinition {
        name: "Ravenous Squirrel",
        cost: cost(&[hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.or(R::Creature),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), g()]),
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                draw(1),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Squirrel Sovereign — {1}{G} 2/2. Other Squirrels you control get +1/+1.
pub fn squirrel_sovereign() -> CardDefinition {
    CardDefinition {
        name: "Squirrel Sovereign",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Squirrels you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Squirrel)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Squirrel Sanctuary — {G}. ETB: a Squirrel; a nontoken creature you
/// control dies: may pay {1} to bounce this to hand.
pub fn squirrel_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Squirrel Sanctuary",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: squirrel_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::NotToken,
                    }),
                effect: Effect::MayPay {
                    description: "Pay {1} to return Squirrel Sanctuary to your hand?".into(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Scurry Oak — {2}{G} 1/2. Evolve; +1/+1 counters put on it → may create
/// a Squirrel.
pub fn scurry_oak() -> CardDefinition {
    CardDefinition {
        name: "Scurry Oak",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![
            evolve(),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                ),
                effect: Effect::MayDo {
                    description: "Create a 1/1 green Squirrel token?".into(),
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: squirrel_token(),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Drey Keeper — {3}{B}{G} 2/2. ETB: two Squirrels; {3}{B}: Squirrels you
/// control get +1/+0 and menace this turn.
pub fn drey_keeper() -> CardDefinition {
    let squirrels = || {
        Selector::EachPermanent(R::HasCreatureType(CreatureType::Squirrel).and(R::ControlledByYou))
    };
    CardDefinition {
        name: "Drey Keeper",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: squirrel_token(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: squirrels(),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: squirrels(),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sylvan Anthem — {G}{G}. Green creatures you control get +1/+1; a green
/// creature entering scries 1.
pub fn sylvan_anthem() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Anthem",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Green creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasColor(Color::Green))
                        .and(R::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasColor(Color::Green)),
                }),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Timeless Dragon — {3}{W}{W} 5/5 flying. Plainscycling {2}; Eternalize
/// {2}{W}{W}.
pub fn timeless_dragon() -> CardDefinition {
    CardDefinition {
        name: "Timeless Dragon",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::Flying,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains),
        ],
        activated_abilities: vec![eternalize(cost(&[generic(2), w(), w()]))],
        ..Default::default()
    }
}

/// Unmarked Grave — {1}{B} Sorcery. Tutor a nonlegendary card straight to
/// the graveyard.
pub fn unmarked_grave() -> CardDefinition {
    CardDefinition {
        name: "Unmarked Grave",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::Not(Box::new(R::HasSupertype(Supertype::Legendary))),
            to: ZoneDest::Graveyard,
        },
        ..Default::default()
    }
}

/// Vile Entomber — {2}{B}{B} 2/2 deathtouch. ETB: entomb any card.
pub fn vile_entomber() -> CardDefinition {
    CardDefinition {
        name: "Vile Entomber",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Any,
            to: ZoneDest::Graveyard,
        })],
        ..Default::default()
    }
}

/// Young Necromancer — {4}{B} 2/3. ETB: may exile two cards from your
/// graveyard; if you do, reanimate a creature card.
pub fn young_necromancer() -> CardDefinition {
    CardDefinition {
        name: "Young Necromancer",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile two cards from your graveyard to reanimate a creature?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Any,
                        }),
                        count: Box::new(Value::Const(2)),
                    },
                    to: ZoneDest::Exile,
                },
                Effect::Reflexive {
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Creature),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Necrogoyf — {3}{B}{B} */4. Power = creature cards in all graveyards;
/// each upkeep that player discards; Madness {1}{B}{B}.
pub fn necrogoyf() -> CardDefinition {
    CardDefinition {
        name: "Necrogoyf",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lhurgoyf],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        dynamic_pt: Some(crate::card::DynamicPt::CreatureCardsInAllGraveyardsPower { base_t: 4 }),
        keywords: vec![Keyword::Madness(cost(&[generic(1), b(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Terminal Agony — {2}{B}{R} Sorcery. Destroy target creature. Madness
/// {B}{R}.
pub fn terminal_agony() -> CardDefinition {
    CardDefinition {
        name: "Terminal Agony",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Madness(cost(&[b(), r()]))],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature),
        },
        ..Default::default()
    }
}

/// Hard Evidence — {U} Sorcery. Create a 0/3 Crab and investigate.
pub fn hard_evidence() -> CardDefinition {
    CardDefinition {
        name: "Hard Evidence",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Crab".into(),
                    power: 0,
                    toughness: 3,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Crab],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::clue_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Brainstone — {1}. {2}, {T}, sac: Brainstorm.
pub fn brainstone() -> CardDefinition {
    CardDefinition {
        name: "Brainstone",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                draw(3),
                Effect::PutOnLibraryFromHand {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sol Talisman — Artifact with no mana cost. Suspend 3—{1}; {T}: add {C}{C}.
pub fn sol_talisman() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Sol Talisman",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1)]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gargadon — {5}{R}{R} 7/5 trample. Suspend 4—{1}{R}.
pub fn gargadon() -> CardDefinition {
    CardDefinition {
        name: "Gargadon",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 7,
        toughness: 5,
        keywords: vec![
            Keyword::Trample,
            Keyword::Suspend(4, cost(&[generic(1), r()])),
        ],
        ..Default::default()
    }
}

// ── Batch 2 — modular Arcbounds + misc value ─────────────────────────────────

use crate::effect::shortcut::{modular_dies, riot};

/// Shared Arcbound shell: 0/0 artifact creature entering with N +1/+1
/// counters and the modular dies-trigger.
fn arcbound(
    name: &'static str,
    mana: &[crate::mana::ManaSymbol],
    types: Vec<CreatureType>,
    n: i32,
    kws: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(mana),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(n))),
        keywords: {
            let mut kws = kws;
            kws.push(Keyword::Modular(n as u32));
            kws
        },
        triggered_abilities: vec![modular_dies()],
        ..Default::default()
    }
}

/// Arcbound Mouser — {W} 0/0 Cat. Lifelink, modular 1.
pub fn arcbound_mouser() -> CardDefinition {
    arcbound(
        "Arcbound Mouser",
        &[w()],
        vec![CreatureType::Cat],
        1,
        vec![Keyword::Lifelink],
    )
}

/// Arcbound Prototype — {1}{W} 0/0. Modular 2.
pub fn arcbound_prototype() -> CardDefinition {
    arcbound(
        "Arcbound Prototype",
        &[generic(1), w()],
        vec![CreatureType::AssemblyWorker],
        2,
        vec![],
    )
}

/// Arcbound Tracker — {2}{R} 0/0 Dog. Menace, modular 2; each spell after
/// your first each turn adds a +1/+1 counter.
pub fn arcbound_tracker() -> CardDefinition {
    let mut def = arcbound(
        "Arcbound Tracker",
        &[generic(2), r()],
        vec![CreatureType::Dog],
        2,
        vec![Keyword::Menace],
    );
    def.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::SpellsCastThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(2),
            },
        ),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
    });
    def
}

/// Arcbound Slasher — {4}{R} 0/0 Cat. Modular 4, riot.
pub fn arcbound_slasher() -> CardDefinition {
    let mut def = arcbound(
        "Arcbound Slasher",
        &[generic(4), r()],
        vec![CreatureType::Cat],
        4,
        vec![],
    );
    def.triggered_abilities.push(riot());
    def
}

/// Arcbound Whelp — {3}{R} 0/0 Dragon. Flying, modular 2; {R}: +1/+0.
pub fn arcbound_whelp() -> CardDefinition {
    let mut def = arcbound(
        "Arcbound Whelp",
        &[generic(3), r()],
        vec![CreatureType::Dragon],
        2,
        vec![Keyword::Flying],
    );
    def.activated_abilities = vec![ActivatedAbility {
        mana_cost: cost(&[r()]),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }];
    def
}

/// Arcbound Shikari — {1}{R}{W} 0/0 Cat Soldier. First strike, modular 2;
/// ETB: +1/+1 counter on each other artifact creature you control.
pub fn arcbound_shikari() -> CardDefinition {
    let mut def = arcbound(
        "Arcbound Shikari",
        &[generic(1), r(), w()],
        vec![CreatureType::Cat, CreatureType::Soldier],
        2,
        vec![Keyword::FirstStrike],
    );
    def.triggered_abilities.push(etb(Effect::AddCounter {
        what: Selector::EachPermanent(
            R::Artifact
                .and(R::Creature)
                .and(R::ControlledByYou)
                .and(R::OtherThanSource),
        ),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    }));
    def
}

/// General Ferrous Rokiric — {1}{R}{W} 3/1. Hexproof from monocolored;
/// casting a multicolored spell mints a 4/4 Golem.
pub fn general_ferrous_rokiric() -> CardDefinition {
    CardDefinition {
        name: "General Ferrous Rokiric",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::HexproofFromMonocolored],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Multicolored,
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Golem".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    colors: vec![Color::Red, Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Golem],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Captain Ripley Vance — {2}{R} 3/2. Your third spell each turn: +1/+1
/// counter, then she deals damage equal to her power to any target.
pub fn captain_ripley_vance() -> CardDefinition {
    CardDefinition {
        name: "Captain Ripley Vance",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::DealDamageEqualToPower {
                    source: Selector::This,
                    target: crate::effect::shortcut::target(),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Abiding Grace — {2}{W}. Your end step, choose one: gain 1 life, or
/// return a mana-value-1 creature card from your graveyard.
pub fn abiding_grace() -> CardDefinition {
    CardDefinition {
        name: "Abiding Grace",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::ChooseMode(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::ManaValueAtMost(1))),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Phantasmal Dreadmaw — {2}{U}{U} 6/6 trample. Targeted → sacrifice it.
pub fn phantasmal_dreadmaw() -> CardDefinition {
    CardDefinition {
        name: "Phantasmal Dreadmaw",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Illusion],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        ..Default::default()
    }
}

/// Flametongue Yearling — {R}{R} 2/1. Multikicker {2}; enters with a +1/+1
/// counter per kick; ETB: deals damage equal to its power to target creature.
pub fn flametongue_yearling() -> CardDefinition {
    CardDefinition {
        name: "Flametongue Yearling",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Multikicker(cost(&[generic(2)]))],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::TimesKicked)),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Jade Avenger — {1}{G} 2/2 Frog Samurai. Bushido 2.
pub fn jade_avenger() -> CardDefinition {
    CardDefinition {
        name: "Jade Avenger",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Samurai],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Bushido(2)],
        ..Default::default()
    }
}

/// Sinister Starfish — {1}{B} 0/3. {T}: Surveil 1.
pub fn sinister_starfish() -> CardDefinition {
    CardDefinition {
        name: "Sinister Starfish",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Starfish],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tavern Scoundrel — {1}{R} 1/3. Win a coin flip: two Treasures; {1}, {T},
/// sac another permanent: flip a coin.
pub fn tavern_scoundrel() -> CardDefinition {
    CardDefinition {
        name: "Tavern Scoundrel",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::WonCoinFlip, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: crabomination_base::tokens::treasure_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_other_filter: Some((R::Permanent, 1)),
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Noop),
                on_tails: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kaleidoscorch — {1}{R} Sorcery. Converge — X damage to any target;
/// Flashback {4}{R}.
pub fn kaleidoscorch() -> CardDefinition {
    CardDefinition {
        name: "Kaleidoscorch",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r()]))],
        effect: Effect::DealDamage {
            to: crate::effect::shortcut::target(),
            amount: Value::ConvergedValue,
        },
        ..Default::default()
    }
}

/// Myr Scrapling — {1} 1/1 Myr. Sacrifice: +1/+1 counter on target creature.
pub fn myr_scrapling() -> CardDefinition {
    CardDefinition {
        name: "Myr Scrapling",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Myr],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tormod's Cryptkeeper — {3} 3/2 vigilance. {T}, sac: exile target
/// player's graveyard.
pub fn tormods_cryptkeeper() -> CardDefinition {
    CardDefinition {
        name: "Tormod's Cryptkeeper",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::Target(0),
                    zone: Zone::Graveyard,
                    filter: R::Any,
                },
                to: ZoneDest::Exile,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Legion Vanguard — {1}{B} 2/2. {1}, sac another creature: explores.
pub fn legion_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Legion Vanguard",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Explore {
                who: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vermin Gorger — {1}{B} 2/2. {T}, sac another creature: each opponent
/// loses 2, you gain 2.
pub fn vermin_gorger() -> CardDefinition {
    CardDefinition {
        name: "Vermin Gorger",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Underworld Hermit — {4}{B}{B} 3/3. ETB: Squirrels equal to your devotion
/// to black.
pub fn underworld_hermit() -> CardDefinition {
    CardDefinition {
        name: "Underworld Hermit",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::DevotionTo(vec![Color::Black]),
            definition: squirrel_token(),
        })],
        ..Default::default()
    }
}

// ── Batch 3 — commons/uncommons on existing primitives ───────────────────────

/// Late to Dinner — {3}{W} Sorcery. Reanimate a creature card; create a Food.
pub fn late_to_dinner() -> CardDefinition {
    CardDefinition {
        name: "Late to Dinner",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::food_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Kitchen Imp — {3}{B} 2/2 flying haste. Madness {B}.
pub fn kitchen_imp() -> CardDefinition {
    CardDefinition {
        name: "Kitchen Imp",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![
            Keyword::Flying,
            Keyword::Haste,
            Keyword::Madness(cost(&[b()])),
        ],
        ..Default::default()
    }
}

/// Hell Mongrel — {3}{B} 4/3. Discard a card: +1/+1 EOT. Madness {2}{B}.
pub fn hell_mongrel() -> CardDefinition {
    CardDefinition {
        name: "Hell Mongrel",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Dog],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Madness(cost(&[generic(2), b()]))],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skophos Reaver — {2}{R} 2/3. +2/+0 during your turn. Madness {1}{R}.
pub fn skophos_reaver() -> CardDefinition {
    CardDefinition {
        name: "Skophos Reaver",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Madness(cost(&[generic(1), r()]))],
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature gets +2/+0.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Foul Watcher — {1}{U} 1/2 flying. ETB surveil 1; delirium +1/+0.
pub fn foul_watcher() -> CardDefinition {
    CardDefinition {
        name: "Foul Watcher",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        static_abilities: vec![StaticAbility {
            description: "Delirium — This creature gets +1/+0 as long as there are four or more card types among cards in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::DeliriumActive {
                    who: PlayerRef::You,
                },
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Rift Sower — {2}{G} 1/3. {T}: any color. Suspend 2—{G}.
pub fn rift_sower() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Rift Sower",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Suspend(2, cost(&[g()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Orchard Strider — {4}{G}{G} 6/4. ETB: two Foods; basic landcycling {1}{G}.
pub fn orchard_strider() -> CardDefinition {
    CardDefinition {
        name: "Orchard Strider",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 6,
        toughness: 4,
        keywords: vec![Keyword::Typecycling(Box::new((
            cost(&[generic(1), g()]),
            R::IsBasicLand,
        )))],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: crabomination_base::tokens::food_token(),
        })],
        ..Default::default()
    }
}

/// Terramorph — {3}{G} Sorcery. Fetch a basic onto the battlefield; rebound.
pub fn terramorph() -> CardDefinition {
    CardDefinition {
        name: "Terramorph",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Rebound],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        ..Default::default()
    }
}

/// Funnel-Web Recluse — {4}{G} 3/5 reach. Morbid ETB: investigate.
pub fn funnel_web_recluse() -> CardDefinition {
    CardDefinition {
        name: "Funnel-Web Recluse",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::ONE,
            },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::clue_token(),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Urban Daggertooth — {2}{G}{G} 4/3 vigilance. Enrage: proliferate.
pub fn urban_daggertooth() -> CardDefinition {
    CardDefinition {
        name: "Urban Daggertooth",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Jewel-Eyed Cobra — {2}{G} 3/1 deathtouch. Dies: a Treasure.
pub fn jewel_eyed_cobra() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Jewel-Eyed Cobra",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: crabomination_base::tokens::treasure_token(),
        })],
        ..Default::default()
    }
}

/// Healer's Flock — {W}{W}{W} 3/3 flying lifelink.
pub fn healers_flock() -> CardDefinition {
    CardDefinition {
        name: "Healer's Flock",
        cost: cost(&[w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Disciple of the Sun — {4}{W} 3/3 lifelink. ETB: return a MV≤3 permanent
/// card from your graveyard to hand.
pub fn disciple_of_the_sun() -> CardDefinition {
    CardDefinition {
        name: "Disciple of the Sun",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Permanent.and(R::ManaValueAtMost(3))),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Fairgrounds Patrol — {1}{W} 2/1. {1}{W}, exile from graveyard: a 1/1
/// flying Thopter. Sorcery speed.
pub fn fairgrounds_patrol() -> CardDefinition {
    CardDefinition {
        name: "Fairgrounds Patrol",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Thopter".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    keywords: vec![Keyword::Flying],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Thopter],
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

/// Knighted Myr — {2}{W} 2/2. {2}{W}: adapt 1; counters put on it grant
/// double strike this turn.
pub fn knighted_myr() -> CardDefinition {
    use crate::effect::shortcut::adapt;
    CardDefinition {
        name: "Knighted Myr",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Myr, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: adapt(1),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::SelfSource,
            ),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Soul of Migration — {5}{W}{W} 2/4 flying. ETB: two 1/1 flying Birds.
/// Evoke {3}{W}.
pub fn soul_of_migration() -> CardDefinition {
    CardDefinition {
        name: "Soul of Migration",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(crate::effect::shortcut::evoke(cost(&[generic(3), w()]))),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Bird".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                keywords: vec![Keyword::Flying],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Bird],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Thraben Watcher — {2}{W}{W} 2/2 flying vigilance. Other nontoken
/// creatures you control get +1/+1 and have vigilance.
pub fn thraben_watcher() -> CardDefinition {
    let others = Selector::EachPermanent(
        R::Creature
            .and(R::ControlledByYou)
            .and(R::NotToken)
            .and(R::OtherThanSource),
    );
    CardDefinition {
        name: "Thraben Watcher",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "Other nontoken creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: others.clone(),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Other nontoken creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others,
                    keyword: Keyword::Vigilance,
                },
            },
        ],
        ..Default::default()
    }
}

/// Floodhound — {U} 1/2. {3}, {T}: investigate.
pub fn floodhound() -> CardDefinition {
    CardDefinition {
        name: "Floodhound",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Dog],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::clue_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mental Journey — {4}{U}{U} Instant. Draw three; basic landcycling {1}{U}.
pub fn mental_journey() -> CardDefinition {
    CardDefinition {
        name: "Mental Journey",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Typecycling(Box::new((
            cost(&[generic(1), u()]),
            R::IsBasicLand,
        )))],
        effect: draw(3),
        ..Default::default()
    }
}

/// Steelfin Whale — {5}{U} 3/4. Affinity for artifacts; an artifact ETB
/// untaps it.
pub fn steelfin_whale() -> CardDefinition {
    CardDefinition {
        name: "Steelfin Whale",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Whale],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        affinity_filter: Some(R::Artifact),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::Untap {
                what: Selector::This,
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Tragic Fall — {1}{B} Instant. -3/-3; hellbent: -13/-13 instead.
pub fn tragic_fall() -> CardDefinition {
    let shrink = |n: i32| {
        Box::new(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(n),
            toughness: Value::Const(n),
            duration: Duration::EndOfTurn,
        })
    };
    CardDefinition {
        name: "Tragic Fall",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::HellbentActive {
                who: PlayerRef::You,
            },
            then: shrink(-13),
            else_: shrink(-3),
        },
        ..Default::default()
    }
}

/// Echoing Return — {B} Sorcery. Return a creature card and its namesakes
/// from your graveyard to hand.
pub fn echoing_return() -> CardDefinition {
    CardDefinition {
        name: "Echoing Return",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::SharingNameWith(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature,
            })),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Lens Flare — {4}{W} Instant. Affinity for artifacts; 5 damage to target
/// attacking or blocking creature.
pub fn lens_flare() -> CardDefinition {
    CardDefinition {
        name: "Lens Flare",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(R::Artifact),
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}
