//! Commander: the cards the **Call for Backup** precon (MOC, Bright-Palm,
//! Soul Awakener) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_brightpalm.rs`.
//!
//! Residuals (each also on its card):
//! - **Dromoka's Command** and **Inscription of Abundance** — the fight mode
//!   targets only your creature; it fights the greatest-power creature you
//!   don't control. The Inscription is never kicked (one mode only).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{backup_with, etb, on_attack, riot, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, VoteOption, VoteTally,
};
use crate::mana::{Color, cost, g, generic, r, w, x};
use crate::sets::{enters_tapped, tap_add};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn countered() -> R {
    R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))
}

fn plus_one(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) }
}

/// "Target creature you control fights target creature you don't control" —
/// one target slot per mode, so the second creature is the greatest-power
/// one you don't control.
fn fight_mode() -> Effect {
    Effect::Fight {
        attacker: target_filtered(R::Creature.and(R::ControlledByYou)),
        defender: Selector::TakeGreatestPower {
            inner: Box::new(Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::ControlledByYou))))),
            count: Box::new(Value::ONE),
        },
    }
}

/// Bright-Palm, Soul Awakener — backup 1; whenever it attacks, double the
/// +1/+1 counters on target creature, which can't be blocked by power 2 or
/// less this turn.
pub fn bright_palm_soul_awakener() -> CardDefinition {
    let attack = || {
        on_attack(Effect::Seq(vec![
            Effect::DoubleCountersOnEach {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBeBlockedByPowerAtMost(2),
                duration: Duration::EndOfTurn,
            },
        ]))
    };
    CardDefinition {
        triggered_abilities: vec![backup_with(1, vec![], vec![attack()]), attack()],
        ..legendary(creature(
            "Bright-Palm, Soul Awakener",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Fox, CreatureType::Shaman],
            4,
            3,
        ))
    }
}

/// Alharu, Solemn Ritualist — on entry, a +1/+1 counter on each of up to two
/// other target creatures; a nontoken creature of yours with a +1/+1 counter
/// dying makes a 1/1 flying Spirit. Partner.
pub fn alharu_solemn_ritualist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature.and(R::OtherThanSource),
                effect: Box::new(plus_one(Selector::Target(0), 1)),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::WithCounter(CounterType::PlusOnePlusOne).and(R::NotToken),
                    },
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Spirit".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    }),
                },
            },
        ],
        ..legendary(creature(
            "Alharu, Solemn Ritualist",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Monk],
            3,
            3,
        ))
    }
}

/// Bretagard Stronghold — enters tapped; {T}: add {G}; {G}{W}{W}, {T},
/// sacrifice it: a +1/+1 counter on each of up to two target creatures you
/// control, which gain vigilance and lifelink until end of turn. Sorcery
/// speed.
pub fn bretagard_stronghold() -> CardDefinition {
    CardDefinition {
        name: "Bretagard Stronghold",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::Green),
            ActivatedAbility {
                mana_cost: cost(&[g(), w(), w()]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                    effect: Box::new(Effect::Seq(vec![
                        plus_one(Selector::Target(0), 1),
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Vigilance,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Lifelink,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dromoka's Command — choose two: prevent a spell's damage; a player
/// sacrifices an enchantment; a +1/+1 counter; your creature fights.
///
/// ⚠ Residual: the fight's second creature is the greatest-power one you
/// don't control.
pub fn dromokas_command() -> CardDefinition {
    CardDefinition {
        name: "Dromoka's Command",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::PreventAllDamageByTargetThisTurn {
                    target: target_filtered(
                        R::IsSpellOnStack
                            .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                    ),
                },
                Effect::Sacrifice { who: target_filtered(R::Player), count: Value::ONE, filter: R::Enchantment },
                plus_one(target_filtered(R::Creature), 1),
                fight_mode(),
            ],
            min: 2,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Emergent Woodwurm — backup 3; whenever it attacks, look at the top X (X
/// its power) and you may put a permanent card with mana value X or less
/// onto the battlefield; the rest go to the bottom at random.
pub fn emergent_woodwurm() -> CardDefinition {
    let attack = || {
        on_attack(Effect::WithX {
            x: Value::PowerOf(Box::new(Selector::This)),
            body: Box::new(Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::XFromCost,
                pick_filter: Some(R::PermanentCard.and(R::ManaValueAtMostXFromCost)),
                take: Some(Value::ONE),
                to_battlefield: true,
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            }))),
        })
    };
    CardDefinition {
        triggered_abilities: vec![backup_with(3, vec![], vec![attack()]), attack()],
        ..creature("Emergent Woodwurm", cost(&[generic(6), g()]), vec![CreatureType::Wurm], 4, 4)
    }
}

/// Falkenrath Exterminator — combat damage to a player grows it; {2}{R}:
/// damage to target creature equal to its +1/+1 counters.
pub fn falkenrath_exterminator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: plus_one(Selector::This, 1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
            },
            ..Default::default()
        }],
        ..creature(
            "Falkenrath Exterminator",
            cost(&[generic(1), r()]),
            vec![CreatureType::Vampire, CreatureType::Archer],
            1,
            1,
        )
    }
}

/// Hamza, Guardian of Arashin — it and your creature spells cost {1} less
/// per creature you control with a +1/+1 counter.
pub fn hamza_guardian_of_arashin() -> CardDefinition {
    let per = || Value::count(yours(countered()));
    CardDefinition {
        self_cost_reduction_per: Some((per(), 1)),
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast cost {1} less for each creature you control with a +1/+1 counter.",
            effect: StaticEffect::CostReductionByValue { filter: R::Creature, amount: per() },
        }],
        ..legendary(creature(
            "Hamza, Guardian of Arashin",
            cost(&[generic(4), g(), w()]),
            vec![CreatureType::Elephant, CreatureType::Warrior],
            5,
            5,
        ))
    }
}

/// Heaven // Earth — Heaven ({X}{G} instant): X damage to each flier. Earth
/// ({X}{R}{R} sorcery, aftermath): X damage to each creature without flying.
pub fn heaven_earth() -> CardDefinition {
    let flying = || R::Creature.and(R::HasKeyword(Keyword::Flying));
    CardDefinition {
        name: "Heaven // Earth",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: Selector::EachPermanent(flying()), amount: Value::XFromCost },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[x(), r(), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
                    amount: Value::XFromCost,
                },
            },
            fuse: false,
            aftermath: true,
        })),
        ..Default::default()
    }
}

/// High Sentinels of Arashin — flying; +1/+1 for each other creature you
/// control with a +1/+1 counter; {3}{W}: a +1/+1 counter on target creature.
pub fn high_sentinels_of_arashin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "+1/+1 for each other creature you control with a +1/+1 counter on it.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: countered().and(R::OtherThanSource),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: plus_one(target_filtered(R::Creature), 1),
            ..Default::default()
        }],
        ..creature(
            "High Sentinels of Arashin",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Inscription of Abundance — choose one: two +1/+1 counters; a player gains
/// life equal to their greatest power; your creature fights.
///
/// ⚠ Residual: no kicker (one mode only); the fight's second creature is the
/// greatest-power one you don't control.
pub fn inscription_of_abundance() -> CardDefinition {
    CardDefinition {
        name: "Inscription of Abundance",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                plus_one(target_filtered(R::Creature), 2),
                Effect::GainLife {
                    who: target_filtered(R::Player),
                    amount: Value::GreatestPowerControlled { who: PlayerRef::Target(0) },
                },
                fight_mode(),
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Mikaeus, the Lunarch — enters with X +1/+1 counters; {T}: a counter on
/// it; {T}, remove a counter: a counter on each other creature you control.
pub fn mikaeus_the_lunarch() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: plus_one(Selector::This, 1), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: plus_one(yours(R::Creature.and(R::OtherThanSource)), 1),
                ..Default::default()
            },
        ],
        ..legendary(creature(
            "Mikaeus, the Lunarch",
            cost(&[x(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            0,
            0,
        ))
    }
}

/// Mirror-Style Master — backup 1; whenever it attacks, a tapped and
/// attacking copy of each attacking modified creature you control, exiled at
/// end of combat.
pub fn mirror_style_master() -> CardDefinition {
    let attack = || {
        on_attack(Effect::ForEach {
            selector: yours(R::Creature.and(R::IsAttacking).and(R::IsModified)),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: true,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::EndOfCombat,
                    body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
                },
            ])),
        })
    };
    CardDefinition {
        triggered_abilities: vec![backup_with(1, vec![], vec![attack()]), attack()],
        ..creature(
            "Mirror-Style Master",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Path of the Pyromancer — discard your hand, add {R} per card discarded,
/// draw that many plus one; then the will of the planeswalkers.
pub fn path_of_the_pyromancer() -> CardDefinition {
    CardDefinition {
        name: "Path of the Pyromancer",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::CardsDiscardedThisEffect),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Sum(vec![Value::CardsDiscardedThisEffect, Value::ONE]),
            },
            Effect::Vote {
                options: vec![
                    VoteOption::new("planeswalk", Effect::Planeswalk { who: PlayerRef::You }),
                    VoteOption::new("chaos", Effect::ChaosEnsues { who: PlayerRef::You }),
                ],
                tally: VoteTally::Majority,
            },
        ]),
        ..Default::default()
    }
}

/// Shalai and Hallar — flying, vigilance; +1/+1 counters put on a creature
/// you control deal that much damage to target opponent.
pub fn shalai_and_hallar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ControlledByYou),
                }),
            effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::TriggerEventAmount },
        }],
        ..legendary(creature(
            "Shalai and Hallar",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Angel, CreatureType::Elf],
            3,
            3,
        ))
    }
}

/// Slurrk, All-Ingesting — enters with five +1/+1 counters; it or another
/// creature of yours dying with a +1/+1 counter grows each of your creatures
/// with one. Partner.
pub fn slurrk_all_ingesting() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::WithCounter(CounterType::PlusOnePlusOne),
                },
            ),
            effect: plus_one(yours(countered()), 1),
        }],
        ..legendary(creature("Slurrk, All-Ingesting", cost(&[generic(5), g()]), vec![CreatureType::Ooze], 0, 0))
    }
}

/// Uncivil Unrest — nontoken creatures you control have riot; a creature of
/// yours with a +1/+1 counter deals double damage.
pub fn uncivil_unrest() -> CardDefinition {
    CardDefinition {
        name: "Uncivil Unrest",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Nontoken creatures you control have riot.",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::Creature.and(R::ControlledByYou).and(R::NotToken),
                    ability: Box::new(riot()),
                },
            },
            StaticAbility {
                description: "Creatures you control with a +1/+1 counter deal double damage.",
                effect: StaticEffect::DoubleDamageFromControlledMatching { filter: countered() },
            },
        ],
        ..Default::default()
    }
}
