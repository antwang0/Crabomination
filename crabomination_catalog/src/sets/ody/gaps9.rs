//! Odyssey (ODY) gap-closing wave 9: the discard/graveyard shell and the
//! Threshold rares. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    Subtypes, TriggeredAbility, WardCost,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// An Aura that enchants a creature.
fn creature_aura(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..enchantment(name, c)
    }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::TurnStep::Upkeep),
            EventScope::YourControl,
        ),
        effect,
    }
}

// ── Discard payoffs ─────────────────────────────────────────────────────────

/// Mindslicer — {2}{B}{B} 4/3 Horror whose death empties every hand.
pub fn mindslicer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(100), // capped at hand size — "their hand"
                random: false,
            },
        }],
        ..creature("Mindslicer", cost(&[generic(2), b(), b()]), vec![CreatureType::Horror], 4, 3)
    }
}

/// Last Rites — {2}{B}. Discard any number, then strip that many nonland
/// cards from a hand.
pub fn last_rites() -> CardDefinition {
    sorcery(
        "Last Rites",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You },
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::CardsDiscardedThisEffect,
                filter: R::Nonland,
            },
        ]),
    )
}

/// Rites of Initiation — {R}. Discard any number of cards; your team gets
/// +1/+0 for each.
pub fn rites_of_initiation() -> CardDefinition {
    instant(
        "Rites of Initiation",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You },
            Effect::PumpPT {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                power: Value::CardsDiscardedThisEffect,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Rites of Refusal — {1}{U}. Discard any number; the countered spell's
/// controller pays {3} per discard or loses it.
pub fn rites_of_refusal() -> CardDefinition {
    instant(
        "Rites of Refusal",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You },
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: ManaCost::default(),
                exile: false,
                extra_generic: Some(Value::Times(
                    Box::new(Value::CardsDiscardedThisEffect),
                    Box::new(Value::Const(3)),
                )),
            },
        ]),
    )
}

/// Pulsating Illusion — {4}{U} 0/1 flier that pumps huge once a turn.
pub fn pulsating_illusion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Pulsating Illusion", cost(&[generic(4), u()]), vec![CreatureType::Illusion], 0, 1)
    }
}

// ── Graveyard costs ─────────────────────────────────────────────────────────

/// Rotting Giant — {1}{B} 3/3 that eats its own graveyard to stay in combat.
pub fn rotting_giant() -> CardDefinition {
    let feed = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::SacrificeSourceUnlessCost { cost: WardCost::ExileFromGraveyard(1) },
    };
    CardDefinition {
        triggered_abilities: vec![feed(EventKind::Attacks), feed(EventKind::Blocks)],
        ..creature(
            "Rotting Giant",
            cost(&[generic(1), b()]),
            vec![CreatureType::Zombie, CreatureType::Giant],
            3,
            3,
        )
    }
}

/// Cursed Monstrosity — {4}{B} 4/3 flier that has to be fed a land whenever
/// anything points at it.
pub fn cursed_monstrosity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSourceUnlessCost {
                cost: WardCost::DiscardMatching(Box::new(R::Land), 1),
            },
        }],
        ..creature("Cursed Monstrosity", cost(&[generic(4), b()]), vec![CreatureType::Horror], 4, 3)
    }
}

/// Tombfire — {B}. Strip every flashback card out of a graveyard.
pub fn tombfire() -> CardDefinition {
    sorcery(
        "Tombfire",
        cost(&[b()]),
        Effect::ExilePlayerGraveyard {
            who: PlayerRef::Target(0),
            filter: Some(R::HasFlashback),
        },
    )
}

/// Haunting Echoes — {3}{B}{B}. Exile a graveyard, then every copy of what it
/// held out of that player's library.
pub fn haunting_echoes() -> CardDefinition {
    sorcery(
        "Haunting Echoes",
        cost(&[generic(3), b(), b()]),
        Effect::Seq(vec![
            Effect::ExilePlayerGraveyard {
                who: PlayerRef::Target(0),
                filter: Some(R::Not(Box::new(R::IsBasicLand))),
            },
            Effect::ExileLibraryCardsNamedLikeExiledThisResolution { who: PlayerRef::Target(0) },
        ]),
    )
}

/// Decaying Soil — {1}{B}{B}. Eats your graveyard each upkeep, and past
/// Threshold buys back the creatures that die.
pub fn decaying_soil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::ExileFromGraveyard {
            who: PlayerRef::You,
            count: Value::ONE,
            filter: R::Any,
        })],
        static_abilities: vec![StaticAbility {
            description: "Threshold — pay {1} to buy back your dying nontoken creatures.",
            effect: StaticEffect::WhileCondition {
                condition: threshold(),
                inner: Box::new(StaticEffect::GrantTriggeredAbility {
                    filter: R::IsSource,
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                            .with_filter(Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::NotToken,
                            }),
                        effect: Effect::UnlessPlayerPays {
                            who: PlayerRef::You,
                            cost: WardCost::Mana(cost(&[generic(1)])),
                            then: Box::new(Effect::Noop),
                            if_paid: Some(Box::new(Effect::Move {
                                what: Selector::TriggerSource,
                                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                            })),
                        },
                    }),
                }),
            },
        }],
        ..enchantment("Decaying Soil", cost(&[generic(1), b(), b()]))
    }
}

/// Gravestorm — {B}{B}{B}. Each upkeep an opponent burns a graveyard card or
/// you draw.
pub fn gravestorm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::UnlessPlayerPays {
            who: PlayerRef::Target(0),
            cost: WardCost::ExileFromGraveyard(1),
            then: Box::new(draw(1)),
            if_paid: None,
        })],
        ..enchantment("Gravestorm", cost(&[b(), b(), b()]))
    }
}

// ── Blue value engines ──────────────────────────────────────────────────────

/// Pedantic Learning — {U}{U}. Turn milled lands into cards for {1} each.
pub fn pedantic_learning() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardMilled, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
            ),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: WardCost::Mana(cost(&[generic(1)])),
                then: Box::new(Effect::Noop),
                if_paid: Some(Box::new(draw(1))),
            },
        }],
        ..enchantment("Pedantic Learning", cost(&[u(), u()]))
    }
}

/// Unifying Theory — {1}{U}. Every spell offers its caster a {2} cantrip.
pub fn unifying_theory() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::Noop),
                if_paid: Some(Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::ONE,
                })),
            },
        }],
        ..enchantment("Unifying Theory", cost(&[generic(1), u()]))
    }
}

/// Aether Burst — {1}{U}. Bounces one creature, plus one per copy already in
/// a graveyard.
pub fn aether_burst() -> CardDefinition {
    instant(
        "Aether Burst",
        cost(&[generic(1), u()]),
        Effect::CapTargetsAt {
            amount: Value::Sum(vec![Value::ONE, Value::CardsNamedLikeSourceInAllGraveyards]),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            }),
        },
    )
}

/// Obstinate Familiar — {R} 1/1 that lets you decline your draws.
pub fn obstinate_familiar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card, you may skip that draw instead.",
            effect: StaticEffect::ControllerMaySkipDraws,
        }],
        ..creature("Obstinate Familiar", cost(&[r()]), vec![CreatureType::Lizard], 1, 1)
    }
}

// ── Red tempo ───────────────────────────────────────────────────────────────

/// Seize the Day — {3}{R}. Untap an attacker and take another combat; comes
/// back once with flashback.
pub fn seize_the_day() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r()]))],
        ..sorcery(
            "Seize the Day",
            cost(&[generic(3), r()]),
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
            ]),
        )
    }
}

/// Dwarven Recruiter — {2}{R} 2/2. Stacks the library with Dwarves.
pub fn dwarven_recruiter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Dwarf),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            count: Value::Const(4),
        })],
        ..creature("Dwarven Recruiter", cost(&[generic(2), r()]), vec![CreatureType::Dwarf], 2, 2)
    }
}

// ── Threshold bodies ────────────────────────────────────────────────────────

/// Repentant Vampire — {3}{B}{B} 3/3 flier that grows off its kills and turns
/// white past Threshold.
pub fn repentant_vampire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Threshold — this creature is white.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::SetColorOfMatching {
                        applies_to: Selector::This,
                        color: Color::White,
                    }),
                },
            },
            StaticAbility {
                description: "Threshold — {T}: Destroy target black creature.",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::This,
                    ability: ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::Destroy {
                            what: target_filtered(R::Creature.and(R::HasColor(Color::Black))),
                        },
                        ..Default::default()
                    },
                    condition: Some(threshold()),
                },
            },
        ],
        ..creature(
            "Repentant Vampire",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire],
            3,
            3,
        )
    }
}

/// Wayward Angel — {4}{W}{W} 4/4 that falls to black past Threshold.
pub fn wayward_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "Threshold — +3/+3 and trample.",
                effect: StaticEffect::PumpSelfIf {
                    condition: threshold(),
                    power: 3,
                    toughness: 3,
                    keywords: vec![Keyword::Trample],
                },
            },
            StaticAbility {
                description: "Threshold — this creature is black.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::SetColorOfMatching {
                        applies_to: Selector::This,
                        color: Color::Black,
                    }),
                },
            },
            StaticAbility {
                description: "Threshold — at the beginning of your upkeep, sacrifice a creature.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::GrantTriggeredAbility {
                        filter: R::IsSource,
                        ability: Box::new(upkeep(Effect::Sacrifice {
                            who: Selector::You,
                            count: Value::ONE,
                            filter: R::Creature,
                        })),
                    }),
                },
            },
        ],
        ..creature(
            "Wayward Angel",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Stone-Tongue Basilisk — {4}{G}{G}{G} 4/5 that kills what it hits and lures
/// the whole board past Threshold.
pub fn stone_tongue_basilisk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Threshold — all creatures able to block this creature do so.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::AllMustBlock,
                condition: threshold(),
            },
        }],
        ..creature(
            "Stone-Tongue Basilisk",
            cost(&[generic(4), g(), g(), g()]),
            vec![CreatureType::Basilisk],
            4,
            5,
        )
    }
}

/// Seton's Desire — {2}{G} Aura. +2/+2, and past Threshold the enchanted
/// creature lures every blocker.
pub fn setons_desire() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::Any,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::AllMustBlock],
                condition: Some(threshold()),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..creature_aura("Seton's Desire", cost(&[generic(2), g()]))
    }
}

/// Verdant Succession — {4}{G}. Every green nontoken creature that dies
/// fetches its twin.
pub fn verdant_succession() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Green).and(R::NotToken),
                },
            ),
            effect: Effect::SearchSameNameToBattlefield {
                who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                what: Selector::TriggerSource,
            },
        }],
        ..enchantment("Verdant Succession", cost(&[generic(4), g()]))
    }
}

/// Balancing Act — {2}{W}{W}. Everyone trims down to the smallest board, then
/// the smallest hand.
pub fn balancing_act() -> CardDefinition {
    sorcery(
        "Balancing Act",
        cost(&[generic(2), w(), w()]),
        Effect::BalanceMatching { filters: vec![R::Permanent], hands: true },
    )
}
