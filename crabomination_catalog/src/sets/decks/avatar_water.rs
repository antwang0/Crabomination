//! Avatar: The Last Airbender — Waterbend (CR 701.67) batch. Tests in
//! `tests/avatar_water.rs`.
//!
//! Waterbend {N} is an additional generic cost where each {1} may be paid by
//! tapping an untapped artifact or creature you control (Convoke restricted to
//! the sub-cost, over artifacts as well as creatures). It rides
//! `CardDefinition.waterbend` (additional cast cost,
//! `GameAction::CastSpellWaterbend`) and `ActivatedAbility.waterbend` (ability
//! cost, `GameAction::ActivateAbilityWaterbend`). Completes the bending family
//! (earthbend / airbend / blight already ship).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, Predicate, SelectionRequirement,
    Selector, SpellSubtype, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    WardCost, Waterbend,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, u, w};

/// Helper: a mandatory "waterbend {N}" additional cost.
fn wb(n: i32) -> Option<Waterbend> {
    Some(Waterbend {
        amount: Value::Const(n),
        optional: false,
    })
}
/// Helper: an optional "you may waterbend {N}" additional cost.
fn wb_opt(n: i32) -> Option<Waterbend> {
    Some(Waterbend {
        amount: Value::Const(n),
        optional: true,
    })
}

// ── Additional-cast-cost waterbend ──────────────────────────────────────────

/// Benevolent River Spirit — {U}{U} 4/5 Spirit. Waterbend {5}. Flying, ward {2};
/// ETB scry 2.
pub fn benevolent_river_spirit() -> CardDefinition {
    CardDefinition {
        name: "Benevolent River Spirit",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        waterbend: wb(5),
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Water Whip — {U}{U} Sorcery (Lesson). Waterbend {5}. Return up to two target
/// creatures to their owners' hands; draw two.
pub fn water_whip() -> CardDefinition {
    CardDefinition {
        name: "Water Whip",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        waterbend: wb(5),
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                filter: SelectionRequirement::Creature,
                max_targets: 2,
                min_targets: 0,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            draw(2),
        ]),
        ..Default::default()
    }
}

/// Waterbending Lesson — {3}{U} Sorcery (Lesson). Draw three cards. Then discard
/// a card unless you waterbend {2}.
pub fn waterbending_lesson() -> CardDefinition {
    CardDefinition {
        name: "Waterbending Lesson",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        waterbend: wb_opt(2),
        effect: Effect::Seq(vec![
            draw(3),
            Effect::If {
                cond: Predicate::SpellWasWaterbend,
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Spirit Water Revival — {1}{U}{U} Sorcery. You may waterbend {6}. Draw two
/// cards; if the additional cost was paid, instead shuffle your graveyard into
/// your library, draw seven, and you have no maximum hand size. Exile it.
pub fn spirit_water_revival() -> CardDefinition {
    CardDefinition {
        name: "Spirit Water Revival",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Sorcery],
        waterbend: wb_opt(6),
        exile_on_resolve: true,
        effect: Effect::If {
            cond: Predicate::SpellWasWaterbend,
            then: Box::new(Effect::Seq(vec![
                Effect::ShuffleGraveyardIntoLibrary {
                    who: PlayerRef::You,
                },
                draw(7),
                Effect::SetNoMaxHandSize { who: Selector::You },
            ])),
            else_: Box::new(draw(2)),
        },
        ..Default::default()
    }
}

/// Waterbender's Restoration — {U}{U} Instant (Lesson). Waterbend {X}. Exile X
/// target creatures you control; return them at the beginning of the next end
/// step.
pub fn waterbenders_restoration() -> CardDefinition {
    CardDefinition {
        name: "Waterbender's Restoration",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        waterbend: Some(Waterbend {
            amount: Value::XFromCost,
            optional: false,
        }),
        effect: Effect::ApplyToTargets {
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            max_targets: u8::MAX,
            min_targets: 0,
            effect: Box::new(Effect::ExileReturnNextEndStep {
                what: Selector::Target(0),
            }),
        },
        ..Default::default()
    }
}

/// Ruinous Waterbending — {1}{B}{B} Sorcery (Lesson). You may waterbend {4}.
/// All creatures get -2/-2 until end of turn. (The paid-rider death-lifegain is
/// dropped — no delayed dies-trigger primitive for one-shot spells yet.)
pub fn ruinous_waterbending() -> CardDefinition {
    CardDefinition {
        name: "Ruinous Waterbending",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        waterbend: wb_opt(4),
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Activated-ability waterbend ─────────────────────────────────────────────

/// Flexible Waterbender — {3}{U} 2/5 Human Warrior Ally. Vigilance. Waterbend
/// {3}: base power and toughness 5/2 until end of turn.
pub fn flexible_waterbender() -> CardDefinition {
    CardDefinition {
        name: "Flexible Waterbender",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Warrior,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(3)]),
            waterbend: true,
            effect: Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Giant Koi — {4}{U}{U} 5/7 Fish. Waterbend {3}: can't be blocked this turn.
/// Islandcycling {2}.
pub fn giant_koi() -> CardDefinition {
    CardDefinition {
        name: "Giant Koi",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Landcycling(
            ManaCost::new(vec![ManaSymbol::Generic(2)]),
            crate::card::LandType::Island,
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(3)]),
            waterbend: true,
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Geyser Leaper — {4}{U} 4/3 Human Warrior Ally. Flying. Waterbend {4}: draw a
/// card, then discard a card.
pub fn geyser_leaper() -> CardDefinition {
    CardDefinition {
        name: "Geyser Leaper",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Warrior,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(4)]),
            waterbend: true,
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ruthless Waterbender — {1}{B} 1/3 Human Soldier Ally. Waterbend {2}: +1/+1
/// until end of turn. Activate only during your turn.
pub fn ruthless_waterbender() -> CardDefinition {
    CardDefinition {
        name: "Ruthless Waterbender",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(2)]),
            waterbend: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Foggy Swamp Vinebender — {3}{G} 4/3 Human Plant Ally. Can't be blocked by
/// creatures with power 2 or less. Waterbend {5}: put a +1/+1 counter on it.
/// Activate only during your turn.
pub fn foggy_swamp_vinebender() -> CardDefinition {
    CardDefinition {
        name: "Foggy Swamp Vinebender",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Plant, CreatureType::Ally],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(5)]),
            waterbend: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Katara, Bending Prodigy — {2}{U} 2/3 Legendary Human Warrior Ally. At the
/// beginning of your end step, if Katara is tapped, put a +1/+1 counter on her.
/// Waterbend {6}: draw a card.
pub fn katara_bending_prodigy() -> CardDefinition {
    CardDefinition {
        name: "Katara, Bending Prodigy",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Warrior,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::Tapped,
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(6)]),
            waterbend: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// North Pole Patrol — {2}{U} 2/3 Human Soldier Ally. {T}: Untap another target
/// permanent you control. Waterbend {3}, {T}: Tap target creature an opponent
/// controls.
pub fn north_pole_patrol() -> CardDefinition {
    CardDefinition {
        name: "North Pole Patrol",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap {
                    what: target_filtered(SelectionRequirement::ControlledByYou),
                    up_to: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::new(vec![ManaSymbol::Generic(3)]),
                waterbend: true,
                effect: Effect::Tap {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Yue, the Moon Spirit — {3}{U} 3/3 Legendary Spirit Ally. Flying, vigilance.
/// Waterbend {5}, {T}: You may cast a noncreature spell from your hand without
/// paying its mana cost.
pub fn yue_the_moon_spirit() -> CardDefinition {
    CardDefinition {
        name: "Yue, the Moon Spirit",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(5)]),
            waterbend: true,
            effect: Effect::CastFromHandWithoutPaying {
                filter: Some(SelectionRequirement::Not(Box::new(
                    SelectionRequirement::Creature,
                ))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Katara, Water Tribe's Hope — {2}{W}{U}{U} 3/3 Legendary Human Warrior Ally.
/// Vigilance. ETB: create a 1/1 white Ally token. Waterbend {X}: creatures you
/// control have base power and toughness X/X until end of turn (X≠0). Activate
/// only during your turn.
pub fn katara_water_tribes_hope() -> CardDefinition {
    CardDefinition {
        name: "Katara, Water Tribe's Hope",
        cost: cost(&[generic(2), w(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Warrior,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::CreateToken {
            definition: TokenDefinition {
                name: "Ally".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Ally],
                    ..Default::default()
                },
                ..Default::default()
            },
            count: Value::ONE,
            who: PlayerRef::You,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::X]),
            waterbend: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::SetBasePT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── More waterbend cards ────────────────────────────────────────────────────

/// Water Tribe Rallier — {1}{W} 2/2 Human Soldier Ally. Waterbend {5}: look at
/// the top four cards; you may reveal a creature card with power 3 or less and
/// put it into your hand. (The rest go to the bottom in random order — modeled
/// as the default "rest stay" disposition.)
pub fn water_tribe_rallier() -> CardDefinition {
    CardDefinition {
        name: "Water Tribe Rallier",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(5)]),
            waterbend: true,
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: false,
                pick_filter: Some(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
                ),
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: true,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aang's Iceberg — {2}{W} Enchantment. Flash. ETB: exile up to one other
/// target nonland permanent until this leaves. Waterbend {3}: sacrifice this; if
/// you do, scry 2.
pub fn aangs_iceberg() -> CardDefinition {
    CardDefinition {
        name: "Aang's Iceberg",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(3)]),
            waterbend: true,
            sac_cost: true,
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Waterbender Ascension — {1}{U} Enchantment. Whenever a creature you control
/// deals combat damage to a player, put a quest counter on it; then if it has
/// four or more, draw a card. Waterbend {4}: target creature can't be blocked
/// this turn.
pub fn waterbender_ascension() -> CardDefinition {
    CardDefinition {
        name: "Waterbender Ascension",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Quest,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Quest,
                        },
                        Value::Const(4),
                    ),
                    then: Box::new(draw(1)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(4)]),
            waterbend: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Unagi of Kyoshi Island — {3}{U}{U} 5/5 Legendary Serpent. Flash. Ward {4}
/// (the printed Ward—Waterbend is approximated as Ward {4} mana). Whenever an
/// opponent draws their second card each turn, you draw two cards.
pub fn the_unagi_of_kyoshi_island() -> CardDefinition {
    CardDefinition {
        name: "The Unagi of Kyoshi Island",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Serpent],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flash, Keyword::Ward(WardCost::generic(4))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl).with_filter(
                Predicate::ValueEquals(
                    Value::CardsDrawnThisTurn(PlayerRef::ActivePlayer),
                    Value::Const(2),
                ),
            ),
            effect: draw(2),
        }],
        ..Default::default()
    }
}

/// Watery Grasp — {U} Aura. Enchant creature. Enchanted creature doesn't untap
/// during its controller's untap step (modeled by tapping it each upkeep, as
/// Narcolepsy does). Waterbend {5}: enchanted creature's owner shuffles it into
/// their library.
pub fn watery_grasp() -> CardDefinition {
    CardDefinition {
        name: "Watery Grasp",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Tap {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(5)]),
            waterbend: true,
            effect: Effect::Move {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::AttachedTo(Box::new(
                        Selector::This,
                    )))),
                    pos: LibraryPosition::Shuffled,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crashing Wave — {U}{U} Sorcery. Waterbend {X}. Tap up to X target creatures,
/// then distribute three stun counters among any number of tapped creatures your
/// opponents control.
pub fn crashing_wave() -> CardDefinition {
    CardDefinition {
        name: "Crashing Wave",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Sorcery],
        waterbend: Some(Waterbend {
            amount: Value::XFromCost,
            optional: false,
        }),
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                filter: SelectionRequirement::Creature,
                max_targets: u8::MAX,
                min_targets: 0,
                effect: Box::new(Effect::Tap {
                    what: Selector::Target(0),
                }),
            },
            Effect::DistributeCounters {
                total: Value::Const(3),
                counter: CounterType::Stun,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent)
                    .and(SelectionRequirement::Tapped),
                max_targets: 3,
            },
        ]),
        ..Default::default()
    }
}
