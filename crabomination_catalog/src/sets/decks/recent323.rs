//! Fifth Dawn (5DN) gap batch 2 — the artifact-matters creatures, the scry
//! commons and the mass-artifact spells. Tests in `recent_b/fdn5`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

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

/// "At the beginning of your upkeep, [effect]."
fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::Upkeep),
            EventScope::SelfSource,
        )
        .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
        effect,
    }
}

/// A cheap artifact card in your graveyard — the Auriok/Leonin recursion filter.
fn small_artifact_in_graveyard() -> R {
    R::InYourGraveyard
        .and(R::Artifact)
        .and(R::ManaValueAtMost(1))
}

// ── Creatures ──

/// Skyhunter Prowler — a flying, vigilant Cat Knight.
pub fn skyhunter_prowler() -> CardDefinition {
    creature(
        "Skyhunter Prowler",
        cost(&[generic(2), w()]),
        1,
        3,
        vec![CreatureType::Cat, CreatureType::Knight],
        vec![Keyword::Flying, Keyword::Vigilance],
    )
}

/// Plasma Elemental — 4/1 and unblockable.
pub fn plasma_elemental() -> CardDefinition {
    creature(
        "Plasma Elemental",
        cost(&[generic(5), u()]),
        4,
        1,
        vec![CreatureType::Elemental],
        vec![Keyword::Unblockable],
    )
}

/// Iron-Barb Hellion — hasty, and never on defense.
pub fn iron_barb_hellion() -> CardDefinition {
    creature(
        "Iron-Barb Hellion",
        cost(&[generic(5), r()]),
        5,
        4,
        vec![CreatureType::Hellion, CreatureType::Beast],
        vec![Keyword::Haste, Keyword::CantBlock],
    )
}

/// Tyrranax — trades power for toughness on demand.
pub fn tyrranax() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(-1),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Tyrranax",
            cost(&[generic(4), g(), g()]),
            5,
            4,
            vec![CreatureType::Dinosaur, CreatureType::Beast],
            vec![],
        )
    }
}

/// Loxodon Stalwart — a vigilant blocker that buys toughness.
pub fn loxodon_stalwart() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ZERO,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Loxodon Stalwart",
            cost(&[generic(3), w(), w()]),
            3,
            3,
            vec![CreatureType::Elephant, CreatureType::Soldier],
            vec![Keyword::Vigilance],
        )
    }
}

/// Loxodon Anchorite — taps to soak up two damage anywhere.
pub fn loxodon_anchorite() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Loxodon Anchorite",
            cost(&[generic(2), w(), w()]),
            2,
            3,
            vec![CreatureType::Elephant, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Nim Grotesque — grows with the artifacts you control.
pub fn nim_grotesque() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Gets +1/+0 for each artifact you control",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Artifact,
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..creature(
            "Nim Grotesque",
            cost(&[generic(6), b()]),
            3,
            6,
            vec![CreatureType::Zombie],
            vec![],
        )
    }
}

/// Relentless Rats — every other Rat of the same name pumps it.
pub fn relentless_rats() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Gets +1/+1 for each other Relentless Rats",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature
                        .and(R::HasName("Relentless Rats".into()))
                        .and(R::OtherThanSource),
                ))),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature(
            "Relentless Rats",
            cost(&[generic(1), b(), b()]),
            2,
            2,
            vec![CreatureType::Rat],
            vec![],
        )
    }
}

/// Vulshok Sorcerer — a hasty pinger.
pub fn vulshok_sorcerer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Vulshok Sorcerer",
            cost(&[generic(1), r(), r()]),
            1,
            1,
            vec![
                CreatureType::Human,
                CreatureType::Shaman,
                CreatureType::Sorcerer,
            ],
            vec![Keyword::Haste],
        )
    }
}

/// Viridian Scout — cashes itself in to shoot down a flier.
pub fn viridian_scout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Viridian Scout",
            cost(&[generic(3), g()]),
            1,
            2,
            vec![
                CreatureType::Elf,
                CreatureType::Warrior,
                CreatureType::Scout,
            ],
            vec![],
        )
    }
}

/// Viridian Lorebearers — pumps by the artifacts across the table.
pub fn viridian_lorebearers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Artifact.and(R::ControlledByOpponent),
                ))),
                toughness: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Artifact.and(R::ControlledByOpponent),
                ))),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Viridian Lorebearers",
            cost(&[generic(3), g()]),
            3,
            3,
            vec![CreatureType::Elf, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Tel-Jilad Lifebreather — Forests buy regeneration shields.
pub fn tel_jilad_lifebreather() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_other_filter: Some((R::HasLandType(LandType::Forest), 1)),
            effect: Effect::Regenerate {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Tel-Jilad Lifebreather",
            cost(&[generic(4), g()]),
            3,
            2,
            vec![CreatureType::Troll, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Krark-Clan Ogre — feeds artifacts to the attack.
pub fn krark_clan_ogre() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Krark-Clan Ogre",
            cost(&[generic(3), r(), r()]),
            3,
            3,
            vec![CreatureType::Ogre],
            vec![],
        )
    }
}

/// Krark-Clan Engineers — two artifacts for one across the table.
pub fn krark_clan_engineers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Artifact, 2)),
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..creature(
            "Krark-Clan Engineers",
            cost(&[generic(3), r()]),
            2,
            2,
            vec![CreatureType::Goblin, CreatureType::Artificer],
            vec![],
        )
    }
}

/// Leonin Squire — buys back a cheap artifact on the way in.
pub fn leonin_squire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: small_artifact_in_graveyard(),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Leonin Squire",
            cost(&[generic(1), w()]),
            2,
            2,
            vec![CreatureType::Cat, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Auriok Salvagers — the same recursion, repeatable.
pub fn auriok_salvagers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: small_artifact_in_graveyard(),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Auriok Salvagers",
            cost(&[generic(3), w()]),
            2,
            4,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Auriok Windwalker — moves Equipment around at instant speed.
pub fn auriok_windwalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Attach {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)
                        .and(R::ControlledByYou),
                },
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByYou),
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Auriok Windwalker",
            cost(&[generic(3), w()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![Keyword::Flying],
        )
    }
}

/// Moriok Rigger — every artifact that dies feeds it.
pub fn moriok_rigger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                },
            ),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Moriok Rigger?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Moriok Rigger",
            cost(&[generic(2), b()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Sylvok Explorer — taps for whatever the other side's lands make.
pub fn sylvok_explorer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColorOpponentCouldProduce,
            },
            ..Default::default()
        }],
        ..creature(
            "Sylvok Explorer",
            cost(&[generic(1), g()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Druid],
            vec![],
        )
    }
}

/// Joiner Adept — every land you control taps for any color.
pub fn joiner_adept() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::effect::shortcut::grant_tap_for_any_color(
            R::Land.and(R::ControlledByYou),
        )],
        ..creature(
            "Joiner Adept",
            cost(&[generic(1), g()]),
            2,
            1,
            vec![CreatureType::Elf, CreatureType::Druid],
            vec![],
        )
    }
}

/// Vedalken Mastermind — rebuys your own permanents.
pub fn vedalken_mastermind() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Permanent.and(R::ControlledByYou),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..creature(
            "Vedalken Mastermind",
            cost(&[u(), u()]),
            1,
            2,
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Tangle Asp — whatever it meets in combat dies when combat ends.
pub fn tangle_asp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::AtNextEndStep {
                    body: Box::new(Effect::Destroy {
                        what: Selector::BlockedAttacker,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::AtNextEndStep {
                    body: Box::new(Effect::Destroy {
                        what: Selector::BlockingCreatures,
                    }),
                },
            },
        ],
        ..creature(
            "Tangle Asp",
            cost(&[generic(1), g()]),
            1,
            2,
            vec![CreatureType::Snake],
            vec![],
        )
    }
}

/// Tornado Elemental — sweeps the skies, then ignores blockers.
pub fn tornado_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            amount: Value::Const(6),
        })],
        ..creature(
            "Tornado Elemental",
            cost(&[generic(5), g(), g()]),
            6,
            6,
            vec![CreatureType::Elemental],
            vec![Keyword::AssignsDamageAsThoughUnblocked],
        )
    }
}

/// Mephidross Vampire — your whole team turns Vampire and grows on damage.
pub fn mephidross_vampire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control are Vampires",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    creature_type: CreatureType::Vampire,
                },
            },
            StaticAbility {
                description: "Your creatures grow when they damage a creature",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::Creature.and(R::ControlledByYou),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::DealsCombatDamageToCreature,
                            EventScope::SelfSource,
                        ),
                        effect: Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                    }),
                },
            },
        ],
        ..creature(
            "Mephidross Vampire",
            cost(&[generic(4), b(), b()]),
            3,
            4,
            vec![CreatureType::Vampire],
            vec![Keyword::Flying],
        )
    }
}

/// Raksha Golden Cub — while equipped, every Cat you control goes wide.
pub fn raksha_golden_cub() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "While equipped, Cats you control get +2/+2",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasCreatureType(CreatureType::Cat)),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::DoubleStrike],
                condition: Predicate::SourceIsEquipped,
            },
        }],
        ..creature(
            "Raksha Golden Cub",
            cost(&[generic(5), w(), w()]),
            3,
            4,
            vec![CreatureType::Cat, CreatureType::Soldier],
            vec![Keyword::Vigilance],
        )
    }
}

/// Razorgrass Screen — a wall that has to throw itself in front of something.
pub fn razorgrass_screen() -> CardDefinition {
    artifact_creature(
        "Razorgrass Screen",
        cost(&[generic(1)]),
        2,
        1,
        vec![CreatureType::Wall],
        vec![Keyword::Defender, Keyword::MustBlock],
    )
}

/// Razormane Masticore — upkeep discard or bust, then a free shot each draw.
pub fn razormane_masticore() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            your_upkeep(Effect::MayDiscard {
                description: "Discard a card or sacrifice Razormane Masticore?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::SacrificeSource)),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::MayDo {
                    description: "Deal 3 damage to target creature?".into(),
                    body: Box::new(Effect::DealDamage {
                        to: target_filtered(R::Creature),
                        amount: Value::Const(3),
                    }),
                },
            },
        ],
        ..artifact_creature(
            "Razormane Masticore",
            cost(&[generic(5)]),
            5,
            5,
            vec![CreatureType::Masticore],
            vec![Keyword::FirstStrike],
        )
    }
}

/// Synod Centurion — a 4/4 that only stands while other artifacts do.
pub fn synod_centurion() -> CardDefinition {
    CardDefinition {
        sacrifice_when_you_control_no_other: Some(R::Artifact),
        ..artifact_creature(
            "Synod Centurion",
            cost(&[generic(4)]),
            4,
            4,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Silent Arbiter — one attacker and one blocker per combat, for everyone.
pub fn silent_arbiter() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "No more than one creature can attack each combat",
                effect: StaticEffect::MaxAttackersPerCombat(1),
            },
            StaticAbility {
                description: "No more than one creature can block each combat",
                effect: StaticEffect::MaxBlockersPerCombat(1),
            },
        ],
        ..artifact_creature(
            "Silent Arbiter",
            cost(&[generic(4)]),
            1,
            5,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Myr Quadropod — flips its stats to attack or block.
pub fn myr_quadropod() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::SwitchPT {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Myr Quadropod",
            cost(&[generic(4)]),
            1,
            4,
            vec![CreatureType::Myr],
            vec![],
        )
    }
}

/// Myr Servitor — every copy in every graveyard comes back together.
pub fn myr_servitor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_upkeep(Effect::Move {
            what: Selector::EachMatching {
                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                filter: R::HasName("Myr Servitor".into()),
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::OwnerOfMoved,
                tapped: false,
            },
        })],
        ..artifact_creature(
            "Myr Servitor",
            cost(&[generic(1)]),
            1,
            1,
            vec![CreatureType::Myr],
            vec![],
        )
    }
}

/// Battered Golem — stays tapped until the next artifact shows up.
pub fn battered_golem() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Doesn't untap during your untap step",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::This,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                },
            ),
            effect: Effect::MayDo {
                description: "Untap Battered Golem?".into(),
                body: Box::new(Effect::Untap {
                    what: Selector::This,
                    up_to: None,
                }),
            },
        }],
        ..artifact_creature(
            "Battered Golem",
            cost(&[generic(3)]),
            3,
            2,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Thermal Navigator — eats an artifact for a turn of flight.
pub fn thermal_navigator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Thermal Navigator",
            cost(&[generic(3)]),
            2,
            2,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Mycosynth Golem — affinity for itself, and for every artifact creature after.
pub fn mycosynth_golem() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::Artifact),
        static_abilities: vec![StaticAbility {
            description: "Your artifact creature spells have affinity for artifacts",
            effect: StaticEffect::GrantAffinityToSpells {
                spell_filter: R::Artifact.and(R::Creature),
                permanent_filter: R::Artifact,
            },
        }],
        ..artifact_creature(
            "Mycosynth Golem",
            cost(&[generic(11)]),
            4,
            5,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Hoverguard Sweepers — a big flier that clears two blockers out of the way.
pub fn hoverguard_sweepers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature(
            "Hoverguard Sweepers",
            cost(&[generic(6), u(), u()]),
            5,
            6,
            vec![CreatureType::Drone],
            vec![Keyword::Flying],
        )
    }
}

/// Lunar Avenger — spends its sunburst counters on a keyword a turn.
pub fn lunar_avenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sunburst],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..artifact_creature(
            "Lunar Avenger",
            cost(&[generic(7)]),
            2,
            2,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Spinal Parasite — a printed -1/-1 that eats counters off anything.
pub fn spinal_parasite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sunburst],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 2)),
            effect: Effect::RemoveAnyCounter {
                what: target_filtered(R::Permanent),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Spinal Parasite",
            cost(&[generic(5)]),
            -1,
            -1,
            vec![CreatureType::Insect],
            vec![],
        )
    }
}

/// Suncrusher — spends sunburst counters on removal, or on saving itself.
pub fn suncrusher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sunburst],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::Destroy {
                    what: target_filtered(R::Creature),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
                ..Default::default()
            },
        ],
        ..artifact_creature(
            "Suncrusher",
            cost(&[generic(9)]),
            3,
            3,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

// ── Spells ──

/// Stand Firm — a pump and a look at the top two.
pub fn stand_firm() -> CardDefinition {
    spell(
        "Stand Firm",
        cost(&[w()]),
        false,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Lose Hope — a shrink and a look at the top two.
pub fn lose_hope() -> CardDefinition {
    spell(
        "Lose Hope",
        cost(&[b()]),
        false,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Tel-Jilad Justice — artifact removal with a look at the top two.
pub fn tel_jilad_justice() -> CardDefinition {
    spell(
        "Tel-Jilad Justice",
        cost(&[generic(1), g()]),
        false,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Screaming Fury — a big swing and haste to use it.
pub fn screaming_fury() -> CardDefinition {
    spell(
        "Screaming Fury",
        cost(&[generic(2), r()]),
        true,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(5),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Vanquish — kills whatever just blocked.
pub fn vanquish() -> CardDefinition {
    spell(
        "Vanquish",
        cost(&[generic(2), w()]),
        false,
        Effect::Destroy {
            what: target_filtered(R::Creature.and(R::IsBlocking)),
        },
    )
}

/// Shattered Dreams — pulls an artifact straight out of their hand.
pub fn shattered_dreams() -> CardDefinition {
    spell(
        "Shattered Dreams",
        cost(&[b()]),
        true,
        Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Artifact,
        },
    )
}

/// Mana Geyser — every tapped land across the table pays out in red.
pub fn mana_geyser() -> CardDefinition {
    spell(
        "Mana Geyser",
        cost(&[generic(3), r(), r()]),
        true,
        Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(
                crate::mana::Color::Red,
                Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Land.and(R::Tapped).and(R::ControlledByOpponent),
                ))),
            ),
        },
    )
}

/// Granulate — sweeps every cheap artifact off the board.
pub fn granulate() -> CardDefinition {
    spell(
        "Granulate",
        cost(&[generic(2), r(), r()]),
        true,
        Effect::Destroy {
            what: Selector::EachPermanent(R::Artifact.and(R::Nonland).and(R::ManaValueAtMost(4))),
        },
    )
}

/// Roar of Reclamation — every artifact in every graveyard comes back.
pub fn roar_of_reclamation() -> CardDefinition {
    spell(
        "Roar of Reclamation",
        cost(&[generic(5), w(), w()]),
        true,
        Effect::Move {
            what: Selector::EachMatching {
                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                filter: R::Artifact,
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::OwnerOfMoved,
                tapped: false,
            },
        },
    )
}

/// Acquire — steal the best artifact out of an opponent's deck.
pub fn acquire() -> CardDefinition {
    spell(
        "Acquire",
        cost(&[generic(3), u(), u()]),
        true,
        Effect::SearchPickedBy {
            who: PlayerRef::Target(0),
            picker: PlayerRef::You,
            filter: R::Artifact,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
    )
}

/// Into Thin Air — an artifact bounce that gets cheaper with your own board.
pub fn into_thin_air() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::Artifact),
        ..spell(
            "Into Thin Air",
            cost(&[generic(5), u()]),
            false,
            Effect::Move {
                what: target_filtered(R::Artifact),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        )
    }
}

/// Rain of Rust — artifact or land, or both for the entwine cost.
pub fn rain_of_rust() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(3), r()]))],
        ..spell(
            "Rain of Rust",
            cost(&[generic(3), r(), r()]),
            false,
            Effect::ChooseMode(vec![
                Effect::Destroy {
                    what: target_filtered(R::Artifact),
                },
                Effect::Destroy {
                    what: target_filtered(R::Land),
                },
            ]),
        )
    }
}

/// Vicious Betrayal — the whole team's worth of power on one creature.
pub fn vicious_betrayal() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeAnyNumber {
            filter: R::Creature.and(R::ControlledByYou),
        }],
        ..spell(
            "Vicious Betrayal",
            cost(&[generic(3), b(), b()]),
            true,
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Times(Box::new(Value::Const(2)), Box::new(Value::SacrificedCount)),
                toughness: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::SacrificedCount),
                ),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Retaliate — kills everything that got through to you this turn.
pub fn retaliate() -> CardDefinition {
    spell(
        "Retaliate",
        cost(&[generic(2), w(), w()]),
        false,
        Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::DealtDamageToControllerThisTurn)),
        },
    )
}
