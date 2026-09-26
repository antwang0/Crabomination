//! Commander: the cards the **Hail, Caesar** precon (PIP, Caesar, Legion's
//! Emperor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_caesar.rs`.
//!
//! Residuals (each also on its card):
//! - **Aradesh, the Founder** — only its own enlist earns the double strike
//!   and draw; another creature of yours that enlists doesn't.
//! - **Colonel Autumn** — "whenever a creature you control exploits" is the
//!   payoff folded into its own and its granted exploit, so a printed exploit
//!   creature (not granted by Autumn) doesn't count, and two Autumns don't
//!   double it.
//! - **Mr. House, President and CEO** — its roll is one die (no extra die
//!   per Treasure mana), and "whenever you roll a 4 or higher" reads a roll's
//!   highest die once.
//! - **Mysterious Stranger** — the exiled cards are picked, not targeted.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{
    battalion, declare_target_opponent, etb, exploit, mint_treasures, on_attack, on_dies, on_you_attack, squad_etb, target_filtered,
    training,
};
use crate::effect::{AttackingTokenCleanup, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, w};
use crate::sets::{enters_tapped, tap_add, tap_add_any_color, tap_add_colorless};
use crabomination_base::tokens::{food_token, junk_token, treasure_token};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn artifact_legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..legend(name, mana, types, p, t) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn make(token: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(token) }
}

fn quests() -> Value {
    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Quest }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

/// A 1/1 white Soldier (or red and white with haste, Caesar's).
fn soldier(colors: Vec<Color>, types: Vec<CreatureType>, haste: bool) -> TokenDefinition {
    TokenDefinition {
        name: if types.len() > 1 { "Human Soldier".into() } else { "Soldier".into() },
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords: if haste { vec![Keyword::Haste] } else { vec![] },
        ..Default::default()
    }
}

fn white_soldier() -> TokenDefinition {
    soldier(vec![Color::White], vec![CreatureType::Soldier], false)
}

fn human_soldier() -> TokenDefinition {
    soldier(vec![Color::White], vec![CreatureType::Human, CreatureType::Soldier], false)
}

fn bobbleheads() -> Value {
    Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasArtifactSubtype(ArtifactSubtype::Bobblehead))
}

fn bobblehead(name: &'static str, second: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Bobblehead], ..Default::default() },
        activated_abilities: vec![tap_add_any_color(), second],
        ..Default::default()
    }
}


/// Aradesh, the Founder — enlist; a creature that enlisted gains double
/// strike, and draws you a card at power 4+. Residual: only Aradesh's own
/// enlist earns the payoff.
pub fn aradesh_the_founder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::EnlistThen {
            then: Box::new(Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::PowerOf(Box::new(Selector::This)), Value::Const(4)),
                    then: Box::new(draw(1)),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..legend(
            "Aradesh, the Founder",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            4,
        )
    }
}

/// Battle of Hoover Dam — as it enters, choose NCR (your end step: a
/// creature card of mana value 3 or less back with a finality counter) or
/// Legion (a creature of yours dying: two +1/+1 counters on a target
/// creature of yours).
pub fn battle_of_hoover_dam() -> CardDefinition {
    let ncr = TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Finality, amount: Value::ONE },
        ]),
    };
    let legion = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
        effect: Effect::AddCounter {
            what: target_filtered(yours(R::Creature)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
    };
    let grant = |t: TriggeredAbility| Effect::GrantTriggeredAbility {
        what: Selector::This,
        trigger: Box::new(t),
        duration: Duration::Permanent,
    };
    CardDefinition {
        name: "Battle of Hoover Dam",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::AsEntersChooseMode(vec![grant(ncr), grant(legion)])),
        ..Default::default()
    }
}

/// Boomer Scrapper — enters or attacks: lose 1 life, a Junk; a token of
/// yours leaving gives it a +1/+1 counter.
pub fn boomer_scrapper() -> CardDefinition {
    let scrap = || {
        Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            make(junk_token(), Value::ONE),
        ])
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(scrap()),
            on_attack(scrap()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken }),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature(
            "Boomer Scrapper",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Butch DeLoria, Tunnel Snake — menace; attacks: +1/+1 per other Rogue
/// and/or Snake you control; {1}{B}: a menace counter on another target
/// creature, which becomes a Rogue too.
pub fn butch_deloria_tunnel_snake() -> CardDefinition {
    let gang = || {
        Value::PermanentCountControlledByMatching(
            PlayerRef::You,
            R::Creature
                .and(R::HasCreatureType(CreatureType::Rogue).or(R::HasCreatureType(CreatureType::Snake)))
                .and(R::OtherThanSource),
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: gang(),
            toughness: gang(),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Seq(vec![
                Effect::AddKeywordCounter {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    keyword: Keyword::Menace,
                    amount: Value::ONE,
                },
                Effect::AddCreatureTypes {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Rogue],
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Butch DeLoria, Tunnel Snake",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Caesar, Legion's Emperor — whenever you attack, you may sacrifice another
/// creature; when you do, choose two: two hasty red and white Soldiers
/// tapped and attacking; draw a card and lose 1 life; damage equal to your
/// creature tokens to target opponent.
pub fn caesar_legions_emperor() -> CardDefinition {
    let legionaries = soldier(vec![Color::Red, Color::White], vec![CreatureType::Soldier], true);
    CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature?".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::ChooseN {
                    // Choose two; the default pair is the Soldiers and the
                    // card (`picks` names the modes, CR 700.2d).
                    picks: vec![0, 1],
                    modes: vec![
                        Effect::CreateTokenAttacking {
                            who: PlayerRef::You,
                            count: Value::Const(2),
                            definition: Arc::new(legionaries),
                            cleanup: AttackingTokenCleanup::None,
                            defender: None,
                        },
                        Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
                        Effect::DealDamage {
                            to: target_filtered(R::OpponentPlayer),
                            amount: Value::PermanentCountControlledByMatching(
                                PlayerRef::You,
                                R::Creature.and(R::IsToken),
                            ),
                        },
                    ],
                }),
            }),
            else_: None,
        })],
        ..legend(
            "Caesar, Legion's Emperor",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Charisma Bobblehead — {T}: any color; {4},{T}: a 1/1 white Soldier per
/// Bobblehead you control, sorcery speed.
pub fn charisma_bobblehead() -> CardDefinition {
    bobblehead(
        "Charisma Bobblehead",
        ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            sorcery_speed: true,
            effect: make(white_soldier(), bobbleheads()),
            ..Default::default()
        },
    )
}

/// Colonel Autumn — lifelink; exploit; other legendary creatures you
/// control have exploit; a creature of yours exploiting puts a +1/+1
/// counter on each creature you control. Residual: the payoff rides on the
/// exploits Autumn has and grants.
pub fn colonel_autumn() -> CardDefinition {
    let payoff = || Effect::AddCounter {
        what: Selector::EachPermanent(yours(R::Creature)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![exploit(payoff())],
        static_abilities: vec![StaticAbility {
            description: "Other legendary creatures you control have exploit.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary))).and(R::OtherThanSource),
                ability: Box::new(exploit(payoff())),
            },
        }],
        ..legend("Colonel Autumn", cost(&[generic(1), w(), b()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Craig Boone, Novac Guard — reach, lifelink; attacking with two or more:
/// two quest counters, then damage equal to its quest counters to up to one
/// target creature unless its controller takes it instead.
pub fn craig_boone_novac_guard() -> CardDefinition {
    let shot = Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::PlayersMayAccept {
            who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            description: "Take the damage yourself to spare your creature?".into(),
            on_accept: Box::new(Effect::Noop),
            if_any: Box::new(Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: quests(),
            }),
            otherwise: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: quests() }),
        }),
    };
    // The target rides on the trigger itself rather than a `Reflexive`: a
    // reflexive body's runtime target isn't carried into the continuation a
    // prompting controller's answer resumes, so the accept was lost.
    let mut boone = on_you_attack(Effect::Seq(vec![
        Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::Const(2) },
        shot,
    ]));
    boone.event = boone.event.with_filter(Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 2 });
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Lifelink],
        triggered_abilities: vec![boone],
        ..legend(
            "Craig Boone, Novac Guard",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Desdemona, Freedom's Edge — vigilance; attacks: a creature card in your
/// graveyard that's an artifact or has mana value 3 or less gains escape
/// until end of turn (its mana cost plus exiling two other cards).
pub fn desdemona_freedoms_edge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::GrantEscapeThisTurn {
            what: target_filtered(
                R::Creature.and(R::Artifact.or(R::ManaValueAtMost(3))).from_your_graveyard(),
            ),
            exile_count: 2,
        })],
        ..legend(
            "Desdemona, Freedom's Edge",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            4,
        )
    }
}

/// Diamond City — enters with a shield counter; {T}: {C}; {T}: move a
/// shield counter from it onto target creature, only if two or more
/// creatures entered under your control this turn.
pub fn diamond_city() -> CardDefinition {
    CardDefinition {
        name: "Diamond City",
        card_types: vec![CardType::Land],
        enters_with_counters: Some((CounterType::Shield, Value::ONE)),
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::ValueAtLeast(
                    Value::CreaturesEnteredThisTurn(PlayerRef::You),
                    Value::Const(2),
                )),
                effect: Effect::MoveCounter {
                    from: Selector::This,
                    to: target_filtered(R::Creature),
                    kind: CounterType::Shield,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// ED-E, Lonesome Eyebot — flying; whenever you attack with more creatures
/// than its quest counters, a quest counter; {2}, sacrifice it: draw one
/// plus one per quest counter.
pub fn ed_e_lonesome_eyebot() -> CardDefinition {
    let mut tally = on_you_attack(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Quest,
        amount: Value::ONE,
    });
    tally.event = tally.event.with_filter(Predicate::ValueAtLeast(
        Value::count(Selector::EachPermanent(yours(R::Creature).and(R::IsAttacking))),
        Value::Sum(vec![quests(), Value::ONE]),
    ));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![tally],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Sum(vec![Value::ONE, quests()]) },
            ..Default::default()
        }],
        ..artifact_legend("ED-E, Lonesome Eyebot", cost(&[generic(3)]), vec![CreatureType::Robot], 2, 1)
    }
}

/// Elder Arthur Maxson — creature tokens you control have training;
/// sacrifice another creature: indestructible until end of turn.
pub fn elder_arthur_maxson() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have training.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: yours(R::Creature.and(R::IsToken)),
                ability: Box::new(training()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..legend(
            "Elder Arthur Maxson",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            2,
        )
    }
}

/// Kellogg, Dangerous Mind — first strike, haste; attacks: a Treasure;
/// sacrifice five Treasures: gain control of target creature while you
/// control Kellogg, sorcery speed.
pub fn kellogg_dangerous_mind() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![on_attack(mint_treasures(1))],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 5)),
            sorcery_speed: true,
            effect: Effect::GainControlWhileYouControlSource { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..legend(
            "Kellogg, Dangerous Mind",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            3,
            2,
        )
    }
}

/// Legate Lanius, Caesar's Ace — Decimate: each opponent sacrifices a tenth
/// of their creatures, rounded up; an opponent sacrificing a creature gives
/// it a +1/+1 counter.
pub fn legate_lanius_caesars_ace() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::ForEachOpponent {
                body: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    count: Value::DivDown(
                        Box::new(Value::Sum(vec![
                            Value::PermanentCountControlledByMatching(PlayerRef::Triggerer, R::Creature),
                            Value::Const(9),
                        ])),
                        10,
                    ),
                    filter: R::Creature,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::OpponentControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..legend(
            "Legate Lanius, Caesar's Ace",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Luck Bobblehead — {T}: any color; {1},{T}: roll a d6 per Bobblehead,
/// a tapped Treasure per even result; seven 6s wins the game.
pub fn luck_bobblehead() -> CardDefinition {
    let treasure = || make(TokenDefinition { tapped: true, ..treasure_token() }, Value::ONE);
    bobblehead(
        "Luck Bobblehead",
        ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::RollDie {
                    sides: 6,
                    count: bobbleheads(),
                    modifier: Value::Const(0),
                    reroll_at_most: 0,
                    ignore_lowest: 0,
                    results: vec![(2, 2, treasure()), (4, 4, treasure()), (6, 6, treasure())],
                    on_doubles: None,
                },
                Effect::If {
                    cond: Predicate::ValueEquals(Value::LastRollFaceCount(6), Value::Const(7)),
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        },
    )
}

/// MacCready, Lamplight Mayor — your power-2-or-less attackers gain skulk;
/// a power-4-or-greater creature attacking you drains its controller 2.
pub fn maccready_lamplight_mayor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::PowerAtMost(2)) },
                ),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Skulk,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::PowerAtLeast(4) },
                ),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
        ],
        ..legend(
            "MacCready, Lamplight Mayor",
            cost(&[w(), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            3,
        )
    }
}

/// Memorial to Glory — enters tapped; {T}: {W}; {3}{W},{T}, sacrifice it:
/// two 1/1 white Soldiers.
pub fn memorial_to_glory() -> CardDefinition {
    CardDefinition {
        name: "Memorial to Glory",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::White),
            ActivatedAbility {
                mana_cost: cost(&[generic(3), w()]),
                tap_cost: true,
                sac_cost: true,
                effect: make(white_soldier(), Value::Const(2)),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mr. House, President and CEO — whenever you roll a 4 or higher, a 3/3
/// Robot (and a Treasure on a 6+); {4},{T}: roll a d6. Residual: the roll is
/// one die, and the trigger reads a roll's highest die once.
pub fn mr_house_president_and_ceo() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::RolledDice, EventScope::YourControl)
                .with_filter(Predicate::DieResultAtLeast(4)),
            effect: Effect::Seq(vec![
                make(robot, Value::ONE),
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(6)),
                    then: Box::new(mint_treasures(1)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::RollDie {
                sides: 6,
                count: Value::ONE,
                modifier: Value::Const(0),
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![],
                on_doubles: None,
            },
            ..Default::default()
        }],
        ..artifact_legend("Mr. House, President and CEO", cost(&[r(), w(), b()]), vec![CreatureType::Human], 0, 4)
    }
}

/// Mysterious Stranger — flash; ETB: exile an instant or sorcery card from
/// each graveyard with one; with two or more exiled, copy one at random
/// and you may cast the copy free. Residual: the cards are picked, not
/// targeted.
pub fn mysterious_stranger() -> CardDefinition {
    let spells = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::ExileLinked {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::Triggerer,
                            zone: Zone::Graveyard,
                            filter: spells(),
                        }),
                        count: Box::new(Value::ONE),
                    },
                }),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::count(Selector::ExiledThisResolution { filter: spells() }),
                    Value::Const(2),
                ),
                then: Box::new(Effect::CopyCardAndCastFree {
                    what: Selector::RandomOf(Box::new(Selector::ExiledThisResolution { filter: spells() })),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Mysterious Stranger", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Rogue], 3, 2)
    }
}

/// Overseer of Vault 76 — it or another creature of yours with power 3 or
/// less entering: a quest counter on it; beginning of combat on your turn,
/// you may remove three quest counters from among your permanents to put a
/// +1/+1 counter on each creature you control, which gain vigilance.
pub fn overseer_of_vault_76() -> CardDefinition {
    let team = || Selector::EachPermanent(yours(R::Creature));
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::PowerAtMost(3)) },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Remove three quest counters to grow your team?".into(),
                    body: Box::new(Effect::RemoveCountersFromAmongThen {
                        kind: CounterType::Quest,
                        count: 3,
                        filter: R::Permanent,
                        then: Box::new(Effect::Seq(vec![
                            Effect::AddCounter { what: team(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                            Effect::GrantKeyword { what: team(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                        ])),
                    }),
                },
            },
        ],
        ..legend(
            "Overseer of Vault 76",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            3,
            3,
        )
    }
}

/// Paladin Elizabeth Taggerdy — battalion: draw a card, then you may put a
/// creature card with mana value up to its power from your hand onto the
/// battlefield tapped and attacking.
pub fn paladin_elizabeth_taggerdy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![battalion(Effect::Seq(vec![
            draw(1),
            Effect::DeployCreatureFromHandAttacking {
                filter: R::Creature.and(R::ManaValueAtMostSourcePower),
                return_to_hand_eot: false,
            },
        ]))],
        ..legend(
            "Paladin Elizabeth Taggerdy",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            2,
        )
    }
}

/// Rose, Cutthroat Raider — first strike; raid: at end of combat on your
/// turn, a Junk per opponent you attacked; sacrificing a Junk adds {R}.
pub fn rose_cutthroat_raider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::EndCombat), EventScope::YourControl)
                    .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
                effect: make(junk_token(), Value::OpponentsAttackedThisCombat),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Junk),
                    },
                ),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) },
            },
        ],
        ..artifact_legend("Rose, Cutthroat Raider", cost(&[generic(2), r(), r()]), vec![CreatureType::Robot], 3, 2)
    }
}

/// Ruthless Radrat — squad (exile four cards from your graveyard per copy);
/// menace.
pub fn ruthless_radrat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Squad(ManaCost::default()), Keyword::Menace],
        squad_extra_cost: Some(AdditionalCastCost::ExileFromGraveyard { filter: R::Any, count: 4 }),
        triggered_abilities: vec![squad_etb()],
        ..creature(
            "Ruthless Radrat",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Rat, CreatureType::Mutant],
            2,
            2,
        )
    }
}

/// Sierra, Nuka's Biggest Fan — your creatures connecting with a player: a
/// quest counter and a Food; sacrificing a Food pumps a target creature of
/// yours by its quest counters.
pub fn sierra_nukas_biggest_fan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
                    make(food_token(), Value::ONE),
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Food),
                    },
                ),
                effect: Effect::PumpPT {
                    what: target_filtered(yours(R::Creature)),
                    power: quests(),
                    toughness: quests(),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..legend(
            "Sierra, Nuka's Biggest Fan",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            3,
            4,
        )
    }
}

/// Survivor's Med Kit — {1},{T}: choose one not chosen before: draw a card;
/// a Food; target player loses all rad counters and you sacrifice it.
pub fn survivors_med_kit() -> CardDefinition {
    CardDefinition {
        name: "Survivor's Med Kit",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    draw(1),
                    make(food_token(), Value::ONE),
                    Effect::Seq(vec![
                        Effect::RemoveAllRadCounters { who: PlayerRef::Target(0) },
                        Effect::SacrificePermanent { what: Selector::This },
                    ]),
                ],
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Nipton Lottery — a random creature: you control it until end of
/// turn, untap it, it gains haste; destroy all other creatures.
pub fn the_nipton_lottery() -> CardDefinition {
    let winner = || Selector::SeparatedPile { chosen: true };
    CardDefinition {
        name: "The Nipton Lottery",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseOneAtRandomAmong {
            what: Selector::EachPermanent(R::Creature),
            chosen: Box::new(Effect::Seq(vec![
                Effect::GainControl { what: winner(), to: None, duration: Duration::EndOfTurn },
                Effect::Untap { what: winner(), up_to: None },
                Effect::GrantKeyword { what: winner(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ])),
            other: Box::new(Effect::Destroy { what: Selector::SeparatedPile { chosen: false } }),
        },
        ..Default::default()
    }
}

/// Thrill-Kill Disciple — squad ({1}, discard a card per copy); dies: a
/// Junk.
pub fn thrill_kill_disciple() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Squad(cost(&[generic(1)]))],
        squad_extra_cost: Some(AdditionalCastCost::Discard { count: 1, filter: None }),
        triggered_abilities: vec![squad_etb(), on_dies(make(junk_token(), Value::ONE))],
        ..creature(
            "Thrill-Kill Disciple",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            3,
            2,
        )
    }
}

/// V.A.T.S. — split second; destroy any number of target creatures with
/// equal toughness.
pub fn v_a_t_s() -> CardDefinition {
    CardDefinition {
        name: "V.A.T.S.",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Creature.and(R::SameToughnessAsTargetSlot(0)),
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: chapters,
        ..Default::default()
    }
}

/// Vault 11: Voter's Dilemma — I: a Human Soldier per opponent. II, III:
/// each player secretly votes for up to one creature; destroy the most
/// voted, or everyone draws if nobody voted.
pub fn vault_11_voters_dilemma() -> CardDefinition {
    let vote = || Effect::SecretCouncilPermanentVoteMost {
        filter: R::Creature,
        on_most: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        on_none: Box::new(Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE }),
    };
    saga(
        "Vault 11: Voter's Dilemma",
        cost(&[generic(2), w(), b()]),
        vec![(1, make(human_soldier(), Value::OpponentCount)), (2, vote()), (3, vote())],
    )
}

/// Vault 75: Middle School — I: exile all creatures with power 4 or
/// greater. II, III: a +1/+1 counter on each creature you control.
pub fn vault_75_middle_school() -> CardDefinition {
    let grow = || Effect::AddCounter {
        what: Selector::EachPermanent(yours(R::Creature)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    saga(
        "Vault 75: Middle School",
        cost(&[generic(2), w(), w()]),
        vec![
            (
                1,
                Effect::Move {
                    what: Selector::EachPermanent(R::Creature.and(R::PowerAtLeast(4))),
                    to: ZoneDest::Exile,
                },
            ),
            (2, grow()),
            (3, grow()),
        ],
    )
}

/// White Glove Gourmand — ETB two 1/1 Human Soldiers; your end step, a Food
/// if another Human died under your control this turn.
pub fn white_glove_gourmand() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(make(human_soldier(), Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                    Predicate::ValueAtLeast(
                        Value::CreatureDeathsThisTurnMatching {
                            filter: yours(R::HasCreatureType(CreatureType::Human)).and(R::OtherThanSource),
                        },
                        Value::ONE,
                    ),
                ),
                effect: make(food_token(), Value::ONE),
            },
        ],
        ..creature(
            "White Glove Gourmand",
            cost(&[generic(2), w(), b()]),
            vec![CreatureType::Human, CreatureType::Noble],
            2,
            2,
        )
    }
}

/// Wild Wasteland — skip your draw step; your upkeep: exile the top two
/// cards, playable this turn.
pub fn wild_wasteland() -> CardDefinition {
    CardDefinition {
        name: "Wild Wasteland",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::ControllerSkipsDrawStep,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Yes Man, Personal Securitron — {T}, your turn only: target opponent gains
/// control of it; you draw two and put a quest counter on it. Leaving the
/// battlefield, its owner makes a tapped 1/1 Soldier per quest counter.
pub fn yes_man_personal_securitron() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Seq(vec![
                declare_target_opponent(),
                Effect::GainControl { what: Selector::This, to: Some(PlayerRef::Target(0)), duration: Duration::Permanent },
                draw(2),
                Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                count: quests(),
                definition: Arc::new(TokenDefinition { tapped: true, ..white_soldier() }),
            },
        }],
        ..artifact_legend("Yes Man, Personal Securitron", cost(&[generic(2), w()]), vec![CreatureType::Robot], 2, 2)
    }
}
