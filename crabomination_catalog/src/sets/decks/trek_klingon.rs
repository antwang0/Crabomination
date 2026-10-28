//! Star Trek Commander (TRC, 2026-11-13) — the Klingon Fury precon's new
//! cards (Worf, Defiant Commander). "Battlefront" is the ability word for
//! effects scaled by the opponents you attacked this turn
//! (`Value::OpponentsAttackedThisTurn`). Tests in
//! `tests/recent_b/trek_klingon.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StationBand, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{battle_cry, etb, on_attack, on_dies, on_you_attack, squad_etb, station, target_filtered};
use crate::effect::{Duration, Effect, GoadLasts, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, w};

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

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

fn spacecraft(name: &'static str, mana: ManaCost, bands: Vec<StationBand>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: bands,
        ..Default::default()
    }
}

/// Klingons and Warriors — the deck's tribal pair.
fn kw() -> R {
    R::HasCreatureType(CreatureType::Klingon).or(R::HasCreatureType(CreatureType::Warrior))
}

fn your_kw_creatures() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(kw()))
}

fn battlefront() -> Value {
    Value::OpponentsAttackedThisTurn
}

/// The 2/1 red Klingon Warrior token with haste.
fn warrior_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Klingon Warrior".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Klingon, CreatureType::Warrior],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    })
}

fn warriors(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: warrior_token() }
}

fn pump_until_eot(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

/// Worf, Defiant Commander — {2}{R}{W}{B} legendary 3/4 Klingon Warrior
/// Officer. Entering makes a hasty 2/1 Klingon Warrior. Battlefront — whenever
/// you attack, exile the top X cards; you may play them until the end of your
/// next turn.
pub fn worf_defiant_commander() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(warriors(Value::ONE)),
            on_you_attack(Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: battlefront(),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            }),
        ],
        ..legend(
            "Worf, Defiant Commander",
            cost(&[generic(2), r(), w(), b()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            3,
            4,
        )
    }
}

/// Kor, Cause of Vengeance — {1}{R}{W}{B} legendary 4/4 Klingon Warrior
/// Officer. Whenever you attack, you may sacrifice another creature; if you
/// do, draw, then your Klingons and Warriors get +2/+2 until end of turn.
pub fn kor_cause_of_vengeance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::MayDo {
            description: "Sacrifice another creature to draw and pump?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature.and(R::OtherThanSource) },
                Effect::If {
                    cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        pump_until_eot(your_kw_creatures(), Value::Const(2), Value::Const(2)),
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..legend(
            "Kor, Cause of Vengeance",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            4,
            4,
        )
    }
}

/// Imperial Hullbreachers — {2}{W} 2/2 Klingon Warrior. Squad {2}, flying,
/// battle cry.
pub fn imperial_hullbreachers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Squad(cost(&[generic(2)])), Keyword::Flying],
        triggered_abilities: vec![squad_etb(), battle_cry(1)],
        ..creature(
            "Imperial Hullbreachers",
            cost(&[generic(2), w()]),
            vec![CreatureType::Klingon, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Indelicate Healer — {2}{W} 3/3 Klingon Warrior Doctor. It or another
/// Klingon or Warrior of yours entering gives your creatures lifelink until
/// end of turn.
pub fn indelicate_healer() -> CardDefinition {
    let lifelink = || Effect::GrantKeyword {
        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
        keyword: Keyword::Lifelink,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(lifelink()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: kw() }),
                effect: lifelink(),
            },
        ],
        ..creature(
            "Indelicate Healer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Doctor],
            3,
            3,
        )
    }
}

/// Korok, Liberated Rage — {3}{W} legendary 2/2 Klingon Borg artifact
/// creature. Battlefront — attacking gives your creatures +X/+X until end of
/// turn.
pub fn korok_liberated_rage() -> CardDefinition {
    let mut c = CardDefinition {
        triggered_abilities: vec![on_attack(pump_until_eot(
            Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            battlefront(),
            battlefront(),
        ))],
        ..legend(
            "Korok, Liberated Rage",
            cost(&[generic(3), w()]),
            vec![CreatureType::Klingon, CreatureType::Borg],
            2,
            2,
        )
    };
    c.card_types.insert(0, CardType::Artifact);
    c
}

/// Martok, Hero of the Empire — {2}{W} legendary 2/2 Klingon Warrior.
/// Battlefront — whenever you attack, a +1/+1 counter on each of up to X
/// target creatures you control.
pub fn martok_hero_of_the_empire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::CapTargetsAt {
            amount: battlefront(),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 7,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByYou),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            }),
        })],
        ..legend(
            "Martok, Hero of the Empire",
            cost(&[generic(2), w()]),
            vec![CreatureType::Klingon, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Merciful Mandate — {1}{W} instant. Untap your creatures; during combat you
/// may end the combat phase (CR 724.2).
pub fn merciful_mandate() -> CardDefinition {
    let in_combat = Predicate::Any(
        [
            TurnStep::BeginCombat,
            TurnStep::DeclareAttackers,
            TurnStep::DeclareBlockers,
            TurnStep::FirstStrikeDamage,
            TurnStep::CombatDamage,
            TurnStep::EndCombat,
        ]
        .into_iter()
        .map(Predicate::CurrentStepIs)
        .collect(),
    );
    CardDefinition {
        name: "Merciful Mandate",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)), up_to: None },
            Effect::If {
                cond: in_combat,
                then: Box::new(Effect::MayDo {
                    description: "End the combat phase?".into(),
                    body: Box::new(Effect::EndTheCombatPhase),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Right of Vengeance — {3}{W} instant. This turn, a creature dealing combat
/// damage to you is destroyed and you make a hasty 2/1 Klingon Warrior.
pub fn right_of_vengeance() -> CardDefinition {
    CardDefinition {
        name: "Right of Vengeance",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::TriggersThisTurn {
            triggered: vec![TriggeredAbility {
                // Combat damage as "a creature's damage in a combat damage step"
                // (the emblem can't hear the combat path's own event).
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::SourceDamagedYou).with_filter(
                    Predicate::All(vec![
                        Predicate::Any(vec![
                            Predicate::CurrentStepIs(TurnStep::CombatDamage),
                            Predicate::CurrentStepIs(TurnStep::FirstStrikeDamage),
                        ]),
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    ]),
                ),
                effect: Effect::Seq(vec![Effect::Destroy { what: Selector::TriggerSource }, warriors(Value::ONE)]),
            }],
        },
        ..Default::default()
    }
}

/// Blood-Sport Champion — {1}{B} 2/2 Klingon Warrior, deathtouch, kicker {2}.
/// Kicked, its arrival gives your other creatures deathtouch until end of turn.
pub fn blood_sport_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Kicker(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Blood-Sport Champion",
            cost(&[generic(1), b()]),
            vec![CreatureType::Klingon, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Kurn, Son of Mogh — {2}{B} legendary 2/3 Klingon Warrior Officer,
/// deathtouch. Battlefront — attacking draws X and loses X life.
pub fn kurn_son_of_mogh() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: battlefront() },
            Effect::LoseLife { who: Selector::You, amount: battlefront() },
        ]))],
        ..legend(
            "Kurn, Son of Mogh",
            cost(&[generic(2), b()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            2,
            3,
        )
    }
}

/// The Promise of Kahless — {4}{B} sorcery. You may sacrifice a Klingon or
/// Warrior; if you do, every other Klingon and Warrior creature card in your
/// graveyard returns to the battlefield.
pub fn the_promise_of_kahless() -> CardDefinition {
    CardDefinition {
        name: "The Promise of Kahless",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::MayDo {
            description: "Sacrifice a Klingon or Warrior?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature.and(kw()) },
                Effect::If {
                    cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                    then: Box::new(Effect::Move {
                        what: Selector::Except(
                            Box::new(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: crate::card::Zone::Graveyard,
                                filter: R::Creature.and(kw()),
                            }),
                            Box::new(Selector::SacrificedThisResolution { filter: R::Any }),
                        ),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Voq, Torchbearer — {3}{B} legendary 4/3 Klingon Spy, menace. May enter as
/// a copy of a creature card in a graveyard, a Klingon Spy with menace too.
pub fn voq_torchbearer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            from_graveyards: true,
            extra_creature_types: vec![CreatureType::Klingon, CreatureType::Spy],
            extra_keywords: vec![Keyword::Menace],
            ..Default::default()
        }),
        ..legend("Voq, Torchbearer", cost(&[generic(3), b()]), vec![CreatureType::Klingon, CreatureType::Spy], 4, 3)
    }
}

/// The Beacon of Kahless — {2}{R} legendary artifact. {T}: one mana of any
/// color. Battlefront — {T}: X mana of one color.
pub fn the_beacon_of_kahless() -> CardDefinition {
    CardDefinition {
        name: "The Beacon of Kahless",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(battlefront()) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ferocious Colonist — {1}{R} 0/1 Klingon Citizen, double strike, melee.
pub fn ferocious_colonist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Melee],
        ..creature(
            "Ferocious Colonist",
            cost(&[generic(1), r()]),
            vec![CreatureType::Klingon, CreatureType::Citizen],
            0,
            1,
        )
    }
}

/// Gagh Slinger — {2}{R} 3/2 Klingon Warrior. Klingon and Warrior spells cost
/// {1} less; another Klingon or Warrior of yours entering deals 1 damage to
/// each opponent.
pub fn gagh_slinger() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Klingon spells and Warrior spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: kw(), amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: kw() }),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        }],
        ..creature("Gagh Slinger", cost(&[generic(2), r()]), vec![CreatureType::Klingon, CreatureType::Warrior], 3, 2)
    }
}

/// Galvanizing Tenor — {3}{R} 4/3 Klingon Performer, menace. Battlefront —
/// attacking goads up to X target creatures.
pub fn galvanizing_tenor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::CapTargetsAt {
            amount: battlefront(),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 7,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Goad { what: Selector::Target(0) }),
            }),
        })],
        ..creature(
            "Galvanizing Tenor",
            cost(&[generic(3), r()]),
            vec![CreatureType::Klingon, CreatureType::Performer],
            4,
            3,
        )
    }
}

/// Glory to the Empire! — {5}{R} enchantment. Entering makes a hasty 2/1
/// Klingon Warrior per opponent; your creatures have melee.
pub fn glory_to_the_empire() -> CardDefinition {
    CardDefinition {
        name: "Glory to the Empire!",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(warriors(Value::OpponentCount))],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have melee.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Melee,
            },
        }],
        ..Default::default()
    }
}

/// Gnarling Targ — {4}{R} 4/5 Boar, trample. Whenever you attack, your
/// Klingons and Warriors get +2/+0 and trample until end of turn.
pub fn gnarling_targ() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
            pump_until_eot(your_kw_creatures(), Value::Const(2), Value::Const(0)),
            Effect::GrantKeyword { what: your_kw_creatures(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Gnarling Targ", cost(&[generic(4), r()]), vec![CreatureType::Boar], 4, 5)
    }
}

/// Grilka, Widowed Warrior — {5}{R} legendary 5/5 Klingon Warrior. Entering
/// steals target creature an opponent controls until end of turn, untapped,
/// with trample, haste and "combat damage to a player makes that many tapped
/// Treasures".
pub fn grilka_widowed_warrior() -> CardDefinition {
    let treasures = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::TriggerEventAmount,
            definition: Arc::new(crabomination_base::tokens::tapped_treasure_token()),
        },
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeywords {
                what: Selector::Target(0),
                keywords: vec![Keyword::Trample, Keyword::Haste],
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(treasures),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..legend(
            "Grilka, Widowed Warrior",
            cost(&[generic(5), r()]),
            vec![CreatureType::Klingon, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Kang, Suspicious Captain — {3}{R} legendary 4/4 Klingon Warrior Officer.
/// Entering goads him and target creature an opponent controls while he
/// stays. One or more goaded creatures attacking puts a +1/+1 counter on
/// target creature you control.
pub fn kang_suspicious_captain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::GoadWhile {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    hold: GoadLasts::WhileSourceOnBattlefield,
                },
                Effect::GoadWhile { what: Selector::This, hold: GoadLasts::WhileSourceOnBattlefield },
            ])),
            TriggeredAbility {
                event: EventSpec {
                    once_per_batch: true,
                    ..EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer)
                        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsGoaded })
                },
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..legend(
            "Kang, Suspicious Captain",
            cost(&[generic(3), r()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            4,
            4,
        )
    }
}

/// Koloth, Tribble Hunter — {3}{R} legendary 3/3 Klingon Warrior Officer.
/// Your end step: each player makes two 1/1 white Tribbles that can't block.
/// All Tribbles get +1/+0 and are goaded.
pub fn koloth_tribble_hunter() -> CardDefinition {
    let tribble = Arc::new(TokenDefinition {
        name: "Tribble".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Tribble], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    });
    let tribbles = || Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Tribble)));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::EachPlayer, count: Value::Const(2), definition: tribble },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "All Tribbles get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: tribbles(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "All Tribbles are goaded.",
                effect: StaticEffect::MatchingAreGoaded { filter: R::HasCreatureType(CreatureType::Tribble) },
            },
        ],
        ..legend(
            "Koloth, Tribble Hunter",
            cost(&[generic(3), r()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            3,
            3,
        )
    }
}

/// Own the Day — {4}{R}{R} sorcery. Battlefront — costs {1} less per opponent
/// you attacked this turn. Untap every creature that attacked this turn; an
/// extra combat and main phase follow this main phase.
pub fn own_the_day() -> CardDefinition {
    CardDefinition {
        name: "Own the Day",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each opponent you attacked this turn.",
            effect: StaticEffect::SelfCostReducedByValue { amount: battlefront() },
        }],
        effect: Effect::Seq(vec![
            Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::AttackedThisTurn)), up_to: None },
            Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Qo'noS Quartermaster — {1}{R} 2/2 Klingon Warrior, menace. Attacking adds
/// one mana of any color that lasts until end of turn.
pub fn qonos_quartermaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::AddManaKeptThisTurnAnyOneColor {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Qo'noS Quartermaster",
            cost(&[generic(1), r()]),
            vec![CreatureType::Klingon, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Alexander, Ship's Fool — {3}{R}{W} legendary 3/5 Klingon Human, vigilance,
/// haste. {T}: parley without a payoff, then X damage to a random opponent,
/// X the greatest mana value revealed.
pub fn alexander_ships_fool() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Parley {
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::RandomOpponent),
                    amount: Value::ParleyGreatestManaValue,
                }),
            },
            ..Default::default()
        }],
        ..legend(
            "Alexander, Ship's Fool",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Klingon, CreatureType::Human],
            3,
            5,
        )
    }
}

/// Chancellor Gorkon — {3}{W}{B} legendary 5/3 Klingon Noble, vigilance.
/// Dying exiles up to two target creatures.
pub fn chancellor_gorkon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_dies(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
        })],
        ..legend(
            "Chancellor Gorkon",
            cost(&[generic(3), w(), b()]),
            vec![CreatureType::Klingon, CreatureType::Noble],
            5,
            3,
        )
    }
}

/// Curzon Dax, Bound by Blood — {2}{R}{W} legendary 4/3 Trill Warrior.
/// Another Klingon or Warrior of yours dying deals 1 to each opponent; dying,
/// he lets you put a creature card with mana value 3 or less from your hand
/// onto the battlefield.
pub fn curzon_dax_bound_by_blood() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: kw() }),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            },
            on_dies(Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::ManaValueAtMost(3)),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            }),
        ],
        ..legend(
            "Curzon Dax, Bound by Blood",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Trill, CreatureType::Warrior],
            4,
            3,
        )
    }
}

/// Rite of Ascension — {2}{R}{W} enchantment. Battlefront — whenever you
/// attack, a hasty 2/1 Klingon Warrior enters tapped and attacking each
/// opponent you attacked this turn.
pub fn rite_of_ascension() -> CardDefinition {
    CardDefinition {
        name: "Rite of Ascension",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::If {
                    cond: Predicate::YouAttackedPlayerThisTurn { who: PlayerRef::Triggerer },
                    then: Box::new(Effect::CreateTokenAttacking {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: warrior_token(),
                        cleanup: Default::default(),
                        defender: Some(PlayerRef::Triggerer),
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Sins of the Father — {3}{B}{R} Saga. I — each opponent sacrifices a
/// nontoken creature. II — a hasty 2/1 Klingon Warrior per opponent. III —
/// exile target player's graveyard; each opponent loses a life per creature
/// card exiled.
pub fn sins_of_the_father() -> CardDefinition {
    let their_creatures = || Selector::CardsInZone {
        who: PlayerRef::Target(0),
        zone: crate::card::Zone::Graveyard,
        filter: R::Creature,
    };
    CardDefinition {
        name: "Sins of the Father",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature.and(R::IsToken.negate()),
                },
            ),
            (2, warriors(Value::OpponentCount)),
            (
                3,
                Effect::Seq(vec![
                    Effect::TargetPlayerThen { filter: R::Player, then: Box::new(Effect::Noop) },
                    // The loss is counted before the exile empties the yard.
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::CountOf(Box::new(their_creatures())),
                    },
                    Effect::Exile {
                        what: Selector::CardsInZone {
                            who: PlayerRef::Target(0),
                            zone: crate::card::Zone::Graveyard,
                            filter: R::Any,
                        },
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// I.K.S. Negh'Var, Imperial Flagship — {3} legendary Spacecraft. Your
/// Klingons and Warriors have vigilance. Station; 10+: 6/6 flying, vigilance.
pub fn i_k_s_neghvar_imperial_flagship() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Klingons and Warriors you control have vigilance.",
            effect: StaticEffect::GrantKeyword { applies_to: your_kw_creatures(), keyword: Keyword::Vigilance },
        }],
        ..spacecraft(
            "I.K.S. Negh'Var, Imperial Flagship",
            cost(&[generic(3)]),
            vec![StationBand {
                min: 10,
                keywords: vec![Keyword::Flying, Keyword::Vigilance],
                pt: Some((6, 6)),
                ..Default::default()
            }],
        )
    }
}

/// K't'inga-Class Battle Cruiser — {3} Spacecraft. Entering or attacking
/// gives one or two target Klingons and/or Warriors of yours flying until end
/// of turn. Station; 4+: 5/5 flying.
pub fn ktinga_class_battle_cruiser() -> CardDefinition {
    let wings = || Effect::ApplyToTargets {
        max_targets: 2,
        min_targets: 1,
        filter: R::Creature.and(R::ControlledByYou).and(kw()),
        effect: Box::new(Effect::GrantKeyword {
            what: Selector::Target(0),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        }),
    };
    CardDefinition {
        triggered_abilities: vec![etb(wings()), on_attack(wings())],
        ..spacecraft(
            "K't'inga-Class Battle Cruiser",
            cost(&[generic(3)]),
            vec![StationBand { min: 4, keywords: vec![Keyword::Flying], pt: Some((5, 5)), ..Default::default() }],
        )
    }
}

/// K'Vort-Class Raider — {4} Spacecraft. During your turn it and your
/// creatures have hexproof. Station; 4+: 5/3 flying, menace, haste.
pub fn kvort_class_raider() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this Spacecraft and creatures you control have hexproof.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::IsSource.or(R::Creature.and(R::ControlledByYou)),
                    ),
                    keyword: Keyword::Hexproof,
                }),
            },
        }],
        ..spacecraft(
            "K'Vort-Class Raider",
            cost(&[generic(4)]),
            vec![StationBand {
                min: 4,
                keywords: vec![Keyword::Flying, Keyword::Menace, Keyword::Haste],
                pt: Some((5, 3)),
                ..Default::default()
            }],
        )
    }
}

/// The Sword of Kahless — {4} legendary Equipment, equip {2}. +2/+2 and
/// "attacking, look at the top five; a Klingon or Warrior creature card may
/// enter; the rest go to the bottom at random".
pub fn the_sword_of_kahless() -> CardDefinition {
    CardDefinition {
        name: "The Sword of Kahless",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            triggered_abilities: vec![on_attack(Effect::LookTopPutMatchingOntoBattlefield {
                count: Value::Const(5),
                filter: R::Creature.and(kw()),
                then: None,
                max: Some(1),
                tapped: false,
                exile_rest: false,
                rest_to_graveyard: false,
                mandatory: false,
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Cha'DIch Investigator — {4}{B} 4/4 Klingon Warrior. Entering makes a hasty
/// 2/1 Klingon Warrior.
pub fn chadich_investigator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(warriors(Value::ONE))],
        ..creature(
            "Cha'DIch Investigator",
            cost(&[generic(4), b()]),
            vec![CreatureType::Klingon, CreatureType::Warrior],
            4,
            4,
        )
    }
}
