//! The Brothers' War (BRO) — 2022. Introduces Prototype (CR 702.160):
//! a colorless artifact creature may instead be cast for a smaller, colored
//! prototype cost, entering with that mana cost, color, and size while
//! keeping its abilities and types (`CardDefinition.prototype` +
//! `GameAction::CastPrototype`).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Prototype,
    SelectionRequirement, Selector, Subtypes, WardCost,
};
use crate::effect::shortcut::{draw, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Value};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// Helper: a Prototype face from its cost symbols and printed size.
fn proto(c: ManaCost, power: i32, toughness: i32) -> Option<Box<Prototype>> {
    Some(Box::new(Prototype { cost: c, power, toughness }))
}

fn construct(creature_types: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types, ..Default::default() }
}

/// Goring Warplow — {6} 5/4 Construct. Prototype {1}{B} — 1/1. Deathtouch.
pub fn goring_warplow() -> CardDefinition {
    CardDefinition {
        name: "Goring Warplow",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Construct]),
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        prototype: proto(cost(&[generic(1), b()]), 1, 1),
        ..Default::default()
    }
}

/// Blitz Automaton — {7} 6/4 Construct. Prototype {2}{R} — 3/2. Haste.
pub fn blitz_automaton() -> CardDefinition {
    CardDefinition {
        name: "Blitz Automaton",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Construct]),
        power: 6,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        prototype: proto(cost(&[generic(2), r()]), 3, 2),
        ..Default::default()
    }
}

/// Rust Goliath — {10} 10/10 Construct. Prototype {3}{G}{G} — 3/5. Reach, trample.
pub fn rust_goliath() -> CardDefinition {
    CardDefinition {
        name: "Rust Goliath",
        cost: cost(&[generic(10)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Construct]),
        power: 10,
        toughness: 10,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        prototype: proto(cost(&[generic(3), g(), g()]), 3, 5),
        ..Default::default()
    }
}

/// Combat Thresher — {7} 3/3 Construct. Prototype {2}{W} — 1/1. Double strike.
/// "When this creature enters, draw a card."
pub fn combat_thresher() -> CardDefinition {
    CardDefinition {
        name: "Combat Thresher",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Construct]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![etb(draw(1))],
        prototype: proto(cost(&[generic(2), w()]), 1, 1),
        ..Default::default()
    }
}

/// Boulderbranch Golem — {7} 6/5 Golem. Prototype {3}{G} — 3/3.
/// "When this creature enters, you gain life equal to its power."
pub fn boulderbranch_golem() -> CardDefinition {
    CardDefinition {
        name: "Boulderbranch Golem",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Golem]),
        power: 6,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        prototype: proto(cost(&[generic(3), g()]), 3, 3),
        ..Default::default()
    }
}

/// Spotter Thopter — {8} 4/5 Thopter. Prototype {3}{U} — 2/3. Flying.
/// "When this creature enters, scry X, where X is its power."
pub fn spotter_thopter() -> CardDefinition {
    CardDefinition {
        name: "Spotter Thopter",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Thopter]),
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        prototype: proto(cost(&[generic(3), u()]), 2, 3),
        ..Default::default()
    }
}

/// Cradle Clearcutter — {6} 3/6 Golem. Prototype {2}{G} — 1/3.
/// "{T}: Add an amount of {G} equal to this creature's power."
pub fn cradle_clearcutter() -> CardDefinition {
    CardDefinition {
        name: "Cradle Clearcutter",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Golem]),
        power: 3,
        toughness: 6,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        prototype: proto(cost(&[generic(2), g()]), 1, 3),
        ..Default::default()
    }
}

/// Fallaji Dragon Engine — {8} 5/5 Dragon. Prototype {2}{R} — 1/3. Flying.
/// "{2}: This creature gets +1/+0 until end of turn."
pub fn fallaji_dragon_engine() -> CardDefinition {
    CardDefinition {
        name: "Fallaji Dragon Engine",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Dragon]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        prototype: proto(cost(&[generic(2), r()]), 1, 3),
        ..Default::default()
    }
}

/// Autonomous Assembler — {5} 4/5 Assembly-Worker. Prototype {1}{W} — 2/2.
/// Vigilance. "{1}, {T}: Put a +1/+1 counter on target Assembly-Worker you
/// control." (The "you control" rider is approximated as any Assembly-Worker.)
pub fn autonomous_assembler() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Autonomous Assembler",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::AssemblyWorker]),
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::HasCreatureType(
                    CreatureType::AssemblyWorker,
                )),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        prototype: proto(cost(&[generic(1), w()]), 2, 2),
        ..Default::default()
    }
}

/// Iron-Craw Crusher — {7} 4/6 Wurm. Prototype {2}{G}{G} — 2/5.
/// "Whenever this creature attacks, target attacking creature gets +X/+0
/// until end of turn, where X is this creature's power."
pub fn iron_craw_crusher() -> CardDefinition {
    CardDefinition {
        name: "Iron-Craw Crusher",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Wurm]),
        power: 4,
        toughness: 6,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::IsAttacking),
            power: Value::PowerOf(Box::new(Selector::This)),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        prototype: proto(cost(&[generic(2), g(), g()]), 2, 5),
        ..Default::default()
    }
}

/// Skitterbeam Battalion — {9} 4/4 Construct. Prototype {3}{R}{R} — 2/2.
/// Trample, haste. "When this creature enters, if you cast it, create two
/// tokens that are copies of it." The `SourceWasCast` gate keeps the token
/// copies (and reanimated bodies) from re-triggering.
pub fn skitterbeam_battalion() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Skitterbeam Battalion",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Construct]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(2),
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
            }),
            else_: Box::new(Effect::Noop),
        })],
        prototype: proto(cost(&[generic(3), r(), r()]), 2, 2),
        ..Default::default()
    }
}

/// Phyrexian Fleshgorger — {7} 7/5 Phyrexian Wurm. Prototype {1}{B}{B} — 3/3.
/// Menace, lifelink. "Ward—Pay life equal to this creature's power."
pub fn phyrexian_fleshgorger() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Fleshgorger",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Phyrexian, CreatureType::Wurm]),
        power: 7,
        toughness: 5,
        keywords: vec![
            Keyword::Menace,
            Keyword::Lifelink,
            Keyword::Ward(WardCost::LifeSourcePower),
        ],
        prototype: proto(cost(&[generic(1), b(), b()]), 3, 3),
        ..Default::default()
    }
}

/// Steel Seraph — {6} 5/4 Angel. Prototype {1}{W}{W} — 3/3. Flying.
/// "At the beginning of combat on your turn, target creature you control
/// gains your choice of flying, vigilance, or lifelink until end of turn."
/// (The modal keyword choice is approximated as granting flying.)
pub fn steel_seraph() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Steel Seraph",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Angel]),
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        }],
        prototype: proto(cost(&[generic(1), w(), w()]), 3, 3),
        ..Default::default()
    }
}

// ── BRO non-prototype cards ──────────────────────────────────────────────────

/// Diabolic Intent — {1}{B} Sorcery. Additional cost: sacrifice a creature.
/// Search your library for a card, put it into your hand, then shuffle.
pub fn diabolic_intent() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Diabolic Intent",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Creature,
            count: 1,
        }],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Recommission — {1}{W} Sorcery. Return target artifact or creature card with
/// mana value 3 or less from your graveyard to the battlefield; if a creature
/// enters this way it gets an additional +1/+1 counter. (The counter is placed
/// on whatever returns; it's inert on a noncreature artifact.)
pub fn recommission() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Recommission",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Artifact)
                        .and(SelectionRequirement::ManaValueAtMost(3)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Depth Charge Colossus — {9} 9/9 Dreadnought. Prototype {4}{U}{U} — 6/6.
/// "This creature doesn't untap during your untap step. / {3}: Untap this."
pub fn depth_charge_colossus() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Depth Charge Colossus",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Dreadnought]),
        power: 9,
        toughness: 9,
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        prototype: proto(cost(&[generic(4), u(), u()]), 6, 6),
        ..Default::default()
    }
}

/// Bitter Reunion — {1}{R} Enchantment. ETB: you may discard a card; if you
/// do, draw two. "{1}, Sacrifice this: Creatures you control gain haste."
pub fn bitter_reunion() -> CardDefinition {
    use crate::effect::shortcut::{each_your_creature, you};
    CardDefinition {
        name: "Bitter Reunion",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "discard a card, then draw two".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: you(), amount: Value::Const(1), random: false },
                draw(2),
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Powerstone Shard — {3} Artifact. "{T}: Add {C} for each artifact you
/// control named Powerstone Shard."
pub fn powerstone_shard() -> CardDefinition {
    CardDefinition {
        name: "Powerstone Shard",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::CountOf(Box::new(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasName("Powerstone Shard".into()),
                }))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tocasia's Welcome — {2}{W} Enchantment. "Whenever one or more creatures you
/// control with mana value 3 or less enter, draw a card. This ability triggers
/// only once each turn."
pub fn tocasias_welcome() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::Predicate;
    CardDefinition {
        name: "Tocasia's Welcome",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
                })
                .once_per_turn(),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// Aeronaut Cavalry — {4}{W} 3/4 Human Soldier. Flying. "When this enters,
/// put a +1/+1 counter on another target Soldier you control."
pub fn aeronaut_cavalry() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Aeronaut Cavalry",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: construct(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Soldier)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Penregon Strongbull — {2}{R} 2/3 Minotaur. "{1}, Sacrifice an artifact:
/// This creature gets +1/+1 until end of turn and deals 1 damage to each
/// opponent."
pub fn penregon_strongbull() -> CardDefinition {
    use crate::effect::shortcut::each_opponent;
    CardDefinition {
        name: "Penregon Strongbull",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: construct(vec![CreatureType::Minotaur]),
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((SelectionRequirement::Artifact, 1)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::DealDamage { to: each_opponent(), amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Phyrexian Warhorse — {3}{B} 3/3 Phyrexian Horse. Kicker {W}; if kicked,
/// ETB creates a 1/1 white Soldier. "{1}, Sacrifice another creature: This
/// creature gets +2/+1 until end of turn."
pub fn phyrexian_warhorse() -> CardDefinition {
    use crate::card::{TokenDefinition, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::Predicate;
    CardDefinition {
        name: "Phyrexian Warhorse",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: construct(vec![CreatureType::Phyrexian, CreatureType::Horse]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Kicker(cost(&[w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Soldier".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: construct(vec![CreatureType::Soldier]),
                        ..Default::default()
                    },
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Frogmyr Enforcer — {7} 4/4 Frog Myr. Prototype {3}{R} — 2/2.
/// Affinity for artifacts (CR 702.41 — this spell costs {1} less per artifact
/// you control).
pub fn frogmyr_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Frogmyr Enforcer",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: construct(vec![CreatureType::Frog, CreatureType::Myr]),
        power: 4,
        toughness: 4,
        affinity_filter: Some(SelectionRequirement::Artifact),
        prototype: proto(cost(&[generic(3), r()]), 2, 2),
        ..Default::default()
    }
}
