//! Darksteel completion batch — the artifact-matters commons/uncommons, the
//! "Echoing" same-name cycle, and the DST half of the Arcbound modular cycle.
//! Tests in `recent_b/dst`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R,
    Selector, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_dies, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, StaticAbility, StaticEffect, ZoneDest,
};
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w};

/// A plain creature body.
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

/// An artifact creature body.
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

/// An instant/sorcery body.
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

/// The DST modular cycle's shared shell: an artifact creature that enters with
/// `n` +1/+1 counters and carries Modular N (CR 702.43).
fn modular_body(
    name: &'static str,
    mv: u32,
    n: u32,
    ct: CreatureType,
    mut keywords: Vec<Keyword>,
) -> CardDefinition {
    keywords.push(Keyword::Modular(n));
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(n as i32))),
        triggered_abilities: vec![crate::effect::shortcut::modular_dies()],
        ..artifact_creature(name, cost(&[generic(mv)]), 0, 0, vec![ct], keywords)
    }
}

// ── Modular (CR 702.43) — the Darksteel five ──

/// Arcbound Crusher — Trample, Modular 1. Whenever another artifact enters,
/// put a +1/+1 counter on this creature.
pub fn arcbound_crusher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact.and(R::OtherThanSource),
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            crate::effect::shortcut::modular_dies(),
        ],
        ..modular_body(
            "Arcbound Crusher",
            4,
            1,
            CreatureType::Juggernaut,
            vec![Keyword::Trample],
        )
    }
}

/// Arcbound Fiend — Fear, Modular 3. At the beginning of your upkeep, you may
/// move a +1/+1 counter from target creature onto this creature.
pub fn arcbound_fiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Move a +1/+1 counter onto Arcbound Fiend".into(),
                    body: Box::new(Effect::MoveCounter {
                        from: target_filtered(R::Creature),
                        to: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            },
            crate::effect::shortcut::modular_dies(),
        ],
        ..modular_body(
            "Arcbound Fiend",
            6,
            3,
            CreatureType::Horror,
            vec![Keyword::Fear],
        )
    }
}

/// Arcbound Lancer — First strike, Modular 4.
pub fn arcbound_lancer() -> CardDefinition {
    modular_body(
        "Arcbound Lancer",
        7,
        4,
        CreatureType::Beast,
        vec![Keyword::FirstStrike],
    )
}

/// Arcbound Overseer — Modular 6. At the beginning of your upkeep, put a +1/+1
/// counter on each creature you control with modular.
pub fn arcbound_overseer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::HasModular),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            crate::effect::shortcut::modular_dies(),
        ],
        ..modular_body("Arcbound Overseer", 8, 6, CreatureType::Golem, vec![])
    }
}

/// Arcbound Reclaimer — Modular 2. Remove a +1/+1 counter from this creature:
/// Put target artifact card from your graveyard on top of your library.
pub fn arcbound_reclaimer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::Move {
                what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: crate::effect::LibraryPosition::Top,
                },
            },
            ..Default::default()
        }],
        ..modular_body("Arcbound Reclaimer", 4, 2, CreatureType::Golem, vec![])
    }
}

// ── Artifact-matters creatures ──

/// Emissary of Despair — Flying. Combat damage to a player drains 1 per
/// artifact that player controls.
pub fn emissary_of_despair() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::count(Selector::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: R::Artifact,
                }),
            },
        }],
        ..creature(
            "Emissary of Despair",
            cost(&[generic(1), b(), b()]),
            2,
            1,
            vec![CreatureType::Spirit],
            vec![Keyword::Flying],
        )
    }
}

/// Emissary of Hope — Flying. Combat damage to a player gains you 1 life per
/// artifact that player controls.
pub fn emissary_of_hope() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: R::Artifact,
                }),
            },
        }],
        ..creature(
            "Emissary of Hope",
            cost(&[generic(1), w(), w()]),
            2,
            1,
            vec![CreatureType::Spirit],
            vec![Keyword::Flying],
        )
    }
}

/// Mephitic Ooze — gets +1/+0 for each artifact you control; its combat damage
/// to a creature destroys that creature, and it can't be regenerated.
pub fn mephitic_ooze() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Mephitic Ooze gets +1/+0 for each artifact you control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::count(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                }),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToCreature,
                EventScope::SelfSource,
            ),
            effect: Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn {
                    what: Selector::Target(0),
                },
                Effect::Destroy {
                    what: Selector::Target(0),
                },
            ]),
        }],
        ..creature(
            "Mephitic Ooze",
            cost(&[generic(4), b()]),
            0,
            5,
            vec![CreatureType::Ooze],
            vec![],
        )
    }
}

/// Nim Abomination — at the beginning of your end step, if it's untapped, you
/// lose 3 life.
pub fn nim_abomination() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::Not(Box::new(R::Tapped)),
            }),
            effect: Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        }],
        ..creature(
            "Nim Abomination",
            cost(&[generic(2), b()]),
            3,
            4,
            vec![CreatureType::Zombie],
            vec![],
        )
    }
}

/// Fangren Firstborn — whenever it attacks, put a +1/+1 counter on each
/// attacking creature.
pub fn fangren_firstborn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::AddCounter {
            what: Selector::EachPermanent(R::IsAttacking),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Fangren Firstborn",
            cost(&[generic(1), g(), g(), g()]),
            4,
            2,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Karstoderm — enters with five +1/+1 counters; loses one whenever an
/// artifact enters.
pub fn karstoderm() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                },
            ),
            effect: Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Karstoderm",
            cost(&[generic(2), g(), g()]),
            0,
            0,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Hoverguard Observer — Flying; can block only creatures with flying.
pub fn hoverguard_observer() -> CardDefinition {
    creature(
        "Hoverguard Observer",
        cost(&[generic(2), u(), u()]),
        3,
        3,
        vec![CreatureType::Drone],
        vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
    )
}

/// Infested Roothold — Defender, protection from artifacts. Whenever an
/// opponent casts an artifact spell, you may create a 1/1 green Insect.
pub fn infested_roothold() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                },
            ),
            effect: Effect::MayDo {
                description: "Create a 1/1 green Insect".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
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
                    },
                }),
            },
        }],
        ..creature(
            "Infested Roothold",
            cost(&[generic(4), g()]),
            0,
            3,
            vec![CreatureType::Wall],
            vec![
                Keyword::Defender,
                Keyword::ProtectionFromCardType(CardType::Artifact),
            ],
        )
    }
}

/// Tel-Jilad Outrider — protection from artifacts.
pub fn tel_jilad_outrider() -> CardDefinition {
    creature(
        "Tel-Jilad Outrider",
        cost(&[generic(3), g()]),
        3,
        1,
        vec![CreatureType::Elf, CreatureType::Warrior],
        vec![Keyword::ProtectionFromCardType(CardType::Artifact)],
    )
}

/// Tel-Jilad Wolf — gets +3/+3 whenever it becomes blocked by an artifact
/// creature.
pub fn tel_jilad_wolf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::BlockingCreatures,
                    filter: R::Artifact.and(R::Creature),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Tel-Jilad Wolf",
            cost(&[generic(2), g()]),
            2,
            2,
            vec![CreatureType::Wolf],
            vec![],
        )
    }
}

/// Tangle Spider — Flash, reach.
pub fn tangle_spider() -> CardDefinition {
    creature(
        "Tangle Spider",
        cost(&[generic(4), g(), g()]),
        3,
        4,
        vec![CreatureType::Spider],
        vec![Keyword::Flash, Keyword::Reach],
    )
}

/// Tanglewalker — your creatures can't be blocked as long as the defending
/// player controls an artifact land.
pub fn tanglewalker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control can't be blocked if defending player controls an artifact land.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Creature.and(R::ControlledByYou),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::ControlledBy {
                        who: PlayerRef::EachOpponent,
                        filter: R::Artifact.and(R::Land),
                    },
                    n: Value::ONE,
                },
                all_players: false,
            },
        }],
        ..creature(
            "Tanglewalker",
            cost(&[generic(2), g()]),
            2,
            2,
            vec![CreatureType::Dryad],
            vec![],
        )
    }
}

/// Grimclaw Bats — Flying. {B}, Pay 1 life: this creature gets +1/+1 until end
/// of turn.
pub fn grimclaw_bats() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 1,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Grimclaw Bats",
            cost(&[generic(1), b()]),
            1,
            1,
            vec![CreatureType::Bat],
            vec![Keyword::Flying],
        )
    }
}

/// Loxodon Mystic — {W}, {T}: Tap target creature.
pub fn loxodon_mystic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Loxodon Mystic",
            cost(&[generic(3), w(), w()]),
            3,
            3,
            vec![CreatureType::Elephant, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Leonin Battlemage — {T}: target creature gets +1/+1 until end of turn.
/// Whenever you cast a spell, you may untap this creature.
pub fn leonin_battlemage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::on_cast(Effect::MayDo {
            description: "Untap Leonin Battlemage".into(),
            body: Box::new(Effect::Untap {
                what: Selector::This,
                up_to: None,
            }),
        })],
        ..creature(
            "Leonin Battlemage",
            cost(&[generic(3), w()]),
            2,
            3,
            vec![CreatureType::Cat, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Neurok Prodigy — Flying. Discard an artifact card: return this creature to
/// its owner's hand.
pub fn neurok_prodigy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Artifact, 1)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Neurok Prodigy",
            cost(&[generic(2), u()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![Keyword::Flying],
        )
    }
}

/// Vedalken Engineer — {T}: add two mana of any one color, spendable only on
/// artifact spells and artifacts' abilities.
pub fn vedalken_engineer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::Const(2))),
                    SpendRestriction::ArtifactOnly,
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Vedalken Engineer",
            cost(&[generic(1), u()]),
            1,
            1,
            vec![CreatureType::Vedalken, CreatureType::Artificer],
            vec![],
        )
    }
}

/// Viridian Acolyte — {1}, {T}: add one mana of any color.
pub fn viridian_acolyte() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: crate::effect::shortcut::add_any_one_color(1),
            ..Default::default()
        }],
        ..creature(
            "Viridian Acolyte",
            cost(&[g()]),
            1,
            1,
            vec![CreatureType::Elf, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Krark-Clan Stoker — {T}, Sacrifice an artifact: Add {R}{R}.
pub fn krark_clan_stoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: crate::effect::shortcut::add_mana(vec![Color::Red, Color::Red]),
            ..Default::default()
        }],
        ..creature(
            "Krark-Clan Stoker",
            cost(&[generic(2), r()]),
            2,
            2,
            vec![CreatureType::Goblin, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Slobad, Goblin Tinkerer — Sacrifice an artifact: target artifact gains
/// indestructible until end of turn.
pub fn slobad_goblin_tinkerer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Artifact),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Slobad, Goblin Tinkerer",
            cost(&[generic(1), r()]),
            1,
            2,
            vec![CreatureType::Goblin, CreatureType::Artificer],
            vec![],
        )
    }
}

/// Vulshok War Boar — when it enters, sacrifice it unless you sacrifice an
/// artifact.
pub fn vulshok_war_boar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::MaySacrifice {
            description: "Sacrifice an artifact to keep Vulshok War Boar?".into(),
            filter: R::Artifact,
            count: Value::ONE,
            then: Box::new(Effect::Noop),
            else_: Some(Box::new(Effect::SacrificeSource)),
        })],
        ..creature(
            "Vulshok War Boar",
            cost(&[generic(2), r(), r()]),
            5,
            5,
            vec![CreatureType::Boar, CreatureType::Beast],
            vec![],
        )
    }
}

// ── Affinity golems ──

/// Spire Golem — Affinity for Islands. Flying.
pub fn spire_golem() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::HasLandType(LandType::Island).and(R::ControlledByYou)),
        ..artifact_creature(
            "Spire Golem",
            cost(&[generic(6)]),
            2,
            4,
            vec![CreatureType::Golem],
            vec![Keyword::Flying],
        )
    }
}

/// Tangle Golem — Affinity for Forests.
pub fn tangle_golem() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::HasLandType(LandType::Forest).and(R::ControlledByYou)),
        ..artifact_creature(
            "Tangle Golem",
            cost(&[generic(7)]),
            5,
            4,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Oxidda Golem — Affinity for Mountains. Haste.
pub fn oxidda_golem() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::HasLandType(LandType::Mountain).and(R::ControlledByYou)),
        ..artifact_creature(
            "Oxidda Golem",
            cost(&[generic(6)]),
            3,
            2,
            vec![CreatureType::Golem],
            vec![Keyword::Haste],
        )
    }
}

// ── Artifacts ──

/// Myr Moonvessel — when it dies, add {C}.
pub fn myr_moonvessel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(crate::effect::shortcut::add_colorless(1))],
        ..artifact_creature(
            "Myr Moonvessel",
            cost(&[generic(1)]),
            1,
            1,
            vec![CreatureType::Myr],
            vec![],
        )
    }
}

/// Voltaic Construct — {2}: Untap target artifact creature.
pub fn voltaic_construct() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Untap {
                what: target_filtered(R::Artifact.and(R::Creature)),
                up_to: None,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Voltaic Construct",
            cost(&[generic(4)]),
            2,
            2,
            vec![CreatureType::Golem, CreatureType::Construct],
            vec![],
        )
    }
}

/// Myr Landshaper — {T}: target land becomes an artifact in addition to its
/// other types until end of turn.
pub fn myr_landshaper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCardTypeIndefinitely {
                what: target_filtered(R::Land),
                card_type: CardType::Artifact,
                until_eot: true,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Myr Landshaper",
            cost(&[generic(3)]),
            1,
            1,
            vec![CreatureType::Myr],
            vec![],
        )
    }
}

/// Myr Matrix — indestructible; Myr get +1/+1; {5}: create a 1/1 Myr.
pub fn myr_matrix() -> CardDefinition {
    CardDefinition {
        name: "Myr Matrix",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Indestructible],
        static_abilities: vec![StaticAbility {
            description: "Myr creatures get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Myr),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Myr".into(),
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Myr],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spawning Pit — Sacrifice a creature: put a charge counter on this.
/// {1}, Remove two charge counters: create a 2/2 colorless Spawn.
pub fn spawning_pit() -> CardDefinition {
    CardDefinition {
        name: "Spawning Pit",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_cost: Some((CounterType::Charge, 2)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Spawn".into(),
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Eldrazi],
                            ..Default::default()
                        },
                        power: 2,
                        toughness: 2,
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Eater of Days — Flying, trample. When it enters, you skip your next two
/// turns.
pub fn eater_of_days() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::SkipTurns {
            who: PlayerRef::You,
            count: Value::Const(2),
        })],
        ..artifact_creature(
            "Eater of Days",
            cost(&[generic(4)]),
            9,
            8,
            vec![CreatureType::Leviathan],
            vec![Keyword::Flying, Keyword::Trample],
        )
    }
}

/// Darksteel Reactor — indestructible. Upkeep: you may add a charge counter.
/// With twenty or more charge counters, you win the game.
pub fn darksteel_reactor() -> CardDefinition {
    CardDefinition {
        name: "Darksteel Reactor",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Put a charge counter on Darksteel Reactor".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    }),
                },
            },
            // CR 603.8 state trigger, modeled off the counter-placement event.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::Charge),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                    Value::Const(20),
                )),
                effect: Effect::WinGame {
                    who: PlayerRef::You,
                },
            },
        ],
        ..Default::default()
    }
}

/// Leonin Bola — equipped creature has "{T}, Unattach: Tap target creature."
pub fn leonin_bola() -> CardDefinition {
    CardDefinition {
        name: "Leonin Bola",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        // The granted line rides the Equipment (the tap cost taps its host).
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::IsHostOfSource),
            effect: Effect::Seq(vec![
                Effect::Unattach {
                    what: Selector::This,
                },
                Effect::Tap {
                    what: target_filtered(R::Creature),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Surestrike Trident — equipped creature has first strike and "{T}, Unattach:
/// This creature deals damage equal to its power to target player or
/// planeswalker."
pub fn surestrike_trident() -> CardDefinition {
    CardDefinition {
        name: "Surestrike Trident",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::IsHostOfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::PowerOf(Box::new(Selector::AttachedTo(Box::new(
                        Selector::This,
                    )))),
                },
                Effect::Unattach {
                    what: Selector::This,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nemesis Mask — all creatures able to block equipped creature do so.
pub fn nemesis_mask() -> CardDefinition {
    CardDefinition {
        name: "Nemesis Mask",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::AllMustBlock],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Spincrusher — whenever it blocks, put a +1/+1 counter on it. Remove a
/// +1/+1 counter: it can't be blocked this turn.
pub fn spincrusher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::blocks(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Spincrusher",
            cost(&[generic(2)]),
            0,
            2,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Mirrodin's Core — {T}: Add {C}. {T}: Put a charge counter on this land.
/// {T}, Remove a charge counter: Add one mana of any color.
pub fn mirrodins_core() -> CardDefinition {
    CardDefinition {
        name: "Mirrodin's Core",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::add_colorless(1),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: crate::effect::shortcut::add_any_one_color(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Spells ──

/// Nourish — you gain 6 life.
pub fn nourish() -> CardDefinition {
    spell(
        "Nourish",
        cost(&[g(), g()]),
        false,
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(6),
        },
    )
}

/// Essence Drain — 3 damage to any target and you gain 3 life.
pub fn essence_drain() -> CardDefinition {
    spell(
        "Essence Drain",
        cost(&[generic(4), b()]),
        true,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(3),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]),
    )
}

/// Metal Fatigue — tap all artifacts.
pub fn metal_fatigue() -> CardDefinition {
    spell(
        "Metal Fatigue",
        cost(&[generic(2), w()]),
        false,
        Effect::Tap {
            what: Selector::EachPermanent(R::Artifact),
        },
    )
}

/// Magnetic Flux — artifact creatures you control gain flying until end of turn.
pub fn magnetic_flux() -> CardDefinition {
    spell(
        "Magnetic Flux",
        cost(&[generic(2), u()]),
        false,
        Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Hunger of the Nim — target creature gets +1/+0 for each artifact you control.
pub fn hunger_of_the_nim() -> CardDefinition {
    spell(
        "Hunger of the Nim",
        cost(&[generic(1), b()]),
        true,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::count(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Artifact,
            }),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Stand Together — two +1/+1 counters on target creature and two on another
/// target creature.
pub fn stand_together() -> CardDefinition {
    spell(
        "Stand Together",
        cost(&[generic(3), g(), g()]),
        false,
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature,
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Unforge — destroy target Equipment; if it was attached, deal 2 damage to
/// that creature.
pub fn unforge() -> CardDefinition {
    spell(
        "Unforge",
        cost(&[generic(2), r()]),
        false,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::AttachedTo(Box::new(Selector::Target(0))),
                amount: Value::Const(2),
            },
            Effect::Destroy {
                what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
            },
        ]),
    )
}

/// Soulscour — destroy all nonartifact permanents.
pub fn soulscour() -> CardDefinition {
    spell(
        "Soulscour",
        cost(&[generic(7), w(), w(), w()]),
        true,
        Effect::Destroy {
            what: Selector::EachPermanent(R::Not(Box::new(R::Artifact))),
        },
    )
}

/// Flamebreak — 3 damage to each creature without flying and each player;
/// those creatures can't be regenerated this turn.
pub fn flamebreak() -> CardDefinition {
    let non_flyers = R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))));
    spell(
        "Flamebreak",
        cost(&[r(), r(), r()]),
        true,
        Effect::Seq(vec![
            Effect::CantBeRegeneratedThisTurn {
                what: Selector::EachPermanent(non_flyers.clone()),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(non_flyers),
                amount: Value::Const(3),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(3),
            },
        ]),
    )
}

/// Last Word — this spell can't be countered. Counter target spell.
pub fn last_word() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        ..spell(
            "Last Word",
            cost(&[generic(2), u(), u()]),
            false,
            crate::effect::shortcut::counter_target_spell(),
        )
    }
}

/// Vex — counter target spell; that spell's controller may draw a card.
pub fn vex() -> CardDefinition {
    spell(
        "Vex",
        cost(&[generic(2), u()]),
        false,
        Effect::Seq(vec![
            crate::effect::shortcut::counter_target_spell(),
            Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ONE,
                }),
            },
        ]),
    )
}

/// Machinate — look at the top X cards of your library, where X is the number
/// of artifacts you control; take one and bottom the rest.
pub fn machinate() -> CardDefinition {
    spell(
        "Machinate",
        cost(&[generic(1), u(), u()]),
        false,
        Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::count(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Artifact,
            }),
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            rest_bottom_random: false,
            picked_lands_to_battlefield: false,
            rest_to_exile: false,
        },
    )
}

/// The "Echoing" cycle: hit a target and every other permanent sharing its
/// name. `body` is built from the slot-0 selector this hands it.
fn echoing(
    name: &'static str,
    mana: ManaCost,
    sorcery: bool,
    filter: R,
    body: impl FnOnce(Selector) -> Effect,
) -> CardDefinition {
    let group = Selector::SharingNameWith(Box::new(Selector::TargetFiltered { slot: 0, filter }));
    spell(name, mana, sorcery, body(group))
}

/// Echoing Calm — destroy target enchantment and every other enchantment with
/// that name.
pub fn echoing_calm() -> CardDefinition {
    echoing(
        "Echoing Calm",
        cost(&[generic(1), w()]),
        false,
        R::Enchantment,
        |what| Effect::Destroy { what },
    )
}

/// Echoing Ruin — destroy target artifact and every other artifact with that name.
pub fn echoing_ruin() -> CardDefinition {
    echoing(
        "Echoing Ruin",
        cost(&[generic(1), r()]),
        true,
        R::Artifact,
        |what| Effect::Destroy { what },
    )
}

/// Echoing Courage — target creature and every other creature with that name
/// get +2/+2 until end of turn.
pub fn echoing_courage() -> CardDefinition {
    echoing(
        "Echoing Courage",
        cost(&[generic(1), g()]),
        false,
        R::Creature,
        |what| Effect::PumpPT {
            what,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Echoing Decay — target creature and every other creature with that name get
/// -2/-2 until end of turn.
pub fn echoing_decay() -> CardDefinition {
    echoing(
        "Echoing Decay",
        cost(&[generic(1), b()]),
        false,
        R::Creature,
        |what| Effect::PumpPT {
            what,
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Aether Snap — remove all counters from all permanents and exile all tokens.
pub fn aether_snap() -> CardDefinition {
    spell(
        "Aether Snap",
        cost(&[generic(3), b(), b()]),
        true,
        Effect::Seq(vec![
            Effect::RemoveAllCounters {
                what: Selector::EachPermanent(R::Any),
            },
            Effect::Exile {
                what: Selector::EachPermanent(R::IsToken),
            },
        ]),
    )
}
