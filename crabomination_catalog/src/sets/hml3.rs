//! Homelands (HML) — the last twelve. Tests in `classic_sets/hml`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    ChainCopyCost, Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::target_filtered,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Autumn Willow — shroud, with a {G} waiver that opens her up to one player.
pub fn autumn_willow() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Shroud],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::WaiveShroudForPlayerThisTurn { player: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..creature("Autumn Willow", cost(&[generic(4), g(), g()]), vec![CreatureType::Avatar], 4, 4)
    }
}

/// Broken Visage — kill an attacker and take its body for the turn.
pub fn broken_visage() -> CardDefinition {
    let dead = || Box::new(Selector::DestroyedThisResolution { filter: R::Creature });
    CardDefinition {
        name: "Broken Visage",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::IsAttacking).and(R::Not(Box::new(R::Artifact)))),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Spirit".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spirit],
                        ..Default::default()
                    },
                    dynamic_pt: Some((
                        Value::PowerOf(dead()),
                        Value::ToughnessOf(dead()),
                    )),
                    ..Default::default()
                }),
            },
            Effect::SacrificeLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}

/// Chain Stasis — tap or untap a creature; its controller may pay {2}{U} to
/// keep the chain going.
pub fn chain_stasis() -> CardDefinition {
    CardDefinition {
        name: "Chain Stasis",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::TapOrUntap { what: target_filtered(R::Creature) },
            Effect::MayCopyThisSpell {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: ChainCopyCost::Mana(cost(&[generic(2), u()])),
            },
        ]),
        ..Default::default()
    }
}

/// Coral Reef — polyp counters off Islands, spent to toughen creatures.
pub fn coral_reef() -> CardDefinition {
    CardDefinition {
        name: "Coral Reef",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Polyp, Value::Const(4))),
        activated_abilities: vec![
            ActivatedAbility {
                sac_other_filter: Some((R::HasLandType(LandType::Island), 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Polyp,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_other_filter: Some(R::Creature.and(R::HasColor(Color::Blue))),
                remove_counter_cost: Some((CounterType::Polyp, 1)),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusZeroPlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dwarven Sea Clan — snipes an Island-holder's combatant at end of combat.
pub fn dwarven_sea_clan() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(before_end_of_combat()),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(
                        R::Creature
                            .and(R::IsAttacking.or(R::IsBlocking))
                            .and(R::ControllerControlsLandType(LandType::Island)),
                    ),
                    amount: Value::Const(2),
                }),
            },
            ..Default::default()
        }],
        ..creature("Dwarven Sea Clan", cost(&[generic(2), r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// "Activate only before the end of combat step."
fn before_end_of_combat() -> Predicate {
    Predicate::Any(
        [
            TurnStep::BeginCombat,
            TurnStep::DeclareAttackers,
            TurnStep::DeclareBlockers,
            TurnStep::CombatDamage,
        ]
        .map(Predicate::CurrentStepIs)
        .to_vec(),
    )
}

/// Giant Albatross — dies and drags down everything that hurt it.
pub fn giant_albatross() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::MayPay {
            description: "Pay {1}{U} to punish everything that damaged the Albatross?".into(),
            mana_cost: cost(&[generic(1), u()]),
            body: Box::new(Effect::DestroyEachUnlessPaysLife {
                filter: R::Creature.and(R::DealtDamageToSourceThisTurn),
                life: 2,
                no_regen: true,
            }),
            else_: None,
        })],
        ..creature("Giant Albatross", cost(&[generic(1), u()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Giant Oyster — tap to freeze a tapped creature and wither it each draw step.
pub fn giant_oyster() -> CardDefinition {
    let release = || Effect::RemoveCounter {
        what: Selector::ChosenPermanentOfSource,
        kind: CounterType::MinusOneMinusOne,
        amount: Value::CountersOn {
            what: Box::new(Selector::ChosenPermanentOfSource),
            kind: CounterType::MinusOneMinusOne,
        },
    };
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::TapAndUntapLock { what: target_filtered(R::Creature.and(R::Tapped)) },
                Effect::RememberPermanentOnSource { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec {
                    filter: Some(Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::Tapped,
                    }),
                    ..EventSpec::new(
                        EventKind::StepBegins(TurnStep::Draw),
                        EventScope::SelfSource,
                    )
                },
                effect: Effect::AddCounter {
                    what: Selector::ChosenPermanentOfSource,
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
                effect: release(),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: release(),
            },
        ],
        ..creature("Giant Oyster", cost(&[generic(2), u(), u()]), vec![CreatureType::Oyster], 0, 3)
    }
}

/// Jinx — retype a land for the turn, then cantrip.
pub fn jinx() -> CardDefinition {
    CardDefinition {
        name: "Jinx",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LandsBecomeChosenBasicType {
                what: target_filtered(R::Land),
                duration: Duration::EndOfTurn,
                            from_chosen_basic: false,
            },
            Effect::AtNextTurnsUpkeep {
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        ]),
        ..Default::default()
    }
}

/// Marjhan — an 8/8 that needs Islands to live, upkeep sacrifices to untap,
/// and pays {U}{U} to shoot down a ground attacker.
pub fn marjhan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessDefenderControlsLandType(LandType::Island)],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        state_trigger: Some(crate::card::StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::EachPermanent(
                    R::HasLandType(LandType::Island).and(R::ControlledByYou),
                ),
            ))),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::IsSource,
            },
        }),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u(), u()]),
                sac_other_filter: Some((R::Creature, 1)),
                condition: Some(Predicate::All(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::IsTurnOf(PlayerRef::You),
                ])),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u(), u()]),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(-1),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::DealDamage {
                        to: target_filtered(
                            R::Creature
                                .and(R::IsAttacking)
                                .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                        ),
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Marjhan", cost(&[generic(5), u(), u()]), vec![CreatureType::Serpent], 8, 8)
    }
}

/// Orcish Mine — three ore counters; when the last comes off, the land dies.
pub fn orcish_mine() -> CardDefinition {
    let strip = || Effect::RemoveCounter {
        what: Selector::This,
        kind: CounterType::Ore,
        amount: Value::ONE,
    };
    CardDefinition {
        name: "Orcish Mine",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        enters_with_counters: Some((CounterType::Ore, Value::Const(3))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                ),
                effect: strip(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::EnchantedBySource),
                effect: strip(),
            },
        ],
        state_trigger: Some(crate::card::StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast {
                counter: CounterType::Ore,
                n: 1,
            })),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(
                        Box::new(Selector::This),
                    )))),
                    amount: Value::Const(2),
                },
                Effect::Destroy { what: Selector::AttachedTo(Box::new(Selector::This)) },
            ]),
        }),
        ..Default::default()
    }
}

/// Retribution — two of an opponent's creatures; they pick which one dies.
pub fn retribution() -> CardDefinition {
    CardDefinition {
        name: "Retribution",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseOneAmong {
            what: Selector::Both(
                Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                }),
                Box::new(Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature
                        .and(R::ControlledByOpponent)
                        .and(R::SameControllerAsTargetSlot(0)),
                }),
            ),
            chooser: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            chosen: Box::new(Effect::SacrificeSelected {
                what: Selector::SeparatedPile { chosen: true },
            }),
            other: Box::new(Effect::AddCounter {
                what: Selector::SeparatedPile { chosen: false },
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            }),
        },
        ..Default::default()
    }
}

/// Rysorian Badger — trades its combat damage for graveyard exile and life.
pub fn rysorian_badger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            // "You may exile up to two target …" — declining every optional
            // slot is the printed "may".
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature.and(R::InOpponentGraveyard),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Exile,
                    }),
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::ExiledThisResolution {
                        filter: R::Any,
                    }),
                    then: Box::new(Effect::Seq(vec![
                        Effect::GainLife {
                            who: Selector::You,
                            amount: Value::CountOf(Box::new(Selector::ExiledThisResolution {
                                filter: R::Any,
                            })),
                        },
                        Effect::GrantKeyword {
                            what: Selector::This,
                            keyword: Keyword::DealsNoCombatDamage,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature("Rysorian Badger", cost(&[generic(2), g()]), vec![CreatureType::Badger], 2, 2)
    }
}
