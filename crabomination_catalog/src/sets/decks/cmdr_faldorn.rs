//! Commander: the cards the **Exit from Exile** precon (CLB, Faldorn, Dread
//! Wolf Herald) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Aurora Phoenix** — a spell given cascade by a trigger (Wild-Magic
//!   Sorcerer) doesn't have the keyword, so it doesn't return the Phoenix.
//! - **Durnan** — exiles the first creature card among the top four (no
//!   choice), and the cast from exile doesn't have undaunted.

use crate::card::{
    ActivatedAbility, Adventure, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, LandType, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{cascade, etb, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate, RevealMissDest,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r};

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

fn wolf() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: std::sync::Arc::new(TokenDefinition {
            name: "Wolf".into(),
            power: 2,
            toughness: 2,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Green],
            subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
            ..Default::default()
        }),
    }
}

fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

fn cast_free(what: Selector) -> Effect {
    Effect::CastWithoutPayingImmediate {
        reduce_generic: 0,
        pay_own_cost: false,
        what,
        source_zone: Zone::Exile,
        exile_after: false,
        copy: false,
    }
}

fn impulse(count: i32, duration: MayPlayDuration, pay_own_cost: bool) -> Effect {
    Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::Const(count),
        duration,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost,
        uncast_penalty: None,
    }
}

/// Faldorn, Dread Wolf Herald — a 2/2 Wolf whenever you cast a spell from
/// exile or a land you control enters from exile; {1}, {T}, discard a card:
/// exile the top card of your library, playable this turn.
pub fn faldorn_dread_wolf_herald() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellFromExile),
                effect: wolf(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Land.and(R::EnteredFromExileThisTurn),
                    }),
                effect: wolf(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: impulse(1, MayPlayDuration::EndOfThisTurn, true),
            ..Default::default()
        }],
        ..creature(
            "Faldorn, Dread Wolf Herald",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Aurora Phoenix — flying, cascade; casting a spell with cascade returns it
/// from your graveyard to your hand.
///
/// Approximation: a spell given cascade by a trigger doesn't count.
pub fn aurora_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Cascade],
        triggered_abilities: vec![
            cascade(6),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasKeyword(Keyword::Cascade),
                    }),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            },
        ],
        ..creature("Aurora Phoenix", cost(&[generic(4), r(), r()]), vec![CreatureType::Phoenix], 5, 3)
    }
}

/// Chaos Wand — {4}, {T}: target opponent exiles cards from the top of their
/// library until an instant or sorcery; you may cast it free; the rest go to
/// the bottom, an uncast find with them.
pub fn chaos_wand() -> CardDefinition {
    CardDefinition {
        name: "Chaos Wand",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![
                    Effect::RevealUntilFind {
                        who: PlayerRef::Target(0),
                        find: R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery)),
                        to: ZoneDest::Exile,
                        cap: Value::Const(60),
                        life_per_revealed: 0,
                        miss_dest: RevealMissDest::BottomRandom,
                    },
                    cast_free(Selector::ExiledThisResolution {
                        filter: R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery)),
                    }),
                    // An uncast find goes to the bottom with the rest.
                    Effect::Move {
                        what: Selector::ExiledThisResolution { filter: R::InExile },
                        to: ZoneDest::Library { who: PlayerRef::Target(0), pos: crate::effect::LibraryPosition::Bottom },
                    },
                ])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dire Fleet Daredevil — first strike; on entry, exile an instant or sorcery
/// card from an opponent's graveyard; you may cast it this turn with any
/// mana, and it's exiled after.
pub fn dire_fleet_daredevil() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::InOpponentGraveyard),
                ),
                to: ZoneDest::Exile,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: true,
                pay_own_cost: true,
                any_color: true,
            },
        ]))],
        ..creature(
            "Dire Fleet Daredevil",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            1,
        )
    }
}

/// Durnan of the Yawning Portal — on attack, exile a creature card from the
/// top four of your library; you may cast it while it stays exiled.
///
/// Approximation: the first creature card among the four is exiled (no
/// choice), and its cast has no undaunted.
pub fn durnan_of_the_yawning_portal() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature,
                    to: ZoneDest::Exile,
                    cap: Value::Const(4),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
                Effect::GrantMayPlay {
                    what: Selector::ExiledThisResolution { filter: R::Creature },
                    duration: MayPlayDuration::WhileExiled,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
        }],
        ..creature(
            "Durnan of the Yawning Portal",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Greater Gargadon — suspend 10—{R}; while suspended, sacrifice an
/// artifact, creature or land: remove a time counter.
pub fn greater_gargadon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(10, cost(&[r()]))],
        activated_abilities: vec![ActivatedAbility {
            from_exile: true,
            sac_other_filter: Some((R::Artifact.or(R::Creature).or(R::Land), 1)),
            condition: Some(Predicate::ValueAtLeast(
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Time },
                Value::ONE,
            )),
            effect: Effect::RemoveTimeCounterFromSuspendedSource,
            ..Default::default()
        }],
        ..creature("Greater Gargadon", cost(&[generic(9), r()]), vec![CreatureType::Beast], 9, 7)
    }
}

/// Green Slime — flash, foretell {G}; on entry, counter an ability of an
/// artifact or enchantment and destroy its source.
pub fn green_slime() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        foretell_cost: Some(cost(&[g()])),
        triggered_abilities: vec![etb(Effect::CounterAbilityAndDestroySource {
            what: target_filtered(R::Artifact.or(R::Enchantment).and(R::HasAbilityOnStack)),
        })],
        ..creature("Green Slime", cost(&[generic(2), g()]), vec![CreatureType::Ooze], 2, 2)
    }
}

/// Highland Forest — snow Mountain Forest that enters tapped.
pub fn highland_forest() -> CardDefinition {
    let mut d = crate::sets::dual_land_with(
        "Highland Forest",
        LandType::Mountain,
        LandType::Forest,
        Color::Red,
        Color::Green,
        vec![],
    );
    d.supertypes.push(Supertype::Snow);
    d.static_abilities.push(crate::sets::enters_tapped());
    d
}

/// Ignite the Future — exile your top three, playable until the end of your
/// next turn (free if this was cast from a graveyard); flashback {7}{R}.
pub fn ignite_the_future() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(7), r()]))],
        ..spell(
            "Ignite the Future",
            cost(&[generic(3), r()]),
            CardType::Sorcery,
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(impulse(3, MayPlayDuration::EndOfControllersNextTurn, false)),
                else_: Box::new(impulse(3, MayPlayDuration::EndOfControllersNextTurn, true)),
            },
        )
    }
}

/// Journey to the Lost City — each upkeep, exile your top four and roll a
/// d20: 1-9 a land from among them onto the battlefield; 10-19 a Wolf with a
/// +1/+1 counter per creature card among them; 20 every permanent card it
/// exiled onto the battlefield, then sacrifice it.
pub fn journey_to_the_lost_city() -> CardDefinition {
    let exiled = |filter: R| Selector::ExiledThisResolution { filter };
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::Seq(vec![
            Effect::ExileTopOfLibrary {
                who: Selector::You,
                amount: Value::Const(4),
                link_to_source: true,
                face_down: false,
            },
            Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::Const(0),
                reroll_at_most: 0,
                results: vec![
                    (
                        1,
                        9,
                        Effect::MayDo {
                            description: "Put a land card from among them onto the battlefield?"
                                .into(),
                            body: Box::new(Effect::Move {
                                what: Selector::Take {
                                    inner: Box::new(exiled(R::Land)),
                                    count: Box::new(Value::ONE),
                                },
                                to: ZoneDest::Battlefield {
                                    controller: PlayerRef::You,
                                    tapped: false,
                                },
                            }),
                        },
                    ),
                    (
                        10,
                        19,
                        Effect::Seq(vec![
                            wolf(),
                            Effect::AddCounter {
                                what: Selector::LastCreatedToken,
                                kind: CounterType::PlusOnePlusOne,
                                amount: Value::CountOf(Box::new(exiled(R::Creature))),
                            },
                        ]),
                    ),
                    (
                        20,
                        20,
                        Effect::Seq(vec![
                            Effect::Move {
                                what: Selector::MatchingAmong {
                                    inner: Box::new(Selector::CardExiledWithSource),
                                    filter: R::PermanentCard,
                                },
                                to: ZoneDest::Battlefield {
                                    controller: PlayerRef::You,
                                    tapped: false,
                                },
                            },
                            Effect::Sacrifice {
                                who: Selector::You,
                                count: Value::ONE,
                                filter: R::IsSource,
                            },
                        ]),
                    ),
                ],
                ignore_lowest: 0,
                on_doubles: None,
            },
        ]))],
        ..spell("Journey to the Lost City", cost(&[generic(3), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Natural Reclamation — cascade; destroy an artifact or enchantment.
pub fn natural_reclamation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![cascade(5)],
        ..spell(
            "Natural Reclamation",
            cost(&[generic(4), g()]),
            CardType::Instant,
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        )
    }
}

/// Sarevok's Tome — take the initiative on entry; {T}: {C} ({C}{C} with the
/// initiative); {3}, {T}, once you've completed a dungeon: exile until a
/// nonland card and cast it free.
pub fn sarevoks_tome() -> CardDefinition {
    let colorless = |n: u32| Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::Colorless(Value::Const(n as i32)),
    };
    CardDefinition {
        name: "Sarevok's Tome",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        triggered_abilities: vec![etb(Effect::TakeInitiative { who: PlayerRef::You })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::If {
                    cond: Predicate::HasInitiative { who: PlayerRef::You },
                    then: Box::new(colorless(2)),
                    else_: Box::new(colorless(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                condition: Some(Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::ONE)),
                effect: Effect::Seq(vec![
                    Effect::ExileTopUntilNonland { who: PlayerRef::You },
                    cast_free(Selector::ExiledThisResolution { filter: R::Nonland }),
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Stolen Strategy — each upkeep, exile the top card of each opponent's
/// library; you may cast those spells this turn with any mana.
pub fn stolen_strategy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::EachOpponent,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: true,
            max_mana_value: None,
            pay_own_cost: false,
            uncast_penalty: None,
        },
                Effect::RestrictMayPlayToCasting { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]))],
        ..spell("Stolen Strategy", cost(&[generic(4), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Sweet-Gum Recluse — flash, cascade, reach; on entry, three +1/+1
/// counters on each of any number of creatures that entered this turn.
pub fn sweet_gum_recluse() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Cascade, Keyword::Reach],
        triggered_abilities: vec![
            cascade(6),
            etb(Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::Creature.and(R::EnteredThisTurn),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                }),
            }),
        ],
        ..creature(
            "Sweet-Gum Recluse",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Spider],
            0,
            3,
        )
    }
}

/// Venture Forth — the first land among the top of your library onto the
/// battlefield, the rest to the bottom; then it suspends itself (three time
/// counters). Suspend 3—{1}{G}.
pub fn venture_forth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1), g()]))],
        ..spell(
            "Venture Forth",
            cost(&[generic(3), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::RevealUntilLandsToBattlefield { count: Value::ONE, tapped: false },
                Effect::ExileSelfSuspended,
            ]),
        )
    }
}

/// Wild-Magic Sorcerer — the first spell you cast from exile each turn has
/// cascade.
pub fn wild_magic_sorcerer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::FirstSpellCastFromExileThisTurn),
            effect: Effect::Cascade {
                max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..creature(
            "Wild-Magic Sorcerer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Orc, CreatureType::Shaman, CreatureType::Sorcerer],
            4,
            3,
        )
    }
}

/// Tlincalli Hunter // Retrieve Prey — trample; once each turn, {0} for a
/// creature spell you cast from exile. Adventure: exile a creature card from
/// your graveyard; you may cast it until the end of your next turn.
pub fn tlincalli_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Once each turn, you may pay {0} rather than pay the mana cost for a creature spell you cast from exile.",
            effect: StaticEffect::FreeExileCastOncePerTurnMatching(R::Creature),
        }],
        adventure: Some(Box::new(Adventure {
            name: "Retrieve Prey",
            cost: cost(&[generic(1), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.from_your_graveyard()),
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
        })),
        ..creature(
            "Tlincalli Hunter",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Scorpion, CreatureType::Scout],
            7,
            7,
        )
    }
}
