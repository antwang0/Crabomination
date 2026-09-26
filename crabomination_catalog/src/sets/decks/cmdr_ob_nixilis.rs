//! Commander: the cards the **Sworn to Darkness** precon (C14, Ob Nixilis of
//! the Black Oath) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_ob_nixilis.rs`.
//!
//! Residuals (each also on its card):
//! - **Infernal Offering** — each "choose an opponent" is the engine's pick,
//!   and each return takes the first creature card in graveyard order.
//! - **Profane Command** — the two modes are resolution-time picks
//!   (`Effect::ChooseN`, default: life loss + −X/−X), so targets follow the
//!   default pair's slots.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    Zone,
};
use crate::effect::shortcut::{target_filtered, target_n};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, x};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// Morbid (ability word, CR 207.2c) — "if a creature died this turn".
fn morbid() -> Predicate {
    Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }
}

fn demon_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Demon".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    })
}

/// Ob Nixilis of the Black Oath — +2: drain each opponent for 1; −2: a 5/5
/// flying Demon for 2 life; −8: an emblem whose "{1}{B}, Sacrifice a creature"
/// rides each of your creatures (CR 114.4).
pub fn ob_nixilis_of_the_black_oath() -> CardDefinition {
    let emblem_ability = ActivatedAbility {
        mana_cost: cost(&[generic(1), b()]),
        sac_cost: true,
        effect: Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::SacrificedPower },
            Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
        ]),
        ..Default::default()
    };
    CardDefinition {
        name: "Ob Nixilis of the Black Oath",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nixilis],
            ..Default::default()
        },
        base_loyalty: 3,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::DrainLifeLost {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: demon_token() },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Ob Nixilis of the Black Oath".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "{1}{B}, Sacrifice a creature: You gain X life and draw X \
                                      cards, where X is the sacrificed creature's power.",
                        effect: StaticEffect::GrantActivatedAbility {
                            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                            ability: emblem_ability,
                            condition: None,
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Demon of Wailing Agonies — lieutenant: +2/+2 and "combat damage to a
/// player makes them sacrifice a creature" while you control your commander.
pub fn demon_of_wailing_agonies() -> CardDefinition {
    let lieutenant = || Predicate::ControlsOwnCommander { who: PlayerRef::You };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Lieutenant — this creature gets +2/+2.",
                effect: StaticEffect::PumpSelfIf { condition: lieutenant(), power: 2, toughness: 2, keywords: vec![] },
            },
            StaticAbility {
                description: "Lieutenant — combat damage to a player: that player sacrifices a creature.",
                effect: StaticEffect::WhileCondition {
                    condition: lieutenant(),
                    inner: Box::new(StaticEffect::GrantTriggeredAbility {
                        filter: R::IsSource,
                        ability: Box::new(TriggeredAbility {
                            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                            effect: Effect::Sacrifice {
                                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                                count: Value::ONE,
                                filter: R::Creature,
                            },
                        }),
                    }),
                },
            },
        ],
        ..creature("Demon of Wailing Agonies", cost(&[generic(3), b(), b()]), vec![CreatureType::Demon], 4, 4)
    }
}

/// Evernight Shade — {B}: +1/+1 until end of turn; undying.
pub fn evernight_shade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Undying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Evernight Shade", cost(&[generic(3), b()]), vec![CreatureType::Shade], 1, 1)
    }
}

/// Flesh Carver — intimidate; {1}{B}, sacrifice another creature: two +1/+1
/// counters; dies: an X/X black Horror, X its last-known power.
pub fn flesh_carver() -> CardDefinition {
    let horror = Arc::new(TokenDefinition {
        name: "Horror".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        dynamic_pt: Some((
            Value::PowerOf(Box::new(Selector::TriggerSource)),
            Value::PowerOf(Box::new(Selector::TriggerSource)),
        )),
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: horror },
        }],
        ..creature(
            "Flesh Carver",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Infernal Offering — you and a chosen opponent each sacrifice a creature
/// and each who did draws two; then you and a chosen opponent each return a
/// creature card from graveyard to the battlefield. Each return takes the
/// first creature card in graveyard order.
pub fn infernal_offering() -> CardDefinition {
    let chosen = || Selector::Player(PlayerRef::ChosenPlayerOfSource);
    let sacrificed = |who: PlayerRef| {
        Predicate::ValueAtLeast(
            Value::SacrificedThisResolutionBy { who, filter: R::Creature },
            Value::ONE,
        )
    };
    let draw_two = |who: Selector| Effect::Draw { who, amount: Value::Const(2) };
    let return_one = |who: PlayerRef| Effect::Move {
        what: Selector::Take {
            inner: Box::new(Selector::CardsInZone { who: who.clone(), zone: Zone::Graveyard, filter: R::Creature }),
            count: Box::new(Value::ONE),
        },
        to: ZoneDest::Battlefield { controller: who, tapped: false },
    };
    spell(
        "Infernal Offering",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ChooseOpponentThen {
                then: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature },
                    Effect::Sacrifice { who: chosen(), count: Value::ONE, filter: R::Creature },
                    Effect::If {
                        cond: sacrificed(PlayerRef::You),
                        then: Box::new(draw_two(Selector::You)),
                        else_: Box::new(Effect::Noop),
                    },
                    Effect::If {
                        cond: sacrificed(PlayerRef::ChosenPlayerOfSource),
                        then: Box::new(draw_two(chosen())),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
            Effect::ChooseOpponentThen {
                then: Box::new(Effect::Seq(vec![
                    return_one(PlayerRef::You),
                    return_one(PlayerRef::ChosenPlayerOfSource),
                ])),
            },
        ]),
    )
}

/// Malicious Affliction — destroy target nonblack creature; morbid: casting it
/// copies it, and the copy may take a new target.
pub fn malicious_affliction() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource).with_filter(morbid()),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::This, count: Value::ONE },
        }],
        ..spell(
            "Malicious Affliction",
            cost(&[b(), b()]),
            CardType::Instant,
            Effect::Destroy { what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())) },
        )
    }
}

/// Morkrut Banshee — morbid: on entering, target creature gets −4/−4.
pub fn morkrut_banshee() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(morbid()),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-4),
                toughness: Value::Const(-4),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Morkrut Banshee", cost(&[generic(3), b(), b()]), vec![CreatureType::Spirit], 4, 4)
    }
}

/// Pestilence Demon — {B}: 1 damage to each creature and each player.
pub fn pestilence_demon() -> CardDefinition {
    let ping_each = |selector: Selector| Effect::ForEach {
        selector,
        body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::ONE }),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Seq(vec![
                ping_each(Selector::EachPermanent(R::Creature)),
                ping_each(Selector::Player(PlayerRef::EachPlayer)),
            ]),
            ..Default::default()
        }],
        ..creature("Pestilence Demon", cost(&[generic(5), b(), b(), b()]), vec![CreatureType::Demon], 7, 6)
    }
}

/// Profane Command — choose two: target player loses X; return target
/// creature card with mana value X or less from your graveyard; target
/// creature gets −X/−X; up to X target creatures gain fear. The picks are made
/// at resolution (`ChooseN`), defaulting to the life loss and the −X/−X.
pub fn profane_command() -> CardDefinition {
    spell(
        "Profane Command",
        cost(&[x(), b(), b()]),
        CardType::Sorcery,
        Effect::ChooseN {
            picks: vec![0, 2],
            modes: vec![
                Effect::LoseLife { who: target_filtered(R::Player), amount: Value::XFromCost },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::ManaValueAtMostXFromCost).from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Negate(Box::new(Value::XFromCost)),
                    toughness: Value::Negate(Box::new(Value::XFromCost)),
                    duration: Duration::EndOfTurn,
                },
                Effect::CapTargetsAtX {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Creature,
                        effect: Box::new(Effect::GrantKeyword {
                            what: target_n(0),
                            keyword: Keyword::Fear,
                            duration: Duration::EndOfTurn,
                        }),
                    }),
                },
            ],
        },
    )
}

/// Raving Dead — deathtouch; each of your combats it attacks an opponent
/// chosen at random if able; its combat damage halves that player's life.
pub fn raving_dead() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::MustAttackChosenPlayer],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::RememberPlayerOnSource { who: PlayerRef::RandomOpponent },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LoseHalfLife {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    rounded_up: false,
                },
            },
        ],
        ..creature("Raving Dead", cost(&[generic(4), b()]), vec![CreatureType::Zombie], 2, 6)
    }
}

/// Reaper from the Abyss — morbid: at each end step, destroy target non-Demon
/// creature.
pub fn reaper_from_the_abyss() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(morbid()),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Demon).negate())),
            },
        }],
        ..creature("Reaper from the Abyss", cost(&[generic(3), b(), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Skirsdag High Priest — morbid: {T}, tap two untapped creatures you control:
/// a 5/5 flying Demon.
pub fn skirsdag_high_priest() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_others_cost: Some((R::Creature.and(R::ControlledByYou), 2)),
            condition: Some(morbid()),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: demon_token() },
            ..Default::default()
        }],
        ..creature(
            "Skirsdag High Priest",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Sudden Spoiling — split second; creatures target player controls lose all
/// abilities and have base power and toughness 0/2 until end of turn.
pub fn sudden_spoiling() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    CardDefinition {
        keywords: vec![Keyword::SplitSecond],
        ..spell(
            "Sudden Spoiling",
            cost(&[generic(1), b(), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::LoseAllAbilities { what: theirs(), duration: Duration::EndOfTurn },
                Effect::SetBasePT {
                    what: theirs(),
                    power: Value::Const(0),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Wake the Dead — only during combat on an opponent's turn: X target creature
/// cards from your graveyard onto the battlefield, sacrificed at the next end
/// step.
pub fn wake_the_dead() -> CardDefinition {
    CardDefinition {
        cast_only_during_combat: true,
        cast_condition: Some(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
        ..spell(
            "Wake the Dead",
            cost(&[x(), b(), b()]),
            CardType::Instant,
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.from_your_graveyard(),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: target_n(0),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::SacrificeAtNextEndStep { what: Selector::LastMoved },
                    ])),
                }),
            },
        )
    }
}

/// Xathrid Demon — each upkeep, sacrifice another creature and each opponent
/// loses its power in life; with none to sacrifice, tap it and lose 7.
pub fn xathrid_demon() -> CardDefinition {
    let other = || R::Creature.and(R::ControlledByYou).and(R::IsSource.negate());
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(other())),
                then: Box::new(Effect::Seq(vec![
                    Effect::SacrificeAndRemember { who: PlayerRef::You, filter: other() },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::SacrificedPower,
                    },
                ])),
                else_: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::This },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(7) },
                ])),
            },
        }],
        ..creature("Xathrid Demon", cost(&[generic(3), b(), b(), b()]), vec![CreatureType::Demon], 7, 7)
    }
}
