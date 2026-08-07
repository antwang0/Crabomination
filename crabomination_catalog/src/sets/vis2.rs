//! Visions (VIS), second wave. Tests in `classic_sets/vis`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword,
    LandType, SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, WardCost,
};
use crate::game::TurnStep;
use crate::effect::shortcut::{add_colorless, etb, target_any, target_filtered};
use crate::effect::{
    CounteredSpellZone, DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
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

/// A land whose only printed mana ability is `{T}: Add {C}`, plus one extra.
fn utility_land(name: &'static str, extra: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_colorless(1), ..Default::default() },
            extra,
        ],
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Scalebane's Elite — {3}{G}{W} 4/4 with protection from black.
pub fn scalebanes_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        ..creature(
            "Scalebane's Elite",
            cost(&[generic(3), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

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

/// Rainbow Efreet — {3}{U} 3/1 flier that can duck out of anything for {U}{U}.
pub fn rainbow_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
            ..Default::default()
        }],
        ..creature("Rainbow Efreet", cost(&[generic(3), u()]), vec![CreatureType::Efreet], 3, 1)
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

/// Brood of Cockroaches — {1}{B} 1/1 that buys itself back a turn later.
pub fn brood_of_cockroaches() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                ])),
            },
        }],
        ..creature("Brood of Cockroaches", cost(&[generic(1), b()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Matopi Golem — {5} 3/3 that regenerates for {1}, shrinking each time.
pub fn matopi_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Regenerated, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(1),
            },
        }],
        ..creature("Matopi Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
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

/// Knight of Valor — {2}{W} 2/2 flanker that can shrink its blockers again.
pub fn knight_of_valor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::BlockingCreatures),
                    filter: R::Not(Box::new(R::HasKeyword(Keyword::Flanking))),
                },
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Knight of Valor",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Quicksand — sacrifices itself to shrink a ground attacker.
pub fn quicksand() -> CardDefinition {
    utility_land(
        "Quicksand",
        ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::IsAttacking.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                },
                power: Value::Const(-1),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        },
    )
}

/// Griffin Canyon — untaps a Griffin and pumps it.
pub fn griffin_canyon() -> CardDefinition {
    utility_land(
        "Griffin Canyon",
        ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: target_filtered(R::HasCreatureType(CreatureType::Griffin)),
                    up_to: None,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        },
    )
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Magma Mine — {4} charges it; sacrificing it throws the whole charge.
pub fn magma_mine() -> CardDefinition {
    artifact(
        "Magma Mine",
        cost(&[generic(1)]),
        vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Pressure,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: target_any(),
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Pressure,
                    },
                },
                ..Default::default()
            },
        ],
    )
}

/// Snake Basket — {4} artifact that dumps X Snakes on the way out.
pub fn snake_basket() -> CardDefinition {
    artifact(
        "Snake Basket",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: TokenDefinition {
                    name: "Snake".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Snake],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
    )
}

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

/// Righteous War — {1}{W}{B}; each half of your board shrugs off the other colour.
pub fn righteous_war() -> CardDefinition {
    let grant = |own: Color, from: Color| StaticAbility {
        description: "Your creatures of one colour have protection from the other.",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::HasColor(own)),
            ),
            keyword: Keyword::Protection(from),
        },
    };
    CardDefinition {
        static_abilities: vec![
            grant(Color::White, Color::Black),
            grant(Color::Black, Color::White),
        ],
        ..enchantment("Righteous War", cost(&[generic(1), w(), b()]))
    }
}

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

/// Flooded Shoreline — bounce a creature by bouncing two Islands.
pub fn flooded_shoreline() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            return_permanent_cost: Some((R::HasLandType(LandType::Island), 2)),
            effect: Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..enchantment("Flooded Shoreline", cost(&[u(), u()]))
    }
}

/// Death Watch — {B} Aura that cashes the host's stats in when it dies.
pub fn death_watch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                        Selector::TriggerSource,
                    ))),
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                },
            ]),
        }],
        ..aura("Death Watch", cost(&[b()]), EquipBonus::default())
    }
}

/// Vanishing — {U} Aura whose host can duck out for {U}{U}.
pub fn vanishing() -> CardDefinition {
    aura(
        "Vanishing",
        cost(&[u()]),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[u(), u()]),
                effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Miraculous Recovery — {4}{W} reanimation with a +1/+1 counter attached.
pub fn miraculous_recovery() -> CardDefinition {
    instant(
        "Miraculous Recovery",
        cost(&[generic(4), w()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
    )
}

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

/// Song of Blood — {1}{R}; mills four and pays out in combat this turn.
pub fn song_of_blood() -> CardDefinition {
    sorcery(
        "Song of Blood",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(4) },
            Effect::GrantTriggeredAbility {
                what: Selector::EachPermanent(R::Any),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::CreatureCardsMilledThisEffect,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Suleiman's Legacy — {R}{W}; no Djinn or Efreet survives it.
pub fn suleimans_legacy() -> CardDefinition {
    let djinn_or_efreet = || {
        R::HasCreatureType(CreatureType::Djinn).or(R::HasCreatureType(CreatureType::Efreet))
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Destroy { what: Selector::EachPermanent(djinn_or_efreet()) },
                Effect::CantBeRegeneratedThisTurn {
                    what: Selector::EachPermanent(djinn_or_efreet()),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(crate::effect::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: djinn_or_efreet(),
                    }),
                effect: Effect::Seq(vec![
                    Effect::CantBeRegeneratedThisTurn { what: Selector::TriggerSource },
                    Effect::Destroy { what: Selector::TriggerSource },
                ]),
            },
        ],
        ..enchantment("Suleiman's Legacy", cost(&[r(), w()]))
    }
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

/// Quirion Druid — {G}, {T}: a land becomes a 2/2 green creature, for good.
pub fn quirion_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(R::Land),
                power: Value::Const(2),
                toughness: Value::Const(2),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..creature(
            "Quirion Druid",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            2,
        )
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

/// Knight of the Mists — {2}{U} 2/2 flanker; pay {U} or a Knight dies.
pub fn knight_of_the_mists() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {U} to spare the Knight?".into(),
            mana_cost: cost(&[u()]),
            body: Box::new(Effect::Noop),
            else_: Some(Box::new(Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn {
                    what: target_filtered(R::HasCreatureType(CreatureType::Knight)),
                },
                Effect::Destroy { what: Selector::Target(0) },
            ]))),
        })],
        ..creature(
            "Knight of the Mists",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Righteous Aura — {W} and two life buys off one source's next hit.
pub fn righteous_aura() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            life_cost: 2,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..enchantment("Righteous Aura", cost(&[generic(1), w()]))
    }
}

/// Phyrexian Marauder — {X} construct that pays again every time it attacks.
pub fn phyrexian_marauder() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        keywords: vec![
            Keyword::CantBlock,
            Keyword::CantAttackOrBlockUnlessPayPerCounter(CounterType::PlusOnePlusOne),
        ],
        ..creature(
            "Phyrexian Marauder",
            cost(&[x()]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            0,
            0,
        )
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

/// Vampirism — {1}{B} Aura that drains the rest of your board into its host.
pub fn vampirism() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get -1/-1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::Not(Box::new(R::IsHostOfSource))),
                power: -1,
                toughness: -1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![etb(Effect::DelayUntil {
            kind: DelayedTriggerKind::YourNextUpkeep,
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        })],
        ..aura(
            "Vampirism",
            cost(&[generic(1), b()]),
            EquipBonus {
                scale: Some(EquipScale {
                    filter: R::Creature.and(R::ControlledByYou),
                    per_power: 1,
                    per_toughness: 1,
                    exclude_host: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
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
