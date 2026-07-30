//! Retro batch — classic creatures with combat/tap/burn abilities, tribal
//! Minotaur lords, cumulative-upkeep beaters, and old Auras (pump / regenerate).
//! Plus two red mass-destruction / land-hate sorceries. Tests in
//! `tests/recent77.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventScope, EventSpec, Keyword, LandType, Predicate,
    StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{Duration, Effect, EventKind, PlayerRef, Selector, Value};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// Vanilla creature helper.
fn vanilla(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
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

/// Firebreathing-style self-pump activated ability: `{cost}: this gets
/// +power/+toughness until end of turn`.
fn self_pump(mana: ManaCost, power: i32, toughness: i32, once_per_turn: bool) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        once_per_turn,
        ..Default::default()
    }
}

/// Elvish Ranger — {2}{G} 4/1 Elf Ranger (vanilla).
pub fn elvish_ranger() -> CardDefinition {
    vanilla(
        "Elvish Ranger",
        cost(&[generic(2), g()]),
        vec![CreatureType::Elf, CreatureType::Ranger],
        4,
        1,
        vec![],
    )
}

/// Mons's Goblin Raiders — {R} 1/1 Goblin (vanilla).
pub fn monss_goblin_raiders() -> CardDefinition {
    vanilla(
        "Mons's Goblin Raiders",
        cost(&[r()]),
        vec![CreatureType::Goblin],
        1,
        1,
        vec![],
    )
}

/// Ambush Party — {4}{R} 3/1 Human Rogue. First strike, haste.
pub fn ambush_party() -> CardDefinition {
    vanilla(
        "Ambush Party",
        cost(&[generic(4), r()]),
        vec![CreatureType::Human, CreatureType::Rogue],
        3,
        1,
        vec![Keyword::FirstStrike, Keyword::Haste],
    )
}

/// Sabretooth Tiger — {2}{R} 2/1 Cat. First strike.
pub fn sabretooth_tiger() -> CardDefinition {
    vanilla(
        "Sabretooth Tiger",
        cost(&[generic(2), r()]),
        vec![CreatureType::Cat],
        2,
        1,
        vec![Keyword::FirstStrike],
    )
}

/// Storm Shaman — {2}{R} 0/4 Human Cleric Shaman. {R}: +1/+0.
pub fn storm_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[r()]), 1, 0, false)],
        ..vanilla(
            "Storm Shaman",
            cost(&[generic(2), r()]),
            vec![
                CreatureType::Human,
                CreatureType::Cleric,
                CreatureType::Shaman,
            ],
            0,
            4,
            vec![],
        )
    }
}

/// Yavimaya Ancients — {3}{G}{G} 2/7 Treefolk. {G}: +1/-2.
pub fn yavimaya_ancients() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[g()]), 1, -2, false)],
        ..vanilla(
            "Yavimaya Ancients",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Treefolk],
            2,
            7,
            vec![],
        )
    }
}

/// Wild Aesthir — {2}{W} 1/1 Bird. Flying, first strike. {W}{W}: +2/+0
/// (activate only once each turn).
pub fn wild_aesthir() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[w(), w()]), 2, 0, true)],
        ..vanilla(
            "Wild Aesthir",
            cost(&[generic(2), w()]),
            vec![CreatureType::Bird],
            1,
            1,
            vec![Keyword::Flying, Keyword::FirstStrike],
        )
    }
}

/// Woolly Spider — {1}{G}{G} 2/3 Spider. Reach; whenever it blocks a creature
/// with flying, it gets +0/+2 until end of turn.
pub fn woolly_spider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::BlockedAttacker,
                    filter: R::HasKeyword(Keyword::Flying),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..vanilla(
            "Woolly Spider",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Spider],
            2,
            3,
            vec![Keyword::Reach],
        )
    }
}

/// Orcish Artillery — {1}{R}{R} 1/3 Orc Warrior. {T}: deal 2 damage to any
/// target and 3 damage to you.
pub fn orcish_artillery() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_any(),
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
            ..Default::default()
        }],
        ..vanilla(
            "Orcish Artillery",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Orc, CreatureType::Warrior],
            1,
            3,
            vec![],
        )
    }
}

/// Brothers of Fire — {1}{R}{R} 2/2 Human Shaman. {1}{R}{R}: deal 1 damage to
/// any target and 1 damage to you.
pub fn brothers_of_fire() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_any(),
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..vanilla(
            "Brothers of Fire",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            2,
            vec![],
        )
    }
}

/// Goblin Digging Team — {R} 1/1 Goblin. {T}, Sacrifice this creature:
/// Destroy target Wall.
pub fn goblin_digging_team() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::HasCreatureType(CreatureType::Wall)),
            },
            ..Default::default()
        }],
        ..vanilla(
            "Goblin Digging Team",
            cost(&[r()]),
            vec![CreatureType::Goblin],
            1,
            1,
            vec![],
        )
    }
}

/// Aysen Bureaucrats — {1}{W} 1/1 Human Advisor. {T}: Tap target creature with
/// power 2 or less.
pub fn aysen_bureaucrats() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
            },
            ..Default::default()
        }],
        ..vanilla(
            "Aysen Bureaucrats",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            1,
            vec![],
        )
    }
}

/// Anaba Spirit Crafter — {2}{R}{R} 1/3 Minotaur Shaman. Minotaur creatures
/// get +1/+0.
pub fn anaba_spirit_crafter() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Minotaur creatures get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Minotaur)),
                power: 1,
                toughness: 0,
            },
        }],
        ..vanilla(
            "Anaba Spirit Crafter",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Shaman],
            1,
            3,
            vec![],
        )
    }
}

/// Anaba Ancestor — {1}{R} 1/1 Minotaur Spirit. {T}: Another target Minotaur
/// creature gets +1/+1 until end of turn.
pub fn anaba_ancestor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Minotaur).and(R::OtherThanSource),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..vanilla(
            "Anaba Ancestor",
            cost(&[generic(1), r()]),
            vec![CreatureType::Minotaur, CreatureType::Spirit],
            1,
            1,
            vec![],
        )
    }
}

/// Elvish Bard — {3}{G}{G} 2/4 Elf Shaman Bard. All creatures able to block it
/// do so (true Lure).
pub fn elvish_bard() -> CardDefinition {
    vanilla(
        "Elvish Bard",
        cost(&[generic(3), g(), g()]),
        vec![CreatureType::Elf, CreatureType::Shaman, CreatureType::Bard],
        2,
        4,
        vec![Keyword::AllMustBlock],
    )
}

/// Marsh Goblins — {B}{R} 1/1 Goblin. Swampwalk.
pub fn marsh_goblins() -> CardDefinition {
    vanilla(
        "Marsh Goblins",
        cost(&[b(), r()]),
        vec![CreatureType::Goblin],
        1,
        1,
        vec![Keyword::Landwalk(LandType::Swamp)],
    )
}

/// Merfolk Assassin — {U}{U} 1/2 Merfolk Assassin. {T}: Destroy target
/// creature with islandwalk.
pub fn merfolk_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    R::Creature.and(R::HasKeyword(Keyword::Landwalk(LandType::Island))),
                ),
            },
            ..Default::default()
        }],
        ..vanilla(
            "Merfolk Assassin",
            cost(&[u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Assassin],
            1,
            2,
            vec![],
        )
    }
}

/// Ghost Hounds — {1}{B} 1/1 Dog Spirit. Vigilance; whenever it blocks or
/// becomes blocked by a white creature, it gains first strike until end of turn.
pub fn ghost_hounds() -> CardDefinition {
    let gain_fs = || Effect::GrantKeyword {
        what: Selector::This,
        keyword: Keyword::FirstStrike,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::BlockedAttacker,
                        filter: R::HasColor(Color::White),
                    },
                ),
                effect: gain_fs(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::BlockingCreatures,
                        filter: R::HasColor(Color::White),
                    }),
                effect: gain_fs(),
            },
        ],
        ..vanilla(
            "Ghost Hounds",
            cost(&[generic(1), b()]),
            vec![CreatureType::Dog, CreatureType::Spirit],
            1,
            1,
            vec![Keyword::Vigilance],
        )
    }
}

/// Yavimaya Ants — {2}{G}{G} 5/1 Insect. Trample, haste. Cumulative upkeep {G}{G}.
pub fn yavimaya_ants() -> CardDefinition {
    vanilla(
        "Yavimaya Ants",
        cost(&[generic(2), g(), g()]),
        vec![CreatureType::Insect],
        5,
        1,
        vec![
            Keyword::Trample,
            Keyword::Haste,
            Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[g(), g()]))),
        ],
    )
}

/// Orcish Oriflamme — {3}{R} Enchantment. Attacking creatures you control get
/// +1/+0.
pub fn orcish_oriflamme() -> CardDefinition {
    CardDefinition {
        name: "Orcish Oriflamme",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Regeneration — {1}{G} Aura. Enchanted creature has "{G}: Regenerate this
/// creature."
pub fn regeneration() -> CardDefinition {
    CardDefinition {
        name: "Regeneration",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"{G}: Regenerate this creature.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    mana_cost: cost(&[g()]),
                    effect: Effect::Regenerate {
                        what: Selector::This,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Carapace — {G} Aura. Enchanted creature gets +0/+2. Sacrifice this Aura:
/// Regenerate enchanted creature.
pub fn carapace() -> CardDefinition {
    CardDefinition {
        name: "Carapace",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 0,
            toughness: 2,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Regenerate {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Feast of the Unicorn — {3}{B} Aura. Enchanted creature gets +4/+0.
pub fn feast_of_the_unicorn() -> CardDefinition {
    CardDefinition {
        name: "Feast of the Unicorn",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 4,
            toughness: 0,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Icequake — {1}{B}{B} Sorcery. Destroy target land. If that land was a snow
/// land, Icequake deals 1 damage to that land's controller.
pub fn icequake() -> CardDefinition {
    CardDefinition {
        name: "Icequake",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasSupertype(Supertype::Snow),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Destroy {
                what: target_filtered(R::Land),
            },
        ]),
        ..Default::default()
    }
}

/// Jökulhaups — {4}{R}{R} Sorcery. Destroy all artifacts, creatures, and lands.
/// They can't be regenerated.
pub fn jokulhaups() -> CardDefinition {
    CardDefinition {
        name: "Jökulhaups",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.or(R::Artifact).or(R::Land)),
            body: Box::new(Effect::DestroyNoRegen {
                what: Selector::TriggerSource,
            }),
        },
        ..Default::default()
    }
}
