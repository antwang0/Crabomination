//! Urza's Destiny (UDS) gap closure. Tests in `classic_sets/uds`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, Value, WardCost,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{deal, on_dies, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::effect::ManaPayload;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
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

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn artifact(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// An Aura that attaches to a creature and carries `bonus`.
fn aura(name: &'static str, c: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// `{2}, Sacrifice this: Draw a card.` — the UDS cantrip-on-death shape.
fn sac_to_draw() -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        sac_cost: true,
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        ..Default::default()
    }
}

// ── Vanilla-ish creatures ───────────────────────────────────────────────────

/// Goblin Berserker — {3}{R} 2/2 with first strike and haste.
pub fn goblin_berserker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        ..creature(
            "Goblin Berserker",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Berserker],
            2,
            2,
        )
    }
}

/// Goliath Beetle — {2}{G} 3/1 trampler.
pub fn goliath_beetle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Goliath Beetle", cost(&[generic(2), g()]), vec![CreatureType::Insect], 3, 1)
    }
}

/// Wild Colos — {2}{R} 2/2 with haste.
pub fn wild_colos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..creature(
            "Wild Colos",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goat, CreatureType::Beast],
            2,
            2,
        )
    }
}

/// Tormented Angel — {3}{W} 1/5 flier.
pub fn tormented_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Tormented Angel", cost(&[generic(3), w()]), vec![CreatureType::Angel], 1, 5)
    }
}

/// Plated Spider — {4}{G} 4/4 with reach.
pub fn plated_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature("Plated Spider", cost(&[generic(4), g()]), vec![CreatureType::Spider], 4, 4)
    }
}

/// Squirming Mass — {1}{B} 1/1 with fear.
pub fn squirming_mass() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        ..creature("Squirming Mass", cost(&[generic(1), b()]), vec![CreatureType::Horror], 1, 1)
    }
}

/// Elvish Lookout — {G} 1/1 with shroud.
pub fn elvish_lookout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        ..creature("Elvish Lookout", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Hulking Ogre — {2}{R} 3/3 that can't block.
pub fn hulking_ogre() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        ..creature("Hulking Ogre", cost(&[generic(2), r()]), vec![CreatureType::Ogre], 3, 3)
    }
}

/// Metathran Soldier — {1}{U} 1/1 that can't be blocked.
pub fn metathran_soldier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        ..creature(
            "Metathran Soldier",
            cost(&[generic(1), u()]),
            vec![CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Voice of Duty — {3}{W} 2/2 flier with protection from green.
pub fn voice_of_duty() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Green)],
        ..creature("Voice of Duty", cost(&[generic(3), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

/// Voice of Reason — {3}{W} 2/2 flier with protection from blue.
pub fn voice_of_reason() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Blue)],
        ..creature("Voice of Reason", cost(&[generic(3), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

/// Thorn Elemental — {5}{G}{G} 7/7 that may assign damage as though unblocked.
pub fn thorn_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
        ..creature(
            "Thorn Elemental",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Elemental],
            7,
            7,
        )
    }
}

/// Taunting Elf — {G} 0/1 that every able blocker must block.
pub fn taunting_elf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBeBlocked],
        ..creature("Taunting Elf", cost(&[g()]), vec![CreatureType::Elf], 0, 1)
    }
}

/// Phyrexian Monitor — {3}{B} 2/2 that regenerates for {B}.
pub fn phyrexian_monitor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Phyrexian Monitor",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Skeleton],
            2,
            2,
        )
    }
}

/// Ancient Silverback — {4}{G}{G} 6/5 that regenerates for {G}.
pub fn ancient_silverback() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Ancient Silverback", cost(&[generic(4), g(), g()]), vec![CreatureType::Ape], 6, 5)
    }
}

// ── Firebreathing-style pumps ───────────────────────────────────────────────

fn pump_ability(c: crate::mana::ManaCost, power: i32, toughness: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Capashen Knight — {1}{W} 1/1 first striker that pumps for {1}{W}.
pub fn capashen_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![pump_ability(cost(&[generic(1), w()]), 1, 0)],
        ..creature(
            "Capashen Knight",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            1,
            1,
        )
    }
}

/// Capashen Templar — {2}{W} 2/2 that toughens for {W}.
pub fn capashen_templar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![pump_ability(cost(&[w()]), 0, 1)],
        ..creature(
            "Capashen Templar",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Colos Yearling — {2}{R} 1/1 mountainwalker that pumps for {R}.
pub fn colos_yearling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        activated_abilities: vec![pump_ability(cost(&[r()]), 1, 0)],
        ..creature(
            "Colos Yearling",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goat, CreatureType::Beast],
            1,
            1,
        )
    }
}

/// Blizzard Elemental — {5}{U}{U} 5/5 flier that untaps itself for {3}{U}.
pub fn blizzard_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..creature(
            "Blizzard Elemental",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Elemental],
            5,
            5,
        )
    }
}

/// Mantis Engine — {5} 3/3 that buys flying or first strike for {2} each.
pub fn mantis_engine() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Mantis Engine", cost(&[generic(5)]), vec![CreatureType::Insect], 3, 3)
    }
}

// ── Sac-to-draw bodies ──────────────────────────────────────────────────────

/// Brass Secretary — {3} 2/1 Construct that cashes itself in for a card.
pub fn brass_secretary() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![sac_to_draw()],
        ..creature("Brass Secretary", cost(&[generic(3)]), vec![CreatureType::Construct], 2, 1)
    }
}

/// Slinking Skirge — {3}{B} 2/1 flier that cashes itself in for a card.
pub fn slinking_skirge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![sac_to_draw()],
        ..creature(
            "Slinking Skirge",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Imp],
            2,
            1,
        )
    }
}

/// Heart Warden — {1}{G} 1/1 mana Elf that cashes itself in for a card.
pub fn heart_warden() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Green, Value::ONE),
                },
                ..Default::default()
            },
            sac_to_draw(),
        ],
        ..creature(
            "Heart Warden",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Marker Beetles — {1}{G}{G} 2/3 that pumps a creature on death and can be
/// cashed in for a card.
pub fn marker_beetles() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![sac_to_draw()],
        ..creature("Marker Beetles", cost(&[generic(1), g(), g()]), vec![CreatureType::Insect], 2, 3)
    }
}

/// Plague Dogs — {4}{B} 3/3 whose death shrinks the whole board.
pub fn plague_dogs() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![sac_to_draw()],
        ..creature(
            "Plague Dogs",
            cost(&[generic(4), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Dog],
            3,
            3,
        )
    }
}

/// Kingfisher — {3}{U} 2/2 flier that replaces itself.
pub fn kingfisher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..creature("Kingfisher", cost(&[generic(3), u()]), vec![CreatureType::Bird], 2, 2)
    }
}

// ── Other death triggers ────────────────────────────────────────────────────

/// Disease Carriers — {2}{B}{B} 2/2 whose death shrinks a creature.
pub fn disease_carriers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Disease Carriers", cost(&[generic(2), b(), b()]), vec![CreatureType::Rat], 2, 2)
    }
}

/// Goblin Gardener — {3}{R} 2/1 whose death breaks a land.
pub fn goblin_gardener() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Destroy { what: target_filtered(R::Land) })],
        ..creature("Goblin Gardener", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 2, 1)
    }
}

/// Goblin Masons — {1}{R} 2/1 whose death breaks a Wall.
pub fn goblin_masons() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Destroy {
            what: target_filtered(R::HasCreatureType(CreatureType::Wall)),
        })],
        ..creature("Goblin Masons", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 2, 1)
    }
}

/// Reliquary Monk — {2}{W} 2/2 whose death breaks an artifact or enchantment.
pub fn reliquary_monk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Destroy {
            what: target_filtered(R::Artifact.or(R::Enchantment)),
        })],
        ..creature(
            "Reliquary Monk",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Monk, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// False Prophet — {2}{W}{W} 2/2 whose death exiles the board.
pub fn false_prophet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Move {
            what: Selector::EachPermanent(R::Creature),
            to: ZoneDest::Exile,
        })],
        ..creature(
            "False Prophet",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Aura Thief — {3}{U} 2/2 flier whose death hands you every enchantment.
pub fn aura_thief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::GainControl {
            what: Selector::EachPermanent(R::Enchantment),
            to: None,
            duration: Duration::Permanent,
        })],
        ..creature("Aura Thief", cost(&[generic(3), u()]), vec![CreatureType::Illusion], 2, 2)
    }
}

// ── "Sacrifice this when you control no …" ──────────────────────────────────

/// Emperor Crocodile — {3}{G} 5/5 that needs company.
pub fn emperor_crocodile() -> CardDefinition {
    CardDefinition {
        sacrifice_when_you_control_no_other: Some(R::Creature),
        ..creature("Emperor Crocodile", cost(&[generic(3), g()]), vec![CreatureType::Crocodile], 5, 5)
    }
}

/// Covetous Dragon — {4}{R} 6/5 flier that needs an artifact.
pub fn covetous_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        sacrifice_when_you_control_no_other: Some(R::Artifact),
        ..creature("Covetous Dragon", cost(&[generic(4), r()]), vec![CreatureType::Dragon], 6, 5)
    }
}

/// Tethered Griffin — {W} 2/3 flier that needs an enchantment.
pub fn tethered_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        sacrifice_when_you_control_no_other: Some(R::Enchantment),
        ..creature("Tethered Griffin", cost(&[w()]), vec![CreatureType::Griffin], 2, 3)
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Braidwood Cup — {3}. Taps for a life.
pub fn braidwood_cup() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Braidwood Cup", cost(&[generic(3)]))
    }
}

/// Braidwood Sextant — {1}. Cashes itself in for a basic land.
pub fn braidwood_sextant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..artifact("Braidwood Sextant", cost(&[generic(1)]))
    }
}

/// Caltrops — {3}. Pricks every attacker.
pub fn caltrops() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer),
            effect: deal(1, Selector::TriggerSource),
        }],
        ..artifact("Caltrops", cost(&[generic(3)]))
    }
}

/// Fodder Cannon — {4}. Feed it a creature to burn one.
pub fn fodder_cannon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: deal(4, target_filtered(R::Creature)),
            ..Default::default()
        }],
        ..artifact("Fodder Cannon", cost(&[generic(4)]))
    }
}

/// Thran Foundry — {1}. Shuffles a graveyard away.
pub fn thran_foundry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..artifact("Thran Foundry", cost(&[generic(1)]))
    }
}

/// Powder Keg — {2}. Ticks up each upkeep, then sweeps everything at its size.
pub fn powder_keg() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Put a fuse counter on Powder Keg".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Fuse,
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DestroyEachMatchingWithManaValue {
                filter: R::Artifact.or(R::Creature),
                value: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Fuse,
                },
            },
            ..Default::default()
        }],
        ..artifact("Powder Keg", cost(&[generic(2)]))
    }
}

/// Metalworker — {3} 1/2 that turns a fistful of artifacts into a pile of mana.
pub fn metalworker() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::RevealAnyNumberFromHand {
                filter: R::Artifact,
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Times(
                        Box::new(Value::CardsRevealedThisEffect),
                        Box::new(Value::Const(2)),
                    )),
                }),
            },
            ..Default::default()
        }],
        ..creature("Metalworker", cost(&[generic(3)]), vec![CreatureType::Construct], 1, 2)
    }
}

/// Masticore — {4} 4/4 that eats a card each upkeep, pings, and regenerates.
pub fn masticore() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: WardCost::Discard(1),
                then: Box::new(Effect::SacrificeSource),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: deal(1, target_filtered(R::Creature)),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
        ],
        ..creature("Masticore", cost(&[generic(4)]), vec![CreatureType::Masticore], 4, 4)
    }
}

/// Extruder — {4} 4/3 with echo that grinds artifacts into counters.
pub fn extruder() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Echo(cost(&[generic(4)]))],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Extruder", cost(&[generic(4)]), vec![CreatureType::Juggernaut], 4, 3)
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Aether Sting — {3}{R}. Every opposing creature spell costs its caster a
/// point of life.
pub fn aether_sting() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: deal(1, Selector::Player(PlayerRef::Triggerer)),
        }],
        ..enchantment("Aether Sting", cost(&[generic(3), r()]))
    }
}

/// Carnival of Souls — {1}{B}. Every creature that enters bleeds you for a
/// black mana.
pub fn carnival_of_souls() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Black, Value::ONE),
                },
            ]),
        }],
        ..enchantment("Carnival of Souls", cost(&[generic(1), b()]))
    }
}

/// Attrition — {1}{B}. Turns spare bodies into removal.
pub fn attrition() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            ..Default::default()
        }],
        ..enchantment("Attrition", cost(&[generic(1), b(), b()]))
    }
}

/// Mental Discipline — {1}{U}{U}. Loots at instant speed.
pub fn mental_discipline() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..enchantment("Mental Discipline", cost(&[generic(1), u(), u()]))
    }
}

/// Yawgmoth's Bargain — {4}{B}{B}. Your draw step is gone; every card costs a
/// life instead.
pub fn yawgmoths_bargain() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::ControllerSkipsDrawStep,
        }],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..enchantment("Yawgmoth's Bargain", cost(&[generic(4), b(), b()]))
    }
}

/// Impatience — {2}{R}. Punishes a player who spent the turn doing nothing.
pub fn impatience() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(
                    Predicate::SpellsCastThisTurnAtLeast {
                        who: PlayerRef::ActivePlayer,
                        at_least: Value::ONE,
                    },
                ))),
            effect: deal(2, Selector::Player(PlayerRef::ActivePlayer)),
        }],
        ..enchantment("Impatience", cost(&[generic(2), r()]))
    }
}

/// Repercussion — {1}{R}{R}. Every point a creature takes is echoed onto its
/// controller.
pub fn repercussion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..enchantment("Repercussion", cost(&[generic(1), r(), r()]))
    }
}

/// Lurking Jackals — {B}. Wakes up as a 3/2 once an opponent is low.
pub fn lurking_jackals() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl).with_filter(
                Predicate::PlayerLifeAtMost { who: PlayerRef::Triggerer, life: 10 },
            ),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(2),
                creature_types: vec![CreatureType::Jackal],
                keywords: vec![],
                duration: Duration::Permanent,
            },
        }],
        ..enchantment("Lurking Jackals", cost(&[b()]))
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Twisted Experiment — {1}{B}. +3/-1.
pub fn twisted_experiment() -> CardDefinition {
    aura(
        "Twisted Experiment",
        cost(&[generic(1), b()]),
        EquipBonus { power: 3, toughness: -1, ..Default::default() },
    )
}

/// Mask of Law and Grace — {W}. Protection from black and from red.
pub fn mask_of_law_and_grace() -> CardDefinition {
    aura(
        "Mask of Law and Grace",
        cost(&[w()]),
        EquipBonus {
            keywords: vec![Keyword::Protection(Color::Black), Keyword::Protection(Color::Red)],
            ..Default::default()
        },
    )
}

/// Illuminated Wings — {1}{U}. Flying, and a card when you're done with it.
pub fn illuminated_wings() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_to_draw()],
        ..aura(
            "Illuminated Wings",
            cost(&[generic(1), u()]),
            EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() },
        )
    }
}

/// Capashen Standard — {W}. +1/+1, and a card when you're done with it.
pub fn capashen_standard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_to_draw()],
        ..aura(
            "Capashen Standard",
            cost(&[w()]),
            EquipBonus { power: 1, toughness: 1, ..Default::default() },
        )
    }
}

/// Momentum — {2}{G}. Grows the host by a counter each upkeep.
pub fn momentum() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Put a growth counter on Momentum".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Growth,
                    amount: Value::ONE,
                }),
            },
        }],
        ..aura(
            "Momentum",
            cost(&[generic(2), g()]),
            EquipBonus {
                scale: Some(EquipScale {
                    filter: R::Any,
                    per_power: 1,
                    per_toughness: 1,
                    count_self_counters: Some(CounterType::Growth),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

/// Mark of Fury — {R}. Haste for a turn, then it comes back for another.
pub fn mark_of_fury() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
        }],
        ..aura(
            "Mark of Fury",
            cost(&[r()]),
            EquipBonus { keywords: vec![Keyword::Haste], ..Default::default() },
        )
    }
}

/// Chime of Night — {1}{B}. Kills a nonblack creature when it falls off.
pub fn chime_of_night() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
        })],
        ..aura("Chime of Night", cost(&[generic(1), b()]), EquipBonus::default())
    }
}

/// Dying Wail — {1}{B}. The host's death costs a player two cards.
pub fn dying_wail() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_dies(Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            })],
            ..Default::default()
        }),
        ..aura("Dying Wail", cost(&[generic(1), b()]), EquipBonus::default())
    }
}

/// Private Research — {U}. Banks page counters, cashed in when the host dies.
pub fn private_research() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Put a page counter on Private Research".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Page,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::EnchantedBySource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Page,
                    },
                },
            },
        ],
        ..aura("Private Research", cost(&[u()]), EquipBonus::default())
    }
}

/// Incendiary — {R}. Banks fuse counters, then explodes when the host dies.
pub fn incendiary() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Put a fuse counter on Incendiary".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Fuse,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::EnchantedBySource),
                effect: Effect::DealDamage {
                    to: target_any(),
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Fuse,
                    },
                },
            },
        ],
        ..aura("Incendiary", cost(&[r()]), EquipBonus::default())
    }
}

/// Festering Wound — {1}{B}. Bleeds the host's controller for its counters.
pub fn festering_wound() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Put an infection counter on Festering Wound".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Infection,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(
                        Box::new(Selector::This),
                    )))),
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Infection,
                    },
                },
            },
        ],
        ..aura("Festering Wound", cost(&[generic(1), b()]), EquipBonus::default())
    }
}

/// Disappear — {2}{U}{U}. Bounces the host (and itself) on demand.
pub fn disappear() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::AttachedTo(Box::new(Selector::This))))),
                },
                Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ]),
            ..Default::default()
        }],
        ..aura("Disappear", cost(&[generic(2), u(), u()]), EquipBonus::default())
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Magnify — {G}. Everything gets +1/+1.
pub fn magnify() -> CardDefinition {
    instant(
        "Magnify",
        cost(&[g()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Solidarity — {3}{W}. Your team gets +0/+5.
pub fn solidarity() -> CardDefinition {
    instant(
        "Solidarity",
        cost(&[generic(3), w()]),
        Effect::PumpPT {
            what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            power: Value::ZERO,
            toughness: Value::Const(5),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Rescue — {U}. Save a permanent by bouncing it.
pub fn rescue() -> CardDefinition {
    instant(
        "Rescue",
        cost(&[u()]),
        Effect::Move {
            what: target_filtered(R::ControlledByYou),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
    )
}

/// Donate — {2}{U}. Hand a permanent to someone else.
pub fn donate() -> CardDefinition {
    sorcery(
        "Donate",
        cost(&[generic(2), u()]),
        Effect::GainControl {
            what: Selector::TargetFiltered { slot: 1, filter: R::ControlledByYou },
            to: Some(PlayerRef::Target(0)),
            duration: Duration::Permanent,
        },
    )
}

/// Flicker — {1}{W}. Blink a nontoken permanent.
pub fn flicker() -> CardDefinition {
    sorcery(
        "Flicker",
        cost(&[generic(1), w()]),
        Effect::ExileAndReturnToOwner {
            what: target_filtered(R::Not(Box::new(R::IsToken))),
        },
    )
}

/// Multani's Decree — {3}{G}. Wraths enchantments and pays you for each.
pub fn multanis_decree() -> CardDefinition {
    sorcery(
        "Multani's Decree",
        cost(&[generic(3), g()]),
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Enchantment) },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::PermanentsDestroyedThisResolution),
                    Box::new(Value::Const(2)),
                ),
            },
        ]),
    )
}

/// Replenish — {3}{W}. Every enchantment in your graveyard comes back.
pub fn replenish() -> CardDefinition {
    sorcery(
        "Replenish",
        cost(&[generic(3), w()]),
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: R::Enchantment,
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Plow Under — {3}{G}{G}. Two lands go back on top.
pub fn plow_under() -> CardDefinition {
    CardDefinition {
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Land,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: crate::effect::LibraryPosition::Top,
                },
            }),
        },
        ..sorcery("Plow Under", cost(&[generic(3), g(), g()]), Effect::Noop)
    }
}

/// Landslide — {R}. Turn Mountains into damage.
pub fn landslide() -> CardDefinition {
    CardDefinition {
        effect: Effect::Seq(vec![
            Effect::SacrificeAnyNumber {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Mountain),
                per_each: Box::new(Effect::Noop),
            },
            Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::SacrificedCount,
            },
        ]),
        ..sorcery("Landslide", cost(&[r()]), Effect::Noop)
    }
}

/// Encroach — {B}. Strip a nonbasic land out of a hand.
pub fn encroach() -> CardDefinition {
    sorcery(
        "Encroach",
        cost(&[b()]),
        Effect::RevealHandDiscardAllMatching {
            who: PlayerRef::Target(0),
            filter: R::Land.and(R::Not(Box::new(R::IsBasicLand))),
        },
    )
}

// ── The Scent cycle (reveal any number of <color> cards) ────────────────────

/// Scent of Cinder — {1}{R}. X damage for X red cards revealed.
pub fn scent_of_cinder() -> CardDefinition {
    sorcery(
        "Scent of Cinder",
        cost(&[generic(1), r()]),
        Effect::RevealAnyNumberFromHand {
            filter: R::HasColor(Color::Red),
            then: Box::new(Effect::DealDamage {
                to: target_any(),
                amount: Value::CardsRevealedThisEffect,
            }),
        },
    )
}

/// Scent of Jasmine — {W}. Two life per white card revealed.
pub fn scent_of_jasmine() -> CardDefinition {
    instant(
        "Scent of Jasmine",
        cost(&[w()]),
        Effect::RevealAnyNumberFromHand {
            filter: R::HasColor(Color::White),
            then: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::CardsRevealedThisEffect),
                    Box::new(Value::Const(2)),
                ),
            }),
        },
    )
}

/// Scent of Ivy — {G}. +X/+X for X green cards revealed.
pub fn scent_of_ivy() -> CardDefinition {
    instant(
        "Scent of Ivy",
        cost(&[g()]),
        Effect::RevealAnyNumberFromHand {
            filter: R::HasColor(Color::Green),
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::CardsRevealedThisEffect,
                toughness: Value::CardsRevealedThisEffect,
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Scent of Nightshade — {1}{B}. -X/-X for X black cards revealed.
pub fn scent_of_nightshade() -> CardDefinition {
    instant(
        "Scent of Nightshade",
        cost(&[generic(1), b()]),
        Effect::RevealAnyNumberFromHand {
            filter: R::HasColor(Color::Black),
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Diff(Box::new(Value::ZERO), Box::new(Value::CardsRevealedThisEffect)),
                toughness: Value::Diff(
                    Box::new(Value::ZERO),
                    Box::new(Value::CardsRevealedThisEffect),
                ),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Scent of Brine — {1}{U}. A soft counter scaled by revealed blue cards.
pub fn scent_of_brine() -> CardDefinition {
    instant(
        "Scent of Brine",
        cost(&[generic(1), u()]),
        Effect::RevealAnyNumberFromHand {
            filter: R::HasColor(Color::Blue),
            then: Box::new(Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[]),
                exile: false,
                extra_generic: Some(Value::CardsRevealedThisEffect),
            }),
        },
    )
}

/// Rofellos's Gift — {G}. Rebuy an enchantment per green card revealed.
pub fn rofellos_gift() -> CardDefinition {
    sorcery(
        "Rofellos's Gift",
        cost(&[g()]),
        Effect::RevealAnyNumberFromHand {
            filter: R::HasColor(Color::Green),
            then: Box::new(Effect::MoveChosen {
                from: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Enchantment,
                },
                filter: None,
                count: Value::CardsRevealedThisEffect,
                up_to: true,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    )
}

// ── The Seer cycle (the Scents on a body) ──────────────────────────────────

fn seer(
    name: &'static str,
    c: crate::mana::ManaCost,
    activation: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    filter: R,
    then: Effect,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: activation,
            tap_cost: true,
            effect: Effect::RevealAnyNumberFromHand { filter, then: Box::new(then) },
            ..Default::default()
        }],
        ..creature(name, c, types, 1, 1)
    }
}

/// Cinder Seer — {3}{R} 1/1. Reveal red cards, deal that much.
pub fn cinder_seer() -> CardDefinition {
    seer(
        "Cinder Seer",
        cost(&[generic(3), r()]),
        cost(&[generic(2), r()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        R::HasColor(Color::Red),
        Effect::DealDamage { to: target_any(), amount: Value::CardsRevealedThisEffect },
    )
}

/// Jasmine Seer — {3}{W} 1/1. Reveal white cards, gain two life each.
pub fn jasmine_seer() -> CardDefinition {
    seer(
        "Jasmine Seer",
        cost(&[generic(3), w()]),
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        R::HasColor(Color::White),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(Box::new(Value::CardsRevealedThisEffect), Box::new(Value::Const(2))),
        },
    )
}

/// Nightshade Seer — {3}{B} 1/1. Reveal black cards, shrink a creature.
pub fn nightshade_seer() -> CardDefinition {
    seer(
        "Nightshade Seer",
        cost(&[generic(3), b()]),
        cost(&[generic(2), b()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        R::HasColor(Color::Black),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Diff(Box::new(Value::ZERO), Box::new(Value::CardsRevealedThisEffect)),
            toughness: Value::Diff(Box::new(Value::ZERO), Box::new(Value::CardsRevealedThisEffect)),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Ivy Seer — {3}{G} 1/1. Reveal green cards, pump a creature.
pub fn ivy_seer() -> CardDefinition {
    seer(
        "Ivy Seer",
        cost(&[generic(3), g()]),
        cost(&[generic(2), g()]),
        vec![CreatureType::Elf, CreatureType::Wizard],
        R::HasColor(Color::Green),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::CardsRevealedThisEffect,
            toughness: Value::CardsRevealedThisEffect,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Brine Seer — {3}{U} 1/1. Reveal blue cards, tax a spell by that much.
pub fn brine_seer() -> CardDefinition {
    seer(
        "Brine Seer",
        cost(&[generic(3), u()]),
        cost(&[generic(2), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        R::HasColor(Color::Blue),
        Effect::CounterUnlessPaid {
            what: Selector::Target(0),
            mana_cost: cost(&[]),
            exile: false,
            extra_generic: Some(Value::CardsRevealedThisEffect),
        },
    )
}

// ── The name-hate cycle ─────────────────────────────────────────────────────

/// Eradicate — {2}{B}{B}. A nonblack creature and every copy of it.
pub fn eradicate() -> CardDefinition {
    sorcery(
        "Eradicate",
        cost(&[generic(2), b(), b()]),
        Effect::ExileAllCopiesOfTargetName {
            what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
        },
    )
}

/// Scour — {2}{W}{W}. An enchantment and every copy of it.
pub fn scour() -> CardDefinition {
    instant(
        "Scour",
        cost(&[generic(2), w(), w()]),
        Effect::ExileAllCopiesOfTargetName { what: target_filtered(R::Enchantment) },
    )
}

/// Sowing Salt — {2}{R}{R}. A nonbasic land and every copy of it.
pub fn sowing_salt() -> CardDefinition {
    sorcery(
        "Sowing Salt",
        cost(&[generic(2), r(), r()]),
        Effect::ExileAllCopiesOfTargetName {
            what: target_filtered(R::Land.and(R::Not(Box::new(R::IsBasicLand)))),
        },
    )
}

/// Splinter — {2}{G}{G}. An artifact and every copy of it.
pub fn splinter() -> CardDefinition {
    sorcery(
        "Splinter",
        cost(&[generic(2), g(), g()]),
        Effect::ExileAllCopiesOfTargetName { what: target_filtered(R::Artifact) },
    )
}

/// Quash — {2}{U}{U}. An instant or sorcery and every copy of it.
pub fn quash() -> CardDefinition {
    instant(
        "Quash",
        cost(&[generic(2), u(), u()]),
        Effect::ExileAllCopiesOfTargetName {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            },
        },
    )
}
