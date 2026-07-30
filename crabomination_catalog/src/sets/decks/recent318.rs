//! Mirrodin (MRD) gap batch 3 — the Entwine modals, the untap-toll artifact
//! creatures, and the artifact-recursion utility. Tests in `recent_b/mrd`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind,
    EventScope, EventSpec, Keyword, Predicate, Selector, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{animate_land, etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

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

/// A two-mode Entwine spell.
fn entwine_spell(
    name: &'static str,
    mana: ManaCost,
    entwine: ManaCost,
    sorcery: bool,
    modes: Vec<Effect>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery { CardType::Sorcery } else { CardType::Instant }],
        keywords: vec![Keyword::Entwine(entwine)],
        effect: Effect::ChooseMode(modes),
        ..Default::default()
    }
}

/// "This creature doesn't untap during your untap step. At the beginning of
/// your upkeep, you may pay `toll`. If you do, untap it." (The Mirrodin
/// juggernaut-with-a-toll pattern.)
fn untap_toll(toll: ManaCost) -> (StaticAbility, TriggeredAbility) {
    (
        StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        },
        TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayPay {
                description: "Untap this creature".into(),
                mana_cost: toll,
                body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
                else_: None,
            },
        },
    )
}

// ── Entwine modals ──

/// Dream's Grip — tap or untap a permanent. Entwine {1}.
pub fn dreams_grip() -> CardDefinition {
    entwine_spell(
        "Dream's Grip",
        cost(&[u()]),
        cost(&[generic(1)]),
        false,
        vec![
            Effect::Tap { what: target_filtered(R::Permanent) },
            Effect::Untap { what: target_filtered(R::Permanent), up_to: None },
        ],
    )
}

/// Blinding Beam — tap two creatures, or lock a player's next untap step.
/// Entwine {1}.
pub fn blinding_beam() -> CardDefinition {
    entwine_spell(
        "Blinding Beam",
        cost(&[generic(2), w()]),
        cost(&[generic(1)]),
        false,
        vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            },
            Effect::CreaturesDontUntapNextUntapStep {
                who: Selector::Player(PlayerRef::Target(0)),
            },
        ],
    )
}

/// Roar of the Kha — team pump, or untap the team. Entwine {1}{W}.
pub fn roar_of_the_kha() -> CardDefinition {
    let mine = R::Creature.and(R::ControlledByYou);
    entwine_spell(
        "Roar of the Kha",
        cost(&[generic(1), w()]),
        cost(&[generic(1), w()]),
        false,
        vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(mine.clone()),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::EachPermanent(mine), up_to: None },
        ],
    )
}

/// Wail of the Nim — regenerate the team, or ping everything for 1.
/// Entwine {B}.
pub fn wail_of_the_nim() -> CardDefinition {
    entwine_spell(
        "Wail of the Nim",
        cost(&[generic(2), b()]),
        cost(&[b()]),
        false,
        vec![
            Effect::Regenerate {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            },
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::ONE,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                },
            ]),
        ],
    )
}

/// Journey of Discovery — two basics to hand, or two extra land drops.
/// Entwine {2}{G}.
pub fn journey_of_discovery() -> CardDefinition {
    entwine_spell(
        "Journey of Discovery",
        cost(&[generic(2), g()]),
        cost(&[generic(2), g()]),
        true,
        vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::HasSupertype(Supertype::Basic).and(R::Land),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(2),
            },
            Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::Const(2) },
        ],
    )
}

/// Incite War — force a player's creatures to attack, or hand your team first
/// strike. Entwine {2}.
pub fn incite_war() -> CardDefinition {
    entwine_spell(
        "Incite War",
        cost(&[generic(2), r()]),
        cost(&[generic(2)]),
        false,
        vec![
            Effect::GrantKeyword {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ],
    )
}

/// One Dozen Eyes — one 5/5 Beast, or five 1/1 Insects. Entwine {G}{G}{G}.
pub fn one_dozen_eyes() -> CardDefinition {
    let token = |name: &str, power: i32, toughness: i32, ct: CreatureType, count: i32| {
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(count),
            definition: TokenDefinition {
                name: name.into(),
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
                power,
                toughness,
                ..Default::default()
            },
        }
    };
    entwine_spell(
        "One Dozen Eyes",
        cost(&[generic(5), g()]),
        cost(&[g(), g(), g()]),
        true,
        vec![
            token("Beast", 5, 5, CreatureType::Beast, 1),
            token("Insect", 1, 1, CreatureType::Insect, 5),
        ],
    )
}

// ── Artifacts ──

/// Galvanic Key — flash; {3}, {T}: untap target artifact.
pub fn galvanic_key() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Artifact), up_to: None },
            ..Default::default()
        }],
        ..artifact("Galvanic Key", cost(&[generic(2)]))
    }
}

/// Leonin Bladetrap — flash; {2}, sacrifice: 2 damage to each attacking
/// creature without flying.
pub fn leonin_bladetrap() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::IsAttacking.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..artifact("Leonin Bladetrap", cost(&[generic(3)]))
    }
}

/// Lifespark Spellbomb — {G}, sacrifice: animate a land; {1}, sacrifice: draw.
pub fn lifespark_spellbomb() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                sac_cost: true,
                effect: animate_land(0, 3),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..artifact("Lifespark Spellbomb", cost(&[generic(1)]))
    }
}

/// Altar of Shadows — banks {B} off its charge counters and buys removal.
pub fn altar_of_shadows() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Black,
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                ),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature) },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Altar of Shadows", cost(&[generic(7)]))
    }
}

// ── Artifact creatures ──

/// Goblin Dirigible — a 4/4 flier that needs {4} each upkeep to swing again.
pub fn goblin_dirigible() -> CardDefinition {
    let (lock, toll) = untap_toll(cost(&[generic(4)]));
    CardDefinition {
        static_abilities: vec![lock],
        triggered_abilities: vec![toll],
        ..artifact_creature(
            "Goblin Dirigible",
            cost(&[generic(6)]),
            4,
            4,
            vec![CreatureType::Construct],
            vec![Keyword::Flying],
        )
    }
}

/// Rust Elemental — feed it an artifact each upkeep or it taps and bleeds you.
pub fn rust_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Artifact.and(R::OtherThanSource),
                }),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Artifact.and(R::OtherThanSource),
                }),
                else_: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::This },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(4) },
                ])),
            },
        }],
        ..artifact_creature(
            "Rust Elemental",
            cost(&[generic(4)]),
            4,
            4,
            vec![CreatureType::Elemental],
            vec![Keyword::Flying],
        )
    }
}

/// Dross Scorpion — any artifact creature dying may untap an artifact.
pub fn dross_scorpion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.and(R::Creature),
                },
            ),
            effect: Effect::MayDo {
                description: "Untap target artifact".into(),
                body: Box::new(Effect::Untap {
                    what: target_filtered(R::Artifact),
                    up_to: None,
                }),
            },
        }],
        ..artifact_creature("Dross Scorpion", cost(&[generic(4)]), 3, 1, vec![CreatureType::Scorpion], vec![])
    }
}

/// Bosh, Iron Golem — {3}{R}, sacrifice an artifact: fling its mana value.
pub fn bosh_iron_golem() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::SacrificedManaValue },
            ..Default::default()
        }],
        ..artifact_creature(
            "Bosh, Iron Golem",
            cost(&[generic(8)]),
            6,
            7,
            vec![CreatureType::Golem],
            vec![Keyword::Trample],
        )
    }
}

// ── Nonartifact creatures ──

/// Copperhoof Vorrac — grows with every untapped permanent across the table.
pub fn copperhoof_vorrac() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusOpponentsUntappedPermanents { base_p: 2, base_t: 2 }),
        ..creature(
            "Copperhoof Vorrac",
            cost(&[generic(3), g(), g()]),
            2,
            2,
            vec![CreatureType::Boar, CreatureType::Beast],
            vec![],
        )
    }
}

/// Loxodon Mender — {W}, {T}: regenerate target artifact.
pub fn loxodon_mender() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Regenerate { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..creature(
            "Loxodon Mender",
            cost(&[generic(5), w()]),
            3,
            3,
            vec![CreatureType::Elephant, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Lumengrid Sentinel — your artifacts entering may tap something down.
pub fn lumengrid_sentinel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::MayDo {
                description: "Tap target permanent".into(),
                body: Box::new(Effect::Tap { what: target_filtered(R::Permanent) }),
            },
        }],
        ..creature(
            "Lumengrid Sentinel",
            cost(&[generic(2), u()]),
            1,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![Keyword::Flying],
        )
    }
}

/// Flayed Nim — its combat damage to a creature drains that creature's
/// controller; {2}{B} regenerates.
pub fn flayed_nim() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Flayed Nim", cost(&[generic(3), b()]), 2, 2, vec![CreatureType::Skeleton], vec![])
    }
}

/// Wurmskin Forger — on entry, spread three +1/+1 counters over up to three
/// creatures.
pub fn wurmskin_forger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::Const(3),
            counter: CounterType::PlusOnePlusOne,
            filter: R::Creature,
            max_targets: 3,
        })],
        ..creature(
            "Wurmskin Forger",
            cost(&[generic(5), g(), g()]),
            2,
            2,
            vec![CreatureType::Elf, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Taj-Nar Swordsmith — on entry, may pay {X} to fetch an Equipment with mana
/// value X or less onto the battlefield.
pub fn taj_nar_swordsmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayPayX {
            description: "Search your library for an Equipment with mana value X or less".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)
                    .and(R::ManaValueAtMostXFromCost),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        })],
        ..creature(
            "Taj-Nar Swordsmith",
            cost(&[generic(3), w()]),
            2,
            3,
            vec![CreatureType::Cat, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Sphere of Purity — artifacts shed a point of damage aimed at you.
pub fn sphere_of_purity() -> CardDefinition {
    CardDefinition {
        name: "Sphere of Purity",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If an artifact would deal damage to you, prevent 1 of that damage.",
            effect: StaticEffect::ReduceDamageToControllerFromSource {
                filter: R::Artifact,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Blinkmoth Well — {T}: {C}; {2}, {T}: tap a noncreature artifact.
pub fn blinkmoth_well() -> CardDefinition {
    CardDefinition {
        name: "Blinkmoth Well",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Tap {
                    what: target_filtered(R::Artifact.and(R::Not(Box::new(R::Creature)))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tangleroot — every creature spell cast refunds its caster {G}.
pub fn tangleroot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::AddMana {
                who: PlayerRef::Triggerer,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
        }],
        ..artifact("Tangleroot", cost(&[generic(3)]))
    }
}

/// Relic Bane — the enchanted artifact bleeds its controller each upkeep.
pub fn relic_bane() -> CardDefinition {
    CardDefinition {
        name: "Relic Bane",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Artifact },
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::This))),
                    amount: Value::Const(2),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Dross Harvester — protection from white, bleeds you 4 each end step, and
/// pays 2 life back on every creature death.
pub fn dross_harvester() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(4) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            },
        ],
        ..creature(
            "Dross Harvester",
            cost(&[generic(1), b(), b()]),
            4,
            4,
            vec![CreatureType::Horror],
            vec![Keyword::Protection(Color::White)],
        )
    }
}
