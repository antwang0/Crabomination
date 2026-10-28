//! Star Trek Commander (TRC, 2026-11-13) — the We Are the Borg precon's new
//! cards (The Borg Queen). Assimilate is `Effect::Assimilate`; the "artifact
//! creatures you control have …" grants are layer-6 statics. Tests in
//! `tests/recent_b/trek_borg.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StationBand, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, station, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, SpendRestriction, b, cost, generic, u, w};

fn borg(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Borg], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legendary(mut c: CardDefinition, types: Vec<CreatureType>) -> CardDefinition {
    c.supertypes = vec![Supertype::Legendary];
    c.subtypes.creature_types = types;
    c
}

fn spacecraft(name: &'static str, mana: ManaCost, bands: Vec<StationBand>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: bands,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn artifact_creatures() -> R {
    R::Artifact.and(R::Creature).and(R::ControlledByYou)
}

fn your_artifact_creatures() -> Selector {
    Selector::EachPermanent(artifact_creatures())
}

fn borg_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Borg".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Borg], ..Default::default() },
        ..Default::default()
    })
}

fn borgs(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: borg_token() }
}

fn anthem(description: &'static str, p: i32, t: i32) -> StaticAbility {
    StaticAbility { description, effect: StaticEffect::PumpPT { applies_to: your_artifact_creatures(), power: p, toughness: t } }
}

fn grant_keyword(description: &'static str, keyword: Keyword) -> StaticAbility {
    StaticAbility { description, effect: StaticEffect::GrantKeyword { applies_to: your_artifact_creatures(), keyword } }
}

/// "Artifact creatures you control have 'Whenever this creature deals combat
/// damage to a player, [effect]'."
fn grant_on_connect(description: &'static str, effect: Effect) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantTriggeredAbility {
            filter: artifact_creatures(),
            ability: Box::new(TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect,
            }),
        },
    }
}

fn declare_target_opponent() -> Effect {
    Effect::TargetPlayerThen { filter: R::OpponentPlayer, then: Box::new(Effect::Noop) }
}

// ── the commander ────────────────────────────────────────────────────────────

/// The Borg Queen — {4}{W}{U}{B} legendary 4/4 Borg Noble artifact creature.
/// Artifact creatures you control get +2/+2. Entering or attacking, target
/// opponent reveals until a creature card, which you assimilate.
pub fn the_borg_queen() -> CardDefinition {
    let assimilate = || {
        Effect::Seq(vec![
            declare_target_opponent(),
            Effect::RevealUntilCreatureAssimilate { who: PlayerRef::Target(0), one_of_all: false },
        ])
    };
    CardDefinition {
        static_abilities: vec![anthem("Artifact creatures you control get +2/+2.", 2, 2)],
        triggered_abilities: vec![etb(assimilate()), on_attack(assimilate())],
        ..legendary(
            borg("The Borg Queen", cost(&[generic(4), w(), u(), b()]), 4, 4),
            vec![CreatureType::Borg, CreatureType::Noble],
        )
    }
}

// ── white ────────────────────────────────────────────────────────────────────

/// Locutus of Borg — {1}{W}{U}{B} legendary 3/3 Borg artifact creature,
/// lifelink. Artifact creatures you control have "connecting makes a 1/1
/// Borg".
pub fn locutus_of_borg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![grant_on_connect(
            "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, create a 1/1 colorless Borg artifact creature token.\"",
            borgs(Value::ONE),
        )],
        ..legendary(borg("Locutus of Borg", cost(&[generic(1), w(), u(), b()]), 3, 3), vec![CreatureType::Borg])
    }
}

/// Borg Incursion — {2}{W} instant. Two 1/1 Borg, then your artifact
/// creatures gain protection from the color of your choice until end of turn.
pub fn borg_incursion() -> CardDefinition {
    spell(
        "Borg Incursion",
        cost(&[generic(2), w()]),
        true,
        Effect::Seq(vec![
            borgs(Value::Const(2)),
            Effect::GrantProtectionFromChosenColor { what: your_artifact_creatures(), duration: Duration::EndOfTurn },
        ]),
    )
}

/// Emergency Regeneration — {1}{W} instant. Your artifacts gain hexproof and
/// indestructible until end of turn.
pub fn emergency_regeneration() -> CardDefinition {
    spell(
        "Emergency Regeneration",
        cost(&[generic(1), w()]),
        true,
        Effect::GrantKeywords {
            what: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
            keywords: vec![Keyword::Hexproof, Keyword::Indestructible],
            duration: Duration::EndOfTurn,
        },
    )
}

/// Join the Collective — {5}{W}{W} sorcery. Destroy all creatures, then a 1/1
/// Borg for each nonartifact creature destroyed.
pub fn join_the_collective() -> CardDefinition {
    spell(
        "Join the Collective",
        cost(&[generic(5), w(), w()]),
        false,
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
            borgs(Value::NonartifactCreaturesDestroyedThisResolution),
        ]),
    )
}

/// Maturation Chamber — {1}{W} artifact. Casting an artifact spell puts a
/// charge counter on it. {4}, {T}: a 1/1 Borg per charge counter; sorcery
/// speed.
pub fn maturation_chamber() -> CardDefinition {
    CardDefinition {
        name: "Maturation Chamber",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: borgs(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge }),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stratagem Emulator — {2}{W} 2/1 Borg artifact creature. Dying makes a 1/1
/// Borg per point of its power.
pub fn stratagem_emulator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(borgs(Value::PowerOf(Box::new(Selector::This))))],
        ..borg("Stratagem Emulator", cost(&[generic(2), w()]), 2, 1)
    }
}

/// Supplementation Node — {2}{W} 2/2 Borg artifact creature. Artifact
/// creatures you control have "connecting puts a +1/+1 counter on it".
pub fn supplementation_node() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant_on_connect(
            "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, put a +1/+1 counter on it.\"",
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        )],
        ..borg("Supplementation Node", cost(&[generic(2), w()]), 2, 2)
    }
}

/// Type-03 Borg Vessel — {3}{W} Spacecraft. Artifact spells cost {1} less.
/// Station; 7+: 5/5 flying, and casting a spell makes a 1/1 Borg.
pub fn type_03_borg_vessel() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Artifact, amount: 1 },
        }],
        ..spacecraft(
            "Type-03 Borg Vessel",
            cost(&[generic(3), w()]),
            vec![StationBand {
                min: 7,
                keywords: vec![Keyword::Flying],
                pt: Some((5, 5)),
                triggers: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                    effect: borgs(Value::ONE),
                }],
                ..Default::default()
            }],
        )
    }
}

/// Command Subadjunct — {W} 2/1 Borg artifact creature. Artifact creatures
/// you control have vigilance.
pub fn command_subadjunct() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant_keyword("Artifact creatures you control have vigilance.", Keyword::Vigilance)],
        ..borg("Command Subadjunct", cost(&[w()]), 2, 1)
    }
}

/// Conquest Mission — {2}{W} enchantment. A nontoken artifact of yours
/// entering adds a quest counter and makes a 1/1 Borg (two at six or more
/// quest counters).
pub fn conquest_mission() -> CardDefinition {
    CardDefinition {
        name: "Conquest Mission",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::IsToken.negate()) },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Quest, n: 6 },
                    then: Box::new(borgs(Value::Const(2))),
                    else_: Box::new(borgs(Value::ONE)),
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── blue ─────────────────────────────────────────────────────────────────────

/// Ambulatory Reactor — {3}{U} 2/5 Borg artifact creature. Artifact creatures
/// you control have "{T}: Add {C}; spend it only on artifact spells or
/// artifact sources' abilities".
pub fn ambulatory_reactor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have \"{T}: Add {C}. Spend this mana only to cast an artifact spell or activate an ability of an artifact source.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: your_artifact_creatures(),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Restricted(
                            Box::new(ManaPayload::Colorless(Value::ONE)),
                            SpendRestriction::ArtifactSpellsOrAbilities,
                        ),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..borg("Ambulatory Reactor", cost(&[generic(3), u()]), 2, 5)
    }
}

/// Borg Scout Sphere — {2}{U} Spacecraft. Entering draws a card. Station; 6+:
/// a 5/5, and artifact creatures you control have flying.
pub fn borg_scout_sphere() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..spacecraft(
            "Borg Scout Sphere",
            cost(&[generic(2), u()]),
            vec![StationBand {
                min: 6,
                pt: Some((5, 5)),
                statics: vec![StaticEffect::GrantKeyword { applies_to: your_artifact_creatures(), keyword: Keyword::Flying }],
                ..Default::default()
            }],
        )
    }
}

/// Co-Option Adjutant — {3}{U} 3/3 Borg artifact creature. Artifact creatures
/// you control have "connecting draws a card".
pub fn co_option_adjutant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant_on_connect(
            "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, draw a card.\"",
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        )],
        ..borg("Co-Option Adjutant", cost(&[generic(3), u()]), 3, 3)
    }
}

/// Gift of Perfection — {4}{U} sorcery. Each opponent reveals until a
/// creature card; you assimilate one of them; the rest go to the bottom.
pub fn gift_of_perfection() -> CardDefinition {
    spell(
        "Gift of Perfection",
        cost(&[generic(4), u()]),
        false,
        Effect::RevealUntilCreatureAssimilate { who: PlayerRef::EachOpponent, one_of_all: true },
    )
}

/// Optimization Annex — {1}{U} 1/3 Borg artifact creature. Charge counters
/// put on your artifacts come one extra.
pub fn optimization_annex() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If one or more charge counters would be put on an artifact you control, that many plus one charge counters are put on it instead.",
            effect: StaticEffect::ExtraCountersOfKindMatching {
                kind: CounterType::Charge,
                filter: R::Artifact.and(R::ControlledByYou),
            },
        }],
        ..borg("Optimization Annex", cost(&[generic(1), u()]), 1, 3)
    }
}

/// Transwarp Hub — {2}{U} Spacecraft. Entering mills five; up to two artifact
/// cards from among them go to your hand. Station; 3+: {T}: target creature
/// can't be blocked this turn.
pub fn transwarp_hub() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MillThenToHandN {
            amount: Value::Const(5),
            filter: R::Artifact,
            take: Value::Const(2),
            otherwise: None,
        })],
        ..spacecraft(
            "Transwarp Hub",
            cost(&[generic(2), u()]),
            vec![StationBand {
                min: 3,
                activated: vec![ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                }],
                ..Default::default()
            }],
        )
    }
}

/// Võx, Child of the Collective — {4}{U} legendary 3/3 Borg artifact
/// creature. Entering and at your first main phase, your next artifact
/// creature spell and next Spacecraft spell this turn have undaunted.
pub fn vox_child_of_the_collective() -> CardDefinition {
    let undaunted = || {
        Effect::Seq(vec![
            Effect::GrantNextMatchingSpellUndauntedThisTurn { filter: R::Artifact.and(R::Creature) },
            Effect::GrantNextMatchingSpellUndauntedThisTurn {
                filter: R::HasArtifactSubtype(ArtifactSubtype::Spacecraft),
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(undaunted()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
                effect: undaunted(),
            },
        ],
        ..legendary(borg("Võx, Child of the Collective", cost(&[generic(4), u()]), 3, 3), vec![CreatureType::Borg])
    }
}

// ── black ────────────────────────────────────────────────────────────────────

/// Assimilate the Wounded — {2}{B} instant. For each opponent, assimilate up
/// to one target creature card from their graveyard that was put there from
/// the battlefield this turn.
pub fn assimilate_the_wounded() -> CardDefinition {
    spell(
        "Assimilate the Wounded",
        cost(&[generic(2), b()]),
        true,
        Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 15,
                min_targets: 0,
                filter: R::Creature.and(R::InOpponentGraveyard).and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                effect: Box::new(Effect::Assimilate { what: Selector::Target(0) }),
            }),
        },
    )
}

/// Assimilation Assessor — {4}{B} 5/5 Borg artifact creature. Entering exiles
/// target opponent's graveyard; attacking assimilates a creature card exiled
/// with it.
pub fn assimilation_assessor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                declare_target_opponent(),
                Effect::Move {
                    what: Selector::CardsInZone { who: PlayerRef::Target(0), zone: crate::card::Zone::Graveyard, filter: R::Any },
                    to: ZoneDest::ExileWithSourceStamp,
                },
            ])),
            on_attack(Effect::Assimilate {
                what: Selector::one_of(Selector::EachMatching {
                    zone: crate::effect::ZoneRef::Exile,
                    filter: R::Creature.and(R::ExiledWithSource),
                }),
            }),
        ],
        ..borg("Assimilation Assessor", cost(&[generic(4), b()]), 5, 5)
    }
}

/// Battle of Wolf 359 — {3}{B}{B} sorcery. Nonartifact creatures get -X/-X
/// until end of turn, X your artifact creatures.
pub fn battle_of_wolf_359() -> CardDefinition {
    let x = || Value::Negate(Box::new(Value::CountOf(Box::new(your_artifact_creatures()))));
    spell(
        "Battle of Wolf 359",
        cost(&[generic(3), b(), b()]),
        false,
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::Artifact.negate())),
            power: x(),
            toughness: x(),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Extraction Integrant — {1}{B} 1/1 Borg artifact creature. Artifact
/// creatures you control have menace and "a creature blocking this gets
/// -1/-1 until end of turn".
pub fn extraction_integrant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            grant_keyword("Artifact creatures you control have menace.", Keyword::Menace),
            StaticAbility {
                description: "Artifact creatures you control have \"Whenever a creature blocks this creature, that creature gets -1/-1 until end of turn.\"",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: artifact_creatures(),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                        effect: Effect::PumpPT {
                            what: Selector::BlockingCreatures,
                            power: Value::Const(-1),
                            toughness: Value::Const(-1),
                            duration: Duration::EndOfTurn,
                        },
                    }),
                },
            },
        ],
        ..borg("Extraction Integrant", cost(&[generic(1), b()]), 1, 1)
    }
}

/// Reclamation Specialist — {2}{B} 2/2 Borg artifact creature. {1},
/// sacrifice an artifact creature: draw a card.
pub fn reclamation_specialist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact.and(R::Creature), 1)),
            sac_other_may_be_source: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..borg("Reclamation Specialist", cost(&[generic(2), b()]), 2, 2)
    }
}

/// Hive Mind Coprocessor — {4}{B} 4/4 Borg artifact creature. Artifact
/// creatures you control have "connecting, you may pay 1 life to draw".
pub fn hive_mind_coprocessor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant_on_connect(
            "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, you may pay 1 life. If you do, draw a card.\"",
            Effect::MayPayLife {
                description: "Pay 1 life to draw a card?".into(),
                amount: Value::ONE,
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        )],
        ..borg("Hive Mind Coprocessor", cost(&[generic(4), b()]), 4, 4)
    }
}

// ── multicolor ───────────────────────────────────────────────────────────────

/// The Best of Both Worlds — {4}{W}{U} Saga. I — assimilate target nonartifact
/// creature an opponent controls. II — artifact creatures you control get
/// +2/+2 until end of turn. III — the Borg it assimilated blinks back to its
/// owner.
pub fn the_best_of_both_worlds() -> CardDefinition {
    CardDefinition {
        name: "The Best of Both Worlds",
        cost: cost(&[generic(4), w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Assimilate {
                    what: target_filtered(R::Creature.and(R::Artifact.negate()).and(R::ControlledByOpponent)),
                },
            ),
            (
                2,
                Effect::PumpPT {
                    what: your_artifact_creatures(),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
            ),
            (3, Effect::ExileAndReturnToOwner { what: Selector::AssimilatedBySource }),
        ],
        ..Default::default()
    }
}

/// Collective Override — {2}{W}{U} instant. Choose one (both with a commander
/// out): a 1/1 Borg per attacking creature; or draw a card per attacking
/// creature.
pub fn collective_override() -> CardDefinition {
    let attackers = || Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::IsAttacking))));
    spell(
        "Collective Override",
        cost(&[generic(2), w(), u()]),
        true,
        Effect::ChooseModesCast {
            modes: vec![borgs(attackers()), Effect::Draw { who: Selector::You, amount: attackers() }],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
    )
    .widening_modes(crate::card::ModesWiden::If(Predicate::YouControlACommander))
}

/// Deliver from Chaos — {3}{W}{B} instant. Exile target artifact or creature;
/// a 1/1 Borg per point of its mana value.
pub fn deliver_from_chaos() -> CardDefinition {
    spell(
        "Deliver from Chaos",
        cost(&[generic(3), w(), b()]),
        true,
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Artifact.or(R::Creature)) },
            borgs(Value::LastExiledManaValue),
        ]),
    )
}

/// Hugh, First of Many — {1}{W}{U}{B} legendary 3/4 Borg artifact creature.
/// Artifact spells you cast have replicate {2}{U}.
pub fn hugh_first_of_many() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast have replicate {2}{U}.",
            effect: StaticEffect::YourSpellsHaveReplicateFor { filter: R::Artifact, cost: cost(&[generic(2), u()]) },
        }],
        ..legendary(borg("Hugh, First of Many", cost(&[generic(1), w(), u(), b()]), 3, 4), vec![CreatureType::Borg])
    }
}

/// Jurati, Cordial Queen — {1}{W}{U} legendary 3/3 Borg Scientist artifact
/// creature. One or more artifact creatures of yours connecting copies target
/// artifact token you control.
pub fn jurati_cordial_queen() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::Creature) },
                )
            },
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Artifact.and(R::IsToken).and(R::ControlledByYou)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
        }],
        ..legendary(
            borg("Jurati, Cordial Queen", cost(&[generic(1), w(), u()]), 3, 3),
            vec![CreatureType::Borg, CreatureType::Scientist],
        )
    }
}

/// One, Borg Evolved — {4}{W}{U}{B} legendary 3/3 Borg Artificer artifact
/// creature. Artifact creatures you control have flying, double strike,
/// vigilance, menace, lifelink and ward {2}.
pub fn one_borg_evolved() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            grant_keyword("Artifact creatures you control have flying.", Keyword::Flying),
            grant_keyword("Artifact creatures you control have double strike.", Keyword::DoubleStrike),
            grant_keyword("Artifact creatures you control have vigilance.", Keyword::Vigilance),
            grant_keyword("Artifact creatures you control have menace.", Keyword::Menace),
            grant_keyword("Artifact creatures you control have lifelink.", Keyword::Lifelink),
            grant_keyword(
                "Artifact creatures you control have ward {2}.",
                Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            ),
        ],
        ..legendary(
            borg("One, Borg Evolved", cost(&[generic(4), w(), u(), b()]), 3, 3),
            vec![CreatureType::Borg, CreatureType::Artificer],
        )
    }
}

/// Locutus, the One Who Speaks — {W}{B} legendary 1/1 Borg artifact creature.
/// Artifact creatures you control get +1/+1. {W}{B}, discard this card: 2
/// damage to target creature and you gain 2 life.
pub fn locutus_the_one_who_speaks() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![anthem("Artifact creatures you control get +1/+1.", 1, 1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), b()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..legendary(borg("Locutus, the One Who Speaks", cost(&[w(), b()]), 1, 1), vec![CreatureType::Borg])
    }
}

// ── colorless ────────────────────────────────────────────────────────────────

/// Class-4 Tactical Cube — {4} Spacecraft. Entering makes two 1/1 Borg.
/// Station; 7+: 4/4 flying, and artifact creatures you control get +1/+1.
pub fn class_4_tactical_cube() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(borgs(Value::Const(2)))],
        ..spacecraft(
            "Class-4 Tactical Cube",
            cost(&[generic(4)]),
            vec![StationBand {
                min: 7,
                keywords: vec![Keyword::Flying],
                pt: Some((4, 4)),
                statics: vec![StaticEffect::PumpPT { applies_to: your_artifact_creatures(), power: 1, toughness: 1 }],
                ..Default::default()
            }],
        )
    }
}

/// Queen's Diamond — {4} legendary Spacecraft. {T}: one mana of a color per
/// charge counter, spendable only on your commander. Station; 8+: 5/5 flying,
/// vigilance, haste.
pub fn queens_diamond() -> CardDefinition {
    let mut c = spacecraft(
        "Queen's Diamond",
        cost(&[generic(4)]),
        vec![StationBand {
            min: 8,
            keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
            pt: Some((5, 5)),
            ..Default::default()
        }],
    );
    c.supertypes = vec![Supertype::Legendary];
    c.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Restricted(
                Box::new(ManaPayload::AnyOneColor(Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                })),
                SpendRestriction::CommanderSpellsOnly,
            ),
        },
        ..Default::default()
    });
    c
}

/// Reconnaissance Partition — {4} 3/3 Borg artifact creature, entering with
/// two charge counters. {T}, remove a counter: fetch a basic land tapped, or
/// draw a card.
pub fn reconnaissance_partition() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Charge, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..borg("Reconnaissance Partition", cost(&[generic(4)]), 3, 3)
    }
}

/// Shore Leave Planet — Land — Planet, enters tapped. {T}: {U}. {4}{U}: until
/// end of turn it becomes a nonlegendary copy of target creature you control,
/// a Robot artifact too.
pub fn shore_leave_planet() -> CardDefinition {
    CardDefinition {
        name: "Shore Leave Planet",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(crate::mana::Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(4), u()]),
                effect: Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: target_filtered(R::Creature.and(R::ControlledByYou)),
                        duration: Duration::EndOfTurn,
                        non_legendary: true,
                    },
                    Effect::AddCardTypeIndefinitely { what: Selector::This, card_type: CardType::Artifact, until_eot: true },
                    Effect::AddCreatureTypes {
                        what: Selector::This,
                        creature_types: vec![CreatureType::Robot],
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
