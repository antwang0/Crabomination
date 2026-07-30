//! Mirrodin (MRD) gap batch 2 — the Nim, the Shards, the artifact-count
//! payoffs and the upkeep taxers. Tests in `recent_b/mrd`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EquipBonus,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, Predicate, Selector,
    SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{blocks, etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
use crate::game::TurnStep;

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
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
        subtypes: Subtypes { creature_types: types, ..Default::default() },
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

/// A bare Aura: enchant `filter`, no printed rider of its own.
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

/// "Affinity for artifacts" as a card-intrinsic cost reduction.
fn affinity_for_artifacts() -> StaticAbility {
    StaticAbility {
        description: "Affinity for artifacts.",
        effect: StaticEffect::SelfCostReducedPerPermanentMatching { filter: R::Artifact, per: 1 },
    }
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery { CardType::Sorcery } else { CardType::Instant }],
        effect,
        ..Default::default()
    }
}

/// The Mirrodin "Shard" cycle: one ability payable with `{3}` and one with a
/// single coloured pip, both tapping the artifact for the same effect.
fn shard(name: &'static str, pip: ManaCost, effect: Effect) -> CardDefinition {
    let ability = |mana: ManaCost| ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        effect: effect.clone(),
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ability(cost(&[generic(3)])), ability(pip)],
        ..artifact(name, cost(&[generic(3)]))
    }
}

/// "At the beginning of each player's upkeep, that player `effect`."
fn each_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
        effect,
    }
}

// ── Nim (artifact-count bodies) ──

/// Nim Lasher — {2}{B} 1/1 that grows with your artifact count.
pub fn nim_lasher() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 1, base_t: 1 }),
        ..creature("Nim Lasher", cost(&[generic(2), b()]), 1, 1, vec![CreatureType::Zombie], vec![])
    }
}

/// Nim Shrieker — a 0/1 flier that grows with your artifact count.
pub fn nim_shrieker() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 0, base_t: 1 }),
        ..creature(
            "Nim Shrieker",
            cost(&[generic(3), b()]),
            0,
            1,
            vec![CreatureType::Zombie],
            vec![Keyword::Flying],
        )
    }
}

/// Nim Shambler — grows with your artifacts; sacrifice a creature to regenerate.
pub fn nim_shambler() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 2, base_t: 1 }),
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Nim Shambler",
            cost(&[generic(2), b(), b()]),
            2,
            1,
            vec![CreatureType::Zombie],
            vec![],
        )
    }
}

/// Nim Replica — {2}{B}, sacrifice: target creature gets -1/-1.
pub fn nim_replica() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature("Nim Replica", cost(&[generic(3)]), 3, 1, vec![CreatureType::Zombie], vec![])
    }
}

// ── Myr and other artifact creatures ──

/// Myr Adapter — +1/+1 for each Equipment attached to it.
pub fn myr_adapter() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusPerAttachedEquipment {
            base_p: 1,
            base_t: 1,
            per: 1,
        }),
        ..artifact_creature("Myr Adapter", cost(&[generic(3)]), 1, 1, vec![CreatureType::Myr], vec![])
    }
}

/// Myr Mindservant — {2}, {T}: shuffle your library.
pub fn myr_mindservant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::ShuffleLibrary { who: PlayerRef::You },
            ..Default::default()
        }],
        ..artifact_creature("Myr Mindservant", cost(&[generic(1)]), 1, 1, vec![CreatureType::Myr], vec![])
    }
}

/// Malachite Golem — {1}{G}: gains trample until end of turn.
pub fn malachite_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature("Malachite Golem", cost(&[generic(6)]), 5, 3, vec![CreatureType::Golem], vec![])
    }
}

/// Needlebug — {4} 2/2 with flash and protection from artifacts.
pub fn needlebug() -> CardDefinition {
    artifact_creature(
        "Needlebug",
        cost(&[generic(4)]),
        2,
        2,
        vec![CreatureType::Insect],
        vec![Keyword::Flash, Keyword::ProtectionFromCardType(CardType::Artifact)],
    )
}

/// Duskworker — regenerates when blocked; {3} pumps its power.
pub fn duskworker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![blocks(Effect::Regenerate { what: Selector::This })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature("Duskworker", cost(&[generic(4)]), 2, 2, vec![CreatureType::Construct], vec![])
    }
}

/// Leveler — a 10/10 that costs you your whole library.
pub fn leveler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileLibraryExceptBottom {
            who: PlayerRef::You,
            keep: Value::ZERO,
        })],
        ..artifact_creature("Leveler", cost(&[generic(5)]), 10, 10, vec![CreatureType::Juggernaut], vec![])
    }
}

/// Nuisance Engine — {2}, {T}: mint a 0/1 Pest.
pub fn nuisance_engine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Pest".into(),
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Pest],
                        ..Default::default()
                    },
                    power: 0,
                    toughness: 1,
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..artifact("Nuisance Engine", cost(&[generic(3)]))
    }
}

/// Serum Tank — every artifact entering charges it; {3}, {T}, remove one: draw.
pub fn serum_tank() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Serum Tank", cost(&[generic(3)]))
    }
}

// ── The Shard cycle ──

/// Granite Shard — {3}, {T} or {R}, {T}: 1 damage to any target.
pub fn granite_shard() -> CardDefinition {
    shard(
        "Granite Shard",
        cost(&[r()]),
        Effect::DealDamage { to: target_any(), amount: Value::ONE },
    )
}

/// Heartwood Shard — {3}, {T} or {G}, {T}: target creature gains trample.
pub fn heartwood_shard() -> CardDefinition {
    shard(
        "Heartwood Shard",
        cost(&[g()]),
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Pearl Shard — {3}, {T} or {W}, {T}: prevent the next 2 damage to any target.
pub fn pearl_shard() -> CardDefinition {
    shard(
        "Pearl Shard",
        cost(&[w()]),
        Effect::PreventNextDamage { target: target_any(), amount: Value::Const(2) },
    )
}

/// Skeleton Shard — {3}, {T} or {B}, {T}: reanimate an artifact creature card
/// to your hand.
pub fn skeleton_shard() -> CardDefinition {
    shard(
        "Skeleton Shard",
        cost(&[b()]),
        Effect::Move {
            what: target_filtered(R::Artifact.and(R::Creature).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Scale of Chiss-Goria — flash, affinity for artifacts; {T}: +0/+1.
pub fn scale_of_chiss_goria() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![affinity_for_artifacts()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ZERO,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Scale of Chiss-Goria", cost(&[generic(3)]))
    }
}

/// Tooth of Chiss-Goria — flash, affinity for artifacts; {T}: +1/+0.
pub fn tooth_of_chiss_goria() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![affinity_for_artifacts()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Tooth of Chiss-Goria", cost(&[generic(3)]))
    }
}

// ── Nonartifact creatures ──

/// Lumengrid Warden — {1}{U} 1/3.
pub fn lumengrid_warden() -> CardDefinition {
    creature(
        "Lumengrid Warden",
        cost(&[generic(1), u()]),
        1,
        3,
        vec![CreatureType::Human, CreatureType::Wizard],
        vec![],
    )
}

/// Luminous Angel — a 4/4 flier that mints a Spirit each upkeep.
pub fn luminous_angel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayDo {
                description: "Create a 1/1 white Spirit with flying".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Spirit".into(),
                        colors: vec![Color::White],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Spirit],
                            ..Default::default()
                        },
                        power: 1,
                        toughness: 1,
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    },
                }),
            },
        }],
        ..creature(
            "Luminous Angel",
            cost(&[generic(4), w(), w(), w()]),
            4,
            4,
            vec![CreatureType::Angel],
            vec![Keyword::Flying],
        )
    }
}

/// Megatog — sacrifice an artifact for +3/+3 and trample.
pub fn megatog() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Megatog", cost(&[generic(4), r(), r()]), 3, 4, vec![CreatureType::Atog], vec![])
    }
}

/// Krark-Clan Grunt — sacrifice an artifact for +1/+0 and first strike.
pub fn krark_clan_grunt() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Krark-Clan Grunt",
            cost(&[generic(2), r()]),
            2,
            2,
            vec![CreatureType::Goblin, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Krark-Clan Shaman — sacrifice an artifact to sweep the ground for 1.
pub fn krark_clan_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasKeyword(
                    Keyword::Flying,
                ))))),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Krark-Clan Shaman",
            cost(&[r()]),
            1,
            1,
            vec![CreatureType::Goblin, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Molder Slug — every upkeep taxes its player an artifact.
pub fn molder_slug() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![each_upkeep(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::ActivePlayer),
            count: Value::ONE,
            filter: R::Artifact,
        })],
        ..creature(
            "Molder Slug",
            cost(&[generic(3), g(), g()]),
            4,
            6,
            vec![CreatureType::Slug, CreatureType::Beast],
            vec![],
        )
    }
}

/// Moriok Scavenger — on entry, may reanimate an artifact creature card to hand.
pub fn moriok_scavenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return an artifact creature card from your graveyard to your hand".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Artifact.and(R::Creature).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Moriok Scavenger",
            cost(&[generic(3), b()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Ogre Leadfoot — an artifact creature that blocks it dies.
pub fn ogre_leadfoot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::BlockingCreatures,
                    filter: R::Artifact,
                }),
            effect: Effect::Destroy { what: Selector::BlockingCreatures },
        }],
        ..creature("Ogre Leadfoot", cost(&[generic(4), r()]), 3, 3, vec![CreatureType::Ogre], vec![])
    }
}

/// Rustmouth Ogre — its combat damage may eat an artifact.
pub fn rustmouth_ogre() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Destroy an artifact that player controls".into(),
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::Artifact.and(R::ControlledByOpponent)),
                }),
            },
        }],
        ..creature("Rustmouth Ogre", cost(&[generic(4), r(), r()]), 5, 4, vec![CreatureType::Ogre], vec![])
    }
}

// ── Enchantments ──

/// Mass Hysteria — every creature has haste.
pub fn mass_hysteria() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All creatures have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Haste,
            },
        }],
        ..enchantment("Mass Hysteria", cost(&[r()]))
    }
}

/// Necrogen Mists — every upkeep costs its player a card.
pub fn necrogen_mists() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![each_upkeep(Effect::Discard {
            who: Selector::Player(PlayerRef::ActivePlayer),
            amount: Value::ONE,
            random: false,
        })],
        ..enchantment("Necrogen Mists", cost(&[generic(2), b()]))
    }
}

/// Contaminated Bond — the enchanted creature bleeds its controller when it
/// attacks or blocks.
pub fn contaminated_bond() -> CardDefinition {
    let drain = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::This))),
            amount: Value::Const(3),
        },
    };
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![drain(EventKind::Attacks), drain(EventKind::Blocks)],
            ..Default::default()
        }),
        ..aura("Contaminated Bond", cost(&[generic(1), b()]), R::Creature)
    }
}

/// Inertia Bubble — the enchanted artifact stops untapping.
pub fn inertia_bubble() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted artifact doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..aura("Inertia Bubble", cost(&[generic(1), u()]), R::Artifact)
    }
}

// ── Spells ──

/// Override — counter unless its controller pays {1} per artifact you control.
pub fn override_card() -> CardDefinition {
    spell(
        "Override",
        cost(&[generic(2), u()]),
        false,
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::CountOf(Box::new(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Artifact,
            }))),
        },
    )
}

/// Soul Nova — exile an attacking creature along with its Equipment.
pub fn soul_nova() -> CardDefinition {
    spell(
        "Soul Nova",
        cost(&[generic(3), w(), w()]),
        false,
        Effect::Seq(vec![
            Effect::Exile {
                what: Selector::AttachedToMe(Box::new(Selector::Target(0))),
            },
            Effect::Exile { what: target_filtered(R::Creature.and(R::IsAttacking)) },
        ]),
    )
}

/// Bloodscent — every creature that can block the target must.
pub fn bloodscent() -> CardDefinition {
    spell(
        "Bloodscent",
        cost(&[generic(3), g()]),
        false,
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::AllMustBlock,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Awe Strike — blank the next damage a creature would deal and bank the life.
pub fn awe_strike() -> CardDefinition {
    spell(
        "Awe Strike",
        cost(&[w()]),
        false,
        Effect::PreventAllDamageFromTargetThisTurn {
            what: target_filtered(R::Creature),
            gain_life: true,
            next_instance_only: true,
        },
    )
}

/// Mindstorm Crown — an upkeep draw, or a point of damage if you kept a card.
pub fn mindstorm_crown() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::You), Value::ZERO),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::ONE,
                }),
            },
        }],
        ..artifact("Mindstorm Crown", cost(&[generic(3)]))
    }
}

/// Goblin War Wagon — a 3/3 that needs {2} each upkeep to keep swinging.
pub fn goblin_war_wagon() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayPay {
                description: "Untap Goblin War Wagon".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
                else_: None,
            },
        }],
        ..artifact_creature(
            "Goblin War Wagon",
            cost(&[generic(4)]),
            3,
            3,
            vec![CreatureType::Juggernaut],
            vec![],
        )
    }
}

/// Neurok Familiar — on entry, mill-or-draw the top card by artifact-ness.
pub fn neurok_familiar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::RevealTopTakeMatchingRestToGraveyard {
            who: PlayerRef::You,
            count: Value::ONE,
            filter: R::Artifact,
        })],
        ..creature(
            "Neurok Familiar",
            cost(&[generic(1), u()]),
            1,
            1,
            vec![CreatureType::Bird],
            vec![Keyword::Flying],
        )
    }
}

