//! Commander: the cards the **Cabaretti Cacophony** precon (NCC, Kitt Kanto,
//! Mayhem Diva) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Excess** — counts creatures you control now that dealt damage to a
//!   player this turn (combat or not).
//! - **Killer Service** — the token sacrificed is the engine's pick.
//! - **Sizzling Soloist** — "attacks during its controller's next combat" is
//!   must-attack until your next turn.
//! - **Vivien's Stampede** — the draw happens at end of combat, not at the
//!   next main phase.
//! - **Zurzoth** — the Devils' draw-and-discard reaches the defending player
//!   of the attack, one player per batch.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, Keyword, SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, mint_treasures, on_attack, target_any, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate,
    VoteOption,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn citizen() -> Arc<TokenDefinition> {
    Arc::new(token("Citizen", vec![Color::Green, Color::White], vec![CreatureType::Citizen], 1, 1))
}

fn rhino() -> TokenDefinition {
    token(
        "Rhino Warrior",
        vec![Color::Green],
        vec![CreatureType::Rhino, CreatureType::Warrior],
        4,
        4,
    )
}

fn make(who: PlayerRef, count: Value, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who, count, definition: def }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn treasure_for(who: PlayerRef) -> Effect {
    make(who, Value::ONE, Arc::new(crabomination_base::tokens::treasure_token()))
}

/// "Whenever another creature you control enters, `effect`" (Alliance).
fn alliance(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
        ),
        effect,
    }
}

/// Bess, Soul Nourisher — base-1/1 creatures entering grow it; attacking,
/// your other base-1/1s get +X/+X for its +1/+1 counters.
pub fn bess_soul_nourisher() -> CardDefinition {
    let one_one = || R::Creature.and(R::BasePowerToughnessIs(1, 1));
    let x = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: one_one() })
                    .once_per_batch(),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            on_attack(Effect::PumpPT {
                what: yours(one_one().and(R::OtherThanSource)),
                power: x(),
                toughness: x(),
                duration: Duration::EndOfTurn,
            }),
        ],
        ..creature(
            "Bess, Soul Nourisher",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            1,
            1,
        )
    }
}

/// Cabaretti Charm — choose one: damage equal to your creatures to a creature
/// or planeswalker; your creatures +1/+1 and trample; two Citizens.
pub fn cabaretti_charm() -> CardDefinition {
    spell(
        "Cabaretti Charm",
        cost(&[r(), g(), w()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Planeswalker)),
                amount: Value::CountOf(Box::new(yours(R::Creature))),
            },
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: yours(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: yours(R::Creature),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            make(PlayerRef::You, Value::Const(2), citizen()),
        ]),
    )
}

/// Cabaretti Confluence — choose three, repeats allowed: a hasty sacrificed
/// copy of your creature; exile an artifact or enchantment; a player's
/// creatures get +1/+1 and first strike.
pub fn cabaretti_confluence() -> CardDefinition {
    spell(
        "Cabaretti Confluence",
        cost(&[generic(3), r(), g(), w()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            min: 3,
            max: 3,
            allow_repeats: true,
            modes: vec![
                Effect::CreateTokenCopiesHasteSac {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    exile: false,
                },
                Effect::Exile { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::ControlledBy {
                            who: PlayerRef::ControllerOf(Box::new(target_filtered(R::Player))),
                            filter: R::Creature,
                        },
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::ControlledBy {
                            who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                            filter: R::Creature,
                        },
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ],
        },
    )
}

/// Crash the Party — a tapped 4/4 Rhino Warrior per tapped creature you
/// control.
pub fn crash_the_party() -> CardDefinition {
    spell(
        "Crash the Party",
        cost(&[generic(5), g()]),
        CardType::Instant,
        make(
            PlayerRef::You,
            Value::CountOf(Box::new(yours(R::Creature.and(R::Tapped)))),
            Arc::new(TokenDefinition { tapped: true, ..rhino() }),
        ),
    )
}

/// False Floor — enters tapped; creatures enter tapped; {2}, {T}, exile it:
/// exile every untapped creature, as a sorcery.
pub fn false_floor() -> CardDefinition {
    CardDefinition {
        name: "False Floor",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "This artifact enters tapped.",
                effect: StaticEffect::EntersTapped { applies_to: Selector::This },
            },
            StaticAbility {
                description: "Creatures enter tapped.",
                effect: StaticEffect::EntersTapped { applies_to: Selector::EachPermanent(R::Creature) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Exile { what: Selector::EachPermanent(R::Creature.and(R::Untapped)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Indulge // Excess — Indulge: this turn, each attack by a creature of
/// yours makes a tapped attacking Citizen. Excess (aftermath): a Treasure per
/// creature of yours that dealt combat damage to a player this turn.
///
/// Approximation: Excess counts your creatures that damaged a player.
pub fn indulge_excess() -> CardDefinition {
    CardDefinition {
        name: "Indulge // Excess",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::OnMatchingAttacksThisTurn {
            filter: R::Creature.and(R::ControlledByYou),
            body: Box::new(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: citizen(),
                cleanup: Default::default(),
                defender: None,
            }),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(1), r()]),
                card_types: vec![CardType::Sorcery],
                effect: make(
                    PlayerRef::You,
                    Value::CountOf(Box::new(yours(R::Creature.and(R::DamagedAPlayerThisTurn)))),
                    Arc::new(crabomination_base::tokens::treasure_token()),
                ),
            },
            fuse: false,
            aftermath: true,
        })),
        ..Default::default()
    }
}

/// Killer Service — a Food per opponent on entry; at your end step you may
/// pay {2} and sacrifice a token for a 4/4 Rhino Warrior.
///
/// Approximation: the token sacrificed is the engine's pick.
pub fn killer_service() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(make(
                PlayerRef::You,
                Value::OpponentCount,
                Arc::new(crabomination_base::tokens::food_token()),
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountOf(Box::new(yours(R::IsToken))),
                        Value::ONE,
                    ),
                    then: Box::new(Effect::MayPay {
                        description: "Pay {2} and sacrifice a token for a 4/4 Rhino Warrior?".into(),
                        mana_cost: cost(&[generic(2)]),
                        // One ask: a token is guaranteed by the gate above, so
                        // the sacrifice is a plain one (a nested `MaySacrifice`
                        // would replay the pay answer — structural audit).
                        body: Box::new(Effect::Seq(vec![
                            Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::IsToken },
                            make(PlayerRef::You, Value::ONE, Arc::new(rhino())),
                        ])),
                        else_: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..spell("Killer Service", cost(&[generic(2), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Life of the Party — first strike, trample, haste; attacking, +X/+0 for
/// your creatures; cast (not a token), each opponent gets a goaded copy.
pub fn life_of_the_party() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![
            on_attack(Effect::PumpPT {
                what: Selector::This,
                power: Value::CountOf(Box::new(yours(R::Creature))),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::IsToken,
                    }))),
                effect: Effect::ForEachOpponent {
                    body: Box::new(Effect::Seq(vec![
                        Effect::CreateTokenCopyOf {
                            who: PlayerRef::Triggerer,
                            count: Value::ONE,
                            source: Selector::This,
                            extra_creature_types: vec![],
                            extra_card_types: vec![],
                            override_pt: None,
                            override_colors: None,
                            enters_tapped: false,
                            non_legendary: false,
                            legendary: false,
                            extra_keywords: vec![],
                        },
                        Effect::GoadForTheGame { what: Selector::LastCreatedToken },
                    ])),
                },
            },
        ],
        ..creature("Life of the Party", cost(&[generic(3), r()]), vec![CreatureType::Elemental], 0, 1)
    }
}

/// Master of Ceremonies — each upkeep, each opponent chooses money, friends
/// or secrets, and you and they each get a Treasure, a Citizen or a card.
pub fn master_of_ceremonies() -> CardDefinition {
    let both = |mk: fn(PlayerRef) -> Effect| Effect::Seq(vec![mk(PlayerRef::You), mk(PlayerRef::CurrentVoter)]);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::EachOpponentChooses {
                prompt: "Choose money, friends, or secrets".into(),
                options: vec![
                    VoteOption::new("Money", both(treasure_for)),
                    VoteOption::new("Friends", both(|p| make(p, Value::ONE, citizen()))),
                    VoteOption::new("Secrets", both(|p| Effect::Draw {
                        who: Selector::Player(p),
                        amount: Value::ONE,
                    })),
                ],
            },
        }],
        ..creature(
            "Master of Ceremonies",
            cost(&[generic(3), w()]),
            vec![CreatureType::Rhino, CreatureType::Druid],
            3,
            4,
        )
    }
}

/// Prosperous Partnership — two Citizens on entry; tap three untapped
/// creatures you control: a Treasure.
pub fn prosperous_partnership() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(PlayerRef::You, Value::Const(2), citizen()))],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Creature.and(R::ControlledByYou).and(R::Untapped), 3)),
            effect: mint_treasures(1),
            ..Default::default()
        }],
        ..spell("Prosperous Partnership", cost(&[generic(1), r(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Rumor Gatherer — alliance: scry 1; the second time each turn, draw
/// instead.
pub fn rumor_gatherer() -> CardDefinition {
    let scry = || Effect::Scry { who: PlayerRef::You, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![alliance(Effect::EscalatingThisTurn {
            modes: vec![scry(), draw(1), scry()],
        })],
        ..creature(
            "Rumor Gatherer",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Elf, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Scepter of Celebration — equipped creature +2/+0 and trample; its combat
/// damage to a player makes that many Citizens. Equip {3}.
pub fn scepter_of_celebration() -> CardDefinition {
    CardDefinition {
        name: "Scepter of Celebration",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            keywords: vec![Keyword::Trample],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: make(PlayerRef::You, Value::TriggerEventAmount, citizen()),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Seize the Spotlight — each opponent chooses fame (you borrow one of
/// their creatures, untapped and hasty, this turn) or fortune (you draw and
/// make a Treasure).
pub fn seize_the_spotlight() -> CardDefinition {
    let chosen = || Selector::SeparatedPile { chosen: true };
    spell(
        "Seize the Spotlight",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::EachOpponentChooses {
            prompt: "Choose fame or fortune".into(),
            options: vec![
                VoteOption::new(
                    "Fame",
                    Effect::ChooseOneAmong {
                        what: Selector::ControlledBy { who: PlayerRef::CurrentVoter, filter: R::Creature },
                        chooser: PlayerRef::You,
                        chosen: Box::new(Effect::Seq(vec![
                            Effect::GainControl {
                                what: chosen(),
                                to: Some(PlayerRef::You),
                                duration: Duration::EndOfTurn,
                            },
                            Effect::Untap { what: chosen(), up_to: None },
                            Effect::GrantKeyword {
                                what: chosen(),
                                keyword: Keyword::Haste,
                                duration: Duration::EndOfTurn,
                            },
                        ])),
                        other: Box::new(Effect::Noop),
                    },
                ),
                VoteOption::new("Fortune", Effect::Seq(vec![draw(1), treasure_for(PlayerRef::You)])),
            ],
        },
    )
}

/// Sizzling Soloist — alliance: an opponent's creature can't block this
/// turn; the second time each turn it also attacks next combat if able.
///
/// Approximation: must-attack until your next turn.
pub fn sizzling_soloist() -> CardDefinition {
    let cant_block = || Effect::GrantKeyword {
        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        keyword: Keyword::CantBlock,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![alliance(Effect::EscalatingThisTurn {
            modes: vec![
                cant_block(),
                Effect::Seq(vec![
                    cant_block(),
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::MustAttack,
                        duration: Duration::UntilNextTurn,
                    },
                ]),
                cant_block(),
            ],
        })],
        ..creature(
            "Sizzling Soloist",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            3,
            2,
        )
    }
}

/// Vivien's Stampede — your creatures gain vigilance, trample and melee;
/// later this turn, draw a card per player dealt combat damage.
///
/// Approximation: the draw comes at end of combat.
pub fn viviens_stampede() -> CardDefinition {
    spell(
        "Vivien's Stampede",
        cost(&[generic(4), g(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GrantKeywords {
                what: yours(R::Creature),
                keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Melee],
                duration: Duration::EndOfTurn,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::EndOfCombat,
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::PlayersDealtCombatDamageThisTurn(PlayerRef::EachPlayer),
                }),
            },
        ]),
    )
}

/// Zurzoth, Chaos Rider — an opponent's first draw outside their turn makes a
/// Devil; your Devils attacking make you and the defender loot at random.
///
/// Approximation: the defender of the attack, one player per batch.
pub fn zurzoth_chaos_rider() -> CardDefinition {
    let devil = TokenDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
        }],
        ..token("Devil", vec![Color::Red], vec![CreatureType::Devil], 1, 1)
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::FirstCardDrawnThisTurn, EventScope::OpponentControl)
                    .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::Triggerer)))),
                effect: make(PlayerRef::You, Value::ONE, Arc::new(devil)),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Devil),
                    })
                    .once_per_batch(),
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::Draw { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::DefendingPlayer),
                        amount: Value::ONE,
                        random: true,
                    },
                ]),
            },
        ],
        ..creature("Zurzoth, Chaos Rider", cost(&[generic(2), r()]), vec![CreatureType::Devil], 2, 3)
    }
}
