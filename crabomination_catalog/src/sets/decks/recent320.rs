//! Mirrodin (MRD) gap batch 5 — the imprint artifacts, the evasion/tax
//! creatures and the upkeep engines. Tests in `recent_b/mrd`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect};
use crate::game::TurnStep;
use crate::mana::{ManaCost, cost, g, generic, r, u};

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

/// "Imprint — When this permanent enters, you may exile target card from a
/// graveyard."
fn imprint_from_graveyard() -> TriggeredAbility {
    etb(Effect::MayDo {
        description: "Exile a card from a graveyard with this?".into(),
        body: Box::new(Effect::ExileWithSource {
            what: target_filtered(R::InGraveyard),
        }),
    })
}

// ── Imprint artifacts ──

/// Mirror Golem — imprints a graveyard card and copies its protections.
pub fn mirror_golem() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![imprint_from_graveyard()],
        static_abilities: vec![StaticAbility {
            description: "This creature has protection from each of the exiled card's card types.",
            effect: StaticEffect::ProtectionFromExiledWithCardTypes,
        }],
        ..artifact_creature(
            "Mirror Golem",
            cost(&[generic(6)]),
            3,
            4,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Mourner's Shield — imprints a graveyard card, then blanks a source sharing
/// its colour.
pub fn mourners_shield() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![imprint_from_graveyard()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PreventAllDamageFromChosenSourceThisTurn {
                filter: R::SharesColorWithExiledBySource,
            },
            ..Default::default()
        }],
        ..artifact("Mourner's Shield", cost(&[generic(4)]))
    }
}

/// Soul Foundry — imprints a creature card from hand and stamps out copies.
pub fn soul_foundry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile a creature card from your hand with this?".into(),
            body: Box::new(Effect::ExileChosenFromHand {
                from: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
                link_to_source: true,
                face_down: false,
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x()]),
            tap_cost: true,
            condition: Some(Predicate::ExiledWithSourceManaValueIsX),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::CardExiledWithSource,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            ..Default::default()
        }],
        ..artifact("Soul Foundry", cost(&[generic(4)]))
    }
}

/// Thought Prison — imprints a card off an opponent's hand and punishes
/// anything that shares its colour or mana value.
pub fn thought_prison() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Look at target player's hand and exile a nonland card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::LookAtHand {
                        who: Selector::Player(PlayerRef::Target(0)),
                    },
                    Effect::ExileChosenFromHand {
                        from: Selector::Player(PlayerRef::Target(0)),
                        count: Value::ONE,
                        filter: R::Nonland,
                        link_to_source: true,
                        face_down: false,
                    },
                ])),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                    .with_filter(Predicate::CastSharesColorOrManaValueWithExiledBySource),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: crate::effect::Value::Const(2),
                },
            },
        ],
        ..artifact("Thought Prison", cost(&[generic(5)]))
    }
}

// ── Artifact engines ──

/// Blinkmoth Urn — every player's first main phase burns their artifact count
/// into colourless mana.
pub fn blinkmoth_urn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::Tapped,
            }))),
            effect: Effect::AddMana {
                who: PlayerRef::ActivePlayer,
                pool: crate::effect::ManaPayload::Colorless(Value::CountOf(Box::new(
                    Selector::ControlledBy {
                        who: PlayerRef::ActivePlayer,
                        filter: R::Artifact,
                    },
                ))),
            },
        }],
        ..artifact("Blinkmoth Urn", cost(&[generic(5)]))
    }
}

/// Farsight Mask — while untapped, opponents' damage refills your hand.
pub fn farsight_mask() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourControl)
                .from_opponent()
                .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Tapped,
                }))),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
        }],
        ..artifact("Farsight Mask", cost(&[generic(5)]))
    }
}

/// Psychogenic Probe — every shuffle costs its owner two.
pub fn psychogenic_probe() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LibraryShuffled, EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            },
        }],
        ..artifact("Psychogenic Probe", cost(&[generic(2)]))
    }
}

/// One half of Power Conduit's "choose one" — split into two activations so
/// each mode declares its own target type.
fn conduit_mode(filter: R, kind: CounterType) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        remove_counter_among_filter: Some((None, 1, R::Permanent)),
        effect: Effect::AddCounter {
            what: target_filtered(filter),
            kind,
            amount: Value::ONE,
        },
        ..Default::default()
    }
}

/// Power Conduit — recycles a counter into a charge or a +1/+1 counter.
pub fn power_conduit() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            conduit_mode(R::Artifact, CounterType::Charge),
            conduit_mode(R::Creature, CounterType::PlusOnePlusOne),
        ],
        ..artifact("Power Conduit", cost(&[generic(2)]))
    }
}

/// Gate to the Aether — each upkeep the top card may deploy itself for free.
pub fn gate_to_the_aether() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::RevealTopMayPutOntoBattlefield {
                who: PlayerRef::ActivePlayer,
                filter: R::Artifact.or(R::Creature).or(R::Enchantment).or(R::Land),
                counter: None,
                extra_types: Vec::new(),
            },
        }],
        ..artifact("Gate to the Aether", cost(&[generic(6)]))
    }
}

// ── Equipment ──

/// Golem-Skin Gauntlets — every Equipment on the host feeds its power.
pub fn golem_skin_gauntlets() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 0,
                count_host_attachments: Some(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..artifact("Golem-Skin Gauntlets", cost(&[generic(1)]))
    }
}

// ── Creatures ──

/// Arc-Slogger — {R} and ten cards off the top for two damage anywhere.
pub fn arc_slogger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            exile_top_cost: 10,
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Arc-Slogger",
            cost(&[generic(3), r(), r()]),
            4,
            5,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Neurok Spy — an artifact on the other side of the table is a free pass.
pub fn neurok_spy() -> CardDefinition {
    creature(
        "Neurok Spy",
        cost(&[generic(2), u()]),
        2,
        2,
        vec![CreatureType::Human, CreatureType::Rogue],
        vec![Keyword::CantBeBlockedIfDefenderControls(Box::new(
            R::Artifact,
        ))],
    )
}

/// Myr Prototype — grows every upkeep, and charges you to use it.
pub fn myr_prototype() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..artifact_creature(
            "Myr Prototype",
            cost(&[generic(5)]),
            2,
            2,
            vec![CreatureType::Myr],
            vec![Keyword::CantAttackOrBlockUnlessPayPerCounter(
                CounterType::PlusOnePlusOne,
            )],
        )
    }
}

/// Wanderguard Sentry — ETB: look at target opponent's hand.
pub fn wanderguard_sentry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookAtHand {
            who: Selector::Player(PlayerRef::Target(0)),
        })],
        ..creature(
            "Wanderguard Sentry",
            cost(&[generic(4), u()]),
            3,
            3,
            vec![CreatureType::Drone],
            vec![],
        )
    }
}

/// Lumengrid Augur — loots a player, and untaps when the loot is an artifact.
pub fn lumengrid_augur() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::DiscardedThisResolution {
                        filter: R::Artifact,
                    }),
                    then: Box::new(Effect::Untap {
                        what: Selector::This,
                        up_to: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Lumengrid Augur",
            cost(&[generic(3), u()]),
            2,
            2,
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Glissa Sunseeker — cracks an artifact whose cost matches your open mana.
pub fn glissa_sunseeker() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Artifact.and(R::ManaValueEqualsYourUnspentMana),
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Glissa Sunseeker",
            cost(&[generic(2), g(), g()]),
            3,
            2,
            vec![CreatureType::Elf, CreatureType::Warrior],
            vec![Keyword::FirstStrike],
        )
    }
}

/// War Elemental — needs blood on the table, then feeds on every point of it.
pub fn war_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::PlayerDamagedThisTurn {
                    who: PlayerRef::EachOpponent,
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::SacrificeSource),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..creature(
            "War Elemental",
            cost(&[r(), r(), r()]),
            1,
            1,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

// ── Enchantments ──

/// March of the Machines — every noncreature artifact stands up as an `MV/MV`.
pub fn march_of_the_machines() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each noncreature artifact is an artifact creature with power and toughness each equal to its mana value.",
            effect: StaticEffect::NoncreatureArtifactsAreCreatures,
        }],
        ..enchantment("March of the Machines", cost(&[generic(3), u()]))
    }
}

/// Fatespinner — each opponent gives up a step every turn.
pub fn fatespinner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            )
            .from_opponent(),
            effect: Effect::ChooseStepToSkipThisTurn {
                who: PlayerRef::ActivePlayer,
            },
        }],
        ..creature(
            "Fatespinner",
            cost(&[generic(1), u(), u()]),
            1,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Fractured Loyalty — whoever points a spell at the enchanted creature takes
/// it home.
pub fn fractured_loyalty() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::EnchantedBySource),
            effect: Effect::GainControl {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: Some(PlayerRef::Triggerer),
                duration: Duration::Permanent,
            },
        }],
        ..aura("Fractured Loyalty", cost(&[generic(1), r()]), R::Creature)
    }
}
