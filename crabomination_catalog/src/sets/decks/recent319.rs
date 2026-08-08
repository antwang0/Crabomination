//! Mirrodin (MRD) gap batch 4 — the Towers, the Clockwork cycle, the artifact
//! Equipment and the entwine finishers. Tests in `recent_b/mrd`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{blocks, etb, on_dies, target_filtered};
use crate::effect::{
    CounteredSpellZone, DelayedTriggerKind, Duration, Effect, LibraryPosition, PlayerRef,
    StaticEffect, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        ..Default::default()
    }
}

fn creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

fn artifact_creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, mana, power, toughness, types, keywords)
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        ..artifact(name, mana)
    }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost, filter: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter },
        },
        ..enchantment(name, mana)
    }
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery {
            CardType::Sorcery
        } else {
            CardType::Instant
        }],
        effect,
        ..Default::default()
    }
}

/// The Mirrodin "Tower" cycle: `{8}, {T}` for one big effect.
fn tower(name: &'static str, effect: Effect) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            tap_cost: true,
            effect,
            ..Default::default()
        }],
        ..artifact(name, cost(&[generic(4)]))
    }
}

/// The Clockwork cycle: enters with N +1/+1 counters and sheds one at end of
/// combat whenever it attacks or blocks.
fn clockwork(
    name: &'static str,
    mana: ManaCost,
    counters: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    let shed = || TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::DelayUntil {
            kind: DelayedTriggerKind::EndOfCombat,
            body: Box::new(Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        },
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(counters))),
        triggered_abilities: vec![
            shed(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                ..shed()
            },
        ],
        ..artifact_creature(name, mana, 0, 0, types, keywords)
    }
}

// ── Towers ──

/// Tower of Eons — {8}, {T}: You gain 10 life.
pub fn tower_of_eons() -> CardDefinition {
    tower(
        "Tower of Eons",
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(10),
        },
    )
}

/// Tower of Fortunes — {8}, {T}: Draw four cards.
pub fn tower_of_fortunes() -> CardDefinition {
    tower(
        "Tower of Fortunes",
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(4),
        },
    )
}

/// Tower of Murmurs — {8}, {T}: Target player mills eight cards.
pub fn tower_of_murmurs() -> CardDefinition {
    tower(
        "Tower of Murmurs",
        Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(8),
        },
    )
}

// ── Clockwork cycle ──

/// Clockwork Beetle — {1} 0/0 with two +1/+1 counters.
pub fn clockwork_beetle() -> CardDefinition {
    clockwork(
        "Clockwork Beetle",
        cost(&[generic(1)]),
        2,
        vec![CreatureType::Insect],
        vec![],
    )
}

/// Clockwork Condor — {4} 0/0 flier with three +1/+1 counters.
pub fn clockwork_condor() -> CardDefinition {
    clockwork(
        "Clockwork Condor",
        cost(&[generic(4)]),
        3,
        vec![CreatureType::Bird],
        vec![Keyword::Flying],
    )
}

/// Clockwork Vorrac — {5} 0/0 trampler with four +1/+1 counters; {T} recharges.
pub fn clockwork_vorrac() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..clockwork(
            "Clockwork Vorrac",
            cost(&[generic(5)]),
            4,
            vec![CreatureType::Boar, CreatureType::Beast],
            vec![Keyword::Trample],
        )
    }
}

/// Clockwork Dragon — {7} 0/0 flier with six +1/+1 counters; {3} recharges.
pub fn clockwork_dragon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..clockwork(
            "Clockwork Dragon",
            cost(&[generic(7)]),
            6,
            vec![CreatureType::Dragon],
            vec![Keyword::Flying],
        )
    }
}

// ── Equipment ──

/// Banshee's Blade — grows by a charge counter each time the equipped creature
/// connects.
pub fn banshees_blade() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggers_on_equipment: true,
            scale: Some(EquipScale {
                filter: R::Artifact,
                per_power: 1,
                per_toughness: 1,
                count_self_counters: Some(CounterType::Charge),
                ..Default::default()
            }),
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                },
                TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToCreature,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                },
            ],
            ..Default::default()
        }),
        ..equipment("Banshee's Blade", cost(&[generic(2)]), cost(&[generic(2)]))
    }
}

/// Nightmare Lash — +1/+1 for each Swamp you control. Equip—Pay 3 life.
pub fn nightmare_lash() -> CardDefinition {
    CardDefinition {
        equip_life_cost: 3,
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::HasLandType(crate::card::LandType::Swamp),
                per_power: 1,
                per_toughness: 1,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..equipment("Nightmare Lash", cost(&[generic(4)]), cost(&[]))
    }
}

/// Dead-Iron Sledge — the equipped creature and whatever it meets in combat
/// both die.
pub fn dead_iron_sledge() -> CardDefinition {
    let mutual_kill = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::This,
            },
            Effect::Destroy {
                what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
            },
        ]),
    };
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![
                mutual_kill(EventKind::Blocks),
                mutual_kill(EventKind::BecomesBlocked),
            ],
            ..Default::default()
        }),
        ..equipment("Dead-Iron Sledge", cost(&[generic(1)]), cost(&[generic(2)]))
    }
}

/// Worldslayer — connect and the board (bar the Equipment) is gone.
pub fn worldslayer() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(R::Permanent.and(R::OtherThanSource)),
                },
            }],
            ..Default::default()
        }),
        ..equipment("Worldslayer", cost(&[generic(5)]), cost(&[generic(5)]))
    }
}

// ── Spells ──

/// Disarm — unattach every Equipment from a creature.
pub fn disarm() -> CardDefinition {
    spell(
        "Disarm",
        cost(&[u()]),
        false,
        Effect::Unattach {
            what: Selector::AttachedToMe(Box::new(target_filtered(R::Creature))),
        },
    )
}

/// Razor Barrier — protection from artifacts or from a colour of your choice.
pub fn razor_barrier() -> CardDefinition {
    spell(
        "Razor Barrier",
        cost(&[generic(1), w()]),
        false,
        Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                keyword: Keyword::ProtectionFromCardType(CardType::Artifact),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantProtectionFromChosenColor {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Assert Authority — affinity for artifacts; counter and exile.
pub fn assert_authority() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Affinity for artifacts.",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                filter: R::Artifact,
                per: 1,
            },
        }],
        ..spell(
            "Assert Authority",
            cost(&[generic(5), u(), u()]),
            false,
            Effect::CounterSpellToZone {
                what: Selector::Target(0),
                zone: CounteredSpellZone::Exile,
            },
        )
    }
}

/// Forge Armor — sacrifice an artifact; its mana value becomes +1/+1 counters.
pub fn forge_armor() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact,
            count: 1,
        }],
        ..spell(
            "Forge Armor",
            cost(&[generic(4), r()]),
            false,
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::SacrificedManaValue,
            },
        )
    }
}

/// Solar Tide — sweep the small creatures or the big ones. Entwine—Sacrifice
/// two lands.
pub fn solar_tide() -> CardDefinition {
    CardDefinition {
        entwine_additional_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 2,
        }),
        ..spell(
            "Solar Tide",
            cost(&[generic(4), w(), w()]),
            true,
            Effect::ChooseMode(vec![
                Effect::Destroy {
                    what: Selector::EachPermanent(R::Creature.and(R::PowerAtMost(2))),
                },
                Effect::Destroy {
                    what: Selector::EachPermanent(R::Creature.and(R::PowerAtLeast(3))),
                },
            ]),
        )
    }
}

/// Betrayal of Flesh — kill a creature or reanimate one. Entwine—Sacrifice
/// three lands.
pub fn betrayal_of_flesh() -> CardDefinition {
    CardDefinition {
        entwine_additional_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 3,
        }),
        ..spell(
            "Betrayal of Flesh",
            cost(&[generic(5), b()]),
            false,
            Effect::ChooseMode(vec![
                Effect::Destroy {
                    what: target_filtered(R::Creature),
                },
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::InYourGraveyard),
                    },
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
        )
    }
}

/// Temporal Cascade — reset both hands or refill them. Entwine {2}.
pub fn temporal_cascade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(2)]))],
        ..spell(
            "Temporal Cascade",
            cost(&[generic(5), u(), u()]),
            true,
            Effect::ChooseMode(vec![
                Effect::ShuffleHandAndGraveyardIntoLibrary {
                    who: PlayerRef::EachPlayer,
                },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(7),
                },
            ]),
        )
    }
}

// ── Creatures ──

/// Vermiculos — swells whenever an artifact enters.
pub fn vermiculos() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Vermiculos",
            cost(&[generic(4), b()]),
            1,
            1,
            vec![CreatureType::Horror],
            vec![],
        )
    }
}

/// Auriok Bladewarden — {T}: target creature gets +X/+X, X = this creature's
/// power.
pub fn auriok_bladewarden() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::PowerOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Auriok Bladewarden",
            cost(&[generic(1), w()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Lodestone Myr — tap an artifact to pump.
pub fn lodestone_myr() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Artifact),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Lodestone Myr",
            cost(&[generic(4)]),
            2,
            2,
            vec![CreatureType::Myr],
            vec![Keyword::Trample],
        )
    }
}

/// Chimney Imp — the classic bad rare: dies, and an opponent stacks a card.
pub fn chimney_imp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::PutCardFromHandOnTopOfLibrary {
            who: Selector::Player(PlayerRef::Target(0)),
        })],
        ..creature(
            "Chimney Imp",
            cost(&[generic(4), b()]),
            1,
            2,
            vec![CreatureType::Imp],
            vec![Keyword::Flying],
        )
    }
}

/// Looming Hoverguard — ETB: put target artifact on top of its owner's library.
pub fn looming_hoverguard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Artifact),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        })],
        ..creature(
            "Looming Hoverguard",
            cost(&[generic(4), u(), u()]),
            3,
            3,
            vec![CreatureType::Drone],
            vec![Keyword::Flying],
        )
    }
}

/// Living Hive — connects and mints an Insect per point of damage.
pub fn living_hive() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::This)),
                definition: Box::new(TokenDefinition {
                    name: "Insect".into(),
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Insect],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                }),
            },
        }],
        ..creature(
            "Living Hive",
            cost(&[generic(6), g(), g()]),
            6,
            6,
            vec![CreatureType::Elemental, CreatureType::Insect],
            vec![Keyword::Trample],
        )
    }
}

/// Groffskithur — becomes blocked and may buy back a second copy.
pub fn groffskithur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![blocks(Effect::MayDo {
            description: "Return a Groffskithur from your graveyard to your hand?".into(),
            body: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard.and(R::HasName("Groffskithur".into())),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Groffskithur",
            cost(&[generic(5), g()]),
            3,
            3,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Reiver Demon — cast it from hand and every nonartifact, nonblack creature
/// dies.
pub fn reiver_demon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CastFromHand),
            effect: Effect::DestroyNoRegen {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::Not(Box::new(R::Artifact)))
                        .and(R::Not(Box::new(R::HasColor(Color::Black)))),
                ),
            },
        }],
        ..creature(
            "Reiver Demon",
            cost(&[generic(4), b(), b(), b(), b()]),
            6,
            6,
            vec![CreatureType::Demon],
            vec![Keyword::Flying],
        )
    }
}

/// Nim Devourer — artifact-scaled body that keeps climbing out of the yard.
pub fn nim_devourer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Artifact,
                per_power: 1,
                per_toughness: 0,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            from_graveyard: true,
            condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Nim Devourer",
            cost(&[generic(3), b(), b()]),
            4,
            1,
            vec![CreatureType::Zombie],
            vec![],
        )
    }
}

// ── Enchantments ──

/// Domineer — an Aura that steals the artifact creature it enchants.
pub fn domineer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        ..aura(
            "Domineer",
            cost(&[generic(1), u(), u()]),
            R::Creature.and(R::Artifact),
        )
    }
}

/// Hum of the Radix — artifact spells tax their caster's own board.
pub fn hum_of_the_radix() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each artifact spell costs {1} more to cast for each artifact its controller controls.",
            effect: StaticEffect::SpellTaxPerControllerPermanent {
                spell_filter: R::Artifact,
                count_filter: R::Artifact,
            },
        }],
        ..enchantment("Hum of the Radix", cost(&[generic(2), g(), g()]))
    }
}

// ── Artifacts ──

/// Sun Droplet — banks the damage you take and bleeds it back as life.
pub fn sun_droplet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::TriggerEventAmount,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::MayDo {
                    description: "Remove a charge counter from Sun Droplet to gain 1 life?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::Charge,
                            amount: Value::ONE,
                        },
                        Effect::GainLife {
                            who: Selector::You,
                            amount: Value::ONE,
                        },
                    ])),
                },
            },
        ],
        ..artifact("Sun Droplet", cost(&[generic(2)]))
    }
}

/// Pentavus — five counters that convert into fliers and back.
pub fn pentavus() -> CardDefinition {
    let pentavite = TokenDefinition {
        name: "Pentavite".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(pentavite),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_other_filter: Some((R::HasName("Pentavite".into()), 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..artifact_creature(
            "Pentavus",
            cost(&[generic(7)]),
            0,
            0,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Myr Incubator — trade the artifacts in your library for a Myr army.
pub fn myr_incubator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::SearchExileThenTokensPerCard {
                filter: R::Artifact,
                definition: Box::new(TokenDefinition {
                    name: "Myr".into(),
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Myr],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..artifact("Myr Incubator", cost(&[generic(6)]))
    }
}

/// Synod Sanctum — blink your board out of a wrath and bring it all back.
pub fn synod_sanctum() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::ExileWithSource {
                    what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                effect: Effect::ReturnExiledBySourceToBattlefield { decayed: false, count: None },
                ..Default::default()
            },
        ],
        ..artifact("Synod Sanctum", cost(&[generic(1)]))
    }
}

/// Jinxed Choker — a hot potato that grows every time it changes hands.
pub fn jinxed_choker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::SelfSource)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::This,
                        to: Some(PlayerRef::Target(0)),
                        duration: Duration::Permanent,
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Jinxed Choker", cost(&[generic(3)]))
    }
}

/// Lightning Coils — five dead creatures buy a hasty Elemental swarm.
pub fn lightning_coils() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::NotToken,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Charge,
                        n: 5,
                    },
                ])),
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        },
                        definition: Box::new(TokenDefinition {
                            name: "Elemental".into(),
                            colors: vec![Color::Red],
                            card_types: vec![CardType::Creature],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Elemental],
                                ..Default::default()
                            },
                            power: 3,
                            toughness: 1,
                            keywords: vec![Keyword::Haste],
                            ..Default::default()
                        }),
                    },
                    Effect::RemoveAllCounters {
                        what: Selector::This,
                    },
                    Effect::ExileLastCreatedTokensAtNextEndStep,
                ]),
            },
        ],
        ..artifact("Lightning Coils", cost(&[generic(3)]))
    }
}

/// Culling Scales — grinds the cheapest nonland permanent off the board.
pub fn culling_scales() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Nonland.and(R::LowestManaValueAmongNonland),
                },
            },
        }],
        ..artifact("Culling Scales", cost(&[generic(3)]))
    }
}

/// Loxodon Peacekeeper — a 4/4 that defects to whoever is losing.
pub fn loxodon_peacekeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::LowestLife),
                duration: Duration::Permanent,
            },
        }],
        ..creature(
            "Loxodon Peacekeeper",
            cost(&[generic(1), w()]),
            4,
            4,
            vec![CreatureType::Elephant, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Vulshok Battlemaster — hoovers up every Equipment on the battlefield.
pub fn vulshok_battlemaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::EachPermanent(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
            to: Selector::This,
        })],
        ..creature(
            "Vulshok Battlemaster",
            cost(&[generic(4), r()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![Keyword::Haste],
        )
    }
}
