//! Tempest (TMP) creatures — the Dauthi shadow corps, the Flowstone pumps and
//! the Rathi Slivers. Tests in `classic_sets/tmp`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{deal, draw, etb, on_dies, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
};
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

/// "{cost}: This creature gets +p/+t until end of turn."
fn self_pump(c: ManaCost, p: i32, t: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(p),
            toughness: Value::Const(t),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Dauthi (shadow) ─────────────────────────────────────────────────────────

fn dauthi(name: &'static str, c: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Shadow], ..creature(name, c, types, p, t) }
}

/// Dauthi Marauder — {2}{B} 3/1 shadow.
pub fn dauthi_marauder() -> CardDefinition {
    dauthi(
        "Dauthi Marauder",
        cost(&[generic(2), b()]),
        vec![CreatureType::Dauthi, CreatureType::Minion],
        3,
        1,
    )
}

/// Dauthi Horror — {1}{B} 2/1 shadow that white creatures can't block.
pub fn dauthi_horror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow, Keyword::CantBeBlockedBy(Box::new(R::HasColor(Color::White)))],
        ..creature(
            "Dauthi Horror",
            cost(&[generic(1), b()]),
            vec![CreatureType::Dauthi, CreatureType::Horror],
            2,
            1,
        )
    }
}

/// Dauthi Mercenary — {2}{B} 2/1 shadow with a firebreathing-style pump.
pub fn dauthi_mercenary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[generic(1), b()]), 1, 0)],
        ..dauthi(
            "Dauthi Mercenary",
            cost(&[generic(2), b()]),
            vec![CreatureType::Dauthi, CreatureType::Knight, CreatureType::Mercenary],
            2,
            1,
        )
    }
}

/// Dauthi Ghoul — {1}{B} 1/1 shadow that grows on every shadow death.
pub fn dauthi_ghoul() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasKeyword(Keyword::Shadow)),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..dauthi(
            "Dauthi Ghoul",
            cost(&[generic(1), b()]),
            vec![CreatureType::Dauthi, CreatureType::Zombie],
            1,
            1,
        )
    }
}

/// Dauthi Mindripper — {3}{B} 2/1 shadow. Unblocked, it can eat itself to strip
/// three cards from the defender.
pub fn dauthi_mindripper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Sacrifice Dauthi Mindripper — defending player discards three cards"
                    .to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificePermanent { what: Selector::This },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::DefendingPlayer),
                        amount: Value::Const(3),
                        random: false,
                    },
                ])),
            },
        }],
        ..dauthi(
            "Dauthi Mindripper",
            cost(&[generic(3), b()]),
            vec![CreatureType::Dauthi, CreatureType::Minion],
            2,
            1,
        )
    }
}

// ── Slivers ─────────────────────────────────────────────────────────────────

/// "All Sliver creatures have '{2}: This creature gets +p/+t until end of turn.'"
fn sliver_pump_lord(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Slivers have \"{2}: This creature gets a pump until end of turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Sliver)),
                ability: self_pump(cost(&[generic(2)]), p, t),
                condition: None,
            },
        }],
        ..creature(name, c, vec![CreatureType::Sliver], 2, 2)
    }
}

/// Armor Sliver — {2}{W} 2/2. All Slivers gain a {2} toughness pump.
pub fn armor_sliver() -> CardDefinition {
    sliver_pump_lord("Armor Sliver", cost(&[generic(2), w()]), 0, 1)
}

/// Barbed Sliver — {2}{R} 2/2. All Slivers gain a {2} power pump.
pub fn barbed_sliver() -> CardDefinition {
    sliver_pump_lord("Barbed Sliver", cost(&[generic(2), r()]), 1, 0)
}

// ── White ───────────────────────────────────────────────────────────────────

/// Advance Scout — {1}{W} 1/1 first strike that hands first strike around.
pub fn advance_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Advance Scout",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Angelic Protector — {3}{W} 2/2 flier that hardens when targeted.
pub fn angelic_protector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ZERO,
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Angelic Protector", cost(&[generic(3), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

/// Auratog — {1}{W} 1/2 that eats its own enchantments.
pub fn auratog() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Enchantment, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Auratog", cost(&[generic(1), w()]), vec![CreatureType::Atog], 1, 2)
    }
}

/// Avenging Angel — {3}{W}{W} 3/3 flier that may shuffle back onto the library
/// top when it dies.
pub fn avenging_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Put Avenging Angel on top of its owner's library".to_string(),
            body: Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Top },
            }),
        })],
        ..creature(
            "Avenging Angel",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Angel],
            3,
            3,
        )
    }
}

/// Clergy en-Vec — {1}{W} 1/1 that taps to shave a point of damage.
pub fn clergy_en_vec() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: crate::effect::shortcut::target_any(),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Clergy en-Vec",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Cloudchaser Eagle — {3}{W} 2/2 flier with an enchantment-kill ETB.
pub fn cloudchaser_eagle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Enchantment),
        })],
        ..creature("Cloudchaser Eagle", cost(&[generic(3), w()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Elite Javelineer — {2}{W} 2/2 that pings an attacker when it blocks.
pub fn elite_javelineer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::blocks(deal(
            1,
            target_filtered(R::Creature.and(R::IsAttacking)),
        ))],
        ..creature(
            "Elite Javelineer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Benthic Behemoth — {5}{U}{U}{U} 7/6 islandwalker.
pub fn benthic_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        ..creature(
            "Benthic Behemoth",
            cost(&[generic(5), u(), u(), u()]),
            vec![CreatureType::Serpent],
            7,
            6,
        )
    }
}

/// Fighting Drake — {2}{U}{U} 2/4 flier.
pub fn fighting_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Fighting Drake", cost(&[generic(2), u(), u()]), vec![CreatureType::Drake], 2, 4)
    }
}

/// Fylamarid — {1}{U}{U} 1/3 flier blue creatures can't block; it can paint a
/// creature blue to widen that.
pub fn fylamarid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CantBeBlockedBy(Box::new(R::HasColor(Color::Blue)))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::Blue],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Fylamarid",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Squid, CreatureType::Beast],
            1,
            3,
        )
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Commander Greven il-Vec — {3}{B}{B}{B} 7/5 fear that demands a creature on
/// arrival.
pub fn commander_greven_il_vec() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::You,
            count: Value::ONE,
            filter: R::Creature,
        })],
        ..creature(
            "Commander Greven il-Vec",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Warrior],
            7,
            5,
        )
    }
}

/// Darkling Stalker — {3}{B} 1/1 shade that also regenerates.
pub fn darkling_stalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
            self_pump(cost(&[b()]), 1, 1),
        ],
        ..creature(
            "Darkling Stalker",
            cost(&[generic(3), b()]),
            vec![CreatureType::Shade, CreatureType::Spirit],
            1,
            1,
        )
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Canyon Drake — {2}{R}{R} 1/2 flier that burns cards for power.
pub fn canyon_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Canyon Drake", cost(&[generic(2), r(), r()]), vec![CreatureType::Drake], 1, 2)
    }
}

/// Firefly — {3}{R} 1/1 flier with firebreathing.
pub fn firefly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![self_pump(cost(&[r()]), 1, 0)],
        ..creature("Firefly", cost(&[generic(3), r()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Flowstone Giant — {2}{R}{R} 3/3 that trades toughness for power.
pub fn flowstone_giant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[r()]), 2, -2)],
        ..creature("Flowstone Giant", cost(&[generic(2), r(), r()]), vec![CreatureType::Giant], 3, 3)
    }
}

/// Flowstone Wyvern — {3}{R}{R} 3/3 flier with the same +2/-2 pump.
pub fn flowstone_wyvern() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![self_pump(cost(&[r()]), 2, -2)],
        ..creature(
            "Flowstone Wyvern",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Drake],
            3,
            3,
        )
    }
}

/// Flowstone Salamander — {3}{R}{R} 3/4 that pings its blockers.
pub fn flowstone_salamander() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: deal(1, Selector::take(Selector::BlockingCreatures, Value::ONE)),
            ..Default::default()
        }],
        ..creature(
            "Flowstone Salamander",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Salamander],
            3,
            4,
        )
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Apes of Rath — {2}{G}{G} 5/4 that stays tapped after it swings.
pub fn apes_of_rath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::SkipNextUntap {
            what: Selector::This,
        })],
        ..creature("Apes of Rath", cost(&[generic(2), g(), g()]), vec![CreatureType::Ape], 5, 4)
    }
}

/// Bayou Dragonfly — {1}{G} 1/1 flying swampwalker.
pub fn bayou_dragonfly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Landwalk(LandType::Swamp)],
        ..creature("Bayou Dragonfly", cost(&[generic(1), g()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Dirtcowl Wurm — {4}{G} 3/4 that grows off opposing land drops.
pub fn dirtcowl_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::OpponentControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Dirtcowl Wurm", cost(&[generic(4), g()]), vec![CreatureType::Wurm], 3, 4)
    }
}

/// Eladamri, Lord of Leaves — {G}{G} 2/2 that makes the rest of the Elves
/// unblockable-through-forests and untargetable.
pub fn eladamri_lord_of_leaves() -> CardDefinition {
    let other_elves = || {
        Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Elf).and(R::OtherThanSource),
        )
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "Other Elf creatures have forestwalk.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: other_elves(),
                    keyword: Keyword::Landwalk(LandType::Forest),
                },
            },
            StaticAbility {
                description: "Other Elves have shroud.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: other_elves(),
                    keyword: Keyword::Shroud,
                },
            },
        ],
        ..creature(
            "Eladamri, Lord of Leaves",
            cost(&[g(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Fugitive Druid — {3}{G} 3/2 that cantrips off Aura spells aimed at it.
pub fn fugitive_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource)
                .caused_by(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            effect: draw(1),
        }],
        ..creature(
            "Fugitive Druid",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            2,
        )
    }
}

// ── Artifact creatures ──────────────────────────────────────────────────────

/// Coiled Tinviper — {3} 2/1 first striker.
pub fn coiled_tinviper() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::FirstStrike],
        ..creature("Coiled Tinviper", cost(&[generic(3)]), vec![CreatureType::Snake], 2, 1)
    }
}

/// Energizer — {4} 2/2 Juggernaut that grows on its own.
pub fn energizer() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Energizer", cost(&[generic(4)]), vec![CreatureType::Juggernaut], 2, 2)
    }
}

// ── Soltari and Thalakos (the rest of the shadow corps) ─────────────────────

fn shadow_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Shadow], ..creature(name, c, types, p, t) }
}

/// Soltari Foot Soldier — {W} 1/1 shadow.
pub fn soltari_foot_soldier() -> CardDefinition {
    shadow_creature(
        "Soltari Foot Soldier",
        cost(&[w()]),
        vec![CreatureType::Soltari, CreatureType::Soldier],
        1,
        1,
    )
}

/// Soltari Crusader — {2}{W} 2/1 shadow with a pump.
pub fn soltari_crusader() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[generic(1), w()]), 1, 0)],
        ..shadow_creature(
            "Soltari Crusader",
            cost(&[generic(2), w()]),
            vec![CreatureType::Soltari, CreatureType::Knight],
            2,
            1,
        )
    }
}

/// Soltari Lancer — {2}{W} 2/2 shadow that first-strikes while attacking.
pub fn soltari_lancer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has first strike as long as it's attacking.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::MatchingAmong {
                    inner: Box::new(Selector::This),
                    filter: R::IsAttacking,
                },
                keyword: Keyword::FirstStrike,
            },
        }],
        ..shadow_creature(
            "Soltari Lancer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Soltari, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Soltari Trooper — {1}{W} 1/1 shadow that swells when it swings.
pub fn soltari_trooper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..shadow_creature(
            "Soltari Trooper",
            cost(&[generic(1), w()]),
            vec![CreatureType::Soltari, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Soltari Emissary — {1}{W} 2/1 that buys shadow by the turn.
pub fn soltari_emissary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Shadow,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Soltari Emissary",
            cost(&[generic(1), w()]),
            vec![CreatureType::Soltari, CreatureType::Soldier],
            2,
            1,
        )
    }
}

/// Thalakos Sentry — {1}{U} 1/2 shadow.
pub fn thalakos_sentry() -> CardDefinition {
    shadow_creature(
        "Thalakos Sentry",
        cost(&[generic(1), u()]),
        vec![CreatureType::Thalakos, CreatureType::Soldier],
        1,
        2,
    )
}

/// Thalakos Seer — {U}{U} 1/1 shadow that cantrips on the way out.
pub fn thalakos_seer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: draw(1),
        }],
        ..shadow_creature(
            "Thalakos Seer",
            cost(&[u(), u()]),
            vec![CreatureType::Thalakos, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Thalakos Mistfolk — {2}{U} 2/1 shadow that can duck back onto the library.
pub fn thalakos_mistfolk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Top },
            },
            ..Default::default()
        }],
        ..shadow_creature(
            "Thalakos Mistfolk",
            cost(&[generic(2), u()]),
            vec![CreatureType::Thalakos, CreatureType::Illusion],
            2,
            1,
        )
    }
}

// ── The rest of the commons and uncommons ──────────────────────────────────

/// Lowland Giant — {2}{R}{R} 4/3.
pub fn lowland_giant() -> CardDefinition {
    creature("Lowland Giant", cost(&[generic(2), r(), r()]), vec![CreatureType::Giant], 4, 3)
}

/// Rootbreaker Wurm — {5}{G}{G} 6/6 trample.
pub fn rootbreaker_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature(
            "Rootbreaker Wurm",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Wurm],
            6,
            6,
        )
    }
}

/// Pincher Beetles — {2}{G} 3/1 shroud.
pub fn pincher_beetles() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        ..creature("Pincher Beetles", cost(&[generic(2), g()]), vec![CreatureType::Insect], 3, 1)
    }
}

/// Heartwood Treefolk — {2}{G}{G} 3/4 forestwalk.
pub fn heartwood_treefolk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..creature(
            "Heartwood Treefolk",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Treefolk],
            3,
            4,
        )
    }
}

/// Skyshroud Troll — {2}{G}{G} 3/3 that regenerates.
pub fn skyshroud_troll() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Skyshroud Troll",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Troll, CreatureType::Giant],
            3,
            3,
        )
    }
}

/// Screeching Harpy — {2}{B}{B} 2/2 flier that regenerates.
pub fn screeching_harpy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Screeching Harpy",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Harpy, CreatureType::Beast],
            2,
            2,
        )
    }
}

/// Ranger en-Vec — {1}{G}{W} 2/2 first striker that regenerates.
pub fn ranger_en_vec() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Ranger en-Vec",
            cost(&[generic(1), g(), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Archer,
                CreatureType::Ranger,
            ],
            2,
            2,
        )
    }
}

/// Sandstone Warrior — {2}{R}{R} 1/3 first striker with firebreathing.
pub fn sandstone_warrior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![self_pump(cost(&[r()]), 1, 0)],
        ..creature(
            "Sandstone Warrior",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Warrior],
            1,
            3,
        )
    }
}

/// Scragnoth — {4}{G} 3/4 that blue can neither counter nor touch.
pub fn scragnoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Protection(Color::Blue)],
        ..creature("Scragnoth", cost(&[generic(4), g()]), vec![CreatureType::Beast], 3, 4)
    }
}

/// Manta Riders — {U} 1/1 that buys flying.
pub fn manta_riders() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Manta Riders", cost(&[u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Wind Dancer — {1}{U} 1/1 flier that lends flight.
pub fn wind_dancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Wind Dancer", cost(&[generic(1), u()]), vec![CreatureType::Faerie], 1, 1)
    }
}

/// Mawcor — {3}{U}{U} 3/3 flier that pings.
pub fn mawcor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: deal(1, crate::effect::shortcut::target_any()),
            ..Default::default()
        }],
        ..creature("Mawcor", cost(&[generic(3), u(), u()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Master Decoy — {1}{W} 1/2 tapper.
pub fn master_decoy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Master Decoy",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Seeker of Skybreak — {1}{G} 2/1 untapper.
pub fn seeker_of_skybreak() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            ..Default::default()
        }],
        ..creature("Seeker of Skybreak", cost(&[generic(1), g()]), vec![CreatureType::Elf], 2, 1)
    }
}

/// Staunch Defenders — {3}{W}{W} 3/4 that gains you 4.
pub fn staunch_defenders() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb_gain_life(4)],
        ..creature(
            "Staunch Defenders",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Souldrinker — {3}{B} 2/2 that trades life for counters.
pub fn souldrinker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 3,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Souldrinker", cost(&[generic(3), b()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Selenia, Dark Angel — {3}{W}{B} 3/3 flier that buys herself back.
pub fn selenia_dark_angel() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature(
            "Selenia, Dark Angel",
            cost(&[generic(3), w(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Angel],
            3,
            3,
        )
    }
}

/// Servant of Volrath — {2}{B} 3/3 that takes a friend with it.
pub fn servant_of_volrath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
            },
        }],
        ..creature("Servant of Volrath", cost(&[generic(2), b()]), vec![CreatureType::Minion], 3, 3)
    }
}

/// Segmented Wurm — {3}{R}{G} 5/5 that shrinks whenever it's targeted.
pub fn segmented_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Segmented Wurm",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Wurm],
            5,
            5,
        )
    }
}

/// Krakilin — {X}{G}{G} 0/0 that enters with X counters and regenerates.
pub fn krakilin() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Krakilin", cost(&[crate::mana::x(), g(), g()]), vec![CreatureType::Beast], 0, 0)
    }
}

/// Spike Drone — {G} 0/0 that enters with a counter it can pass on.
pub fn spike_drone() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Spike Drone",
            cost(&[g()]),
            vec![CreatureType::Spike, CreatureType::Drone],
            0,
            0,
        )
    }
}

/// Knight of Dawn — {1}{W}{W} 2/2 first striker that buys protection.
pub fn knight_of_dawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            effect: Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Knight of Dawn",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Knight of Dusk — {1}{B}{B} 2/2 that murders its blocker.
pub fn knight_of_dusk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            effect: Effect::Destroy {
                what: Selector::take(Selector::BlockingCreatures, Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Knight of Dusk",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Renegade Warlord — {4}{R} 3/3 first striker that rallies the attack.
pub fn renegade_warlord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(
                R::Creature.and(R::IsAttacking).and(R::OtherThanSource),
            ),
            power: Value::ONE,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Renegade Warlord",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Rootwalla — {2}{G} 2/2 with a once-a-turn pump.
pub fn rootwalla() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            ..self_pump(cost(&[generic(1), g()]), 2, 2)
        }],
        ..creature("Rootwalla", cost(&[generic(2), g()]), vec![CreatureType::Lizard], 2, 2)
    }
}

/// Pit Imp — {B} 0/1 flier with a twice-a-turn pump.
pub fn pit_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            max_activations_per_turn: Some(2),
            ..self_pump(cost(&[b()]), 1, 0)
        }],
        ..creature("Pit Imp", cost(&[b()]), vec![CreatureType::Imp], 0, 1)
    }
}

/// Verdant Force — {5}{G}{G}{G} 7/7 that spawns a Saproling every upkeep.
pub fn verdant_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crate::card::TokenDefinition {
                    name: "Saproling".to_string(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Saproling],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..creature(
            "Verdant Force",
            cost(&[generic(5), g(), g(), g()]),
            vec![CreatureType::Elemental],
            7,
            7,
        )
    }
}

/// Mongrel Pack — {3}{G} 4/1 that leaves a litter behind.
pub fn mongrel_pack() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(4),
            definition: crate::card::TokenDefinition {
                name: "Dog".to_string(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Dog],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..creature("Mongrel Pack", cost(&[generic(3), g()]), vec![CreatureType::Dog], 4, 1)
    }
}

/// Marble Titan — {3}{W} 3/3 that locks down the big creatures.
pub fn marble_titan() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures with power 3 or greater don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: Selector::EachPermanent(R::Creature.and(R::PowerAtLeast(3))),
                condition: None,
            },
        }],
        ..creature("Marble Titan", cost(&[generic(3), w()]), vec![CreatureType::Giant], 3, 3)
    }
}

/// Orim, Samite Healer — {1}{W}{W} 1/3 that taps to soak 3 damage.
pub fn orim_samite_healer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: crate::effect::shortcut::target_any(),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..creature(
            "Orim, Samite Healer",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}
