//! Multi-block batch (CR 509.1b): the printed "can block an additional
//! creature" / "can block any number of creatures" cards not already in the
//! catalog, plus their granters and payoffs. Tests in `recent_b/recent_306`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{embalm, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, StaticAbility, StaticEffect};
use crate::mana::{cost, g, generic, r, w};

/// "Creatures you control have [keyword]." — the static anthem shape.
fn your_creatures_have(keyword: Keyword, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword,
        },
    }
}

// ── "Can block any number of creatures" ──────────────────────────────────────

/// Palace Guard — {2}{W} 1/4 Human Soldier that can block any number of
/// creatures.
pub fn palace_guard() -> CardDefinition {
    CardDefinition {
        name: "Palace Guard",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::CanBlockAnyNumber],
        ..Default::default()
    }
}

/// Wall of Glare — {1}{W} 0/5 Wall with defender that can block any number of
/// creatures.
pub fn wall_of_glare() -> CardDefinition {
    CardDefinition {
        name: "Wall of Glare",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Defender, Keyword::CanBlockAnyNumber],
        ..Default::default()
    }
}

/// Avatar of Hope — {6}{W}{W} 4/9 Avatar with flying that can block any number
/// of creatures. Costs {6} less while you're at 3 or less life.
pub fn avatar_of_hope() -> CardDefinition {
    CardDefinition {
        name: "Avatar of Hope",
        cost: cost(&[generic(6), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 4,
        toughness: 9,
        keywords: vec![Keyword::Flying, Keyword::CanBlockAnyNumber],
        static_abilities: vec![StaticAbility {
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 6,
                condition: Predicate::PlayerLifeAtMost {
                    who: PlayerRef::You,
                    life: 3,
                },
            },
            description: "If you have 3 or less life, this spell costs {6} less to cast.",
        }],
        ..Default::default()
    }
}

/// Ironfist Crusher — {4}{W} 2/4 Human Soldier that can block any number of
/// creatures. Morph {3}{W}.
pub fn ironfist_crusher() -> CardDefinition {
    CardDefinition {
        name: "Ironfist Crusher",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![
            Keyword::CanBlockAnyNumber,
            Keyword::Morph(cost(&[generic(3), w()])),
        ],
        ..Default::default()
    }
}

// ── "Can block an additional creature each combat" ───────────────────────────

/// Foriysian Brigade — {3}{W} 2/4 Human Soldier that can block an additional
/// creature each combat.
pub fn foriysian_brigade() -> CardDefinition {
    CardDefinition {
        name: "Foriysian Brigade",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::CanBlockAdditional(1)],
        ..Default::default()
    }
}

/// Foriysian Interceptor — {3}{W} 0/5 Human Soldier with flash and defender
/// that can block an additional creature each combat.
pub fn foriysian_interceptor() -> CardDefinition {
    CardDefinition {
        name: "Foriysian Interceptor",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        keywords: vec![
            Keyword::Flash,
            Keyword::Defender,
            Keyword::CanBlockAdditional(1),
        ],
        ..Default::default()
    }
}

/// Selesnya Sagittars — {3}{G}{W} 2/5 Elf Archer with reach that can block an
/// additional creature each combat.
pub fn selesnya_sagittars() -> CardDefinition {
    CardDefinition {
        name: "Selesnya Sagittars",
        cost: cost(&[generic(3), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::CanBlockAdditional(1)],
        ..Default::default()
    }
}

/// Spike-Tailed Ceratops — {4}{G} 4/4 Dinosaur that can block an additional
/// creature each combat.
pub fn spike_tailed_ceratops() -> CardDefinition {
    CardDefinition {
        name: "Spike-Tailed Ceratops",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::CanBlockAdditional(1)],
        ..Default::default()
    }
}

/// Two-Headed Giant of Foriys — {4}{R} 4/4 Giant with trample that can block an
/// additional creature each combat.
pub fn two_headed_giant_of_foriys() -> CardDefinition {
    CardDefinition {
        name: "Two-Headed Giant of Foriys",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::CanBlockAdditional(1)],
        ..Default::default()
    }
}

/// Ghastbark Twins — {5}{G}{G} 7/7 Treefolk with trample that can block an
/// additional creature each combat.
pub fn ghastbark_twins() -> CardDefinition {
    CardDefinition {
        name: "Ghastbark Twins",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample, Keyword::CanBlockAdditional(1)],
        ..Default::default()
    }
}

/// Night Market Guard — {3} 3/1 Construct artifact creature that can block an
/// additional creature each combat.
pub fn night_market_guard() -> CardDefinition {
    CardDefinition {
        name: "Night Market Guard",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::CanBlockAdditional(1)],
        ..Default::default()
    }
}

/// Two-Headed Dragon — {4}{R}{R} 4/4 Dragon with flying and menace that can
/// block an additional creature each combat. {1}{R}: +2/+0.
pub fn two_headed_dragon() -> CardDefinition {
    CardDefinition {
        name: "Two-Headed Dragon",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Flying,
            Keyword::Menace,
            Keyword::CanBlockAdditional(1),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Trueheart Duelist — {1}{W} 2/2 Human Warrior that can block an additional
/// creature each combat. Embalm {2}{W}.
pub fn trueheart_duelist() -> CardDefinition {
    CardDefinition {
        name: "Trueheart Duelist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CanBlockAdditional(1)],
        activated_abilities: vec![embalm(cost(&[generic(2), w()]))],
        ..Default::default()
    }
}

/// Kemba's Legion — {5}{W}{W} 4/6 Cat Soldier with vigilance that can block an
/// additional creature each combat for each Equipment attached to it.
pub fn kembas_legion() -> CardDefinition {
    CardDefinition {
        name: "Kemba's Legion",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            effect: StaticEffect::SelfCanBlockAdditionalPerAttachedEquipment,
            description: "This creature can block an additional creature each combat for \
                          each Equipment attached to this creature.",
        }],
        ..Default::default()
    }
}

/// Entourage of Trest — {4}{G} 4/4 Elf Soldier. ETB: become the monarch; can
/// block an additional creature each combat while you're the monarch.
pub fn entourage_of_trest() -> CardDefinition {
    CardDefinition {
        name: "Entourage of Trest",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::BecomeMonarch {
                who: PlayerRef::You,
            },
        }],
        static_abilities: vec![StaticAbility {
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::CanBlockAdditional(1),
                condition: Predicate::IsMonarch {
                    who: PlayerRef::You,
                },
            },
            description: "This creature can block an additional creature each combat as \
                          long as you're the monarch.",
        }],
        ..Default::default()
    }
}

// ── Activated / one-shot grants ──────────────────────────────────────────────

/// Anurid Swarmsnapper — {2}{G} 1/4 Frog Beast with reach. {1}{G}: can block an
/// additional creature this turn.
pub fn anurid_swarmsnapper() -> CardDefinition {
    CardDefinition {
        name: "Anurid Swarmsnapper",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: block_additional_self(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mounted Archers — {3}{W} 2/3 Human Soldier Archer with reach. {W}: can block
/// an additional creature this turn.
pub fn mounted_archers() -> CardDefinition {
    CardDefinition {
        name: "Mounted Archers",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Archer,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: block_additional_self(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Luminous Guardian — {3}{W} 1/4 Human Nomad. {W}: +0/+1; {2}: can block an
/// additional creature this turn.
pub fn luminous_guardian() -> CardDefinition {
    CardDefinition {
        name: "Luminous Guardian",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Nomad],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(0),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: block_additional_self(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Act of Heroism — {1}{W} Instant. Untap target creature; it gets +2/+2 and
/// can block an additional creature this turn.
pub fn act_of_heroism() -> CardDefinition {
    CardDefinition {
        name: "Act of Heroism",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: target_filtered(R::Creature),
                up_to: None,
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CanBlockAdditional(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Give No Ground — {3}{W} Instant. Target creature gets +2/+6 and can block
/// any number of creatures this turn.
pub fn give_no_ground() -> CardDefinition {
    CardDefinition {
        name: "Give No Ground",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(6),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CanBlockAnyNumber,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Blaze of Glory — {W} Instant, castable only during combat before blockers
/// are declared. Target creature defending player controls can block any number
/// of creatures this turn and blocks each attacking creature this turn if able.
pub fn blaze_of_glory() -> CardDefinition {
    CardDefinition {
        name: "Blaze of Glory",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        cast_only_before_blockers: true,
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CanBlockAnyNumber,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::MustBlock,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Static granters ──────────────────────────────────────────────────────────

/// High Ground — {W} Enchantment. Each creature you control can block an
/// additional creature each combat.
pub fn high_ground() -> CardDefinition {
    CardDefinition {
        name: "High Ground",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![your_creatures_have(
            Keyword::CanBlockAdditional(1),
            "Each creature you control can block an additional creature each combat.",
        )],
        ..Default::default()
    }
}

/// Brave the Sands — {1}{W} Enchantment. Creatures you control have vigilance
/// and can block an additional creature each combat.
pub fn brave_the_sands() -> CardDefinition {
    CardDefinition {
        name: "Brave the Sands",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            your_creatures_have(Keyword::Vigilance, "Creatures you control have vigilance."),
            your_creatures_have(
                Keyword::CanBlockAdditional(1),
                "Each creature you control can block an additional creature each combat.",
            ),
        ],
        ..Default::default()
    }
}

/// Cenn's Tactician — {W} 1/1 Kithkin Soldier. {W}, {T}: +1/+1 counter on
/// target Soldier; your creatures with a +1/+1 counter can block an additional
/// creature each combat.
pub fn cenns_tactician() -> CardDefinition {
    CardDefinition {
        name: "Cenn's Tactician",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::HasCreatureType(CreatureType::Soldier)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::CanBlockAdditional(1),
            },
            description: "Each creature you control with a +1/+1 counter on it can block \
                          an additional creature each combat.",
        }],
        ..Default::default()
    }
}

// ── Auras & Equipment ────────────────────────────────────────────────────────

/// Entangler — {2}{W}{W} Aura. Enchanted creature can block any number of
/// creatures.
pub fn entangler() -> CardDefinition {
    CardDefinition {
        name: "Entangler",
        cost: cost(&[generic(2), w(), w()]),
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
            keywords: vec![Keyword::CanBlockAnyNumber],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Iona's Blessing — {3}{W} Aura. Enchanted creature gets +2/+2, has vigilance,
/// and can block an additional creature each combat.
pub fn ionas_blessing() -> CardDefinition {
    CardDefinition {
        name: "Iona's Blessing",
        cost: cost(&[generic(3), w()]),
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
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Vigilance, Keyword::CanBlockAdditional(1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Echo Circlet — {2} Equipment. Equipped creature can block an additional
/// creature each combat. Equip {1}.
pub fn echo_circlet() -> CardDefinition {
    CardDefinition {
        name: "Echo Circlet",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CanBlockAdditional(1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Vanguard's Shield — {2} Equipment. Equipped creature gets +0/+3 and can
/// block an additional creature each combat. Equip {3}.
pub fn vanguards_shield() -> CardDefinition {
    CardDefinition {
        name: "Vanguard's Shield",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            toughness: 3,
            keywords: vec![Keyword::CanBlockAdditional(1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// "This creature can block an additional creature this turn" — the shared
/// body of the activated granters.
fn block_additional_self() -> Effect {
    Effect::GrantKeyword {
        what: Selector::This,
        keyword: Keyword::CanBlockAdditional(1),
        duration: Duration::EndOfTurn,
    }
}
