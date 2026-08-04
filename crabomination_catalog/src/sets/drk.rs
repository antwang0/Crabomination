//! The Dark (DRK) — 1994. Tests in `classic_sets/drk`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility, Subtypes,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::target_filtered,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

fn artifact_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, c, types, p, t)
    }
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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
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
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Apprentice Wizard — {U} into three colorless.
pub fn apprentice_wizard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(3)),
            },
            ..Default::default()
        }],
        ..creature(
            "Apprentice Wizard",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            1,
        )
    }
}

/// Carnivorous Plant — {3}{G} 4/5 Wall.
pub fn carnivorous_plant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        ..creature(
            "Carnivorous Plant",
            cost(&[generic(3), g()]),
            vec![CreatureType::Plant, CreatureType::Wall],
            4,
            5,
        )
    }
}

/// Cave People — swings harder but softer, and hands out mountainwalk.
pub fn cave_people() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Landwalk(LandType::Mountain),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Cave People", cost(&[generic(1), r(), r()]), vec![CreatureType::Human], 1, 4)
    }
}

/// Coal Golem — a 3/3 that burns down into {R}{R}{R}.
pub fn coal_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::Const(3)),
            },
            ..Default::default()
        }],
        ..artifact_creature("Coal Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
    }
}

/// Diabolic Machine — {3}: regenerate.
pub fn diabolic_machine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..artifact_creature(
            "Diabolic Machine",
            cost(&[generic(7)]),
            vec![CreatureType::Construct],
            4,
            4,
        )
    }
}

/// Drowned — {B}: regenerate.
pub fn drowned() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Drowned", cost(&[generic(1), u()]), vec![CreatureType::Zombie], 1, 1)
    }
}

/// Electric Eel — a 1/1 that shocks you coming and going.
pub fn electric_eel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage { to: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::DealDamage { to: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature("Electric Eel", cost(&[u()]), vec![CreatureType::Fish], 1, 1)
    }
}

/// Exorcist — {1}{W}, {T}: destroy target black creature.
pub fn exorcist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black))),
            },
            ..Default::default()
        }],
        ..creature(
            "Exorcist",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Goblins of the Flarg — mountainwalkers who won't share with Dwarves.
pub fn goblins_of_the_flarg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours),
            effect: Effect::If {
                cond: crate::card::Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Dwarf).and(R::ControlledByYou),
                )),
                then: Box::new(Effect::SacrificeSource),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Goblins of the Flarg",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Knights of Thorn — {3}{W} 2/2 with protection from red and banding.
pub fn knights_of_thorn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red), Keyword::Banding],
        ..creature(
            "Knights of Thorn",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Land Leeches — {1}{G}{G} 2/2 first striker.
pub fn land_leeches() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature("Land Leeches", cost(&[generic(1), g(), g()]), vec![CreatureType::Leech], 2, 2)
    }
}

/// Miracle Worker — {T}: shrug off an Aura on one of your creatures.
pub fn miracle_worker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Miracle Worker",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Murk Dwellers — hits harder when nobody blocks.
pub fn murk_dwellers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfCombat,
            },
        }],
        ..creature("Murk Dwellers", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Niall Silvain — a slow, repeatable regeneration engine.
pub fn niall_silvain() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g(), g(), g()]),
            tap_cost: true,
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Niall Silvain", cost(&[g(), g(), g()]), vec![CreatureType::Ouphe], 2, 2)
    }
}

/// People of the Woods — as tough as your Forest count.
pub fn people_of_the_woods() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatchingToughness {
            base_p: 1,
            base_t: 0,
            filter: Box::new(R::HasLandType(LandType::Forest)),
        }),
        ..creature("People of the Woods", cost(&[g(), g()]), vec![CreatureType::Human], 1, 0)
    }
}

/// Pikemen — {1}{W} 1/2 with first strike and banding.
pub fn pikemen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Banding],
        ..creature(
            "Pikemen",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Savaen Elves — {G}{G}, {T}: destroy an Aura on a land.
pub fn savaen_elves() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura),
                ),
            },
            ..Default::default()
        }],
        ..creature("Savaen Elves", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Scarwood Goblins — {R}{G} 2/2, no text.
pub fn scarwood_goblins() -> CardDefinition {
    creature("Scarwood Goblins", cost(&[r(), g()]), vec![CreatureType::Goblin], 2, 2)
}

/// Scavenger Folk — a one-shot Naturalize on legs.
pub fn scavenger_folk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..creature("Scavenger Folk", cost(&[g()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Sisters of the Flame — {T}: add {R}.
pub fn sisters_of_the_flame() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Sisters of the Flame",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Squire — {1}{W} 1/2, no text.
pub fn squire() -> CardDefinition {
    creature(
        "Squire",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Soldier],
        1,
        2,
    )
}

/// Uncle Istvan — creatures simply can't hurt him.
pub fn uncle_istvan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PreventDamageFromMatching(Box::new(R::Creature))],
        ..creature("Uncle Istvan", cost(&[generic(1), b(), b(), b()]), vec![CreatureType::Human], 1, 3)
    }
}

/// Water Wurm — grows while an opponent holds an Island.
pub fn water_wurm() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +0/+1 as long as an opponent controls an Island.",
            effect: StaticEffect::PumpSelfIf {
                condition: crate::card::Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(LandType::Island).and(R::ControlledByOpponent),
                )),
                power: 0,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..creature("Water Wurm", cost(&[u()]), vec![CreatureType::Wurm], 1, 1)
    }
}

/// Witch Hunter — pings faces and bounces an opponent's creature.
pub fn witch_hunter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w(), w()]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Witch Hunter",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Wormwood Treefolk — buys evasion with its controller's life.
pub fn wormwood_treefolk() -> CardDefinition {
    let walk = |lt: LandType| ActivatedAbility {
        mana_cost: match lt {
            LandType::Forest => cost(&[g(), g()]),
            _ => cost(&[b(), b()]),
        },
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Landwalk(lt),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![walk(LandType::Forest), walk(LandType::Swamp)],
        ..creature(
            "Wormwood Treefolk",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Treefolk],
            4,
            4,
        )
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Amnesia — strips a hand down to its lands.
pub fn amnesia() -> CardDefinition {
    sorcery(
        "Amnesia",
        cost(&[generic(3), u(), u(), u()]),
        Effect::RevealHandDiscardAllMatching {
            who: PlayerRef::Target(0),
            filter: R::Not(Box::new(R::Land)),
        },
    )
}

/// Ashes to Ashes — two creatures gone, five life spent.
pub fn ashes_to_ashes() -> CardDefinition {
    CardDefinition {
        ..sorcery(
            "Ashes to Ashes",
            cost(&[generic(1), b(), b()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::Not(Box::new(R::Artifact)))),
                    to: ZoneDest::Exile,
                },
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::Not(Box::new(R::Artifact))),
                    },
                    to: ZoneDest::Exile,
                },
                Effect::DealDamage { to: Selector::You, amount: Value::Const(5) },
            ]),
        )
    }
}

/// Dust to Dust — exiles two artifacts.
pub fn dust_to_dust() -> CardDefinition {
    CardDefinition {
        ..sorcery(
            "Dust to Dust",
            cost(&[generic(1), w(), w()]),
            Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Artifact), to: ZoneDest::Exile },
                Effect::Move {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Artifact },
                    to: ZoneDest::Exile,
                },
            ]),
        )
    }
}

/// Marsh Gas — every creature loses two power.
pub fn marsh_gas() -> CardDefinition {
    instant(
        "Marsh Gas",
        cost(&[b()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Const(-2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Morale — attackers get +1/+1.
pub fn morale() -> CardDefinition {
    instant(
        "Morale",
        cost(&[generic(1), w(), w()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Riptide — taps every blue creature.
pub fn riptide() -> CardDefinition {
    instant(
        "Riptide",
        cost(&[u()]),
        Effect::Tap {
            what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Blue))),
        },
    )
}

/// Tivadar's Crusade — kills every Goblin.
pub fn tivadars_crusade() -> CardDefinition {
    sorcery(
        "Tivadar's Crusade",
        cost(&[generic(1), w(), w()]),
        Effect::Destroy {
            what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Goblin)),
        },
    )
}


// ── Enchantments ───────────────────────────────────────────────────────────

/// Dark Heart of the Wood — Forests into life.
pub fn dark_heart_of_the_wood() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                R::HasLandType(LandType::Forest).and(R::ControlledByYou),
                1,
            )),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..enchantment("Dark Heart of the Wood", cost(&[b(), g()]))
    }
}

/// Hidden Path — every green creature walks the Forests.
pub fn hidden_path() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Green creatures have forestwalk.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasColor(Color::Green)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Landwalk(LandType::Forest)],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..enchantment("Hidden Path", cost(&[generic(2), g(), g(), g(), g()]))
    }
}

/// Sunken City — a blue anthem with an upkeep rent.
pub fn sunken_city() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Blue creatures get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasColor(Color::Blue)),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[u(), u()]) },
        }],
        ..enchantment("Sunken City", cost(&[u(), u()]))
    }
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Bone Flute — {2}, {T}: every creature loses a point of power.
pub fn bone_flute() -> CardDefinition {
    artifact(
        "Bone Flute",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-1),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Book of Rass — life into cards.
pub fn book_of_rass() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        ..artifact(
            "Book of Rass",
            cost(&[generic(6)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                life_cost: 2,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            }],
        )
    }
}

/// Fountain of Youth — {2}, {T}: gain 1 life.
pub fn fountain_of_youth() -> CardDefinition {
    artifact(
        "Fountain of Youth",
        ManaCost::default(),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
    )
}

/// Skull of Orm — buys back an enchantment.
pub fn skull_of_orm() -> CardDefinition {
    artifact(
        "Skull of Orm",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Enchantment.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
    )
}

/// Standing Stones — any colour, for a life.
pub fn standing_stones() -> CardDefinition {
    artifact(
        "Standing Stones",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            life_cost: 1,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
    )
}

/// Stone Calendar — your spells cost {1} less.
pub fn stone_calendar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Any, amount: 1 },
        }],
        ..artifact("Stone Calendar", cost(&[generic(5)]), vec![])
    }
}

/// Tower of Coireall — Walls can't stop the chosen creature.
pub fn tower_of_coireall() -> CardDefinition {
    artifact(
        "Tower of Coireall",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBeBlockedByCreatureType(CreatureType::Wall),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}
