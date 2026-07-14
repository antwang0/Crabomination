//! A twelfth wave — an Equipment-matters package (Equipment payoffs, equip-cost
//! and equip-speed statics, two-target Attach spells, living weapons). Tests in
//! `crabomination/src/tests/recent12.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, w, Color};

// ── Equipment payoff creatures ───────────────────────────────────────────────

/// Leonin Shikari — {1}{W} Cat Soldier 2/2. You may activate equip abilities
/// any time you could cast an instant.
pub fn leonin_shikari() -> CardDefinition {
    CardDefinition {
        name: "Leonin Shikari",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "You may activate equip abilities any time you could cast an instant.",
            effect: StaticEffect::ControllerEquipAtInstantSpeed,
        }],
        ..Default::default()
    }
}

/// Kemba, Kha Regent — {1}{W}{W} Legendary Cat Cleric 2/4. At your upkeep,
/// create a 2/2 white Cat token for each Equipment attached to Kemba.
pub fn kemba_kha_regent() -> CardDefinition {
    use crate::game::types::TurnStep;
    let cat = TokenDefinition {
        name: "Cat".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Kemba, Kha Regent",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountMatching {
                    sel: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
                    filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                },
                definition: cat,
            },
        }],
        ..Default::default()
    }
}

/// Danitha Capashen, Paragon — {2}{W} Legendary Human Knight 2/2 with first
/// strike, vigilance, lifelink. Aura and Equipment spells you cast cost {1} less.
pub fn danitha_capashen() -> CardDefinition {
    CardDefinition {
        name: "Danitha Capashen, Paragon",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Aura and Equipment spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Auriok Steelshaper — {1}{W} Human Soldier 1/1. Equip costs you pay cost {1}
/// less. As long as it's equipped, each Soldier or Knight you control gets
/// +1/+1.
pub fn auriok_steelshaper() -> CardDefinition {
    CardDefinition {
        name: "Auriok Steelshaper",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![
            StaticAbility {
                description: "Equip costs you pay cost {1} less.",
                effect: StaticEffect::EquipCostReduction { amount: 1 },
            },
            StaticAbility {
                description: "While equipped, Soldiers and Knights you control get +1/+1.",
                effect: StaticEffect::PumpTeamIf {
                    condition: crate::effect::Predicate::SourceIsEquipped,
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::ControlledByYou.and(SelectionRequirement::Creature).and(
                            SelectionRequirement::HasCreatureType(CreatureType::Soldier)
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Knight)),
                        ),
                    ),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                },
            },
        ],
        ..Default::default()
    }
}

/// Balan, Wandering Knight — {2}{W}{W} Legendary Cat Knight 3/3 with first
/// strike; has double strike while two or more Equipment are attached to it.
/// {1}{W}: Attach all Equipment you control to Balan.
pub fn balan_wandering_knight() -> CardDefinition {
    CardDefinition {
        name: "Balan, Wandering Knight",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Double strike while two or more Equipment are attached to it.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::DoubleStrike,
                condition: SelectionRequirement::EquippedByAtLeast(2),
            },
        }],
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Attach {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact
                        .and(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                to: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goblin Gaveleer — {R} Goblin Warrior 1/1 with trample. Gets +2/+0 for each
/// Equipment attached to it.
pub fn goblin_gaveleer() -> CardDefinition {
    CardDefinition {
        name: "Goblin Gaveleer",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::BasePlusPerAttachedEquipment { base_p: 1, base_t: 1, per: 2 }),
        ..Default::default()
    }
}

/// Valduk, Keeper of the Flame — {2}{R} Legendary Human Shaman 3/2. At combat on
/// your turn, for each Aura and Equipment attached to Valduk, create a 3/1 red
/// Elemental with trample and haste, then exile those tokens at the next end step.
pub fn valduk_keeper_of_the_flame() -> CardDefinition {
    use crate::game::types::TurnStep;
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 3,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        keywords: vec![Keyword::Trample, Keyword::Haste],
        ..Default::default()
    };
    CardDefinition {
        name: "Valduk, Keeper of the Flame",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountMatching {
                        sel: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                            .or(SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
                    },
                    definition: elemental,
                },
                Effect::ExileLastCreatedTokensAtNextEndStep,
            ]),
        }],
        ..Default::default()
    }
}

// ── ETB-attach Equipment ─────────────────────────────────────────────────────

/// Maul of the Skyclaves — {2}{W} Equipment. ETB: attach to target creature you
/// control. Equipped creature gets +2/+2 and has flying and first strike.
/// Equip {2}{W}{W}.
pub fn maul_of_the_skyclaves() -> CardDefinition {
    CardDefinition {
        name: "Maul of the Skyclaves",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2), w(), w()]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying, Keyword::FirstStrike],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        })],
        ..Default::default()
    }
}

/// Embercleave — {4}{R}{R} Legendary Equipment with flash. Costs {1} less for
/// each attacking creature you control. ETB: attach to target creature you
/// control. Equipped creature gets +1/+1, double strike, trample. Equip {3}.
/// (Cost reduction counts creatures that attacked this turn — exact during your
/// own combat, the flash window the card is built for.)
pub fn embercleave() -> CardDefinition {
    CardDefinition {
        name: "Embercleave",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each attacking creature you control.",
            effect: StaticEffect::SelfCostReducedPerCreatureAttackedThisTurn {
                per: 1,
                all_players: false,
            },
        }],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::DoubleStrike, Keyword::Trample],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        })],
        ..Default::default()
    }
}

// ── Equip-trigger Equipment ──────────────────────────────────────────────────

/// Armory of Iroas — {2} Equipment. Whenever equipped creature attacks, put a
/// +1/+1 counter on it. Equip {2}.
pub fn armory_of_iroas() -> CardDefinition {
    CardDefinition {
        name: "Armory of Iroas",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Flayer Husk — {1} Equipment with living weapon (mints a 0/0 Germ and
/// attaches itself). Equipped creature gets +1/+1. Equip {2}.
pub fn flayer_husk() -> CardDefinition {
    let germ = TokenDefinition {
        name: "Phyrexian Germ".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Flayer Husk",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: germ },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// Lizard Blades — {1}{R} Artifact Creature — Equipment Lizard 1/1 with double
/// strike. Equipped creature has double strike. Reconfigure {2}.
pub fn lizard_blades() -> CardDefinition {
    CardDefinition {
        name: "Lizard Blades",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::DoubleStrike, Keyword::Reconfigure(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::DoubleStrike],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Plain stat/keyword Equipment ─────────────────────────────────────────────

/// Cathar's Shield — {0} Equipment. +0/+3 and vigilance. Equip {3}.
pub fn cathars_shield() -> CardDefinition {
    super::modern::simple_equipment(
        "Cathar's Shield",
        cost(&[]),
        cost(&[generic(3)]),
        0,
        3,
        vec![Keyword::Vigilance],
    )
}

/// Leonin Scimitar — {1} Equipment. +1/+1. Equip {1}.
pub fn leonin_scimitar() -> CardDefinition {
    super::modern::simple_equipment(
        "Leonin Scimitar",
        cost(&[generic(1)]),
        cost(&[generic(1)]),
        1,
        1,
        vec![],
    )
}

/// Bladed Pinions — {2} Equipment. Flying and first strike. Equip {2}.
pub fn bladed_pinions() -> CardDefinition {
    super::modern::simple_equipment(
        "Bladed Pinions",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        0,
        0,
        vec![Keyword::Flying, Keyword::FirstStrike],
    )
}

// ── Equipment-matters spells & walkers ───────────────────────────────────────

/// Magnetic Theft — {R} Instant. Attach target Equipment to target creature.
/// (Control of the Equipment doesn't change.)
pub fn magnetic_theft() -> CardDefinition {
    CardDefinition {
        name: "Magnetic Theft",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Attach {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
            },
            to: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Creature },
        },
        ..Default::default()
    }
}

/// Sram's Expertise — {2}{W}{W} Sorcery. Create three 1/1 colorless Servo
/// artifact tokens, then you may cast a spell with mana value 3 or less from
/// your hand without paying its mana cost.
pub fn srams_expertise() -> CardDefinition {
    let servo = TokenDefinition {
        name: "Servo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Servo], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Sram's Expertise",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: servo },
            Effect::CastFromHandWithoutPaying {
                filter: Some(SelectionRequirement::ManaValueAtMost(3)),
            },
        ]),
        ..Default::default()
    }
}

// ── Undaunted (CR 702.125) — costs {1} less per opponent ─────────────────────

/// `Undaunted` static — "this spell costs {1} less to cast for each opponent."
fn undaunted() -> StaticAbility {
    StaticAbility {
        description: "This spell costs {1} less to cast for each opponent.",
        effect: StaticEffect::SelfCostReducedPerOpponent { per: 1 },
    }
}

/// Sublime Exhalation — {6}{W} Sorcery with Undaunted. Destroy all creatures.
pub fn sublime_exhalation() -> CardDefinition {
    CardDefinition {
        name: "Sublime Exhalation",
        cost: cost(&[generic(6), w()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![undaunted()],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Curtains' Call — {5}{B} Instant with Undaunted. Destroy two target creatures.
pub fn curtains_call() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Curtains' Call",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![undaunted()],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// Coastal Breach — {6}{U} Sorcery with Undaunted. Return all nonland
/// permanents to their owners' hands.
pub fn coastal_breach() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Coastal Breach",
        cost: cost(&[generic(6), u()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![undaunted()],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Nonland),
            body: Box::new(Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
            }),
        },
        ..Default::default()
    }
}

/// Nahiri, the Harbinger — {2}{R}{W} Legendary Planeswalker — Nahiri, 3 loyalty.
/// +2: you may discard a card; if you do, draw a card.
/// −2: exile target enchantment, tapped artifact, or tapped creature.
/// −8: search your library for an artifact or creature card and put it onto the
/// battlefield. (Printed haste + return-to-hand rider on the −8 is dropped.)
pub fn nahiri_the_harbinger() -> CardDefinition {
    CardDefinition {
        name: "Nahiri, the Harbinger",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nahiri],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::MayDo {
                    description: "Discard a card. If you do, draw a card.".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Enchantment
                            .or(SelectionRequirement::Artifact
                                .and(SelectionRequirement::Tapped))
                            .or(SelectionRequirement::Creature
                                .and(SelectionRequirement::Tapped)),
                    ),
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}
