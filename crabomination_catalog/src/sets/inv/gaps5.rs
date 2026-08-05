//! Invasion (INV) gap-closing wave 5: the pile-splitting rares and the last
//! utility shell. Tests in `classic_sets/inv_gaps5`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, pump_target, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn upkeep(scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), scope)
}

/// The pile the chooser took / the pile left over.
fn chosen_pile() -> Selector {
    Selector::SeparatedPile { chosen: true }
}
fn other_pile() -> Selector {
    Selector::SeparatedPile { chosen: false }
}

// ── The pile-splitting cycle ────────────────────────────────────────────────

/// Do or Die — you split the target player's creatures; they pick the pile
/// that dies.
pub fn do_or_die() -> CardDefinition {
    sorcery(
        "Do or Die",
        cost(&[generic(1), b()]),
        Effect::SeparateIntoPiles {
            what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
            splitter: PlayerRef::You,
            chooser: PlayerRef::Target(0),
            chosen: Box::new(Effect::DestroyNoRegen { what: chosen_pile() }),
            other: Box::new(Effect::Noop),
        },
    )
}

/// Death or Glory — you split your graveyard's creatures; an opponent exiles
/// one pile and the other returns.
pub fn death_or_glory() -> CardDefinition {
    sorcery(
        "Death or Glory",
        cost(&[generic(4), w()]),
        Effect::SeparateIntoPiles {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature,
            },
            splitter: PlayerRef::You,
            chooser: PlayerRef::EachOpponent,
            chosen: Box::new(Effect::Move { what: chosen_pile(), to: ZoneDest::Exile }),
            other: Box::new(Effect::Move {
                what: other_pile(),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
    )
}

/// Bend or Break — each player splits their own nontoken lands; an opponent
/// picks the pile that is destroyed and the rest taps.
pub fn bend_or_break() -> CardDefinition {
    sorcery(
        "Bend or Break",
        cost(&[generic(3), r()]),
        Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::SeparateIntoPiles {
                what: Selector::ControlledBy {
                    who: PlayerRef::Triggerer,
                    filter: R::Land.and(R::NotToken),
                },
                splitter: PlayerRef::Triggerer,
                chooser: PlayerRef::OpponentOf(Box::new(PlayerRef::Triggerer)),
                chosen: Box::new(Effect::Destroy { what: chosen_pile() }),
                other: Box::new(Effect::Tap { what: other_pile() }),
            }),
        },
    )
}

/// Fight or Flight — you split the attacking player's creatures; only the pile
/// they pick may attack.
pub fn fight_or_flight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::OpponentControl,
            ),
            effect: Effect::SeparateIntoPiles {
                what: Selector::ControlledBy { who: PlayerRef::ActivePlayer, filter: R::Creature },
                splitter: PlayerRef::You,
                chooser: PlayerRef::ActivePlayer,
                chosen: Box::new(Effect::Noop),
                other: Box::new(Effect::GrantKeyword {
                    what: other_pile(),
                    keyword: Keyword::CantAttack,
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..enchantment("Fight or Flight", cost(&[generic(3), w()]))
    }
}

/// Stand or Fall — you split each defender's creatures; only the pile they
/// pick may block.
pub fn stand_or_fall() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::SeparateIntoPiles {
                    what: Selector::ControlledBy {
                        who: PlayerRef::Triggerer,
                        filter: R::Creature,
                    },
                    splitter: PlayerRef::You,
                    chooser: PlayerRef::Triggerer,
                    chosen: Box::new(Effect::Noop),
                    other: Box::new(Effect::GrantKeyword {
                        what: other_pile(),
                        keyword: Keyword::CantBlock,
                        duration: Duration::EndOfTurn,
                    }),
                }),
            },
        }],
        ..enchantment("Stand or Fall", cost(&[generic(3), r()]))
    }
}

/// Barrin's Spite — two creatures controlled by the same player; their
/// controller sacrifices one and the other bounces.
pub fn barrins_spite() -> CardDefinition {
    sorcery(
        "Barrin's Spite",
        cost(&[generic(2), u(), b()]),
        Effect::ChooseOneAmong {
            what: Selector::Both(
                Box::new(Selector::TargetFiltered { slot: 0, filter: R::Creature }),
                Box::new(Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::SameControllerAsTargetSlot(0)),
                }),
            ),
            chooser: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            chosen: Box::new(Effect::SacrificeSelected { what: chosen_pile() }),
            other: Box::new(Effect::Move {
                what: other_pile(),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        },
    )
}

// ── Utility spells ──────────────────────────────────────────────────────────

/// Coalition Victory — win with all five basic land types and a creature of
/// each colour.
pub fn coalition_victory() -> CardDefinition {
    sorcery(
        "Coalition Victory",
        cost(&[generic(3), w(), u(), b(), r(), g()]),
        Effect::If {
            cond: Predicate::All(vec![
                Predicate::ControlsLandOfEachBasicType(PlayerRef::You),
                Predicate::ControlsCreatureOfEachColor(PlayerRef::You),
            ]),
            then: Box::new(Effect::WinGame { who: PlayerRef::You }),
            else_: Box::new(Effect::Noop),
        },
    )
}

fn saproling() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    }
}

/// Artifact Mutation — destroy an artifact, then make that many Saprolings.
pub fn artifact_mutation() -> CardDefinition {
    instant(
        "Artifact Mutation",
        cost(&[r(), g()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen { what: target_filtered(R::Artifact) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ManaValueOf(Box::new(Selector::Target(0))),
                definition: saproling(),
            },
        ]),
    )
}

/// Bind — counter an activated ability and draw.
pub fn bind() -> CardDefinition {
    instant(
        "Bind",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::CounterAbility { what: target_filtered(R::HasAbilityOnStack) },
            draw(1),
        ]),
    )
}

/// Chaotic Strike — after blockers, a coin flip may pump; it always cantrips.
pub fn chaotic_strike() -> CardDefinition {
    CardDefinition {
        cast_only_after_blockers: true,
        ..instant(
            "Chaotic Strike",
            cost(&[generic(1), r()]),
            Effect::Seq(vec![
                Effect::FlipCoin {
                    count: Value::Const(1),
                    on_heads: Box::new(pump_target(1, 1)),
                    on_tails: Box::new(Effect::Noop),
                },
                draw(1),
            ]),
        )
    }
}

/// Crystal Spray — swap a colour word on a permanent and cantrip.
pub fn crystal_spray() -> CardDefinition {
    instant(
        "Crystal Spray",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![
            Effect::ReplaceColorWord {
                what: target_filtered(R::Permanent),
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Samite Ministration — a prevention shield against one source, refunding
/// life for damage prevented from a black or red one.
pub fn samite_ministration() -> CardDefinition {
    instant(
        "Samite Ministration",
        cost(&[generic(1), w()]),
        Effect::PreventAllDamageFromChosenSourceThisTurn {
            filter: R::Any,
            gain_life_from_colors: vec![Color::Black, Color::Red],
        },
    )
}

/// Protective Sphere — a repeatable prevention shield against one source that
/// shares a colour with the mana spent on the activation.
pub fn protective_sphere() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            life_cost: 1,
            effect: Effect::PreventAllDamageFromChosenSourceThisTurn {
                filter: R::SharesColorWithManaSpent,
                gain_life_from_colors: vec![],
            },
            ..Default::default()
        }],
        ..enchantment("Protective Sphere", cost(&[generic(2), w()]))
    }
}

/// Orim's Touch — a 2-point prevention shield, 4 when kicked.
pub fn orims_touch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1)]))],
        ..instant(
            "Orim's Touch",
            cost(&[w()]),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::PreventNextDamage {
                    target: target_any(),
                    amount: Value::Const(4),
                }),
                else_: Box::new(Effect::PreventNextDamage {
                    target: target_any(),
                    amount: Value::Const(2),
                }),
            },
        )
    }
}

/// Overabundance — tapping a land doubles its mana and pings its controller.
pub fn overabundance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
            ),
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::Triggerer,
                    pool: ManaPayload::AnyTypeTriggerSourceProduces,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..enchantment("Overabundance", cost(&[generic(1), r(), g()]))
    }
}

/// Pure Reflection — every creature spell wipes the Reflections and mints a
/// new one sized to that spell.
pub fn pure_reflection() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::Seq(vec![
                Effect::DestroyNoRegen {
                    what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Reflection)),
                },
                Effect::CreateToken {
                    who: PlayerRef::Triggerer,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Reflection".into(),
                        power: 0,
                        toughness: 0,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Reflection],
                            ..Default::default()
                        },
                        dynamic_pt: Some((
                            Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                            Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        )),
                        ..Default::default()
                    },
                },
            ]),
        }],
        ..enchantment("Pure Reflection", cost(&[generic(2), w()]))
    }
}

/// Spreading Plague — any creature entering wipes the rest of its colour.
pub fn spreading_plague() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::DestroyNoRegen {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::SharingColorWith(Box::new(Selector::TriggerSource))),
                    filter: R::Creature,
                },
            },
        }],
        ..enchantment("Spreading Plague", cost(&[generic(4), b()]))
    }
}

/// Temporal Distortion — tapping adds an hourglass counter, counters stop the
/// untap, and each upkeep clears the active player's.
pub fn temporal_distortion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.or(R::Land),
                    },
                ),
                effect: Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::Hourglass,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: upkeep(EventScope::AnyPlayer),
                effect: Effect::RemoveCounter {
                    what: Selector::ControlledBy {
                        who: PlayerRef::ActivePlayer,
                        filter: R::WithCounter(CounterType::Hourglass),
                    },
                    kind: CounterType::Hourglass,
                    amount: Value::Const(99),
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Each permanent with an hourglass counter doesn't untap.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::WithCounter(CounterType::Hourglass)),
                keyword: Keyword::DoesntUntapWhileCounter(CounterType::Hourglass),
            },
        }],
        ..enchantment("Temporal Distortion", cost(&[generic(3), u(), u()]))
    }
}

/// Vile Consumption — every creature must be bought off for 1 life each
/// upkeep.
pub fn vile_consumption() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All creatures have \"At the beginning of your upkeep, sacrifice this \
                          creature unless you pay 1 life.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature,
                ability: Box::new(TriggeredAbility {
                    event: upkeep(EventScope::YourControl),
                    effect: Effect::UnlessPlayerPays {
                        who: PlayerRef::You,
                        cost: WardCost::Life(1),
                        then: Box::new(Effect::SacrificeSource),
                        if_paid: None,
                    },
                }),
            },
        }],
        ..enchantment("Vile Consumption", cost(&[generic(1), u(), b()]))
    }
}

/// Yawgmoth's Agenda — one spell a turn, play from your graveyard, and your
/// cards are exiled instead of binned.
pub fn yawgmoths_agenda() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You can't cast more than one spell each turn.",
                effect: StaticEffect::OneSpellPerTurn,
            },
            StaticAbility {
                description: "You may play lands from your graveyard.",
                effect: StaticEffect::MayPlayLandsFromGraveyard,
            },
            StaticAbility {
                description: "You may cast permanent spells from your graveyard.",
                effect: StaticEffect::MayCastPermanentsFromGraveyard,
            },
            StaticAbility {
                description: "If a card would be put into your graveyard from anywhere, exile it \
                              instead.",
                effect: StaticEffect::ExileCardsBoundForGraveyard {
                    opponents_only: false,
                    own_only: true,
                    colors: None,
                    card_types: None,
                    void_counter: false,
                stamp_source: false,
                },
            },
        ],
        ..enchantment("Yawgmoth's Agenda", cost(&[generic(3), b(), b()]))
    }
}

/// Metathran Aerostat — deploy a creature of the paid mana value, then bounce
/// itself.
pub fn metathran_aerostat() -> CardDefinition {
    CardDefinition {
        name: "Metathran Aerostat",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Metathran], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), u()]),
            effect: Effect::Seq(vec![
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::ManaValueExactlyXFromCost),
                    count: Value::Const(1),
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::LastMoved),
                    then: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Atalya, Samite Master — {X} of prevention or {X} of life, X paid in white.
pub fn atalya_samite_master() -> CardDefinition {
    CardDefinition {
        name: "Atalya, Samite Master",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            x_mana_color: Some(Color::White),
            effect: Effect::ChooseMode(vec![
                Effect::PreventNextDamage {
                    target: target_filtered(R::Creature),
                    amount: Value::XFromCost,
                },
                Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Global Ruin — every player keeps one land of each basic type and sacrifices
/// the rest.
pub fn global_ruin() -> CardDefinition {
    sorcery(
        "Global Ruin",
        cost(&[generic(4), w()]),
        Effect::EachPlayerKeepsOneOfEachBasicTypeSacrificesRest,
    )
}

/// Desperate Research — name a card, keep every copy from the top seven and
/// exile the rest.
pub fn desperate_research() -> CardDefinition {
    sorcery(
        "Desperate Research",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            Effect::NameCard { what: Selector::This, restrict_to: None },
            Effect::RevealTopTakeNamedExileRest { count: Value::Const(7) },
        ]),
    )
}

/// Loafing Giant — attacking or blocking mills; a milled land blanks its
/// combat damage.
pub fn loafing_giant() -> CardDefinition {
    CardDefinition {
        name: "Loafing Giant",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 4,
        toughness: 6,
        triggered_abilities: vec![
            mill_and_maybe_blank(EventKind::Attacks),
            mill_and_maybe_blank(EventKind::Blocks),
        ],
        ..Default::default()
    }
}

fn mill_and_maybe_blank(kind: EventKind) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::MillThenBranchByType {
            land: Box::new(Effect::PreventAllCombatDamageByMatchingThisTurn {
                filter: R::IsSource,
            }),
            creature: Box::new(Effect::Noop),
            noncreature: Box::new(Effect::Noop),
        },
    }
}

/// Aether Rift — a random discard each upkeep; a discarded creature comes back
/// unless someone pays 5 life.
pub fn aether_rift() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: true },
                Effect::UnlessPlayerPays {
                    who: PlayerRef::EachPlayer,
                    cost: WardCost::Life(5),
                    then: Box::new(Effect::Move {
                        what: Selector::DiscardedThisResolution { filter: R::Creature },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    if_paid: None,
                },
            ]),
        }],
        ..enchantment("Aether Rift", cost(&[generic(1), r(), g()]))
    }
}

/// Tsabo's Decree — name a creature type, then strip it from a player's hand
/// and board.
pub fn tsabos_decree() -> CardDefinition {
    instant(
        "Tsabo's Decree",
        cost(&[generic(5), b()]),
        Effect::Seq(vec![
            Effect::NameCreatureType { what: Selector::This },
            Effect::RevealHandDiscardAllMatching {
                who: PlayerRef::Target(0),
                filter: R::Creature.and(R::IsSourceChosenCreatureType),
            },
            Effect::DestroyNoRegen {
                what: Selector::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: R::Creature.and(R::IsSourceChosenCreatureType),
                },
            },
        ]),
    )
}

/// Cauldron Dance — combat-only double reanimation: one from your graveyard
/// (bounced at end of turn) and one from hand (sacrificed).
pub fn cauldron_dance() -> CardDefinition {
    CardDefinition {
        cast_only_during_combat: true,
        ..instant(
            "Cauldron Dance",
            cost(&[generic(4), b(), r()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword {
                    what: Selector::LastMoved,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                },
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    count: Value::Const(1),
                    tapped: false,
                    haste: true,
                    sacrifice_eot: true,
                    return_eot: false,
                    then: None,
                },
            ]),
        )
    }
}

/// Spinal Embrace — combat-only theft; the creature is sacrificed at end of
/// turn for its toughness in life.
pub fn spinal_embrace() -> CardDefinition {
    CardDefinition {
        cast_only_during_combat: true,
        ..instant(
            "Spinal Embrace",
            cost(&[generic(3), u(), u(), b()]),
            Effect::Seq(vec![
                Effect::Untap {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    up_to: None,
                },
                Effect::GainControl {
                    what: Selector::Target(0),
                    to: None,
                    duration: Duration::Permanent,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::If {
                        cond: Predicate::SelectorExists(Selector::Target(0)),
                        then: Box::new(Effect::Seq(vec![
                            Effect::GainLife {
                                who: Selector::You,
                                amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
                            },
                            Effect::SacrificeSelected { what: Selector::Target(0) },
                        ])),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
        )
    }
}

/// Pledge of Loyalty — the enchanted creature continuously dodges the colours
/// of permanents its controller controls, keeping this Aura on.
pub fn pledge_of_loyalty() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::ProtectionFromMatching(Box::new(
                R::SharesColorWithPermanentYouControl,
            ))],
            protection_keeps_self: true,
            ..Default::default()
        }),
        ..enchantment("Pledge of Loyalty", cost(&[generic(1), w()]))
    }
}

// ── The colour-matters shell ────────────────────────────────────────────────

/// Well-Laid Plans — creatures can't hurt creatures of a shared colour.
pub fn well_laid_plans() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to a creature by another \
                          creature if they share a color.",
            effect: StaticEffect::PreventDamageBetweenSharedColorCreatures,
        }],
        ..enchantment("Well-Laid Plans", cost(&[generic(2), u()]))
    }
}

/// Harsh Judgment — instants and sorceries of the chosen colour burn their own
/// caster instead of you.
pub fn harsh_judgment() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "If an instant or sorcery spell of the chosen color would deal damage \
                          to you, it deals that damage to its controller instead.",
            effect: StaticEffect::RedirectChosenColorSpellDamageToController,
        }],
        ..enchantment("Harsh Judgment", cost(&[generic(2), w(), w()]))
    }
}

/// Pulse of Llanowar — your basic lands produce the colour you named.
pub fn pulse_of_llanowar() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "If a basic land you control is tapped for mana, it produces mana of \
                          the chosen color instead of any other type.",
            effect: StaticEffect::YourBasicLandsProduceChosenColorInstead,
        }],
        ..enchantment("Pulse of Llanowar", cost(&[generic(3), g()]))
    }
}

/// Mana Maze — nothing may share a colour with the turn's last cast.
pub fn mana_maze() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can't cast spells that share a color with the spell most \
                          recently cast this turn.",
            effect: StaticEffect::CantCastSharingColorWithLastCastSpell,
        }],
        ..enchantment("Mana Maze", cost(&[generic(1), u()]))
    }
}

/// Traveler's Cloak — landwalk of a type chosen as it enters, plus a cantrip.
pub fn travelers_cloak() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        as_enters_effect: Some(Effect::ChooseBasicLandTypeForSource),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantChosenTypeLandwalk { what: Selector::attached_to(Selector::This) },
            draw(1),
        ]))],
        ..enchantment("Traveler's Cloak", cost(&[generic(2), u()]))
    }
}

/// Teferi's Response — counter an opponent's land-targeting spell or ability
/// (killing a permanent source) and draw two.
pub fn teferis_response() -> CardDefinition {
    instant(
        "Teferi's Response",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::CounterAbilityAndDestroySource {
                what: target_filtered(R::TargetsALandYouControl),
            },
            draw(2),
        ]),
    )
}

/// Mages' Contest — bid life for the right to counter a spell.
pub fn mages_contest() -> CardDefinition {
    instant(
        "Mages' Contest",
        cost(&[generic(1), r(), r()]),
        Effect::BidLifeToCounterTargetSpell { what: target_filtered(R::IsSpellOnStack) },
    )
}

/// Pain // Suffering — a split card: a discard, or a Stone Rain.
pub fn pain_suffering() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(3), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Destroy { what: target_filtered(R::Land) },
            },
            fuse: false,
            aftermath: false,
        })),
        ..sorcery(
            "Pain // Suffering",
            cost(&[b()]),
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            },
        )
    }
}

/// Essence Leak — a red or green enchanted permanent has to be bought back
/// each upkeep.
pub fn essence_leak() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::attached_to(Selector::This),
                filter: R::HasColor(Color::Red).or(R::HasColor(Color::Green)),
            }),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: WardCost::ManaCostOfAttached,
                then: Box::new(Effect::SacrificeSelected {
                    what: Selector::attached_to(Selector::This),
                }),
                if_paid: None,
            },
        }],
        ..enchantment("Essence Leak", cost(&[u()]))
    }
}

/// Psychic Battle — every targeting decision is contested by a top-card
/// reveal; the biggest mana value may repoint it.
pub fn psychic_battle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ChoseTargets, EventScope::AnyPlayer),
            effect: Effect::RevealTopGreatestMayChangeTargets,
        }],
        ..enchantment("Psychic Battle", cost(&[generic(3), u(), u()]))
    }
}
