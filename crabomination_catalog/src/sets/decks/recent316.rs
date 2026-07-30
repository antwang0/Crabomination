//! Mirrodin (MRD) gap batch 1 — the Myr cycle, the Slith cycle, the golems,
//! the Tel-Jilad elves, the Equipment, and the easy spells. Tests in
//! `recent_b/mrd`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{blocks, etb, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, PlayerStaticTarget, StaticEffect,
    ZoneDest,
};
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

/// A Myr mana creature: 1/1 artifact creature, `{T}`: add one `color`.
fn myr(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![color]),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            name,
            cost(&[generic(2)]),
            1,
            1,
            vec![CreatureType::Myr],
            vec![],
        )
    }
}

/// `{cost}`: regenerate this creature.
fn regenerate_self(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::Regenerate {
            what: Selector::This,
        },
        ..Default::default()
    }
}

/// The Slith trigger: combat damage to a player grows it permanently.
fn slith_growth() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
    }
}

/// `{cost}`: this creature gains `keyword` until end of turn.
fn self_keyword_pump(mana: ManaCost, keyword: Keyword) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Myr ──

/// Alpha Myr — {2} 2/1 artifact creature.
pub fn alpha_myr() -> CardDefinition {
    artifact_creature(
        "Alpha Myr",
        cost(&[generic(2)]),
        2,
        1,
        vec![CreatureType::Myr],
        vec![],
    )
}

/// Omega Myr — {2} 1/2 artifact creature.
pub fn omega_myr() -> CardDefinition {
    artifact_creature(
        "Omega Myr",
        cost(&[generic(2)]),
        1,
        2,
        vec![CreatureType::Myr],
        vec![],
    )
}

/// Copper Myr — {T}: Add {G}.
pub fn copper_myr() -> CardDefinition {
    myr("Copper Myr", Color::Green)
}

/// Silver Myr — {T}: Add {U}.
pub fn silver_myr() -> CardDefinition {
    myr("Silver Myr", Color::Blue)
}

/// Iron Myr — {T}: Add {R}.
pub fn iron_myr() -> CardDefinition {
    myr("Iron Myr", Color::Red)
}

/// Leaden Myr — {T}: Add {B}.
pub fn leaden_myr() -> CardDefinition {
    myr("Leaden Myr", Color::Black)
}

// ── Sliths ──

/// Slith Ascendant — {1}{W}{W} 1/1 flier that grows on combat damage.
pub fn slith_ascendant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![slith_growth()],
        ..creature(
            "Slith Ascendant",
            cost(&[generic(1), w(), w()]),
            1,
            1,
            vec![CreatureType::Slith],
            vec![Keyword::Flying],
        )
    }
}

/// Slith Bloodletter — {B}{B} 1/1 that grows on combat damage; {1}{B} regenerates.
pub fn slith_bloodletter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![slith_growth()],
        activated_abilities: vec![regenerate_self(cost(&[generic(1), b()]))],
        ..creature(
            "Slith Bloodletter",
            cost(&[b(), b()]),
            1,
            1,
            vec![CreatureType::Slith],
            vec![],
        )
    }
}

/// Slith Firewalker — {R}{R} 1/1 haste that grows on combat damage.
pub fn slith_firewalker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![slith_growth()],
        ..creature(
            "Slith Firewalker",
            cost(&[r(), r()]),
            1,
            1,
            vec![CreatureType::Slith],
            vec![Keyword::Haste],
        )
    }
}

/// Slith Predator — {G}{G} 1/1 trample that grows on combat damage.
pub fn slith_predator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![slith_growth()],
        ..creature(
            "Slith Predator",
            cost(&[g(), g()]),
            1,
            1,
            vec![CreatureType::Slith],
            vec![Keyword::Trample],
        )
    }
}

/// Slith Strider — {1}{U}{U} 1/1 that draws when blocked and grows on damage.
pub fn slith_strider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
            slith_growth(),
        ],
        ..creature(
            "Slith Strider",
            cost(&[generic(1), u(), u()]),
            1,
            1,
            vec![CreatureType::Slith],
            vec![],
        )
    }
}

// ── Golems and other artifact creatures ──

/// Cobalt Golem — {1}{U}: gains flying until end of turn.
pub fn cobalt_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword_pump(cost(&[generic(1), u()]), Keyword::Flying)],
        ..artifact_creature(
            "Cobalt Golem",
            cost(&[generic(4)]),
            2,
            3,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Titanium Golem — {1}{W}: gains first strike until end of turn.
pub fn titanium_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword_pump(
            cost(&[generic(1), w()]),
            Keyword::FirstStrike,
        )],
        ..artifact_creature(
            "Titanium Golem",
            cost(&[generic(5)]),
            3,
            3,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Pewter Golem — {1}{B}: regenerate.
pub fn pewter_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![regenerate_self(cost(&[generic(1), b()]))],
        ..artifact_creature(
            "Pewter Golem",
            cost(&[generic(5)]),
            4,
            2,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Hematite Golem — {1}{R}: +2/+0 until end of turn.
pub fn hematite_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Hematite Golem",
            cost(&[generic(4)]),
            1,
            4,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

/// Grid Monitor — a 4/6 body that locks its controller out of creature spells.
pub fn grid_monitor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You can't cast creature spells.",
            effect: StaticEffect::ControllerCantCastCreatureSpells,
        }],
        ..artifact_creature(
            "Grid Monitor",
            cost(&[generic(4)]),
            4,
            6,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Wizard Replica — {U}, sacrifice: counter target spell unless its controller
/// pays {2}.
pub fn wizard_replica() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Wizard Replica",
            cost(&[generic(3)]),
            1,
            3,
            vec![CreatureType::Wizard],
            vec![Keyword::Flying],
        )
    }
}

/// Soldier Replica — {1}{W}, sacrifice: 3 damage to an attacking or blocking
/// creature.
pub fn soldier_replica() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Soldier Replica",
            cost(&[generic(3)]),
            1,
            3,
            vec![CreatureType::Soldier],
            vec![],
        )
    }
}

/// Goblin Replica — {3}{R}, sacrifice: destroy target artifact.
pub fn goblin_replica() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Goblin Replica",
            cost(&[generic(3)]),
            2,
            2,
            vec![CreatureType::Goblin],
            vec![],
        )
    }
}

/// Rustspore Ram — on entry, destroy target Equipment.
pub fn rustspore_ram() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
        })],
        ..artifact_creature(
            "Rustspore Ram",
            cost(&[generic(4)]),
            1,
            3,
            vec![CreatureType::Sheep],
            vec![],
        )
    }
}

// ── Nonartifact creatures ──

/// Fangren Hunter — {3}{G}{G} 4/4 trample.
pub fn fangren_hunter() -> CardDefinition {
    creature(
        "Fangren Hunter",
        cost(&[generic(3), g(), g()]),
        4,
        4,
        vec![CreatureType::Beast],
        vec![Keyword::Trample],
    )
}

/// Plated Slagwurm — {4}{G}{G}{G} 8/8 hexproof.
pub fn plated_slagwurm() -> CardDefinition {
    creature(
        "Plated Slagwurm",
        cost(&[generic(4), g(), g(), g()]),
        8,
        8,
        vec![CreatureType::Wurm],
        vec![Keyword::Hexproof],
    )
}

/// Goblin Striker — {1}{R} 1/1 first strike, haste.
pub fn goblin_striker() -> CardDefinition {
    creature(
        "Goblin Striker",
        cost(&[generic(1), r()]),
        1,
        1,
        vec![CreatureType::Goblin, CreatureType::Berserker],
        vec![Keyword::FirstStrike, Keyword::Haste],
    )
}

/// Vulshok Berserker — {3}{R} 3/2 haste.
pub fn vulshok_berserker() -> CardDefinition {
    creature(
        "Vulshok Berserker",
        cost(&[generic(3), r()]),
        3,
        2,
        vec![CreatureType::Human, CreatureType::Berserker],
        vec![Keyword::Haste],
    )
}

/// Dross Prowler — {2}{B} 2/1 fear.
pub fn dross_prowler() -> CardDefinition {
    creature(
        "Dross Prowler",
        cost(&[generic(2), b()]),
        2,
        1,
        vec![CreatureType::Zombie],
        vec![Keyword::Fear],
    )
}

/// Woebearer — fear; its combat damage may reanimate a creature card to hand.
pub fn woebearer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return a creature card from your graveyard to your hand".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature(
            "Woebearer",
            cost(&[generic(4), b()]),
            2,
            3,
            vec![CreatureType::Zombie],
            vec![Keyword::Fear],
        )
    }
}

/// Tel-Jilad Chosen — {1}{G} 2/1 with protection from artifacts.
pub fn tel_jilad_chosen() -> CardDefinition {
    creature(
        "Tel-Jilad Chosen",
        cost(&[generic(1), g()]),
        2,
        1,
        vec![CreatureType::Elf, CreatureType::Warrior],
        vec![Keyword::ProtectionFromCardType(CardType::Artifact)],
    )
}

/// Tel-Jilad Archers — {4}{G} 2/4 with protection from artifacts and reach.
pub fn tel_jilad_archers() -> CardDefinition {
    creature(
        "Tel-Jilad Archers",
        cost(&[generic(4), g()]),
        2,
        4,
        vec![CreatureType::Elf, CreatureType::Archer],
        vec![
            Keyword::ProtectionFromCardType(CardType::Artifact),
            Keyword::Reach,
        ],
    )
}

/// Tel-Jilad Exile — {3}{G} 2/3; {1}{G} regenerates.
pub fn tel_jilad_exile() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![regenerate_self(cost(&[generic(1), g()]))],
        ..creature(
            "Tel-Jilad Exile",
            cost(&[generic(3), g()]),
            2,
            3,
            vec![CreatureType::Troll, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Troll Ascetic — {1}{G}{G} 3/2 hexproof; {1}{G} regenerates.
pub fn troll_ascetic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![regenerate_self(cost(&[generic(1), g()]))],
        ..creature(
            "Troll Ascetic",
            cost(&[generic(1), g(), g()]),
            3,
            2,
            vec![CreatureType::Troll, CreatureType::Shaman],
            vec![Keyword::Hexproof],
        )
    }
}

/// Trolls of Tel-Jilad — {1}{G}: regenerate target green creature.
pub fn trolls_of_tel_jilad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Green))),
            },
            ..Default::default()
        }],
        ..creature(
            "Trolls of Tel-Jilad",
            cost(&[generic(5), g(), g()]),
            5,
            6,
            vec![CreatureType::Troll, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Leonin Elder — every artifact entering may buy you 1 life.
pub fn leonin_elder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                },
            ),
            effect: Effect::MayDo {
                description: "Gain 1 life".into(),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Leonin Elder",
            cost(&[w()]),
            1,
            1,
            vec![CreatureType::Cat, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Leonin Abunas — your artifacts have hexproof.
pub fn leonin_abunas() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifacts you control have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                keyword: Keyword::Hexproof,
            },
        }],
        ..creature(
            "Leonin Abunas",
            cost(&[generic(3), w()]),
            2,
            5,
            vec![CreatureType::Cat, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Leonin Den-Guard — +1/+1 and vigilance while equipped.
pub fn leonin_den_guard() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While equipped, this creature gets +1/+1 and has vigilance.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsEquipped,
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Vigilance],
            },
        }],
        ..creature(
            "Leonin Den-Guard",
            cost(&[generic(1), w()]),
            1,
            3,
            vec![CreatureType::Cat, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Skyhunter Cub — +1/+1 and flying while equipped.
pub fn skyhunter_cub() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While equipped, this creature gets +1/+1 and has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsEquipped,
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..creature(
            "Skyhunter Cub",
            cost(&[generic(2), w()]),
            2,
            2,
            vec![CreatureType::Cat, CreatureType::Knight],
            vec![],
        )
    }
}

/// Loxodon Punisher — +2/+2 for each Equipment attached to it.
pub fn loxodon_punisher() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusPerAttachedEquipment {
            base_p: 2,
            base_t: 2,
            per: 2,
        }),
        ..creature(
            "Loxodon Punisher",
            cost(&[generic(3), w()]),
            2,
            2,
            vec![CreatureType::Elephant, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Vedalken Archmage — every artifact spell you cast draws a card.
pub fn vedalken_archmage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact)),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Vedalken Archmage",
            cost(&[generic(2), u(), u()]),
            0,
            2,
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Viridian Joiner — {T}: add {G} equal to this creature's power.
pub fn viridian_joiner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..creature(
            "Viridian Joiner",
            cost(&[generic(2), g()]),
            1,
            2,
            vec![CreatureType::Elf, CreatureType::Druid],
            vec![],
        )
    }
}

/// Psychic Membrane — a 0/3 Wall that may draw when it blocks.
pub fn psychic_membrane() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![blocks(Effect::MayDo {
            description: "Draw a card".into(),
            body: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            }),
        })],
        ..creature(
            "Psychic Membrane",
            cost(&[generic(2), u()]),
            0,
            3,
            vec![CreatureType::Wall],
            vec![Keyword::Defender],
        )
    }
}

// ── Equipment and other artifacts ──

/// Vulshok Battlegear — equipped creature gets +3/+3. Equip {3}.
pub fn vulshok_battlegear() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            ..Default::default()
        }),
        ..equipment(
            "Vulshok Battlegear",
            cost(&[generic(3)]),
            cost(&[generic(3)]),
        )
    }
}

/// Slagwurm Armor — equipped creature gets +0/+6. Equip {3}.
pub fn slagwurm_armor() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            toughness: 6,
            ..Default::default()
        }),
        ..equipment("Slagwurm Armor", cost(&[generic(1)]), cost(&[generic(3)]))
    }
}

/// Vorrac Battlehorns — trample and can't be blocked by more than one creature.
pub fn vorrac_battlehorns() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Trample, Keyword::CantBeBlockedByMoreThanOne],
            ..Default::default()
        }),
        ..equipment(
            "Vorrac Battlehorns",
            cost(&[generic(2)]),
            cost(&[generic(1)]),
        )
    }
}

/// Vulshok Gauntlets — +4/+2, but the equipped creature stops untapping.
pub fn vulshok_gauntlets() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            power: 4,
            toughness: 2,
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..equipment(
            "Vulshok Gauntlets",
            cost(&[generic(2)]),
            cost(&[generic(3)]),
        )
    }
}

/// Empyrial Plate — equipped creature gets +1/+1 for each card in your hand.
pub fn empyrial_plate() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                per_power: 1,
                per_toughness: 1,
                count_host_controller_hand: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..equipment("Empyrial Plate", cost(&[generic(2)]), cost(&[generic(2)]))
    }
}

/// Viridian Longbow — equipped creature gains a {T}: ping-1 ability.
pub fn viridian_longbow() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: target_any(),
                    amount: Value::ONE,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..equipment("Viridian Longbow", cost(&[generic(1)]), cost(&[generic(3)]))
    }
}

/// Leonin Sun Standard — {1}{W}: your creatures get +1/+1 until end of turn.
pub fn leonin_sun_standard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Leonin Sun Standard", cost(&[generic(2)]))
    }
}

/// Tanglebloom — {1}, {T}: gain 1 life.
pub fn tanglebloom() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..artifact("Tanglebloom", cost(&[generic(1)]))
    }
}

/// Tower of Champions — {8}, {T}: target creature gets +6/+6 until end of turn.
pub fn tower_of_champions() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(6),
                toughness: Value::Const(6),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Tower of Champions", cost(&[generic(4)]))
    }
}

/// Krark's Thumb — CR 705.3: flip two coins and ignore one.
pub fn krarks_thumb() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "If you would flip a coin, instead flip two coins and ignore one.",
            effect: StaticEffect::CoinFlipAdvantage {
                target: PlayerStaticTarget::Controller,
            },
        }],
        ..artifact("Krark's Thumb", cost(&[generic(2)]))
    }
}

// ── Spells ──

/// Battlegrowth — put a +1/+1 counter on target creature.
pub fn battlegrowth() -> CardDefinition {
    spell(
        "Battlegrowth",
        cost(&[g()]),
        false,
        Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
    )
}

/// Barter in Blood — each player sacrifices two creatures.
pub fn barter_in_blood() -> CardDefinition {
    spell(
        "Barter in Blood",
        cost(&[generic(2), b(), b()]),
        true,
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::Const(2),
            filter: R::Creature,
        },
    )
}

/// Altar's Light — exile target artifact or enchantment.
pub fn altars_light() -> CardDefinition {
    spell(
        "Altar's Light",
        cost(&[generic(2), w(), w()]),
        false,
        Effect::Exile {
            what: target_filtered(R::Artifact.or(R::HasCardType(CardType::Enchantment))),
        },
    )
}

/// Deconstruct — destroy target artifact and refund {G}{G}{G}.
pub fn deconstruct() -> CardDefinition {
    spell(
        "Deconstruct",
        cost(&[generic(2), g()]),
        true,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green, Color::Green, Color::Green]),
            },
        ]),
    )
}

/// Turn to Dust — destroy target Equipment and refund {G}.
pub fn turn_to_dust() -> CardDefinition {
    spell(
        "Turn to Dust",
        cost(&[g()]),
        false,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
        ]),
    )
}

/// Fabricate — tutor an artifact to hand.
pub fn fabricate() -> CardDefinition {
    spell(
        "Fabricate",
        cost(&[generic(2), u()]),
        true,
        Effect::Search {
            who: PlayerRef::You,
            filter: R::Artifact,
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Irradiate — target creature gets -1/-1 for each artifact you control.
pub fn irradiate() -> CardDefinition {
    spell(
        "Irradiate",
        cost(&[generic(3), b()]),
        false,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Times(
                Box::new(Value::CountOf(Box::new(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                }))),
                Box::new(Value::Const(-1)),
            ),
            toughness: Value::Times(
                Box::new(Value::CountOf(Box::new(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                }))),
                Box::new(Value::Const(-1)),
            ),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Predator's Strike — +3/+3 and trample until end of turn.
pub fn predators_strike() -> CardDefinition {
    spell(
        "Predator's Strike",
        cost(&[generic(1), g()]),
        false,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Regress — return target permanent to its owner's hand.
pub fn regress() -> CardDefinition {
    spell(
        "Regress",
        cost(&[generic(2), u()]),
        false,
        Effect::Move {
            what: target_filtered(R::Permanent),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Tempest of Light — destroy all enchantments.
pub fn tempest_of_light() -> CardDefinition {
    spell(
        "Tempest of Light",
        cost(&[generic(2), w()]),
        false,
        Effect::Destroy {
            what: Selector::EachPermanent(R::HasCardType(CardType::Enchantment)),
        },
    )
}

/// Tel-Jilad Stylus — {T}: bottom a permanent you own.
pub fn tel_jilad_stylus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::OwnedByYou)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..artifact("Tel-Jilad Stylus", cost(&[generic(1)]))
    }
}
