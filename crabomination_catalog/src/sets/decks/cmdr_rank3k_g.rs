//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), seventh file. Tests in `tests/recent_b/cmdr_rank3k_g.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use std::sync::Arc;

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
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn equipment(
    name: &'static str,
    mana: ManaCost,
    equip: ManaCost,
    bonus: EquipBonus,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter,
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn counters(what: Selector, n: Value) -> Effect {
    Effect::AddCounter {
        what,
        kind: CounterType::PlusOnePlusOne,
        amount: n,
    }
}

fn draw(n: Value) -> Effect {
    Effect::Draw {
        who: Selector::You,
        amount: n,
    }
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn your_end_step() -> EventSpec {
    EventSpec::new(
        EventKind::StepBegins(TurnStep::End),
        EventScope::YourControl,
    )
}

// ── Ian the Reckless ─────────────────────────────────────────────────────────

/// Butcher's Cleaver — {3} Equipment: +3/+0; lifelink while the equipped
/// creature is a Human. Equip {3}.
pub fn butchers_cleaver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as equipped creature is a Human, it has lifelink.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    filter: R::HasCreatureType(CreatureType::Human),
                },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    keyword: Keyword::Lifelink,
                }),
            },
        }],
        ..equipment(
            "Butcher's Cleaver",
            cost(&[generic(3)]),
            cost(&[generic(3)]),
            EquipBonus {
                power: 3,
                ..Default::default()
            },
        )
    }
}

/// Auntie Blyte, Bad Influence — {2}{R} legendary 2/2 flying Devil Advisor.
/// Whenever a source you control deals damage to you, that many +1/+1 counters
/// on her. {1}{R}, {T}, remove X +1/+1 counters: X damage to any target.
pub fn auntie_blyte_bad_influence() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PlayerDamaged,
                EventScope::YourSourceDamagedAnyPlayer,
            )
            .with_filter(Predicate::SamePlayer(
                PlayerRef::TriggerEventPlayer,
                PlayerRef::You,
            )),
            effect: counters(Selector::This, Value::TriggerEventAmount),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            remove_counter_x: Some(CounterType::PlusOnePlusOne),
            effect: Effect::DealDamage {
                to: target_filtered(R::Any),
                amount: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..legend(
            "Auntie Blyte, Bad Influence",
            cost(&[generic(2), r()]),
            vec![CreatureType::Devil, CreatureType::Advisor],
            2,
            2,
        )
    }
}

// ── Doctor Strange, Surgeon ──────────────────────────────────────────────────

/// Doctor Strange, Surgeon — {4}{W} legendary 2/5 lifelink. You gain twice the
/// life you would. At the beginning of each combat, if you have at least 10
/// life more than your starting total, your creatures get +2/+2 and vigilance
/// until end of turn.
pub fn doctor_strange_surgeon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain twice that much life instead.",
            effect: StaticEffect::LifeGainMultiplier {
                target: PlayerStaticTarget::Controller,
                factor: 2,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::PlayerLifeAtLeastAboveStarting {
                who: PlayerRef::You,
                delta: 10,
            }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: yours(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: yours(R::Creature),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..legend(
            "Doctor Strange, Surgeon",
            cost(&[generic(4), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Doctor,
                CreatureType::Hero,
            ],
            2,
            5,
        )
    }
}

/// MJ, Rising Star — {2}{W} legendary 2/3 vigilance; whenever you gain life, a
/// +1/+1 counter on her.
pub fn mj_rising_star() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: counters(Selector::This, Value::ONE),
        }],
        ..legend(
            "MJ, Rising Star",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Performer],
            2,
            3,
        )
    }
}

// ── Wiccan, Rising Magician ──────────────────────────────────────────────────

/// Wiccan, Rising Magician — {4}{U} legendary 4/4 flyer. Whenever you cast a
/// noncreature spell, exile another target nonland, nontoken permanent; it
/// returns under its owner's control at the beginning of the next end step.
pub fn wiccan_rising_magician() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(trigger_is(R::HasCardType(CardType::Creature).negate())),
            effect: Effect::ExileReturnToOwnerNextEndStep {
                what: target_filtered(
                    R::Permanent
                        .and(R::Land.negate())
                        .and(R::NotToken)
                        .and(R::OtherThanSource),
                ),
                tapped: false,
            },
        }],
        ..legend(
            "Wiccan, Rising Magician",
            cost(&[generic(4), u()]),
            vec![
                CreatureType::Mutant,
                CreatureType::Warlock,
                CreatureType::Hero,
            ],
            4,
            4,
        )
    }
}

/// H.E.R.B.I.E. Scout Unit — {4} 2/1 flying Robot Scout artifact creature.
/// ETB: draw, then you may put a land card from your hand onto the battlefield
/// tapped.
pub fn herbie_scout_unit() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(Value::ONE),
            Effect::PutLandsFromHandOntoBattlefieldTapped { count: Value::ONE },
        ]))],
        ..creature(
            "H.E.R.B.I.E. Scout Unit",
            cost(&[generic(4)]),
            vec![CreatureType::Robot, CreatureType::Scout],
            2,
            1,
        )
    }
}

// ── Devil Dinosaur ───────────────────────────────────────────────────────────

/// Moon-Boy, Dino Rider — {1}{G} legendary 2/2. Dinosaur spells you cast cost
/// {1} less. Attacks while you control a Dinosaur: +1/+1 until end of turn.
pub fn moon_boy_dino_rider() -> CardDefinition {
    let dino = || R::HasCreatureType(CreatureType::Dinosaur);
    let mut t = on_attack(Effect::PumpPT {
        what: Selector::This,
        power: Value::ONE,
        toughness: Value::ONE,
        duration: Duration::EndOfTurn,
    });
    t.event = t
        .event
        .with_filter(Predicate::SelectorExists(yours(R::Creature.and(dino()))));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Dinosaur spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: dino(),
                amount: 1,
            },
        }],
        triggered_abilities: vec![t],
        ..legend(
            "Moon-Boy, Dino Rider",
            cost(&[generic(1), g()]),
            vec![CreatureType::Ape, CreatureType::Human],
            2,
            2,
        )
    }
}

/// Sauron, Dino Devotee — {3}{G}{G} legendary 4/4 flyer. Enters or attacks,
/// choose one: gain 3 life; or a saurian counter on another target creature —
/// it's a green Dinosaur with base power and toughness 5/5 while it has one.
pub fn sauron_dino_devotee() -> CardDefinition {
    let saurian = || Duration::WhileHasCounter(CounterType::Saurian);
    let modes = || {
        Effect::ChooseMode(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    kind: CounterType::Saurian,
                    amount: Value::ONE,
                },
                Effect::BecomeCreatureType {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Dinosaur],
                    duration: saurian(),
                },
                Effect::BecomeColor {
                    what: Selector::Target(0),
                    colors: vec![Color::Green],
                    duration: saurian(),
                    additive: false,
                },
                Effect::SetBasePT {
                    what: Selector::Target(0),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    duration: saurian(),
                },
            ]),
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(modes()), on_attack(modes())],
        ..legend(
            "Sauron, Dino Devotee",
            cost(&[generic(3), g(), g()]),
            vec![
                CreatureType::Dinosaur,
                CreatureType::Vampire,
                CreatureType::Villain,
            ],
            4,
            4,
        )
    }
}

// ── Nyla, Shirshu Sleuth ─────────────────────────────────────────────────────

/// Nyla, Shirshu Sleuth — {4}{B} legendary 4/5 Mole Beast. ETB: exile up to one
/// target creature card from your graveyard; if you do, lose X life and create
/// X Clues, X its mana value. Your end step, if you control no Clues: return
/// target card exiled with Nyla to its owner's hand.
pub fn nyla_shirshu_sleuth() -> CardDefinition {
    let x = || Value::ManaValueOf(Box::new(Selector::Target(0)));
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                        to: ZoneDest::ExileWithSourceStamp,
                    },
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: x(),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: x(),
                        definition: Arc::new(crabomination_base::tokens::clue_token()),
                    },
                ])),
            }),
            TriggeredAbility {
                event: your_end_step().with_filter(Predicate::Not(Box::new(
                    Predicate::SelectorExists(yours(R::HasArtifactSubtype(ArtifactSubtype::Clue))),
                ))),
                effect: Effect::Move {
                    what: target_filtered(R::ExiledWithSource),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
            },
        ],
        ..legend(
            "Nyla, Shirshu Sleuth",
            cost(&[generic(4), b()]),
            vec![CreatureType::Mole, CreatureType::Beast],
            4,
            5,
        )
    }
}

/// Apex Witchstalker — {4}{B}{B} 6/4 menace Wolf. Enters or dies: gain 2 life.
/// Basic landcycling {2}.
pub fn apex_witchstalker() -> CardDefinition {
    let gain = || Effect::GainLife {
        who: Selector::You,
        amount: Value::Const(2),
    };
    CardDefinition {
        keywords: vec![
            Keyword::Menace,
            Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand))),
        ],
        triggered_abilities: vec![etb(gain()), on_dies(gain())],
        ..creature(
            "Apex Witchstalker",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Wolf],
            6,
            4,
        )
    }
}

// ── Lulu, Loyal Hollyphant ───────────────────────────────────────────────────

/// Lulu, Loyal Hollyphant — {3}{W} legendary 3/2 flying Elephant Angel, choose
/// a Background. Your end step, if a permanent you controlled left the
/// battlefield this turn: a +1/+1 counter on each tapped creature you control,
/// then untap them.
pub fn lulu_loyal_hollyphant() -> CardDefinition {
    let tapped = || yours(R::Creature.and(R::Tapped));
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step().with_filter(Predicate::RevoltActive {
                who: PlayerRef::You,
            }),
            effect: Effect::Seq(vec![
                counters(tapped(), Value::ONE),
                Effect::Untap {
                    what: tapped(),
                    up_to: None,
                },
            ]),
        }],
        ..legend(
            "Lulu, Loyal Hollyphant",
            cost(&[generic(3), w()]),
            vec![CreatureType::Elephant, CreatureType::Angel],
            3,
            2,
        )
    }
}

/// Steward of Solidarity — {1}{W} 2/2 Human Warrior. {T}, exert it: a 1/1 white
/// vigilant Warrior. (Residual: the exert lands on resolution.)
pub fn steward_of_solidarity() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::SkipNextUntap {
                    what: Selector::This,
                },
                make(
                    Value::ONE,
                    TokenDefinition {
                        name: "Warrior".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        keywords: vec![Keyword::Vigilance],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Warrior],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Steward of Solidarity",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Rent Is Due — {W} enchantment. Your end step: you may tap two untapped
/// creatures and/or Treasures you control; if you do, draw a card; otherwise
/// sacrifice it.
pub fn rent_is_due() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step(),
            effect: Effect::MayTap {
                description: "Tap two creatures and/or Treasures to draw a card?".into(),
                filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Treasure)),
                count: Value::Const(2),
                then: Box::new(draw(Value::ONE)),
                else_: Some(Box::new(Effect::SacrificeSource)),
            },
        }],
        ..spell(
            "Rent Is Due",
            cost(&[w()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Denethor, Ruling Steward ─────────────────────────────────────────────────

/// Gríma Wormtongue — {2}{B} legendary 1/4. Your opponents can't gain life.
/// {T}, sacrifice another creature: target player loses 1 life; if the
/// sacrificed creature was legendary, amass Orcs 2.
pub fn grima_wormtongue() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: PlayerStaticTarget::EachOpponent,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: target_filtered(R::Player),
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::SacrificedCard,
                        filter: R::HasSupertype(Supertype::Legendary),
                    },
                    then: Box::new(Effect::Amass {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        extra_type: Some(CreatureType::Orc),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Gríma Wormtongue",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            4,
        )
    }
}

/// Stir Up Trouble — {B} sorcery; as an additional cost, sacrifice an artifact
/// or creature or pay {4}. Destroy target creature.
pub fn stir_up_trouble() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay {
            filter: R::Artifact.or(R::Creature),
            pay: 4,
        }],
        ..spell(
            "Stir Up Trouble",
            cost(&[b()]),
            CardType::Sorcery,
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
        )
    }
}

// ── Vision, Synthezoid Avenger ───────────────────────────────────────────────

/// Manhole Cover — {2} artifact, flash. ETB: target creature gains
/// indestructible until end of turn. {2}, sacrifice it: target player draws.
pub fn manhole_cover() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Indestructible,
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw {
                who: target_filtered(R::Player),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..spell(
            "Manhole Cover",
            cost(&[generic(2)]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Sting, Bilbo's Sword — {2} legendary Equipment, flash. ETB: a hone counter on
/// it per creature target opponent controls; attach it to up to one target
/// creature you control. Each hone counter grants +1/+0 to the equipped
/// creature. Equip {3}.
pub fn sting_bilbos_sword() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![StaticAbility {
            description: "Each hone counter on this Equipment grants +1/+0 to equipped creature.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Hone,
                },
                toughness: Value::ZERO,
            },
        }],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Hone,
                    amount: Value::PermanentCountControlledByMatching(
                        PlayerRef::Target(0),
                        R::Creature,
                    ),
                },
                Effect::Attach {
                    what: Selector::This,
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                },
            ])),
        })],
        ..equipment(
            "Sting, Bilbo's Sword",
            cost(&[generic(2)]),
            cost(&[generic(3)]),
            EquipBonus::default(),
        )
    }
}

// ── Starke of Rath ───────────────────────────────────────────────────────────

/// Conquering Manticore — {4}{R}{R} 5/5 flyer. ETB: gain control of target
/// creature an opponent controls until end of turn; untap it; it gains haste.
pub fn conquering_manticore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Conquering Manticore",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Manticore],
            5,
            5,
        )
    }
}

/// Glacial Crevasses — {2}{R} enchantment. Sacrifice a snow Mountain: prevent
/// all combat damage this turn.
pub fn glacial_crevasses() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                R::HasLandType(LandType::Mountain).and(R::HasSupertype(Supertype::Snow)),
                1,
            )),
            effect: Effect::PreventAllCombatDamageThisTurn,
            ..Default::default()
        }],
        ..spell(
            "Glacial Crevasses",
            cost(&[generic(2), r()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Skyfire Kirin ────────────────────────────────────────────────────────────

/// Culling Dais — {2} artifact. {T}, sacrifice a creature: a charge counter.
/// {1}, sacrifice it: draw a card per charge counter.
pub fn culling_dais() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: draw(Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                }),
                ..Default::default()
            },
        ],
        ..spell(
            "Culling Dais",
            cost(&[generic(2)]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Hazoret's Favor — {2}{R} enchantment. At the beginning of combat on your
/// turn, you may have target creature you control get +2/+0 and haste until end
/// of turn; if you do, sacrifice it at the beginning of the next end step.
pub fn hazorets_favor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Give a creature +2/+0 and haste, sacrificing it at end step?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        power: Value::Const(2),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::SacrificeAtNextEndStep {
                        what: Selector::Target(0),
                    },
                ])),
            },
        }],
        ..spell(
            "Hazoret's Favor",
            cost(&[generic(2), r()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Astrid Peth ──────────────────────────────────────────────────────────────

/// Technodrome — {2} 3/3 reach, trample Construct artifact creature; can't
/// attack or block unless its power is 6 or greater. {T}, sacrifice another
/// artifact: draw a card and a +1/+1 counter on it.
pub fn technodrome() -> CardDefinition {
    let small = || {
        Predicate::Not(Box::new(Predicate::EntityMatches {
            what: Selector::This,
            filter: R::PowerAtLeast(6),
        }))
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Reach, Keyword::Trample],
        static_abilities: [Keyword::CantAttack, Keyword::CantBlock]
            .into_iter()
            .map(|keyword| StaticAbility {
                description: "This creature can't attack or block unless its power is 6 or greater.",
                effect: StaticEffect::WhileCondition {
                    condition: small(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword }),
                },
            })
            .collect(),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Seq(vec![draw(Value::ONE), counters(Selector::This, Value::ONE)]),
            ..Default::default()
        }],
        ..creature("Technodrome", cost(&[generic(2)]), vec![CreatureType::Construct], 3, 3)
    }
}

/// Angelic Purge — {2}{W} sorcery; as an additional cost, sacrifice a
/// permanent. Exile target artifact, creature or enchantment.
pub fn angelic_purge() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Permanent,
            count: 1,
        }],
        ..spell(
            "Angelic Purge",
            cost(&[generic(2), w()]),
            CardType::Sorcery,
            Effect::Exile {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Enchantment)),
            },
        )
    }
}
