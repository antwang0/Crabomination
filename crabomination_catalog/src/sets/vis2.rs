//! Visions (VIS), second wave. Tests in `classic_sets/vis`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EventKind, EventScope, EventSpec, Keyword,
    LandType, SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, WardCost,
};
use crate::game::TurnStep;
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{
    CounteredSpellZone, DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}


fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}


fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

// ── Creatures ───────────────────────────────────────────────────────────────


/// Viashivan Dragon — {2}{R}{R}{G}{G} 4/4 flier that pumps in either direction.
pub fn viashivan_dragon() -> CardDefinition {
    let pump = |color, p, t| ActivatedAbility {
        mana_cost: cost(&[color]),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(p),
            toughness: Value::Const(t),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![pump(r(), 1, 0), pump(g(), 0, 1)],
        ..creature(
            "Viashivan Dragon",
            cost(&[generic(2), r(), r(), g(), g()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}


/// Mundungu — {1}{U}{B} 1/1 that taxes every spell by {1} and a life.
pub fn mundungu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CounterUnless {
                what: Selector::Target(0),
                cost: WardCost::ManaAndLife(cost(&[generic(1)]), 1),
            },
            ..Default::default()
        }],
        ..creature(
            "Mundungu",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}


/// Bogardan Phoenix — {2}{R}{R}{R} 3/3 flier; the first death brings it back.
pub fn bogardan_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::If {
                cond: crate::effect::Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Death,
                    },
                    Value::Const(1),
                ),
                then: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Exile }),
                else_: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Death,
                        amount: Value::Const(1),
                    },
                ])),
            },
        }],
        ..creature("Bogardan Phoenix", cost(&[generic(2), r(), r(), r()]), vec![CreatureType::Phoenix], 3, 3)
    }
}


// ── Lands ───────────────────────────────────────────────────────────────────


// ── Artifacts ───────────────────────────────────────────────────────────────


/// Diamond Kaleidoscope — mints Prisms that cash in for any colour.
pub fn diamond_kaleidoscope() -> CardDefinition {
    artifact(
        "Diamond Kaleidoscope",
        cost(&[generic(4)]),
        vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Prism".into(),
                        power: 0,
                        toughness: 1,
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::IsToken.and(R::HasName("Prism".into())), 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
        ],
    )
}

// ── Enchantments ────────────────────────────────────────────────────────────


/// Squandered Resources — every land is a Dark Ritual on the way out.
pub fn squandered_resources() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyTypeSacrificedLandProduces,
            },
            ..Default::default()
        }],
        ..enchantment("Squandered Resources", cost(&[b(), g()]))
    }
}


// ── Spells ──────────────────────────────────────────────────────────────────


/// Desertion — {3}{U}{U}; countering an artifact or creature spell steals it.
pub fn desertion() -> CardDefinition {
    instant(
        "Desertion",
        cost(&[generic(3), u(), u()]),
        Effect::CounterSpellToZone {
            what: Selector::Target(0),
            zone: CounteredSpellZone::CountererBattlefieldIfMatching(Box::new(
                R::Creature.or(R::Artifact),
            )),
        },
    )
}


/// Tithe — {W}; one Plains, or two if the opponent is ahead on lands.
pub fn tithe() -> CardDefinition {
    instant(
        "Tithe",
        cost(&[w()]),
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Plains),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::If {
                cond: crate::effect::Predicate::ValueAtLeast(
                    Value::CountMatching {
                        sel: Box::new(Selector::ControlledBy {
                            who: PlayerRef::Target(0),
                            filter: R::Land,
                        }),
                        filter: R::Land,
                    },
                    Value::Sum(vec![
                        Value::CountMatching {
                            sel: Box::new(Selector::EachPermanent(R::Land.and(R::ControlledByYou))),
                            filter: R::Land,
                        },
                        Value::Const(1),
                    ]),
                ),
                then: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Plains),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Eye of Singularity — {3}{W} World enchantment; the board goes singleton.
pub fn eye_of_singularity() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn {
                    what: Selector::EachPermanent(
                        R::SharesNameWithAnotherPermanent.and(R::Not(Box::new(R::IsBasicLand))),
                    ),
                },
                Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::SharesNameWithAnotherPermanent.and(R::Not(Box::new(R::IsBasicLand))),
                    ),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(crate::effect::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Not(Box::new(R::IsBasicLand)),
                    }),
                effect: Effect::Seq(vec![
                    Effect::CantBeRegeneratedThisTurn {
                        what: Selector::SharingNameWith(Box::new(Selector::TriggerSource)),
                    },
                    Effect::Destroy {
                        what: Selector::SharingNameWith(Box::new(Selector::TriggerSource)),
                    },
                ]),
            },
        ],
        ..enchantment("Eye of Singularity", cost(&[generic(3), w()]))
    }
}

// ── The Chimera cycle ───────────────────────────────────────────────────────

/// The four Chimeras: {4} 2/2 artifact creatures that pass their keyword — and
/// a +2/+2 counter — to another Chimera on the way out.
fn chimera(name: &'static str, keyword: Keyword) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![keyword.clone()],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::HasCreatureType(CreatureType::Chimera)),
                    kind: CounterType::PlusTwoPlusTwo,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword,
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..creature(name, cost(&[generic(4)]), vec![CreatureType::Chimera], 2, 2)
    }
}

/// Brass-Talon Chimera — passes first strike.
pub fn brass_talon_chimera() -> CardDefinition {
    chimera("Brass-Talon Chimera", Keyword::FirstStrike)
}

/// Iron-Heart Chimera — passes vigilance.
pub fn iron_heart_chimera() -> CardDefinition {
    chimera("Iron-Heart Chimera", Keyword::Vigilance)
}

/// Lead-Belly Chimera — passes trample.
pub fn lead_belly_chimera() -> CardDefinition {
    chimera("Lead-Belly Chimera", Keyword::Trample)
}

/// Tin-Wing Chimera — passes flying.
pub fn tin_wing_chimera() -> CardDefinition {
    chimera("Tin-Wing Chimera", Keyword::Flying)
}

// ── Wave four ───────────────────────────────────────────────────────────────

/// Sands of Time — nobody untaps; instead every board flips at each upkeep.
pub fn sands_of_time() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each player skips their untap step.",
            effect: StaticEffect::SkipStep { step: TurnStep::Untap, all_players: true },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::SwapTappedState {
                what: Selector::ControlledBy {
                    who: PlayerRef::ActivePlayer,
                    filter: R::Artifact.or(R::Creature).or(R::Land),
                },
            },
        }],
        ..artifact("Sands of Time", cost(&[generic(4)]), vec![])
    }
}

/// City of Solitude — everyone acts only on their own turn.
pub fn city_of_solitude() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can cast spells and activate abilities only during their own turns.",
            effect: StaticEffect::PlayersActOnlyOnTheirOwnTurn,
        }],
        ..enchantment("City of Solitude", cost(&[generic(2), g()]))
    }
}


/// Kookus — {3}{R}{R} 3/5 trampler that hurts you and charges in unattended.
pub fn kookus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep().with_filter(crate::effect::Predicate::Not(Box::new(
                crate::effect::Predicate::SelectorExists(Selector::EachPermanent(
                    R::ControlledByYou.and(R::HasName("Keeper of Kookus".into())),
                )),
            ))),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::You, amount: Value::Const(3) },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::MustAttack,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Kookus", cost(&[generic(3), r(), r()]), vec![CreatureType::Djinn], 3, 5)
    }
}


/// Ovinomancer — three basics to keep it, and it turns anything into a Sheep.
pub fn ovinomancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessCost {
            cost: WardCost::ReturnMatchingToHand(Box::new(R::IsBasicLand), 3),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            return_permanent_cost: Some((R::IsSource, 1)),
            effect: Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
                Effect::Destroy { what: Selector::Target(0) },
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Sheep".into(),
                        power: 0,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Sheep],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Ovinomancer",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            1,
        )
    }
}


/// Lightning Cloud — {R} on any red spell turns into a ping.
pub fn lightning_cloud() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Red),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {R} to ping?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..enchantment("Lightning Cloud", cost(&[generic(3), r()]))
    }
}

/// Juju Bubble — a life-gain engine that pops the moment you play anything.
pub fn juju_bubble() -> CardDefinition {
    let pop = || TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
        effect: Effect::Sacrifice {
            who: Selector::You,
            count: Value::Const(1),
            filter: R::IsSource,
        },
    };
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![
            pop(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: R::IsSource,
                },
            },
        ],
        ..artifact(
            "Juju Bubble",
            cost(&[generic(1)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            }],
        )
    }
}

/// Infernal Harvest — {1}{B}; X Swamps back to hand becomes X damage.
pub fn infernal_harvest() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::ReturnToHand {
            filter: R::HasLandType(LandType::Swamp),
            count: 1,
            count_x: true,
        }],
        ..sorcery(
            "Infernal Harvest",
            cost(&[generic(1), b()]),
            Effect::DealDamageDivided {
                total: Value::XFromCost,
                filter: R::Creature,
                max_targets: 6,
                retaliate_to_source: false,
            },
        )
    }
}

/// Time and Tide — everything phased out comes back, and every phaser leaves.
pub fn time_and_tide() -> CardDefinition {
    instant(
        "Time and Tide",
        cost(&[u(), u()]),
        Effect::SwapPhasedState { filter: R::Creature },
    )
}

/// Katabatic Winds — a phasing enchantment that grounds every flier.
pub fn katabatic_winds() -> CardDefinition {
    let fliers = || Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying)));
    let ground = |description, keyword| StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword { applies_to: fliers(), keyword },
    };
    CardDefinition {
        keywords: vec![Keyword::Phasing],
        static_abilities: vec![
            ground("Creatures with flying can't attack.", Keyword::CantAttack),
            ground("Creatures with flying can't block.", Keyword::CantBlock),
            ground(
                "Their activated abilities with {T} in their costs can't be activated.",
                Keyword::CantActivateTapAbilities,
            ),
        ],
        ..enchantment("Katabatic Winds", cost(&[generic(2), g()]))
    }
}

/// Teferi's Realm — every upkeep, a whole card type blinks out.
pub fn teferis_realm() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::ActivePlayer,
                body: Box::new(Effect::Seq(vec![
                    Effect::ChooseCardTypeForSource,
                    Effect::PhaseOut {
                        what: Selector::EachPermanent(R::IsSourceChosenCardType.and(R::NotToken)),
                        until_source_leaves: false,
                    },
                ])),
            },
        }],
        ..enchantment("Teferi's Realm", cost(&[generic(1), u(), u()]))
    }
}

/// Equipoise — each upkeep, the opponent's excess of each type phases out.
pub fn equipoise() -> CardDefinition {
    let excess = |filter: R| {
        let theirs = Value::CountMatching {
            sel: Box::new(Selector::ControlledBy {
                who: PlayerRef::Target(0),
                filter: filter.clone(),
            }),
            filter: filter.clone(),
        };
        let yours = Value::CountMatching {
            sel: Box::new(Selector::EachPermanent(filter.clone().and(R::ControlledByYou))),
            filter: filter.clone(),
        };
        Effect::PhaseOut {
            what: Selector::Take {
                inner: Box::new(Selector::ControlledBy { who: PlayerRef::Target(0), filter }),
                count: Box::new(Value::NonNeg(Box::new(Value::Diff(
                    Box::new(theirs),
                    Box::new(yours),
                )))),
            },
            until_source_leaves: false,
        }
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::Seq(vec![excess(R::Land), excess(R::Artifact), excess(R::Creature)]),
        }],
        ..enchantment("Equipoise", cost(&[generic(2), w()]))
    }
}

/// Guiding Spirit — {T}: shove a creature card back on top of a library.
pub fn guiding_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TopOfGraveyardToLibraryTop {
                who: PlayerRef::Target(0),
                filter: R::Creature,
            },
            ..Default::default()
        }],
        ..creature(
            "Guiding Spirit",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Angel, CreatureType::Spirit],
            1,
            2,
        )
    }
}

/// Wand of Denial — {T}: peek at a library top and pay 2 life to bin it.
pub fn wand_of_denial() -> CardDefinition {
    artifact(
        "Wand of Denial",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LookTopMayPayLifeToBin {
                who: PlayerRef::Target(0),
                filter: R::Nonland,
                life: 2,
            },
            ..Default::default()
        }],
    )
}

/// Pillar Tombs of Aku — {2}{B}{B} World enchantment: a creature a turn, or 5 life.
pub fn pillar_tombs_of_aku() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ActivePlayer,
                cost: WardCost::SacrificeCreature,
                then: Box::new(Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::ActivePlayer),
                        amount: Value::Const(5),
                    },
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: R::IsSource,
                    },
                ])),
                if_paid: None,
            },
        }],
        ..enchantment("Pillar Tombs of Aku", cost(&[generic(2), b(), b()]))
    }
}

// ── Wave five ───────────────────────────────────────────────────────────────

/// Vision Charm — {U} for a mill, a land-type swap, or a phase-out.
pub fn vision_charm() -> CardDefinition {
    instant(
        "Vision Charm",
        cost(&[u()]),
        Effect::ChooseMode(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(4),
            },
            Effect::LandsBecomeChosenBasicType {
                what: Selector::EachPermanent(R::Land),
                duration: Duration::EndOfTurn,
                from_chosen_basic: true,
            },
            Effect::PhaseOut { what: target_filtered(R::Artifact), until_source_leaves: false },
        ]),
    )
}

/// Elephant Grass — {G}; black creatures can't attack you, and the rest pay.
pub fn elephant_grass() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        static_abilities: vec![
            StaticAbility {
                description: "Black creatures can't attack you.",
                effect: StaticEffect::CreaturesCantAttackController {
                    protect_planeswalkers: false,
                    filter: Some(R::HasColor(Color::Black)),
                },
            },
            StaticAbility {
                description: "Nonblack creatures can't attack you unless their controller pays {2} for each.",
                effect: StaticEffect::AttackTaxToController {
                    amount: Value::Const(2),
                    protect_planeswalkers: false,
                    filter: Some(R::Not(Box::new(R::HasColor(Color::Black)))),
                },
            },
        ],
        ..enchantment("Elephant Grass", cost(&[g()]))
    }
}

/// Heat Wave — {2}{R}; blue creatures can't block you, and the rest bleed for it.
pub fn heat_wave() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[r()])))],
        static_abilities: vec![
            StaticAbility {
                description: "Blue creatures can't block creatures you control.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::HasColor(Color::Blue)).and(R::ControlledByOpponent),
                    ),
                    keyword: Keyword::CantBlock,
                },
            },
            StaticAbility {
                description: "Nonblue creatures can't block unless their controller pays 1 life for each.",
                effect: StaticEffect::BlockTaxToController {
                    amount: Value::Const(1),
                    only_while_attacking: false,
                    filter: Some(R::Not(Box::new(R::HasColor(Color::Blue)))),
                    life: true,
                },
            },
        ],
        ..enchantment("Heat Wave", cost(&[generic(2), r()]))
    }
}

/// Corrosion — {1}{B}{R}; rust piles up on an opponent's artifacts until they crumble.
pub fn corrosion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::ControlledBy {
                            who: PlayerRef::Target(0),
                            filter: R::Artifact,
                        },
                        kind: CounterType::Rust,
                        amount: Value::Const(1),
                    },
                    Effect::CantBeRegeneratedThisTurn {
                        what: Selector::EachPermanent(
                            R::Artifact.and(R::ManaValueAtMostOwnCounters(CounterType::Rust)),
                        ),
                    },
                    Effect::Destroy {
                        what: Selector::EachPermanent(
                            R::Artifact.and(R::ManaValueAtMostOwnCounters(CounterType::Rust)),
                        ),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::RemoveCounter {
                    what: Selector::EachPermanent(R::WithAnyCounter),
                    kind: CounterType::Rust,
                    amount: Value::Const(99),
                },
            },
        ],
        ..enchantment("Corrosion", cost(&[generic(1), b(), r()]))
    }
}

/// Dream Tides — {2}{U}{U}; creatures stay tapped unless you buy them back.
pub fn dream_tides() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: Selector::EachPermanent(R::Creature),
                condition: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::ActivePlayer,
                body: Box::new(Effect::MayPayRepeatedly {
                    who: PlayerRef::You,
                    description: "Pay {2} to untap a tapped nongreen creature?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(Effect::Untap {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachPermanent(
                                R::Creature
                                    .and(R::ControlledByYou)
                                    .and(R::Tapped)
                                    .and(R::Not(Box::new(R::HasColor(Color::Green)))),
                            )),
                            count: Box::new(Value::Const(1)),
                        },
                        up_to: None,
                    }),
                }),
            },
        }],
        ..enchantment("Dream Tides", cost(&[generic(2), u(), u()]))
    }
}

/// Three Wishes — {1}{U}{U}; three cards off the top, playable until your next turn.
pub fn three_wishes() -> CardDefinition {
    instant(
        "Three Wishes",
        cost(&[u(), u(), generic(1)]),
        Effect::Seq(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(3),
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Graveyard,
                }),
            },
        ]),
    )
}

/// Foreshadow — {1}{U}; name a card, strip their top, and draw either way.
pub fn foreshadow() -> CardDefinition {
    instant(
        "Foreshadow",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::NameCard { what: Selector::This, restrict_to: None },
            Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(1) },
            Effect::If {
                cond: crate::effect::Predicate::ValueAtLeast(
                    Value::CardsMilledThisEffectMatching { filter: R::NamedBySource },
                    Value::Const(1),
                ),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        ]),
    )
}

/// Necromancy — {2}{B}; an Aura that reanimates and takes the creature with it.
pub fn necromancy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::Attach { what: Selector::This, to: Selector::LastMoved },
            ])),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::SacrificeSelected {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                },
            },
        ],
        ..enchantment("Necromancy", cost(&[generic(2), b()]))
    }
}

/// Undiscovered Paradise — any colour, but it bounces itself next untap step.
pub fn undiscovered_paradise() -> CardDefinition {
    CardDefinition {
        name: "Undiscovered Paradise",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                Effect::ReturnToHandAtYourNextUntapStep { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Desolation — {1}{B}{B}; tapping a land for mana costs you one at end step.
pub fn desolation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::If {
                    cond: crate::effect::Predicate::TappedLandForManaThisTurn(PlayerRef::You),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice {
                            who: Selector::You,
                            count: Value::Const(1),
                            filter: R::Land,
                        },
                        Effect::If {
                            cond: crate::effect::Predicate::EntityMatches {
                                what: Selector::SacrificedCard,
                                filter: R::HasLandType(LandType::Plains),
                            },
                            then: Box::new(Effect::DealDamage {
                                to: Selector::You,
                                amount: Value::Const(2),
                            }),
                            else_: Box::new(Effect::Noop),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        ..enchantment("Desolation", cost(&[generic(1), b(), b()]))
    }
}

/// Elkin Lair — {3}{R} World enchantment; every upkeep gambles a card from hand.
pub fn elkin_lair() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::ExileRandomFromHandMayPlayThisTurn { who: PlayerRef::ActivePlayer },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Graveyard,
                    }),
                },
            ]),
        }],
        ..enchantment("Elkin Lair", cost(&[generic(3), r()]))
    }
}
