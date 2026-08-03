//! Legends (LEG) — the CR 702.22 "bands with other" cycle (the five
//! legendary-band lands, Master of the Hunt, Shelkin Brownie, Tolaria) plus
//! the set's plain bodies and one-line spells. Tests in `classic_sets/leg`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EquipBonus, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value,
    shortcut::target_filtered,
};
use crate::mana::{Color, ManaCost, cost, g, generic};

/// "Bands with other legendary creatures" — the quality the five Legends
/// band lands hand out.
fn bands_with_legends() -> Keyword {
    Keyword::BandsWithOther(Box::new(
        R::Creature.and(R::HasSupertype(Supertype::Legendary)),
    ))
}

/// The Legends band-land cycle: one colour's legendary creatures band together.
fn band_land(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "Legendary creatures you control of this land's color have \
                          \"bands with other legendary creatures.\"",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasSupertype(Supertype::Legendary))
                        .and(R::HasColor(color)),
                ),
                keyword: bands_with_legends(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Adventurers' Guildhouse — the green band land.
pub fn adventurers_guildhouse() -> CardDefinition {
    band_land("Adventurers' Guildhouse", Color::Green)
}

/// Cathedral of Serra — the white band land.
pub fn cathedral_of_serra() -> CardDefinition {
    band_land("Cathedral of Serra", Color::White)
}

/// Mountain Stronghold — the red band land.
pub fn mountain_stronghold() -> CardDefinition {
    band_land("Mountain Stronghold", Color::Red)
}

/// Seafarer's Quay — the blue band land.
pub fn seafarers_quay() -> CardDefinition {
    band_land("Seafarer's Quay", Color::Blue)
}

/// Unholy Citadel — the black band land.
pub fn unholy_citadel() -> CardDefinition {
    band_land("Unholy Citadel", Color::Black)
}

/// Master of the Hunt — a wolf factory whose tokens band with each other.
pub fn master_of_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "Master of the Hunt",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Wolves of the Hunt".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Wolf],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::BandsWithOther(Box::new(R::HasName(
                        "Wolves of the Hunt".into(),
                    )))],
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shelkin Brownie — strips a creature's "bands with other".
pub fn shelkin_brownie() -> CardDefinition {
    CardDefinition {
        name: "Shelkin Brownie",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ouphe], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LoseKeyword {
                what: target_filtered(R::Creature),
                keyword: bands_with_legends(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tolaria — blue mana, or a band-hosing tap during any upkeep.
pub fn tolaria() -> CardDefinition {
    CardDefinition {
        name: "Tolaria",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Blue]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::LoseKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Banding,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::LoseKeyword {
                        what: target_filtered(R::Creature),
                        keyword: bands_with_legends(),
                        duration: Duration::EndOfTurn,
                    },
                ]),
                condition: Some(crate::card::Predicate::CurrentStepIs(
                    crate::game::types::TurnStep::Upkeep,
                )),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Wave 2: the plain bodies and one-line spells ────────────────────────────

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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

/// A 0/1 Kobold for nothing — the Kher Keep cycle.
fn kobold(name: &'static str) -> CardDefinition {
    CardDefinition {
        color_override: Some(vec![Color::Red]),
        ..creature(name, ManaCost::default(), vec![CreatureType::Kobold], 0, 1)
    }
}

/// Crimson Kobolds — a free 0/1.
pub fn crimson_kobolds() -> CardDefinition {
    kobold("Crimson Kobolds")
}

/// Crookshank Kobolds — a free 0/1.
pub fn crookshank_kobolds() -> CardDefinition {
    kobold("Crookshank Kobolds")
}

/// Kobolds of Kher Keep — a free 0/1.
pub fn kobolds_of_kher_keep() -> CardDefinition {
    kobold("Kobolds of Kher Keep")
}

/// Kobold Taskmaster — the Kobold power lord.
pub fn kobold_taskmaster() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Kobold creatures you control get +1/+0.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Kobold)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
                power: 1,
                toughness: 0,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..creature(
            "Kobold Taskmaster",
            cost(&[generic(1), crate::mana::r()]),
            vec![CreatureType::Kobold],
            1,
            2,
        )
    }
}

/// Kobold Drill Sergeant — the Kobold toughness-and-trample lord.
pub fn kobold_drill_sergeant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Kobold creatures you control get +0/+1 and have trample.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Kobold)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
                power: 0,
                toughness: 1,
                keywords: vec![Keyword::Trample],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..creature(
            "Kobold Drill Sergeant",
            cost(&[generic(1), crate::mana::r()]),
            vec![CreatureType::Kobold, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Kobold Overlord — first strike, and it shares.
pub fn kobold_overlord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Other Kobold creatures you control have first strike.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Kobold)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..creature(
            "Kobold Overlord",
            cost(&[generic(1), crate::mana::r()]),
            vec![CreatureType::Kobold],
            1,
            2,
        )
    }
}

/// Headless Horseman — a vanilla 2/2.
pub fn headless_horseman() -> CardDefinition {
    creature(
        "Headless Horseman",
        cost(&[generic(2), crate::mana::b()]),
        vec![CreatureType::Zombie, CreatureType::Knight],
        2,
        2,
    )
}

/// Barbary Apes — a vanilla 2/2.
pub fn barbary_apes() -> CardDefinition {
    creature("Barbary Apes", cost(&[generic(1), g()]), vec![CreatureType::Ape], 2, 2)
}

/// Hornet Cobra — a first-striking 2/1.
pub fn hornet_cobra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature("Hornet Cobra", cost(&[generic(1), g(), g()]), vec![CreatureType::Snake], 2, 1)
    }
}

/// Cat Warriors — forestwalking beaters.
pub fn cat_warriors() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(crate::card::LandType::Forest)],
        ..creature(
            "Cat Warriors",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Devouring Deep — an islandwalking 1/2.
pub fn devouring_deep() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(crate::card::LandType::Island)],
        ..creature(
            "Devouring Deep",
            cost(&[generic(2), crate::mana::u()]),
            vec![CreatureType::Fish],
            1,
            2,
        )
    }
}

/// Wall of Earth — a 0/6 roadblock.
pub fn wall_of_earth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        ..creature(
            "Wall of Earth",
            cost(&[generic(1), crate::mana::r()]),
            vec![CreatureType::Wall],
            0,
            6,
        )
    }
}

/// Walking Dead — a regenerating 1/1.
pub fn walking_dead() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Walking Dead",
            cost(&[generic(1), crate::mana::b()]),
            vec![CreatureType::Zombie],
            1,
            1,
        )
    }
}

/// Amrou Kithkin — small blockers can't stop it.
pub fn amrou_kithkin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(3)],
        ..creature(
            "Amrou Kithkin",
            cost(&[crate::mana::w(), crate::mana::w()]),
            vec![CreatureType::Kithkin],
            1,
            1,
        )
    }
}

/// Emerald Dragonfly — a flier that can trade up.
pub fn emerald_dragonfly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Emerald Dragonfly", cost(&[generic(1), g()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Fire Sprites — a flier that taps for red.
pub fn fire_sprites() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g()]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red]),
            },
            ..Default::default()
        }],
        ..creature("Fire Sprites", cost(&[generic(1), g()]), vec![CreatureType::Faerie], 1, 1)
    }
}

/// Ghosts of the Damned — a repeatable shrink.
pub fn ghosts_of_the_damned() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Ghosts of the Damned",
            cost(&[generic(1), crate::mana::b(), crate::mana::b()]),
            vec![CreatureType::Spirit],
            0,
            2,
        )
    }
}

/// Relic Barrier — a repeatable artifact tapper.
pub fn relic_barrier() -> CardDefinition {
    CardDefinition {
        name: "Relic Barrier",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flash Counter — a counterspell for instants only.
pub fn flash_counter() -> CardDefinition {
    instant(
        "Flash Counter",
        cost(&[generic(1), crate::mana::u()]),
        Effect::CounterSpell {
            what: target_filtered(
                R::IsSpellOnStack.and(R::HasCardType(CardType::Instant)),
            ),
        },
    )
}

/// Remove Soul — a counterspell for creatures only.
pub fn remove_soul() -> CardDefinition {
    instant(
        "Remove Soul",
        cost(&[generic(1), crate::mana::u()]),
        Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::Creature)),
        },
    )
}

/// Shield Wall — a team toughness pump.
pub fn shield_wall() -> CardDefinition {
    instant(
        "Shield Wall",
        cost(&[generic(1), crate::mana::w()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ZERO,
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Hell Swarm — a board-wide power shave.
pub fn hell_swarm() -> CardDefinition {
    instant(
        "Hell Swarm",
        cost(&[crate::mana::b()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Const(-1),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Divine Offering — Disenchant that pays you back.
pub fn divine_offering() -> CardDefinition {
    instant(
        "Divine Offering",
        cost(&[generic(1), crate::mana::w()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
        ]),
    )
}

/// Great Defender — a toughness pump the size of the creature.
pub fn great_defender() -> CardDefinition {
    instant(
        "Great Defender",
        cost(&[crate::mana::w()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::ZERO,
            toughness: Value::ManaValueOf(Box::new(Selector::Target(0))),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Winds of Change — everyone reshuffles and redraws.
pub fn winds_of_change() -> CardDefinition {
    CardDefinition {
        name: "Winds of Change",
        cost: cost(&[crate::mana::r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ShuffleHandsDrawSame { who: PlayerRef::EachPlayer },
        ..Default::default()
    }
}

/// Field of Dreams — everyone plays off a revealed top card.
pub fn field_of_dreams() -> CardDefinition {
    CardDefinition {
        name: "Field of Dreams",
        cost: cost(&[generic(2), crate::mana::u()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::World],
        static_abilities: vec![StaticAbility {
            description: "Players play with the top card of their libraries revealed.",
            effect: StaticEffect::AllLibraryTopsRevealed,
        }],
        ..Default::default()
    }
}

/// Gaseous Form — the creature stops mattering in combat.
pub fn gaseous_form() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to and dealt by \
                          enchanted creature.",
            effect: StaticEffect::PreventAllCombatDamageToAndFromEnchanted,
        }],
        ..aura_shell("Gaseous Form", cost(&[generic(2), crate::mana::u()]), EquipBonus::default())
    }
}

/// Anti-Magic Aura — the creature can't be targeted.
pub fn anti_magic_aura() -> CardDefinition {
    aura_shell(
        "Anti-Magic Aura",
        cost(&[generic(2), crate::mana::u()]),
        EquipBonus { keywords: vec![Keyword::Shroud], ..Default::default() },
    )
}

/// Immolation — a brutal trade of toughness for power.
pub fn immolation() -> CardDefinition {
    aura_shell(
        "Immolation",
        cost(&[crate::mana::r()]),
        EquipBonus { power: 2, toughness: -2, ..Default::default() },
    )
}

/// Eternal Warrior — vigilance in Aura form.
pub fn eternal_warrior() -> CardDefinition {
    aura_shell(
        "Eternal Warrior",
        cost(&[crate::mana::r()]),
        EquipBonus { keywords: vec![Keyword::Vigilance], ..Default::default() },
    )
}

/// The Brute — a pump Aura with a regeneration button.
pub fn the_brute() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::r(), crate::mana::r(), crate::mana::r()]),
            effect: Effect::Regenerate { what: host() },
            ..Default::default()
        }],
        ..aura_shell(
            "The Brute",
            cost(&[generic(1), crate::mana::r()]),
            EquipBonus { power: 1, ..Default::default() },
        )
    }
}

/// "Enchant creature" with an `EquipBonus` body.
fn aura_shell(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// The permanent this Aura is attached to.
fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}
