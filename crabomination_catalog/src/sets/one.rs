//! Phyrexia: All Will Be One — Incubate (CR 701.53). "Incubate N" creates an
//! Incubator double-faced token with N +1/+1 counters; `{2}: Transform` flips
//! it to a 0/0 Phyrexian artifact creature (so it becomes N/N).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{
    deal, drain, draw, etb, gain_life, on_attack, on_dies, target_any, target_filtered,
};
use crate::effect::{LookPick, Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, phyrexian, r, u, w};

/// Anthem: "Phyrexians you control have `keyword`."
fn phyrexians_have(keyword: Keyword) -> StaticEffect {
    StaticEffect::GrantKeyword {
        applies_to: Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Phyrexian)
                .and(SelectionRequirement::ControlledByYou),
        ),
        keyword,
    }
}

fn incubate(amount: u32) -> Effect {
    Effect::Incubate {
        who: PlayerRef::You,
        amount: Value::Const(amount as i32),
    }
}

/// Eyes of Gitaxias — {2}{U} Sorcery. Incubate 3. Draw a card.
pub fn eyes_of_gitaxias() -> CardDefinition {
    CardDefinition {
        name: "Eyes of Gitaxias",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            incubate(3),
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Merciless Repurposing — {4}{B}{B} Instant. Exile target creature. Incubate 3.
pub fn merciless_repurposing() -> CardDefinition {
    CardDefinition {
        name: "Merciless Repurposing",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Exile,
            },
            incubate(3),
        ]),
        ..Default::default()
    }
}

/// Phyrexian Awakening — {2}{W} Enchantment. ETB: incubate 4. Phyrexians you
/// control have vigilance.
pub fn phyrexian_awakening() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Awakening",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(incubate(4))],
        static_abilities: vec![StaticAbility {
            description: "Phyrexians you control have vigilance.",
            effect: phyrexians_have(Keyword::Vigilance),
        }],
        ..Default::default()
    }
}

/// Tangled Skyline — {4}{G} Enchantment. ETB: gain 5 life and incubate 5.
/// Phyrexians you control have reach.
pub fn tangled_skyline() -> CardDefinition {
    CardDefinition {
        name: "Tangled Skyline",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![gain_life(5), incubate(5)]))],
        static_abilities: vec![StaticAbility {
            description: "Phyrexians you control have reach.",
            effect: phyrexians_have(Keyword::Reach),
        }],
        ..Default::default()
    }
}

/// Injector Crocodile — {4}{B}{B} Creature — Phyrexian Crocodile 5/5. When it
/// dies, incubate 3. Swampcycling {2}.
pub fn injector_crocodile() -> CardDefinition {
    CardDefinition {
        name: "Injector Crocodile",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Crocodile],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Landcycling(
            cost(&[generic(2)]),
            crate::card::LandType::Swamp,
        )],
        triggered_abilities: vec![on_dies(incubate(3))],
        ..Default::default()
    }
}

/// Essence of Orthodoxy — {3}{W}{W} Creature — Phyrexian 3/3, flying. Whenever
/// this or another Phyrexian you control enters, incubate 2.
pub fn essence_of_orthodoxy() -> CardDefinition {
    CardDefinition {
        name: "Essence of Orthodoxy",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Phyrexian),
                }),
            effect: incubate(2),
        }],
        ..Default::default()
    }
}

/// Compleated Huntmaster — {2}{B} Creature — Phyrexian Elf Warrior 2/3.
/// {1}, {T}, Sacrifice another creature or artifact: Incubate 3.
pub fn compleated_huntmaster() -> CardDefinition {
    CardDefinition {
        name: "Compleated Huntmaster",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Elf,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                1,
            )),
            effect: incubate(3),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Incisor Glider — {1}{W} Artifact Creature — Phyrexian Construct 1/3 with
/// flying. Corrupted (CR 702.166) — whenever it attacks, if an opponent has
/// three or more poison counters, creatures you control get +1/+1 until EOT.
pub fn incisor_glider() -> CardDefinition {
    CardDefinition {
        name: "Incisor Glider",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::CorruptedActive {
                    who: PlayerRef::You,
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: crate::effect::Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Vivisection Evangelist — {3}{W}{B} Creature — Phyrexian Cleric 4/4 with
/// vigilance. Corrupted (CR 702.166) — when it enters, if an opponent has three
/// or more poison counters, destroy target creature or planeswalker an opponent
/// controls.
pub fn vivisection_evangelist() -> CardDefinition {
    CardDefinition {
        name: "Vivisection Evangelist",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CorruptedActive {
                    who: PlayerRef::You,
                }),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Ravenous Necrotitan — {2}{B}{B} Creature — Phyrexian Horror 6/6. Corrupted
/// (CR 702.166) — when it enters, sacrifice a creature unless an opponent has
/// three or more poison counters.
pub fn ravenous_necrotitan() -> CardDefinition {
    CardDefinition {
        name: "Ravenous Necrotitan",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CorruptedActive {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::You),
                count: Value::ONE,
                filter: SelectionRequirement::Creature,
            }),
        })],
        ..Default::default()
    }
}

/// Fleshless Gladiator — {1}{B} Creature — Phyrexian Skeleton 2/2. Corrupted
/// (CR 702.166) — {2}{B}: return this card from your graveyard to the
/// battlefield tapped and lose 1 life (only while an opponent has 3+ poison).
pub fn fleshless_gladiator() -> CardDefinition {
    CardDefinition {
        name: "Fleshless Gladiator",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            condition: Some(Predicate::CorruptedActive {
                who: PlayerRef::You,
            }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sinew Dancer — {W} Creature — Phyrexian Soldier 1/1. {3}{W}, {T}: Tap target
/// creature. Corrupted (CR 702.166) — {W}, {T}: Tap target creature (only while
/// an opponent has three or more poison counters).
pub fn sinew_dancer() -> CardDefinition {
    let tap_target = || Effect::Tap {
        what: target_filtered(SelectionRequirement::Creature),
    };
    CardDefinition {
        name: "Sinew Dancer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), w()]),
                tap_cost: true,
                effect: tap_target(),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                condition: Some(Predicate::CorruptedActive {
                    who: PlayerRef::You,
                }),
                effect: tap_target(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Apostle of Invasion — {4}{W}{W} Creature — Phyrexian Angel 4/4 with flying.
/// Corrupted (CR 702.166) — has double strike while an opponent has three or
/// more poison counters.
pub fn apostle_of_invasion() -> CardDefinition {
    CardDefinition {
        name: "Apostle of Invasion",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Angel],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Corrupted — has double strike.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::DoubleStrike,
                condition: Predicate::CorruptedActive {
                    who: PlayerRef::You,
                },
            },
        }],
        ..Default::default()
    }
}

/// Bloated Contaminator — {2}{G} Creature — Phyrexian Beast 4/4 with trample and
/// toxic 1. Whenever it deals combat damage to a player, proliferate.
pub fn bloated_contaminator() -> CardDefinition {
    CardDefinition {
        name: "Bloated Contaminator",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Toxic(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Bonepicker Skirge — {2}{B} Creature — Phyrexian Imp 2/2 with flying.
/// Corrupted (CR 702.166) — as long as an opponent has three or more poison
/// counters, it has deathtouch and lifelink.
pub fn bonepicker_skirge() -> CardDefinition {
    let corrupted_keyword = |keyword: Keyword| StaticAbility {
        description: "Corrupted — has deathtouch and lifelink.",
        effect: StaticEffect::SelfHasKeywordIf {
            keyword,
            condition: Predicate::CorruptedActive {
                who: PlayerRef::You,
            },
        },
    };
    CardDefinition {
        name: "Bonepicker Skirge",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Imp],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            corrupted_keyword(Keyword::Deathtouch),
            corrupted_keyword(Keyword::Lifelink),
        ],
        ..Default::default()
    }
}

/// Bilious Skulldweller — {B} Creature — Phyrexian Insect 1/1 with deathtouch
/// and toxic 1.
pub fn bilious_skulldweller() -> CardDefinition {
    CardDefinition {
        name: "Bilious Skulldweller",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch, Keyword::Toxic(1)],
        ..Default::default()
    }
}

/// Branchblight Stalker — {1}{G} Creature — Phyrexian Elf Scout 3/1 with toxic 2.
pub fn branchblight_stalker() -> CardDefinition {
    CardDefinition {
        name: "Branchblight Stalker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Elf,
                CreatureType::Scout,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Toxic(2)],
        ..Default::default()
    }
}

/// Jawbone Duelist — {1}{W} Creature — Phyrexian Soldier 1/1 with double strike
/// and toxic 1.
pub fn jawbone_duelist() -> CardDefinition {
    CardDefinition {
        name: "Jawbone Duelist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::DoubleStrike, Keyword::Toxic(1)],
        ..Default::default()
    }
}

/// Ichorspit Basilisk — {2}{G} Creature — Phyrexian Basilisk 1/3 with deathtouch
/// and toxic 1.
pub fn ichorspit_basilisk() -> CardDefinition {
    CardDefinition {
        name: "Ichorspit Basilisk",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Basilisk],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch, Keyword::Toxic(1)],
        ..Default::default()
    }
}

/// Swooping Lookout — {W} Creature — Phyrexian Bird 1/2 with flying and vigilance.
pub fn swooping_lookout() -> CardDefinition {
    CardDefinition {
        name: "Swooping Lookout",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    }
}

/// Malcator's Watcher — {1}{U} Artifact Creature — Phyrexian Bird 1/1 with flying
/// and vigilance. When it dies, draw a card.
pub fn malcators_watcher() -> CardDefinition {
    CardDefinition {
        name: "Malcator's Watcher",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Drone],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Sheoldred's Headcleaver — {3}{B} Creature — Phyrexian Horror 2/4 with menace
/// and toxic 2.
pub fn sheoldreds_headcleaver() -> CardDefinition {
    CardDefinition {
        name: "Sheoldred's Headcleaver",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Menace, Keyword::Toxic(2)],
        ..Default::default()
    }
}

/// Chimney Rabble — {3}{R} Creature — Phyrexian Goblin 3/3 with haste. When it
/// enters, create a 1/1 red Phyrexian Goblin creature token.
pub fn chimney_rabble() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Phyrexian Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Chimney Rabble",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(goblin),
        })],
        ..Default::default()
    }
}

/// Chrome Prowler — {2}{U} Artifact Creature — Phyrexian Insect 3/2 with flash.
/// When it enters, tap target creature an opponent controls.
pub fn chrome_prowler() -> CardDefinition {
    CardDefinition {
        name: "Chrome Prowler",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cat],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Cutthroat Centurion — {2}{B} Creature — Phyrexian Warrior 2/2. Sacrifice
/// another artifact or creature: this gets +2/+2 until end of turn. Activate
/// only once each turn.
pub fn cutthroat_centurion() -> CardDefinition {
    CardDefinition {
        name: "Cutthroat Centurion",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            sac_other_filter: Some((
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                1,
            )),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shrapnel Slinger — {1}{R} Creature — Phyrexian Rebel 2/2. When it enters, you
/// may sacrifice a creature; if you do, destroy target artifact an opponent
/// controls.
pub fn shrapnel_slinger() -> CardDefinition {
    CardDefinition {
        name: "Shrapnel Slinger",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Phyrexian],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "sacrifice a creature".into(),
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// A 1/1 colorless Phyrexian Mite artifact creature token with toxic 1 that
/// can't block (Skrelv's Hive, Crawling Chorus).
fn mite_token() -> TokenDefinition {
    crabomination_base::tokens::phyrexian_mite_token()
}

/// Tyrranax Rex — {4}{G}{G}{G} Creature — Phyrexian Dinosaur 8/8. Can't be
/// countered. Trample, ward {4}, haste, toxic 4.
pub fn tyrranax_rex() -> CardDefinition {
    CardDefinition {
        name: "Tyrranax Rex",
        cost: cost(&[generic(4), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![
            Keyword::CantBeCountered,
            Keyword::Trample,
            Keyword::Ward(WardCost::generic(4)),
            Keyword::Haste,
            Keyword::Toxic(4),
        ],
        ..Default::default()
    }
}

/// Thrun, Breaker of Silence — {3}{G}{G} Legendary Creature — Troll Shaman 5/5.
/// Can't be countered. Trample. Can't be the target of nongreen spells or
/// abilities opponents control. During your turn, Thrun has indestructible.
pub fn thrun_breaker_of_silence() -> CardDefinition {
    CardDefinition {
        name: "Thrun, Breaker of Silence",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Shaman],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::CantBeCountered,
            Keyword::Trample,
            Keyword::HexproofExceptColors(vec![Color::Green]),
        ],
        static_abilities: vec![StaticAbility {
            description: "During your turn, Thrun has indestructible.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Indestructible,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Mondrak, Glory Dominus — {2}{W}{W} Legendary Creature — Phyrexian Horror 4/4.
/// Token doubler. {1}{W/P}{W/P}, Sacrifice two other artifacts and/or creatures:
/// Put an indestructible counter on Mondrak.
pub fn mondrak_glory_dominus() -> CardDefinition {
    CardDefinition {
        name: "Mondrak, Glory Dominus",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, twice that many are created instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), phyrexian(Color::White), phyrexian(Color::White)]),
            sac_other_filter: Some((
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                2,
            )),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Indestructible,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kuldotha Cackler — {2}{R} Creature — Phyrexian Hyena 2/3. Trample. Whenever
/// it attacks, it gets +X/+0 until end of turn, where X is the number of
/// permanents you control with oil counters on them.
pub fn kuldotha_cackler() -> CardDefinition {
    CardDefinition {
        name: "Kuldotha Cackler",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Hyena],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::WithCounter(CounterType::Oil)
                    .and(SelectionRequirement::ControlledByYou),
            ))),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Evolving Adaptive — {G} Creature — Phyrexian Warrior 0/0. Enters with an oil
/// counter; gets +1/+1 for each oil counter on it. Whenever another creature you
/// control enters with greater power or toughness, put an oil counter on this.
pub fn evolving_adaptive() -> CardDefinition {
    CardDefinition {
        name: "Evolving Adaptive",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::Oil, Value::ONE)),
        dynamic_pt: Some(DynamicPt::BasePlusCountersOnSelf {
            per_p: 1,
            per_t: 1,
            counter_type: CounterType::Oil,
            base_p: 0,
            base_t: 0,
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::OtherThanSource),
                    },
                    Predicate::Any(vec![
                        Predicate::Not(Box::new(Predicate::ValueAtMost(
                            Value::PowerOf(Box::new(Selector::TriggerSource)),
                            Value::PowerOf(Box::new(Selector::This)),
                        ))),
                        Predicate::Not(Box::new(Predicate::ValueAtMost(
                            Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                            Value::ToughnessOf(Box::new(Selector::This)),
                        ))),
                    ]),
                ])),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Skrelv's Hive — {1}{W} Enchantment. At the beginning of your upkeep, lose 1
/// life and create a Phyrexian Mite token. Corrupted — while an opponent has 3+
/// poison counters, creatures you control with toxic have lifelink.
pub fn skrelvs_hive() -> CardDefinition {
    CardDefinition {
        name: "Skrelv's Hive",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(mite_token()),
                },
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "Corrupted — creatures you control with toxic have lifelink.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::CorruptedActive {
                    who: PlayerRef::You,
                },
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasToxic.and(SelectionRequirement::ControlledByYou),
                ),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Prologue to Phyresis — {1}{U} Instant. Each opponent gets a poison counter.
/// Draw a card.
pub fn prologue_to_phyresis() -> CardDefinition {
    CardDefinition {
        name: "Prologue to Phyresis",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddPoison {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Corrupted Conviction — {B} Instant. As an additional cost, sacrifice a
/// creature. Draw two cards.
pub fn corrupted_conviction() -> CardDefinition {
    CardDefinition {
        name: "Corrupted Conviction",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            count: 1,
        }],
        effect: draw(2),
        ..Default::default()
    }
}

/// Whisper of the Dross — {B} Instant. Target creature gets -1/-1 until end of
/// turn. Proliferate.
pub fn whisper_of_the_dross() -> CardDefinition {
    CardDefinition {
        name: "Whisper of the Dross",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Bring the Ending — {1}{U} Instant. Counter target spell unless its controller
/// pays {2}. Corrupted — counter it outright instead if its controller has three
/// or more poison counters.
pub fn bring_the_ending() -> CardDefinition {
    CardDefinition {
        name: "Bring the Ending",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::CorruptedActive {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            }),
            else_: Box::new(Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            }),
        },
        ..Default::default()
    }
}

/// Vraska's Fall — {2}{B} Instant. Each opponent sacrifices a creature or
/// planeswalker of their choice and gets a poison counter.
pub fn vraskas_fall() -> CardDefinition {
    CardDefinition {
        name: "Vraska's Fall",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            },
            Effect::AddPoison {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Crawling Chorus — {W} Creature — Phyrexian Horror 1/1 with toxic 1. When it
/// dies, create a Phyrexian Mite token.
pub fn crawling_chorus() -> CardDefinition {
    CardDefinition {
        name: "Crawling Chorus",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(mite_token()),
        })],
        ..Default::default()
    }
}

/// Slaughter Singer — {G}{W} Creature — Phyrexian Cleric 2/2 with toxic 2.
/// Whenever another creature you control with toxic attacks, it gets +1/+1
/// until end of turn.
pub fn slaughter_singer() -> CardDefinition {
    CardDefinition {
        name: "Slaughter Singer",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Toxic(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasToxic,
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Zealot of the God-Pharaoh — {3}{R} Creature — Minotaur Archer 4/3.
/// {4}{R}: This creature deals 2 damage to target opponent or planeswalker.
pub fn zealot_of_the_god_pharaoh() -> CardDefinition {
    CardDefinition {
        name: "Zealot of the God-Pharaoh",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Archer],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            effect: deal(
                2,
                target_filtered(
                    SelectionRequirement::Player.or(SelectionRequirement::Planeswalker),
                ),
            ),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mandibular Kite — {W} Artifact — Equipment. Living weapon. Equipped creature
/// gets +1/+1 and has flying. Equip {3}{W}.
pub fn mandibular_kite() -> CardDefinition {
    let germ = TokenDefinition {
        name: "Phyrexian Germ".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Mandibular Kite",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3), w()]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(germ),
            },
            Effect::Attach {
                what: Selector::This,
                to: Selector::LastCreatedToken,
            },
        ]))],
        ..Default::default()
    }
}

/// Migloz, Maze Crusher — {1}{R}{G} Legendary Creature — Phyrexian Beast 4/4.
/// Enters with five oil counters. Three activated abilities spend oil: gain
/// vigilance+menace; +2/+2; or destroy an artifact/enchantment.
pub fn migloz_maze_crusher() -> CardDefinition {
    CardDefinition {
        name: "Migloz, Maze Crusher",
        cost: cost(&[generic(1), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        enters_with_counters: Some((CounterType::Oil, Value::Const(5))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_cost: Some((CounterType::Oil, 1)),
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Menace,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                remove_counter_cost: Some((CounterType::Oil, 2)),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                remove_counter_cost: Some((CounterType::Oil, 3)),
                effect: Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ichor Drinker — {B} Creature — Phyrexian Vampire 1/1 with lifelink.
/// {B}, Exile this card from your graveyard: Incubate 2. Sorcery speed.
pub fn ichor_drinker() -> CardDefinition {
    CardDefinition {
        name: "Ichor Drinker",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Vampire],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: incubate(2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vraan, Executioner Thane — {1}{B} Legendary Creature — Phyrexian Vampire 2/2.
/// Whenever one or more other creatures you control die, each opponent loses 2
/// life and you gain 2 life. Only once each turn.
pub fn vraan_executioner_thane() -> CardDefinition {
    CardDefinition {
        name: "Vraan, Executioner Thane",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Vampire],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .once_per_turn(),
            effect: drain(2),
        }],
        ..Default::default()
    }
}

/// Karumonix, the Rat King — {1}{B}{B} Legendary Creature — Phyrexian Rat 3/3
/// with toxic 1. Other Rats you control have toxic 1. When it enters, look at the
/// top five cards of your library; put any number of Rat cards into your hand and
/// the rest on the bottom.
pub fn karumonix_the_rat_king() -> CardDefinition {
    CardDefinition {
        name: "Karumonix, the Rat King",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Rat],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Toxic(1)],
        static_abilities: vec![StaticAbility {
            description: "Other Rats you control have toxic 1.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Rat)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Toxic(1),
            },
        }],
        triggered_abilities: vec![etb(Effect::RevealTopTakeMatchingToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            filter: SelectionRequirement::HasCreatureType(CreatureType::Rat),
            distinct_powers: false,
        })],
        ..Default::default()
    }
}

/// Vindictive Flamestoker — {R} Creature — Phyrexian Wizard 1/2. Whenever you
/// cast a noncreature spell, put an oil counter on it. {6}{R}, Sacrifice it:
/// Discard your hand, then draw four cards. This ability costs {1} less to
/// activate for each oil counter on it.
pub fn vindictive_flamestoker() -> CardDefinition {
    CardDefinition {
        name: "Vindictive Flamestoker",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), r()]),
            sac_cost: true,
            self_counter_cost_reduction: Some(CounterType::Oil),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::CardsInHandMatching {
                        who: PlayerRef::You,
                        filter: SelectionRequirement::Any,
                    },
                    random: false,
                },
                draw(4),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gitaxian Anatomist — {3}{U} Creature — Phyrexian Wizard 2/5. When it enters,
/// you may tap it; if you do, proliferate.
pub fn gitaxian_anatomist() -> CardDefinition {
    CardDefinition {
        name: "Gitaxian Anatomist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: Selector::This,
            },
            Effect::Proliferate,
        ]))],
        ..Default::default()
    }
}

/// Basilica Shepherd — {3}{W}{W} Creature — Phyrexian Angel 3/3 with flying.
/// When it enters, create two Phyrexian Mite tokens.
pub fn basilica_shepherd() -> CardDefinition {
    CardDefinition {
        name: "Basilica Shepherd",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Angel],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Box::new(mite_token()),
        })],
        ..Default::default()
    }
}

/// Infectious Bite — {1}{G} Instant. Target creature you control deals damage
/// equal to its power to target creature you don't control. Each opponent gets a
/// poison counter.
pub fn infectious_bite() -> CardDefinition {
    CardDefinition {
        name: "Infectious Bite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPower {
                source: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
            Effect::AddPoison {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Gulping Scraptrap — {4}{B} Creature — Phyrexian Horror 4/4. When it enters or
/// dies, proliferate.
pub fn gulping_scraptrap() -> CardDefinition {
    CardDefinition {
        name: "Gulping Scraptrap",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Proliferate), on_dies(Effect::Proliferate)],
        ..Default::default()
    }
}

/// Deadly Derision — {2}{B}{B} Instant. Destroy target creature or planeswalker.
/// Create a Treasure token.
pub fn deadly_derision() -> CardDefinition {
    CardDefinition {
        name: "Deadly Derision",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(crate::game::effects::treasure_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Kill-Zone Acrobat — {2}{B} Creature — Human Soldier 3/2. Whenever it attacks,
/// you may sacrifice another creature or artifact; if you do, it gains flying
/// until end of turn.
pub fn kill_zone_acrobat() -> CardDefinition {
    CardDefinition {
        name: "Kill-Zone Acrobat",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "sacrifice another creature or artifact".into(),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Artifact)
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Blightbelly Rat — {1}{B} Creature — Phyrexian Rat 2/2 with toxic 1. When it
/// dies, proliferate.
pub fn blightbelly_rat() -> CardDefinition {
    CardDefinition {
        name: "Blightbelly Rat",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Rat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![on_dies(Effect::Proliferate)],
        ..Default::default()
    }
}

/// Sawblade Scamp — {R} Creature — Phyrexian Beast 1/1 with haste. Whenever you
/// cast a noncreature spell, put an oil counter on it. {T}, Remove an oil
/// counter: it deals 1 damage to each opponent.
pub fn sawblade_scamp() -> CardDefinition {
    CardDefinition {
        name: "Sawblade Scamp",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: deal(1, Selector::Player(PlayerRef::EachOpponent)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Furnace Punisher — {2}{R} Creature — Phyrexian Warrior 3/3 with menace. At
/// the beginning of each player's upkeep, deals 2 damage to that player unless
/// they control two or more basic lands.
pub fn furnace_punisher() -> CardDefinition {
    CardDefinition {
        name: "Furnace Punisher",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::ControlledBy {
                        who: PlayerRef::ActivePlayer,
                        filter: SelectionRequirement::IsBasicLand,
                    },
                    n: Value::Const(2),
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(deal(2, Selector::Player(PlayerRef::ActivePlayer))),
            },
        }],
        ..Default::default()
    }
}

/// Necrogen Communion — {1}{B} Aura. Enchant creature you control; it has
/// toxic 2. When it dies, return that card to the battlefield under your
/// control.
pub fn necrogen_communion() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Necrogen Communion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Toxic(2)],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        }],
        ..Default::default()
    }
}

/// Adaptive Sporesinger — {2}{G} 2/2 Phyrexian Druid with vigilance. ETB
/// choose one: target creature +2/+2 and vigilance EOT, or proliferate.
pub fn adaptive_sporesinger() -> CardDefinition {
    CardDefinition {
        name: "Adaptive Sporesinger",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Proliferate,
        ]))],
        ..Default::default()
    }
}

/// Annihilating Glare — {B} Sorcery. As an additional cost, pay {4} or
/// sacrifice an artifact or creature. Destroy target creature or planeswalker.
pub fn annihilating_glare() -> CardDefinition {
    CardDefinition {
        name: "Annihilating Glare",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeOrPay {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            pay: 4,
        }],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
        },
        ..Default::default()
    }
}

/// Axiom Engraver — {1}{R} 1/3 Phyrexian Wizard, enters with two oil counters.
/// {T}, remove an oil counter, discard a card: draw a card.
pub fn axiom_engraver() -> CardDefinition {
    CardDefinition {
        name: "Axiom Engraver",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 1)),
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bladegraft Aspirant — {2}{R} 2/3 Phyrexian Warrior with menace. Equipment
/// spells cost {1} less; equip abilities cost {1} less (the printed
/// "targeting this creature" scope widened to all your equips).
pub fn bladegraft_aspirant() -> CardDefinition {
    CardDefinition {
        name: "Bladegraft Aspirant",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        static_abilities: vec![
            StaticAbility {
                description: "Equipment spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Equip abilities you activate cost {1} less to activate.",
                effect: StaticEffect::EquipCostReduction { amount: 1 },
            },
        ],
        ..Default::default()
    }
}

/// Blazing Crescendo — {1}{R} Instant. Target creature +3/+1 EOT; impulse the
/// top card until the end of your next turn.
pub fn blazing_crescendo() -> CardDefinition {
    use crate::card::MayPlayDuration;
    CardDefinition {
        name: "Blazing Crescendo",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: false,
                uncast_penalty: None,
            },
        ]),
        ..Default::default()
    }
}

/// Against All Odds — {3}{W} Sorcery. Choose one or both: flicker target
/// artifact/creature you control; reanimate an artifact/creature card MV≤3.
pub fn against_all_odds() -> CardDefinition {
    CardDefinition {
        name: "Against All Odds",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Seq(vec![
                    Effect::Exile {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: SelectionRequirement::Artifact
                                .or(SelectionRequirement::Creature)
                                .and(SelectionRequirement::ControlledByYou),
                        },
                    },
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOfMoved,
                            tapped: false,
                        },
                    },
                ]),
                // Per-mode targets each occupy slot 0 inside their mode; the
                // outer cast supplies them positionally in pick order.
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Artifact
                            .or(SelectionRequirement::Creature)
                            .and(SelectionRequirement::InYourGraveyard)
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    },
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ],
        },
        ..Default::default()
    }
}

/// Annex Sentry — {2}{W} 1/4 Phyrexian Cleric artifact creature, toxic 1. ETB
/// exile target opposing artifact/creature MV≤3 until this leaves.
pub fn annex_sentry() -> CardDefinition {
    CardDefinition {
        name: "Annex Sentry",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .and(SelectionRequirement::ControlledByOpponent)
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            ),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Armored Scrapgorger — {1}{G} 0/3 Phyrexian Beast; +3/+0 with 3+ oil.
/// {T}: any color. Becomes tapped → exile a graveyard card, add an oil.
pub fn armored_scrapgorger() -> CardDefinition {
    CardDefinition {
        name: "Armored Scrapgorger",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "This creature gets +3/+0 as long as it has three or more oil counters on it.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Oil,
                    },
                    Value::Const(3),
                ),
                power: 3,
                toughness: 0,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::InGraveyard),
                    to: ZoneDest::Exile,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Ambulatory Edifice — {2}{B} 3/2 Phyrexian Construct. ETB you may pay 2
/// life; when you do, target creature gets -1/-1 until end of turn.
pub fn ambulatory_edifice() -> CardDefinition {
    CardDefinition {
        name: "Ambulatory Edifice",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayPayLife {
            description: "Pay 2 life for -1/-1?".into(),
            amount: Value::Const(2),
            body: Box::new(Effect::Reflexive {
                body: Box::new(Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Atmosphere Surgeon — {1}{U} 2/1 Phyrexian Wizard. Noncreature cast → oil
/// counter. Remove an oil: target creature gains flying EOT (sorcery only).
pub fn atmosphere_surgeon() -> CardDefinition {
    CardDefinition {
        name: "Atmosphere Surgeon",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Oil, 1)),
            sorcery_speed: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bladed Ambassador — {1}{W} 3/1 Phyrexian Soldier, enters with an oil
/// counter. {1}, remove an oil counter: indestructible until end of turn.
pub fn bladed_ambassador() -> CardDefinition {
    CardDefinition {
        name: "Bladed Ambassador",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        enters_with_counters: Some((CounterType::Oil, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Black Sun's Twilight — {X}{B} Instant. Up to one target creature -X/-X EOT;
/// if X ≥ 5, reanimate a creature card MV≤X tapped.
pub fn black_suns_twilight() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Black Sun's Twilight",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
                toughness: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                then: Box::new(Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                            filter: SelectionRequirement::Creature,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Carnivorous Canopy — {2}{G} Sorcery. Destroy target artifact, enchantment,
/// or flying creature; proliferate if its mana value was 3 or less.
pub fn carnivorous_canopy() -> CardDefinition {
    let filter = || {
        SelectionRequirement::Artifact
            .or(SelectionRequirement::Enchantment)
            .or(SelectionRequirement::Creature
                .and(SelectionRequirement::HasKeyword(Keyword::Flying)))
    };
    CardDefinition {
        name: "Carnivorous Canopy",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::ManaValueAtMost(3),
            },
            then: Box::new(Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(filter()),
                },
                Effect::Proliferate,
            ])),
            else_: Box::new(Effect::Destroy {
                what: target_filtered(filter()),
            }),
        },
        ..Default::default()
    }
}

/// Chrome Cat — {3} 3/2 Cat artifact creature. ETB scry 1.
pub fn chrome_cat() -> CardDefinition {
    CardDefinition {
        name: "Chrome Cat",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Distorted Curiosity — {2}{U} Sorcery. Corrupted — costs {2} less if an
/// opponent has three or more poison counters. Draw two cards.
pub fn distorted_curiosity() -> CardDefinition {
    CardDefinition {
        name: "Distorted Curiosity",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "Corrupted — This spell costs {2} less to cast if an opponent has three or more poison counters.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::CorruptedActive {
                    who: PlayerRef::You,
                },
                amount: 2,
            },
        }],
        effect: draw(2),
        ..Default::default()
    }
}

// ── Proliferate-matters (CR 701.34 — EventKind::Proliferated) ────────────────

/// Tekuthal, Inquiry Dominus — {2}{U}{U} 3/5 flying. If you would proliferate,
/// proliferate twice instead. {1}{U/P}{U/P}, Remove three counters from among
/// other permanents you control: put an indestructible counter on it.
pub fn tekuthal_inquiry_dominus() -> CardDefinition {
    CardDefinition {
        name: "Tekuthal, Inquiry Dominus",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If you would proliferate, proliferate twice instead.",
            effect: StaticEffect::ProliferateTwice,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), phyrexian(Color::Blue), phyrexian(Color::Blue)]),
            remove_counter_among_filter: Some((
                None,
                3,
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .or(SelectionRequirement::Planeswalker)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            )),
            effect: Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Scheming Aspirant — {1}{B} 1/3. Whenever you proliferate, each opponent
/// loses 2 life and you gain 2 life.
pub fn scheming_aspirant() -> CardDefinition {
    CardDefinition {
        name: "Scheming Aspirant",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Proliferated, EventScope::YourControl),
            effect: drain(2),
        }],
        ..Default::default()
    }
}

/// Ezuri, Stalker of Spheres — {2}{G}{U} 3/3. ETB: you may pay {3}; if you do,
/// proliferate twice. Whenever you proliferate, draw a card.
pub fn ezuri_stalker_of_spheres() -> CardDefinition {
    CardDefinition {
        name: "Ezuri, Stalker of Spheres",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Elf,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::MayPay {
                description: "Pay {3} to proliferate twice?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::Seq(vec![Effect::Proliferate, Effect::Proliferate])),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Proliferated, EventScope::YourControl),
                effect: draw(1),
            },
        ],
        ..Default::default()
    }
}

/// Voidwing Hybrid — {U}{B} 2/1 flying, toxic 1. When you proliferate, return
/// this card from your graveyard to your hand.
pub fn voidwing_hybrid() -> CardDefinition {
    CardDefinition {
        name: "Voidwing Hybrid",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Bat],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Toxic(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Proliferated, EventScope::FromYourGraveyard),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Melira, the Living Cure — {G}{W} 3/3. If you would get one or more poison
/// counters, instead you get one and can't get more this turn. Exile Melira:
/// when another target creature or artifact is put into a graveyard this turn,
/// return it to the battlefield under its owner's control.
pub fn melira_the_living_cure() -> CardDefinition {
    CardDefinition {
        name: "Melira, the Living Cure",
        cost: cost(&[g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "If you would get one or more poison counters, instead you get one poison counter and you can't get additional poison counters this turn.",
            effect: StaticEffect::PoisonCappedAtOnePerTurn,
        }],
        activated_abilities: vec![ActivatedAbility {
            exile_self_cost: true,
            effect: Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                        tapped: false,
                    },
                }),
                slot: 0,
                filter: Some(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Artifact)
                        .and(SelectionRequirement::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ovika, Enigma Goliath — {5}{U}{R} 6/6 flying, Ward—{3}, Pay 3 life.
/// Whenever you cast a noncreature spell, create X 1/1 red Phyrexian Goblin
/// tokens, X = that spell's mana value; they gain haste until end of turn.
pub fn ovika_enigma_goliath() -> CardDefinition {
    CardDefinition {
        name: "Ovika, Enigma Goliath",
        cost: cost(&[generic(5), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Nightmare],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![
            Keyword::Flying,
            Keyword::Ward(WardCost::ManaAndLife(cost(&[generic(3)]), 3)),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(SelectionRequirement::Noncreature),
            ),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: Box::new(phyrexian_goblin_token()),
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedTokens,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// 1/1 red Phyrexian Goblin (Ovika, Churning Reservoir).
fn phyrexian_goblin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Vivisurgeon's Insight — {3}{U}{U} Sorcery. Draw three cards. Proliferate.
pub fn vivisurgeons_insight() -> CardDefinition {
    CardDefinition {
        name: "Vivisurgeon's Insight",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![draw(3), Effect::Proliferate]),
        ..Default::default()
    }
}

/// Experimental Augury — {1}{U} Instant. Look at the top three cards; put one
/// into your hand and the rest on the bottom. Proliferate.
pub fn experimental_augury() -> CardDefinition {
    CardDefinition {
        name: "Experimental Augury",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
    ..Default::default()
})),
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Unnatural Restoration — {1}{G} Sorcery. Return target permanent card from
/// your graveyard to your hand. Proliferate.
pub fn unnatural_restoration() -> CardDefinition {
    CardDefinition {
        name: "Unnatural Restoration",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::PermanentCard.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Copper Longlegs — {1}{G} 1/3 reach. {1}{G}, Sacrifice this: Proliferate.
pub fn copper_longlegs() -> CardDefinition {
    CardDefinition {
        name: "Copper Longlegs",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Spider],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reject Imperfection — {1}{U}{U} Instant. Counter target spell. If its mana
/// value was 3 or less, proliferate.
pub fn reject_imperfection() -> CardDefinition {
    CardDefinition {
        name: "Reject Imperfection",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::ManaValueAtMost(3),
            },
            then: Box::new(Effect::Seq(vec![
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
                Effect::Proliferate,
            ])),
            else_: Box::new(Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            }),
        },
        ..Default::default()
    }
}

/// Serum Snare — {1}{U} Instant. Return target nonland permanent to its
/// owner's hand. If it had mana value 3 or less, proliferate.
pub fn serum_snare() -> CardDefinition {
    let bounce = || Effect::Move {
        what: target_filtered(SelectionRequirement::Nonland),
        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
    };
    CardDefinition {
        name: "Serum Snare",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::ManaValueAtMost(3),
            },
            then: Box::new(Effect::Seq(vec![bounce(), Effect::Proliferate])),
            else_: Box::new(bounce()),
        },
        ..Default::default()
    }
}

/// Thirsting Roots — {G} Sorcery. Choose one — search your library for a basic
/// land card to hand; or proliferate.
pub fn thirsting_roots() -> CardDefinition {
    CardDefinition {
        name: "Thirsting Roots",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Venomous Brutalizer — {2}{G}{G} 4/4, toxic 3. ETB: you may pay {1}{G}; if
/// you do, proliferate.
pub fn venomous_brutalizer() -> CardDefinition {
    CardDefinition {
        name: "Venomous Brutalizer",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Toxic(3)],
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {1}{G} to proliferate?".into(),
            mana_cost: cost(&[generic(1), g()]),
            body: Box::new(Effect::Proliferate),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Tainted Observer — {1}{G}{U} 2/3 flying, toxic 1. Whenever another creature
/// you control enters, you may pay {2}; if you do, proliferate.
pub fn tainted_observer() -> CardDefinition {
    CardDefinition {
        name: "Tainted Observer",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Toxic(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::MayPay {
                description: "Pay {2} to proliferate?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Proliferate),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Infectious Inquiry — {2}{B} Sorcery. You draw two cards and lose 2 life.
/// Each opponent gets a poison counter.
pub fn infectious_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Infectious Inquiry",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::AddPoison {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Mesmerizing Dose — {1}{U}{U} Aura. ETB: tap enchanted creature, then
/// proliferate. Enchanted creature doesn't untap during its controller's
/// untap step.
pub fn mesmerizing_dose() -> CardDefinition {
    CardDefinition {
        name: "Mesmerizing Dose",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
            Effect::Proliferate,
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

// ── For Mirrodin! Equipment + equipment-matters ──────────────────────────────

/// CR 702.163 — For Mirrodin! ETB mints a 2/2 red Rebel and self-attaches.
fn for_mirrodin(
    name: &'static str,
    mana: crate::mana::ManaCost,
    equip: crate::mana::ManaCost,
    bonus: crate::card::EquipBonus,
) -> CardDefinition {
    let rebel = TokenDefinition {
        name: "Rebel".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rebel],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(rebel),
            },
            Effect::Attach {
                what: Selector::This,
                to: Selector::LastCreatedToken,
            },
        ]))],
        ..Default::default()
    }
}

/// Dragonwing Glider — {3}{R}{R} For Mirrodin! Equipped gets +2/+2, flying,
/// haste. Equip {3}{R}{R}.
pub fn dragonwing_glider() -> CardDefinition {
    for_mirrodin(
        "Dragonwing Glider",
        cost(&[generic(3), r(), r()]),
        cost(&[generic(3), r(), r()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying, Keyword::Haste],
            ..Default::default()
        },
    )
}

/// Hexgold Halberd — {1}{R} For Mirrodin! During your turn, equipped creature
/// has first strike and trample. Equip {2}{R}.
pub fn hexgold_halberd() -> CardDefinition {
    for_mirrodin(
        "Hexgold Halberd",
        cost(&[generic(1), r()]),
        cost(&[generic(2), r()]),
        EquipBonus {
            during_your_turn_keywords: vec![Keyword::FirstStrike, Keyword::Trample],
            ..Default::default()
        },
    )
}

/// Mirran Bardiche — {4}{W} For Mirrodin! Equipped gets +2/+1 and vigilance.
/// Equip {3}{W}.
pub fn mirran_bardiche() -> CardDefinition {
    for_mirrodin(
        "Mirran Bardiche",
        cost(&[generic(4), w()]),
        cost(&[generic(3), w()]),
        EquipBonus {
            power: 2,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        },
    )
}

/// Vulshok Splitter — {3}{R} For Mirrodin! Equipped gets +2/+0. Equip {2}{R}.
pub fn vulshok_splitter() -> CardDefinition {
    for_mirrodin(
        "Vulshok Splitter",
        cost(&[generic(3), r()]),
        cost(&[generic(2), r()]),
        EquipBonus {
            power: 2,
            ..Default::default()
        },
    )
}

/// Sylvok Battle-Chair — {4}{G}{G} For Mirrodin! Equipped gets +4/+4 and
/// trample. Equip {5}{G}{G}.
pub fn sylvok_battle_chair() -> CardDefinition {
    for_mirrodin(
        "Sylvok Battle-Chair",
        cost(&[generic(4), g(), g()]),
        cost(&[generic(5), g(), g()]),
        EquipBonus {
            power: 4,
            toughness: 4,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        },
    )
}

/// Hexgold Hoverwings — {3}{W} For Mirrodin! Equipped has flying; your
/// equipped creatures get +1/+0. Equip {2}{W}.
pub fn hexgold_hoverwings() -> CardDefinition {
    let mut def = for_mirrodin(
        "Hexgold Hoverwings",
        cost(&[generic(3), w()]),
        cost(&[generic(2), w()]),
        EquipBonus {
            keywords: vec![Keyword::Flying],
            ..Default::default()
        },
    );
    def.static_abilities = vec![StaticAbility {
        description: "Creatures you control that are equipped get +1/+0.",
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::EquippedByAtLeast(1)),
            ),
            power: 1,
            toughness: 0,
        },
    }];
    def
}

/// Bladehold War-Whip — {1}{R}{W} For Mirrodin! Equipped has double strike;
/// your other Equipment's equip abilities cost {1} less. Equip {3}{R}{W}.
pub fn bladehold_war_whip() -> CardDefinition {
    let mut def = for_mirrodin(
        "Bladehold War-Whip",
        cost(&[generic(1), r(), w()]),
        cost(&[generic(3), r(), w()]),
        EquipBonus {
            keywords: vec![Keyword::DoubleStrike],
            ..Default::default()
        },
    );
    def.static_abilities = vec![StaticAbility {
        description: "Equip abilities you activate of other Equipment cost {1} less to activate.",
        effect: StaticEffect::EquipCostReduction { amount: 1 },
    }];
    def
}

/// Infested Fleshcutter — {1}{W} Equipment. Equipped gets +2/+0; whenever it
/// attacks, create a 1/1 Phyrexian Mite (toxic 1, can't block). Equip {2}{W}.
pub fn infested_fleshcutter() -> CardDefinition {
    CardDefinition {
        name: "Infested Fleshcutter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2), w()]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            triggered_abilities: vec![on_attack(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(mite_token()),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Prosthetic Injector — {1} Equipment. Equipped gets +0/+2 and has toxic 1.
/// Equip {1}.
pub fn prosthetic_injector() -> CardDefinition {
    CardDefinition {
        name: "Prosthetic Injector",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            toughness: 2,
            keywords: vec![Keyword::Toxic(1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Oxidda Finisher — {5}{R}{R} 7/5 trample. Affinity for Equipment.
pub fn oxidda_finisher() -> CardDefinition {
    CardDefinition {
        name: "Oxidda Finisher",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rebel],
            ..Default::default()
        },
        power: 7,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        affinity_filter: Some(SelectionRequirement::HasArtifactSubtype(
            ArtifactSubtype::Equipment,
        )),
        ..Default::default()
    }
}

/// Rebel Salvo — {2}{R} Instant. Affinity for Equipment. Deal 5 to target
/// creature or planeswalker; it loses indestructible until end of turn.
pub fn rebel_salvo() -> CardDefinition {
    CardDefinition {
        name: "Rebel Salvo",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(SelectionRequirement::HasArtifactSubtype(
            ArtifactSubtype::Equipment,
        )),
        effect: Effect::Seq(vec![
            Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
            },
            deal(
                5,
                target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            ),
        ]),
        ..Default::default()
    }
}

/// Leonin Lightbringer — {2}{W} 3/2 Ward {2}; +1/+1 while equipped.
pub fn leonin_lightbringer() -> CardDefinition {
    CardDefinition {
        name: "Leonin Lightbringer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Rebel],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is equipped, it gets +1/+1.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsEquipped,
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Resistance Reunited — {1}{W} Instant. Target creature gets +2/+2; your
/// equipped creatures gain indestructible until end of turn.
pub fn resistance_reunited() -> CardDefinition {
    CardDefinition {
        name: "Resistance Reunited",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::EquippedByAtLeast(1)),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Plated Onslaught — {3}{W}{W} Instant. Affinity for artifacts. Creatures you
/// control get +2/+1 until end of turn.
pub fn plated_onslaught() -> CardDefinition {
    CardDefinition {
        name: "Plated Onslaught",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(SelectionRequirement::Artifact),
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(2),
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Jor Kadeen, First Goldwarden — {R}{W} 2/2 trample. On attack: +X/+X EOT,
/// X = your equipped creatures; then if power ≥ 4, draw a card.
pub fn jor_kadeen_first_goldwarden() -> CardDefinition {
    let equipped_count = Value::count(Selector::EachPermanent(
        SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::EquippedByAtLeast(1)),
    ));
    CardDefinition {
        name: "Jor Kadeen, First Goldwarden",
        cost: cost(&[r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: equipped_count.clone(),
                toughness: equipped_count,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(Selector::This)),
                    Value::Const(4),
                ),
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Kemba, Kha Enduring — {1}{W} 2/2. Whenever Kemba or another Cat you control
/// enters, attach up to one target Equipment you control to it. Your equipped
/// creatures get +1/+1. {3}{W}{W}: create a 2/2 white Cat.
pub fn kemba_kha_enduring() -> CardDefinition {
    let cat = TokenDefinition {
        name: "Cat".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Kemba, Kha Enduring",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Cat),
                }),
            effect: Effect::Attach {
                what: target_filtered(
                    SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                to: Selector::TriggerSource,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Equipped creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::EquippedByAtLeast(1)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), w()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(cat),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sword of Forge and Frontier — {3} Equipment. +2/+2, pro-red and pro-green;
/// combat damage to a player: impulse-exile two + an extra land play. Equip {2}.
pub fn sword_of_forge_and_frontier() -> CardDefinition {
    CardDefinition {
        name: "Sword of Forge and Frontier",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![
                Keyword::Protection(Color::Red),
                Keyword::Protection(Color::Green),
            ],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: false,
                        uncast_penalty: None,
                    },
                    Effect::GrantExtraLandPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                    },
                ]),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Sphere lands + skullbombs + commons ──────────────────────────────────────

/// The five common "Land — Sphere" taplands: enters tapped, {T}: Add [color],
/// {1}[color], {T}, Sacrifice: draw a card.
fn sphere_land(name: &'static str, color: Color) -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Sphere],
            ..Default::default()
        },
        activated_abilities: vec![
            super::tap_add(color),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), crate::mana::colored(color)]),
                tap_cost: true,
                sac_cost: true,
                effect: draw(1),
                ..Default::default()
            },
        ],
        triggered_abilities: vec![super::etb_tap()],
        ..Default::default()
    }
}

pub fn the_autonomous_furnace() -> CardDefinition {
    sphere_land("The Autonomous Furnace", Color::Red)
}
pub fn the_dross_pits() -> CardDefinition {
    sphere_land("The Dross Pits", Color::Black)
}
pub fn the_fair_basilica() -> CardDefinition {
    sphere_land("The Fair Basilica", Color::White)
}
pub fn the_hunter_maze() -> CardDefinition {
    sphere_land("The Hunter Maze", Color::Green)
}
pub fn the_surgical_bay() -> CardDefinition {
    sphere_land("The Surgical Bay", Color::Blue)
}

/// Skullbomb: {1}, Sacrifice: draw a card — plus a colored sorcery-speed
/// sacrifice mode that also draws.
fn skullbomb(name: &'static str, mode_cost: crate::mana::ManaCost, mode: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: draw(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: mode_cost,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![mode, draw(1)]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Basilica Skullbomb — {2}{W}, Sac: your creature gets +2/+2 and flying EOT;
/// draw.
pub fn basilica_skullbomb() -> CardDefinition {
    skullbomb(
        "Basilica Skullbomb",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Dross Skullbomb — {2}{B}, Sac: return a creature card from your graveyard
/// to your hand; draw.
pub fn dross_skullbomb() -> CardDefinition {
    skullbomb(
        "Dross Skullbomb",
        cost(&[generic(2), b()]),
        Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Furnace Skullbomb — {1}{R}, Sac: put two oil counters on your artifact or
/// creature; draw.
pub fn furnace_skullbomb() -> CardDefinition {
    skullbomb(
        "Furnace Skullbomb",
        cost(&[generic(1), r()]),
        Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::Oil,
            amount: Value::Const(2),
        },
    )
}

/// Surgical Skullbomb — {2}{U}, Sac: bounce target creature; draw.
pub fn surgical_skullbomb() -> CardDefinition {
    skullbomb(
        "Surgical Skullbomb",
        cost(&[generic(2), u()]),
        Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
    )
}

/// Maze Skullbomb — {2}{G}, Sac: your creature gets +3/+3 and trample EOT; draw.
pub fn maze_skullbomb() -> CardDefinition {
    skullbomb(
        "Maze Skullbomb",
        cost(&[generic(2), g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
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

/// Tyrranax Atrocity — {3}{G}{G} 4/4 haste, toxic 3.
pub fn tyrranax_atrocity() -> CardDefinition {
    CardDefinition {
        name: "Tyrranax Atrocity",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste, Keyword::Toxic(3)],
        ..Default::default()
    }
}

/// Resistance Skywarden — {3}{R}{R} 5/5 menace, reach.
pub fn resistance_skywarden() -> CardDefinition {
    CardDefinition {
        name: "Resistance Skywarden",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rebel],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Menace, Keyword::Reach],
        ..Default::default()
    }
}

/// Skyscythe Engulfer — {5}{G} 6/5 reach, trample; can't be blocked by
/// creatures with flying.
pub fn skyscythe_engulfer() -> CardDefinition {
    CardDefinition {
        name: "Skyscythe Engulfer",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![
            Keyword::Reach,
            Keyword::Trample,
            Keyword::CantBeBlockedBy(Box::new(SelectionRequirement::HasKeyword(Keyword::Flying))),
        ],
        ..Default::default()
    }
}

/// Duelist of Deep Faith — {1}{W} 2/2 toxic 1; first strike during your turn.
pub fn duelist_of_deep_faith() -> CardDefinition {
    CardDefinition {
        name: "Duelist of Deep Faith",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Toxic(1)],
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::FirstStrike,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Paladin of Predation — {5}{G}{G} 6/7 toxic 6; can't be blocked by power ≤ 2.
pub fn paladin_of_predation() -> CardDefinition {
    CardDefinition {
        name: "Paladin of Predation",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Knight],
            ..Default::default()
        },
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::Toxic(6), Keyword::CantBeBlockedByPowerAtMost(2)],
        ..Default::default()
    }
}

/// Minor Misstep — {U} Instant. Counter target spell with mana value 1 or less.
pub fn minor_misstep() -> CardDefinition {
    CardDefinition {
        name: "Minor Misstep",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(SelectionRequirement::ManaValueAtMost(1)),
            ),
        },
        ..Default::default()
    }
}

/// Quicksilver Fisher — {3}{U}{U} 4/3 flying. ETB: draw a card, then discard
/// a card.
pub fn quicksilver_fisher() -> CardDefinition {
    CardDefinition {
        name: "Quicksilver Fisher",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Drake],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(1),
            Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: false,
            },
        ]))],
        ..Default::default()
    }
}

/// Free from Flesh — {R} Instant. Target creature gets +2/+2 until end of
/// turn; put two oil counters on it.
pub fn free_from_flesh() -> CardDefinition {
    CardDefinition {
        name: "Free from Flesh",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Oil,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Compleat Devotion — {1}{W} Instant. Your creature gets +2/+2 EOT; if it has
/// toxic, draw a card.
pub fn compleat_devotion() -> CardDefinition {
    CardDefinition {
        name: "Compleat Devotion",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasToxic,
                },
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Hexgold Slash — {R} Instant. Deal 2 to target creature — 4 instead if it
/// has toxic.
pub fn hexgold_slash() -> CardDefinition {
    CardDefinition {
        name: "Hexgold Slash",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::HasToxic,
            },
            then: Box::new(deal(4, target_filtered(SelectionRequirement::Creature))),
            else_: Box::new(deal(2, target_filtered(SelectionRequirement::Creature))),
        },
        ..Default::default()
    }
}

/// Offer Immortality — {1}{B} Instant. Target creature gains deathtouch and
/// indestructible until end of turn.
pub fn offer_immortality() -> CardDefinition {
    CardDefinition {
        name: "Offer Immortality",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Orthodoxy Enforcer — {3}{W} 2/4 vigilance; +2/+0 while you control 2+
/// artifacts.
pub fn orthodoxy_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Orthodoxy Enforcer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+0 as long as you control two or more artifacts.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(2),
                },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Cephalopod Sentry — {2}{W}{U} Artifact Creature */5 flying; power = your
/// artifact count (CR 604.3 CDA).
pub fn cephalopod_sentry() -> CardDefinition {
    CardDefinition {
        name: "Cephalopod Sentry",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Squid],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower {
            base_p: 0,
            base_t: 5,
        }),
        ..Default::default()
    }
}

// ── Oil-counter engine cards + more commons ──────────────────────────────────

/// "Whenever you cast a noncreature spell, put an oil counter on this."
fn oil_on_noncreature_cast() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::CastSpellMatches(SelectionRequirement::Noncreature),
        ),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Oil,
            amount: Value::ONE,
        },
    }
}

/// "Whenever another creature or artifact you control is put into a graveyard
/// from the battlefield, put an oil counter on this."
fn oil_on_another_dying() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::CreatureOrArtifactDied,
            EventScope::AnotherOfYours,
        ),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Oil,
            amount: Value::ONE,
        },
    }
}

/// Trawler Drake — {2}{U} 0/0 flying; enters with an oil counter; +1/+1 per
/// oil; noncreature casts add oil.
pub fn trawler_drake() -> CardDefinition {
    CardDefinition {
        name: "Trawler Drake",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Drake],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Oil, Value::ONE)),
        dynamic_pt: Some(DynamicPt::BasePlusCountersOnSelf {
            counter_type: CounterType::Oil,
            base_p: 0,
            base_t: 0,
            per_p: 1,
            per_t: 1,
        }),
        triggered_abilities: vec![oil_on_noncreature_cast()],
        ..Default::default()
    }
}

/// Necrosquito — {3}{B} 0/0 flying; enters with two oil; +1/+1 per oil;
/// another creature/artifact dying adds oil.
pub fn necrosquito() -> CardDefinition {
    CardDefinition {
        name: "Necrosquito",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        dynamic_pt: Some(DynamicPt::BasePlusCountersOnSelf {
            counter_type: CounterType::Oil,
            base_p: 0,
            base_t: 0,
            per_p: 1,
            per_t: 1,
        }),
        triggered_abilities: vec![oil_on_another_dying()],
        ..Default::default()
    }
}

/// Exuberant Fuseling — {R} 0/1 trample; +1/+0 per oil; ETB and another
/// creature/artifact dying add oil.
pub fn exuberant_fuseling() -> CardDefinition {
    CardDefinition {
        name: "Exuberant Fuseling",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Goblin,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        toughness: 1,
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::BasePlusCountersOnSelf {
            counter_type: CounterType::Oil,
            base_p: 0,
            base_t: 1,
            per_p: 1,
            per_t: 0,
        }),
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Oil,
                amount: Value::ONE,
            }),
            oil_on_another_dying(),
        ],
        ..Default::default()
    }
}

/// Serum Sovereign — {4}{U} 4/4 flying; noncreature casts add oil; {U},
/// remove an oil: draw, then scry 2.
pub fn serum_sovereign() -> CardDefinition {
    CardDefinition {
        name: "Serum Sovereign",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Sphinx],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![oil_on_noncreature_cast()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ichor Synthesizer — {1}{U} 1/3; noncreature casts add oil; at 4+ oil it
/// gets +2/+0 and can't be blocked.
pub fn ichor_synthesizer() -> CardDefinition {
    CardDefinition {
        name: "Ichor Synthesizer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![oil_on_noncreature_cast()],
        static_abilities: vec![StaticAbility {
            description: "At four or more oil counters: +2/+0 and can't be blocked.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Oil,
                    },
                    Value::Const(4),
                ),
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
            },
        }],
        ..Default::default()
    }
}

/// Gitaxian Raptor — {2}{U} 1/4 flying; enters with three oil; remove an oil:
/// +1/-1 until end of turn.
pub fn gitaxian_raptor() -> CardDefinition {
    CardDefinition {
        name: "Gitaxian Raptor",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Oil, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Furnace Strider — {4}{R} 4/5; enters with two oil; remove an oil: target
/// creature you control gains haste until end of turn.
pub fn furnace_strider() -> CardDefinition {
    CardDefinition {
        name: "Furnace Strider",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Norn's Wellspring — {1}{W}. Your creatures dying scry 1 + add oil; {1},
/// {T}, remove two oil: draw a card.
pub fn norns_wellspring() -> CardDefinition {
    CardDefinition {
        name: "Norn's Wellspring",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 2)),
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vat of Rebirth — {B}. Another artifact/creature dying adds oil; {2}{B},
/// {T}, remove four oil: reanimate a creature card (sorcery-only).
pub fn vat_of_rebirth() -> CardDefinition {
    CardDefinition {
        name: "Vat of Rebirth",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![oil_on_another_dying()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            sorcery_speed: true,
            remove_counter_cost: Some((CounterType::Oil, 4)),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tablet of Compleation — {2}. {T}: add an oil counter. {T}: Add {C} (2+
/// oil). {1}, {T}: draw (5+ oil).
pub fn tablet_of_compleation() -> CardDefinition {
    let oil_at_least = |n: i32| {
        Predicate::ValueAtLeast(
            Value::CountersOn {
                what: Box::new(Selector::This),
                kind: CounterType::Oil,
            },
            Value::Const(n),
        )
    };
    CardDefinition {
        name: "Tablet of Compleation",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(oil_at_least(2)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                condition: Some(oil_at_least(5)),
                effect: draw(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Font of Progress — {U}. Enters with two oil; {3}, {T}: target player mills
/// X, X = oil counters on this.
pub fn font_of_progress() -> CardDefinition {
    CardDefinition {
        name: "Font of Progress",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Oil,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Incubation Sac — {G}. Enters with three oil; {4}, {T}, remove an oil: make
/// a 3/3 Golem (sorcery-only).
pub fn incubation_sac() -> CardDefinition {
    CardDefinition {
        name: "Incubation Sac",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Oil, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sorcery_speed: true,
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Phyrexian Golem".into(),
                    power: 3,
                    toughness: 3,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Urabrask's Forge — {2}{R}. At combat on your turn: add an oil counter,
/// then mint an X/1 trample haste Horror (X = oil), sacrificed at end step.
pub fn urabrasks_forge() -> CardDefinition {
    let oil = Value::CountersOn {
        what: Box::new(Selector::This),
        kind: CounterType::Oil,
    };
    CardDefinition {
        name: "Urabrask's Forge",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Phyrexian Horror".into(),
                        power: 0,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Trample, Keyword::Haste],
                        dynamic_pt: Some((oil.clone(), Value::ONE)),
                        ..Default::default()
                    }),
                },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ]),
        }],
        ..Default::default()
    }
}

/// Watchful Blisterzoa — {4}{U}{U} 4/4 flying; enters with an oil counter;
/// dies: draw cards equal to its oil counters.
pub fn watchful_blisterzoa() -> CardDefinition {
    CardDefinition {
        name: "Watchful Blisterzoa",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Jellyfish],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Oil, Value::ONE)),
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::CountersOn {
                what: Box::new(Selector::This),
                kind: CounterType::Oil,
            },
        })],
        ..Default::default()
    }
}

/// Magmatic Sprinter — {2}{R} 3/2 haste. ETB: two oil on your artifact or
/// creature; your end step: bounce it unless it sheds two oil.
pub fn magmatic_sprinter() -> CardDefinition {
    CardDefinition {
        name: "Magmatic Sprinter",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::Oil,
                amount: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Oil,
                        },
                        Value::Const(2),
                    ),
                    then: Box::new(Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Oil,
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Lattice-Blade Mantis — {3}{G} 4/3; enters with two oil; on attack, may
/// shed an oil to untap it and get +1/+1 until end of turn.
pub fn lattice_blade_mantis() -> CardDefinition {
    CardDefinition {
        name: "Lattice-Blade Mantis",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Oil,
                },
                Value::ONE,
            ),
            then: Box::new(Effect::MayDo {
                description: "Remove an oil counter to untap and pump?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Oil,
                        amount: Value::ONE,
                    },
                    Effect::Untap {
                        what: Selector::This,
                        up_to: None,
                    },
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Forgehammer Centurion — {2}{R} 3/2; another creature/artifact dying adds
/// oil; on attack, may shed two oil: target creature can't block this turn.
pub fn forgehammer_centurion() -> CardDefinition {
    CardDefinition {
        name: "Forgehammer Centurion",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            oil_on_another_dying(),
            on_attack(Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Oil,
                    },
                    Value::Const(2),
                ),
                then: Box::new(Effect::MayDo {
                    description: "Remove two oil counters: a creature can't block this turn?"
                        .into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::Oil,
                            amount: Value::Const(2),
                        },
                        Effect::Reflexive {
                            body: Box::new(Effect::CantBlockSourceThisTurn {
                                target: target_filtered(SelectionRequirement::Creature),
                            }),
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..Default::default()
    }
}

/// Predation Steward — {1}{G} 2/2; enters with two oil; {2}{G}, {T}, remove
/// an oil: target creature +2/+2 until end of turn (sorcery-only).
pub fn predation_steward() -> CardDefinition {
    CardDefinition {
        name: "Predation Steward",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Elf,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            sorcery_speed: true,
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Evolved Spinoderm — {2}{G}{G} 5/5; enters with four oil; trample at ≤ 2
/// oil, hexproof otherwise; upkeep sheds an oil, sacrificed when dry.
pub fn evolved_spinoderm() -> CardDefinition {
    let oil = || Value::CountersOn {
        what: Box::new(Selector::This),
        kind: CounterType::Oil,
    };
    CardDefinition {
        name: "Evolved Spinoderm",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        enters_with_counters: Some((CounterType::Oil, Value::Const(4))),
        static_abilities: vec![
            StaticAbility {
                description: "Trample as long as it has two or fewer oil counters.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Trample,
                    condition: Predicate::ValueAtMost(oil(), Value::Const(2)),
                },
            },
            StaticAbility {
                description: "Otherwise, hexproof.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Hexproof,
                    condition: Predicate::ValueAtLeast(oil(), Value::Const(3)),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtMost(oil(), Value::Const(0)),
                    then: Box::new(Effect::SacrificePermanent {
                        what: Selector::This,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Eye of Malcator — {2}{U}. ETB scry 2; another artifact entering animates
/// it into a 4/4 Phyrexian Eye until end of turn.
pub fn eye_of_malcator() -> CardDefinition {
    CardDefinition {
        name: "Eye of Malcator",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Artifact,
                    }),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Phyrexian, CreatureType::Eye],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Mandible Justiciar — {1}{W} 2/1 lifelink; another artifact entering gives
/// it +1/+1 until end of turn.
pub fn mandible_justiciar() -> CardDefinition {
    CardDefinition {
        name: "Mandible Justiciar",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Escaped Experiment — {1}{U} 2/1; on attack, an opponent's creature gets
/// -X/-0 until end of turn, X = your artifact count.
pub fn escaped_experiment() -> CardDefinition {
    CardDefinition {
        name: "Escaped Experiment",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Diff(
                Box::new(Value::Const(0)),
                Box::new(Value::count(Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                ))),
            ),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Meldweb Strider — {4}{U} Vehicle 5/5 vigilance; enters with an oil
/// counter; remove an oil: becomes a creature until end of turn. Crew 3.
pub fn meldweb_strider() -> CardDefinition {
    CardDefinition {
        name: "Meldweb Strider",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Vigilance, Keyword::Crew(3)],
        enters_with_counters: Some((CounterType::Oil, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(5),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ribskiff — {4} Vehicle 4/4, toxic 2; ETB draw a card. Crew 3.
pub fn ribskiff() -> CardDefinition {
    CardDefinition {
        name: "Ribskiff",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Toxic(2), Keyword::Crew(3)],
        triggered_abilities: vec![etb(draw(1))],
        ..Default::default()
    }
}

/// Gleeful Demolition — {R} Sorcery. Destroy target artifact; if you
/// controlled it, create three 1/1 red Phyrexian Goblins.
pub fn gleeful_demolition() -> CardDefinition {
    CardDefinition {
        name: "Gleeful Demolition",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::ControlledByYou,
            },
            then: Box::new(Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Artifact),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: Box::new(phyrexian_goblin_token()),
                },
            ])),
            else_: Box::new(Effect::Destroy {
                what: target_filtered(SelectionRequirement::Artifact),
            }),
        },
        ..Default::default()
    }
}

/// Testament Bearer — {3}{B} 4/1; dies: look at the top three, one to hand,
/// rest to graveyard.
pub fn testament_bearer() -> CardDefinition {
    CardDefinition {
        name: "Testament Bearer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
    ..Default::default()
})))],
        ..Default::default()
    }
}

/// Meldweb Curator — {3}{U} 3/4; ETB: up to one target instant or sorcery
/// card from your graveyard goes on top of your library.
pub fn meldweb_curator() -> CardDefinition {
    CardDefinition {
        name: "Meldweb Curator",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                    .and(SelectionRequirement::InYourGraveyard),
            ),
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: crate::effect::LibraryPosition::Top,
            },
        })],
        ..Default::default()
    }
}

/// Nimraiser Paladin — {4}{B} 4/4, toxic 2; ETB: return a creature card with
/// mana value ≤ 3 from your graveyard to your hand.
pub fn nimraiser_paladin() -> CardDefinition {
    CardDefinition {
        name: "Nimraiser Paladin",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Toxic(2)],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::InYourGraveyard)
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Stinging Hivemaster — {2}{B} 3/2, toxic 1; dies: create a Phyrexian Mite.
pub fn stinging_hivemaster() -> CardDefinition {
    CardDefinition {
        name: "Stinging Hivemaster",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(mite_token()),
        })],
        ..Default::default()
    }
}

/// Flensing Raptor — {2}{W} 2/2 flying, toxic 1; ETB: another target toxic
/// creature you control gets +1/+1 and flying until end of turn.
pub fn flensing_raptor() -> CardDefinition {
    CardDefinition {
        name: "Flensing Raptor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Toxic(1)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasToxic)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Myr Kinsmith — {4} 3/1; ETB: search your library for a Myr card to hand.
pub fn myr_kinsmith() -> CardDefinition {
    CardDefinition {
        name: "Myr Kinsmith",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Myr],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCreatureType(CreatureType::Myr),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Myr Convert — {2} 2/1, toxic 1; {T}, pay 2 life: add one mana of any color.
pub fn myr_convert() -> CardDefinition {
    CardDefinition {
        name: "Myr Convert",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Myr],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Toxic(1)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Myr Custodian — {3} 2/3; ETB: scry 2, then each opponent may scry 1.
pub fn myr_custodian() -> CardDefinition {
    CardDefinition {
        name: "Myr Custodian",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Myr],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Scry {
                who: PlayerRef::EachOpponent,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Feed the Infection — {3}{B} Sorcery. Draw three, lose 3. Corrupted — each
/// opponent with three or more poison counters loses 3 life (1v1: gated on
/// the Corrupted check).
pub fn feed_the_infection() -> CardDefinition {
    CardDefinition {
        name: "Feed the Infection",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            draw(3),
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::CorruptedActive {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Plague Nurse — {3}{G} 3/4, toxic 2; {2}{G}: other toxic creatures you
/// control gain toxic 1 until end of turn (once per turn).
pub fn plague_nurse() -> CardDefinition {
    CardDefinition {
        name: "Plague Nurse",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Toxic(2)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            once_per_turn: true,
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasToxic)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Toxic(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dune Mover — {2} 2/1, toxic 1; ETB: may search a basic land, then shuffle
/// and put it on top.
pub fn dune_mover() -> CardDefinition {
    CardDefinition {
        name: "Dune Mover",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: crate::effect::LibraryPosition::Top,
            },
        })],
        ..Default::default()
    }
}

// ── Wave 4: commons/uncommons + modal spells ─────────────────────────────────

/// Rustvine Cultivator — {G} 1/2. {T}: add an oil counter; {T}, remove an
/// oil: untap target land.
pub fn rustvine_cultivator() -> CardDefinition {
    CardDefinition {
        name: "Rustvine Cultivator",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Elf,
                CreatureType::Druid,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Oil, 1)),
                effect: Effect::Untap {
                    what: target_filtered(SelectionRequirement::Land),
                    up_to: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Oil-Gorger Troll — {3}{G}{G} 3/4. ETB: gain 3 life; if you control a
/// permanent with an oil counter, draw a card.
pub fn oil_gorger_troll() -> CardDefinition {
    CardDefinition {
        name: "Oil-Gorger Troll",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Troll,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            gain_life(3),
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::WithCounter(CounterType::Oil)
                        .and(SelectionRequirement::ControlledByYou),
                )),
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Molten Rebuke — {4}{R} Sorcery. Choose one or both — 5 damage to a
/// creature or planeswalker; destroy target Equipment.
pub fn molten_rebuke() -> CardDefinition {
    CardDefinition {
        name: "Molten Rebuke",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                deal(
                    5,
                    target_filtered(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                ),
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::HasArtifactSubtype(
                        ArtifactSubtype::Equipment,
                    )),
                },
            ],
        },
        ..Default::default()
    }
}

/// Tamiyo's Immobilizer — {3}{U}. Enters with four oil; {T}, remove an oil:
/// tap target artifact or creature.
pub fn tamiyos_immobilizer() -> CardDefinition {
    CardDefinition {
        name: "Tamiyo's Immobilizer",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Oil, Value::Const(4))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ruthless Predation — {1}{G} Sorcery. Your creature gets +1/+2, then
/// fights a creature you don't control.
pub fn ruthless_predation() -> CardDefinition {
    CardDefinition {
        name: "Ruthless Predation",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::ONE,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::Not(
                        Box::new(SelectionRequirement::ControlledByYou),
                    )),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Maze's Mantle — {2}{G} flash Aura. +2/+2; ETB: if enchanted creature has
/// toxic, it gains hexproof until end of turn.
pub fn mazes_mantle() -> CardDefinition {
    CardDefinition {
        name: "Maze's Mantle",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                filter: SelectionRequirement::HasToxic,
            },
            then: Box::new(Effect::GrantKeyword {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Porcelain Zealot — {3}{W} 2/3. At combat on your turn: your creature gets
/// +1/+1 — +2/+2 instead if it has toxic.
pub fn porcelain_zealot() -> CardDefinition {
    let pump = |n: i32| Effect::PumpPT {
        what: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        power: Value::Const(n),
        toughness: Value::Const(n),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Porcelain Zealot",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasToxic,
                },
                then: Box::new(pump(2)),
                else_: Box::new(pump(1)),
            },
        }],
        ..Default::default()
    }
}

/// Cinderslash Ravager — {4}{R}{G} 5/5 vigilance; costs {1} less per
/// oil-countered permanent you control; ETB: 1 damage to each opposing creature.
pub fn cinderslash_ravager() -> CardDefinition {
    CardDefinition {
        name: "Cinderslash Ravager",
        cost: cost(&[generic(4), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each permanent you control with oil counters on it.",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                filter: SelectionRequirement::WithCounter(CounterType::Oil)
                    .and(SelectionRequirement::ControlledByYou),
                per: 1,
            },
        }],
        triggered_abilities: vec![etb(deal(
            1,
            Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        ))],
        ..Default::default()
    }
}

/// Charge of the Mites — {2}{W} Instant. Choose one — damage equal to your
/// creature count to a creature or planeswalker; or two Mites.
pub fn charge_of_the_mites() -> CardDefinition {
    CardDefinition {
        name: "Charge of the Mites",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Box::new(mite_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Mite Overseer — {3}{W} 4/2 first strike. During your turn, your creature
/// tokens get +1/+0 and have first strike. {3}{W/P}: create a Mite.
pub fn mite_overseer() -> CardDefinition {
    CardDefinition {
        name: "Mite Overseer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "During your turn, creature tokens you control get +1/+0 and have first strike.",
            effect: StaticEffect::AnthemForFilter {
                filter: SelectionRequirement::Creature.and(SelectionRequirement::IsToken),
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
                opponents: false,
                all_players: false,
                only_your_turn: true,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), phyrexian(Color::White)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(mite_token()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Veil of Assimilation — {1}{W}. This or another artifact entering: target
/// creature you control gets +1/+1 and gains vigilance until end of turn.
pub fn veil_of_assimilation() -> CardDefinition {
    CardDefinition {
        name: "Veil of Assimilation",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Urabrask's Anointer — {3}{R} 4/2. ETB: X damage to any target, X = your
/// oil-countered permanents.
pub fn urabrasks_anointer() -> CardDefinition {
    CardDefinition {
        name: "Urabrask's Anointer",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::count(Selector::EachPermanent(
                SelectionRequirement::WithCounter(CounterType::Oil)
                    .and(SelectionRequirement::ControlledByYou),
            )),
        })],
        ..Default::default()
    }
}

/// Planar Disruption — {1}{W} Aura on an artifact, creature, or planeswalker:
/// it can't attack or block and its activated abilities can't be activated.
pub fn planar_disruption() -> CardDefinition {
    CardDefinition {
        name: "Planar Disruption",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .or(SelectionRequirement::Planeswalker),
            ),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::CantAttack,
                Keyword::CantBlock,
                Keyword::CantActivateAbilities,
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Grafted from the parallel session's commons batch ───────────────────────

/// Cruel Grimnarch — {5}{B} 5/5 Phyrexian Cleric with deathtouch. ETB: each
/// opponent discards; you gain 4 life per opponent who couldn't.
pub fn cruel_grimnarch() -> CardDefinition {
    CardDefinition {
        name: "Cruel Grimnarch",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::EachOpponent), Value::ONE),
            then: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            }),
            else_: Box::new(gain_life(4)),
        })],
        ..Default::default()
    }
}

/// Awaken the Sleeper — {3}{R} Sorcery. Threaten target creature (untap,
/// haste); you may destroy all Equipment attached to it.
pub fn awaken_the_sleeper() -> CardDefinition {
    CardDefinition {
        name: "Awaken the Sleeper",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::MayDo {
                description: "Destroy all Equipment attached to it?".into(),
                body: Box::new(Effect::Destroy {
                    what: Selector::AttachedToMe(Box::new(Selector::Target(0))),
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── ONE remainder wave 1: rares/uncommons on existing primitives ────────────

/// A 3/3 colorless Phyrexian Golem artifact creature token (Malcator).
fn golem_token() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Chittering Skitterling — {2}{B} 1/4 Phyrexian Rat. Corrupted — Sacrifice an
/// artifact or creature: Draw a card. Only if an opponent has 3+ poison, once
/// each turn.
pub fn chittering_skitterling() -> CardDefinition {
    CardDefinition {
        name: "Chittering Skitterling",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                1,
            )),
            once_per_turn: true,
            condition: Some(Predicate::CorruptedActive {
                who: PlayerRef::You,
            }),
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Filigree Sylex — {2} Legendary Artifact. {T}: oil counter. {T}, Sac:
/// destroy each nonland permanent with MV = oil count. {T}, remove ten oil
/// from among your permanents, Sac: 10 damage to any target.
pub fn the_filigree_sylex() -> CardDefinition {
    CardDefinition {
        name: "The Filigree Sylex",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DestroyEachNonlandWithManaValue {
                    value: Value::TotalCountersOn {
                        what: Box::new(Selector::This),
                    },
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                remove_counter_among_filter: Some((
                    Some(CounterType::Oil),
                    10,
                    SelectionRequirement::Permanent.and(SelectionRequirement::ControlledByYou),
                )),
                effect: deal(10, target_any()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tamiyo's Logbook — {2}{U} Artifact — Book. {5}{U}, {T}: Draw a card. Costs
/// {1} less to activate per other artifact you control.
pub fn tamiyos_logbook() -> CardDefinition {
    CardDefinition {
        name: "Tamiyo's Logbook",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            tap_cost: true,
            cost_reduction_per: Some(
                SelectionRequirement::Artifact
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Staff of Compleation — {3} Artifact. {T}, Pay 1/2/3/4 life: destroy your
/// permanent / any-color mana / proliferate / draw. {5}: Untap it.
pub fn staff_of_compleation() -> CardDefinition {
    CardDefinition {
        name: "Staff of Compleation",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Permanent.and(SelectionRequirement::OwnedByYou),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                life_cost: 2,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                life_cost: 3,
                effect: Effect::Proliferate,
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                life_cost: 4,
                effect: draw(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                effect: Effect::Untap {
                    what: Selector::This,
                    up_to: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Koth, Fire of Resistance — {2}{R}{R} Planeswalker — Koth, loyalty 4.
/// +2: tutor a basic Mountain to hand. −3: damage = your Mountains to a
/// creature. −7: emblem "Mountain enters → 4 damage to any target".
pub fn koth_fire_of_resistance() -> CardDefinition {
    let mountain = SelectionRequirement::HasLandType(crate::card::LandType::Mountain);
    CardDefinition {
        name: "Koth, Fire of Resistance",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Koth],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            crate::effect::LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand.and(mountain.clone()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            crate::effect::LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Creature),
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(
                        mountain.clone().and(SelectionRequirement::ControlledByYou),
                    ))),
                },
                ..Default::default()
            },
            crate::effect::LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Koth, Fire of Resistance".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::EntersBattlefield,
                            EventScope::YourControl,
                        )
                        .with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: mountain,
                        }),
                        effect: deal(4, target_any()),
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Malcator, Purity Overseer — {1}{W}{U} 1/1. ETB: a 3/3 Golem. Your end step,
/// if three or more artifacts entered under your control this turn: a 3/3 Golem.
pub fn malcator_purity_overseer() -> CardDefinition {
    let mint_golem = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Box::new(golem_token()),
    };
    CardDefinition {
        name: "Malcator, Purity Overseer",
        cost: cost(&[generic(1), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Elephant,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            etb(mint_golem()),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::ArtifactsEnteredThisTurn(PlayerRef::You),
                    Value::Const(3),
                )),
                effect: mint_golem(),
            },
        ],
        ..Default::default()
    }
}

/// Geth, Thane of Contracts — {1}{B}{B} 3/4. Other creatures you control get
/// -1/-1. {1}{B}{B}, {T}, sorcery: reanimate a creature from your graveyard.
/// (The "exile it if it would leave" rider is a finality counter.)
pub fn geth_thane_of_contracts() -> CardDefinition {
    CardDefinition {
        name: "Geth, Thane of Contracts",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Zombie],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: -1,
                toughness: -1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::Finality,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ichorplate Golem — {3} 2/3. Your creatures entering with oil get another
/// oil counter; your oil-countered creatures get +1/+1.
pub fn ichorplate_golem() -> CardDefinition {
    CardDefinition {
        name: "Ichorplate Golem",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::WithCounter(CounterType::Oil)),
                }),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with oil counters on them get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CounterType::Oil)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Necrogen Rotpriest — {2}{B}{G} 1/5, toxic 2. Your toxic creatures' combat
/// damage to a player adds a poison counter. {1}{B}{G}: a toxic creature you
/// control gains deathtouch until end of turn.
pub fn necrogen_rotpriest() -> CardDefinition {
    CardDefinition {
        name: "Necrogen Rotpriest",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Zombie,
                CreatureType::Cleric,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Toxic(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasToxic,
            }),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasToxic),
                ),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Indoctrination Attendant — {3}{W} 3/4, toxic 1. ETB: you may bounce another
/// permanent you control; if you do, create a Mite.
pub fn indoctrination_attendant() -> CardDefinition {
    CardDefinition {
        name: "Indoctrination Attendant",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return another permanent you control to hand for a Mite?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::ChoosePermanentForSource {
                    filter: SelectionRequirement::Permanent
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                },
                Effect::Move {
                    what: Selector::ChosenPermanentOfSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(
                        Selector::ChosenPermanentOfSource,
                    ))),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(mite_token()),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Mirrex — Land — Sphere. {T}: {C}. {T}: any color if it entered this turn.
/// {3}, {T}: create a Mite.
pub fn mirrex() -> CardDefinition {
    CardDefinition {
        name: "Mirrex",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![crate::card::LandType::Sphere],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::EnteredThisTurn,
                }),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(mite_token()),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Monumental Facade — Land — Sphere, enters with two oil counters.
/// {T}: {C}. {T}, remove an oil counter: put an oil counter on target
/// artifact or creature you control. Sorcery only.
pub fn the_monumental_facade() -> CardDefinition {
    CardDefinition {
        name: "The Monumental Facade",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![crate::card::LandType::Sphere],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                remove_counter_cost: Some((CounterType::Oil, 1)),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Artifact
                            .or(SelectionRequirement::Creature)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Seedcore — Land — Sphere. {T}: {C}. {T}: any color for Phyrexian
/// creature spells. Corrupted — {T}: target 1/1 creature gets +2/+1 until EOT.
pub fn the_seedcore() -> CardDefinition {
    CardDefinition {
        name: "The Seedcore",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![crate::card::LandType::Sphere],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Restricted(
                        Box::new(crate::effect::ManaPayload::AnyOneColor(Value::ONE)),
                        crate::mana::SpendRestriction::CreatureOfType(CreatureType::Phyrexian),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::CorruptedActive {
                    who: PlayerRef::You,
                }),
                effect: Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::PowerAtLeast(1))
                            .and(SelectionRequirement::PowerAtMost(1))
                            .and(SelectionRequirement::ToughnessAtLeast(1))
                            .and(SelectionRequirement::ToughnessAtMost(1)),
                    ),
                    power: Value::Const(2),
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Zealot's Conviction — {W} Aura with flash. +1/+1; Corrupted — an additional
/// +1/+0 and first strike.
pub fn zealots_conviction() -> CardDefinition {
    CardDefinition {
        name: "Zealot's Conviction",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            conditional: vec![crate::card::ConditionalEquipBonus {
                host_filter: SelectionRequirement::Creature,
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
                condition: Some(Predicate::CorruptedActive {
                    who: PlayerRef::You,
                }),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Transplant Theorist — {3}{U} 2/4. Your artifacts (including this) entering
/// let you loot. {2}: bottom target card from your graveyard.
pub fn transplant_theorist() -> CardDefinition {
    CardDefinition {
        name: "Transplant Theorist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::MayDo {
                description: "Draw a card, then discard a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    draw(1),
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::InYourGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Phyrexian Atlas — {3} Artifact. {T}: any color. Corrupted — on becoming
/// tapped, each poisoned-out opponent loses 1 life (exact in two-player).
pub fn phyrexian_atlas() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Atlas",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource).with_filter(
                Predicate::CorruptedActive {
                    who: PlayerRef::You,
                },
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Slobad, Iron Goblin — {2}{R} 3/3. {T}, Sacrifice an artifact: add {R} equal
/// to its mana value, spendable only on artifacts.
pub fn slobad_iron_goblin() -> CardDefinition {
    CardDefinition {
        name: "Slobad, Iron Goblin",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Goblin,
                CreatureType::Artificer,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((SelectionRequirement::Artifact, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Restricted(
                    Box::new(crate::effect::ManaPayload::OfColor(
                        Color::Red,
                        Value::SacrificedManaValue,
                    )),
                    crate::mana::SpendRestriction::ArtifactOnly,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Venerated Rotpriest — {G} 1/2, toxic 1. Whenever a creature you control
/// becomes the target of a spell, target opponent gets a poison counter.
pub fn venerated_rotpriest() -> CardDefinition {
    CardDefinition {
        name: "Venerated Rotpriest",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted),
            effect: Effect::AddPoison {
                who: target_filtered(SelectionRequirement::OpponentPlayer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Unctus, Grand Metatect — {1}{U}{U} 2/4. Other blue creatures you control
/// loot when tapped; other artifact creatures get +1/+1. {U/P}, sorcery:
/// target creature you control becomes a blue artifact in addition until EOT.
pub fn unctus_grand_metatect() -> CardDefinition {
    CardDefinition {
        name: "Unctus, Grand Metatect",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Vedalken],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Other blue creatures you control have \"Whenever this creature becomes \
                     tapped, draw a card, then discard a card.\"",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasColor(Color::Blue))
                        .and(SelectionRequirement::OtherThanSource),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                        effect: Effect::Seq(vec![
                            draw(1),
                            Effect::Discard {
                                who: Selector::You,
                                amount: Value::ONE,
                                random: false,
                            },
                        ]),
                    }),
                },
            },
            StaticAbility {
                description: "Other artifact creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Artifact
                            .and(SelectionRequirement::Creature)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[phyrexian(Color::Blue)]),
            sorcery_speed: true,
            effect: Effect::AddCardTypeIndefinitely {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                card_type: CardType::Artifact,
                until_eot: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tyvar, Jubilant Brawler — {1}{B}{G} Planeswalker — Tyvar, loyalty 3.
/// Your creatures' abilities activate as though they had haste. +1: untap a
/// creature. −2: mill 3, then may reanimate a MV≤2 creature.
pub fn tyvar_jubilant_brawler() -> CardDefinition {
    CardDefinition {
        name: "Tyvar, Jubilant Brawler",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Tyvar],
            ..Default::default()
        },
        base_loyalty: 3,
        static_abilities: vec![StaticAbility {
            description: "You may activate abilities of creatures you control as though those creatures \
                 had haste.",
            effect: StaticEffect::ControllerCreatureAbilitiesAsThoughHaste,
        }],
        loyalty_abilities: vec![
            crate::effect::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Untap {
                    what: target_filtered(SelectionRequirement::Creature),
                    up_to: None,
                },
                ..Default::default()
            },
            crate::effect::LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Mill {
                        who: Selector::You,
                        amount: Value::Const(3),
                    },
                    Effect::MayDo {
                        description: "Return a creature with mana value 2 or less?".into(),
                        body: Box::new(Effect::Move {
                            what: target_filtered(
                                SelectionRequirement::Creature
                                    .and(SelectionRequirement::InYourGraveyard)
                                    .and(SelectionRequirement::ManaValueAtMost(2)),
                            ),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nahiri's Sacrifice — {1}{R} Sorcery. Sacrifice an artifact or creature with
/// mana value X; deal X damage divided among any number of target creatures.
pub fn nahiris_sacrifice() -> CardDefinition {
    CardDefinition {
        name: "Nahiri's Sacrifice",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: 1,
        }],
        effect: Effect::DealDamageDivided {
            retaliate_to_source: false,
            total: Value::SacrificedManaValue,
            filter: SelectionRequirement::Creature,
            max_targets: 5,
        },
        ..Default::default()
    }
}

/// Atraxa's Skitterfang — {3} 2/2, enters with three oil counters. At combat
/// on your turn, may remove one: a creature you control gains flying,
/// vigilance, deathtouch, or lifelink until end of turn.
pub fn atraxas_skitterfang() -> CardDefinition {
    let grant = |keyword: Keyword| Effect::GrantKeyword {
        what: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        keyword,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Atraxa's Skitterfang",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((CounterType::Oil, Value::Const(3))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayDo {
                description: "Remove an oil counter to grant a keyword?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Oil,
                        amount: Value::ONE,
                    },
                    Effect::ChooseMode(vec![
                        grant(Keyword::Flying),
                        grant(Keyword::Vigilance),
                        grant(Keyword::Deathtouch),
                        grant(Keyword::Lifelink),
                    ]),
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Serum-Core Chimera — {2}{U}{R} 2/4 flying; noncreature casts add oil.
/// Remove three oil, sorcery: draw, then may discard for 3 damage.
pub fn serum_core_chimera() -> CardDefinition {
    CardDefinition {
        name: "Serum-Core Chimera",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Chimera],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_noncreature()),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            remove_counter_cost: Some((CounterType::Oil, 3)),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::MayDiscard {
                    description: "Discard a nonland card for 3 damage?".into(),
                    count: Value::ONE,
                    then: Box::new(Effect::DealDamage {
                        to: target_filtered(
                            SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                        ),
                        amount: Value::Const(3),
                    }),
                    else_: None,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── ONE planeswalkers (CR 702.150 Compleated + the uncompleated four) ────────

use crate::card::PlaneswalkerSubtype;
use crate::effect::LoyaltyAbility;
use crate::mana::phyrexian_hybrid;

/// A 3/3 green Phyrexian Beast token with toxic 1 (Lukka −1, Goliath Hatchery).
fn beast_token() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Beast".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        keywords: vec![Keyword::Toxic(1)],
        ..Default::default()
    }
}

fn target_player() -> Selector {
    target_filtered(SelectionRequirement::Player)
}

/// Jace, the Perfected Mind — {2}{U}{U/P}, Compleated, loyalty 5. +1: up to one
/// creature gets -3/-0 until your next turn. −2: mill 3, draw 3 if a graveyard
/// holds 20+, else 1. −X: target player mills 3X.
pub fn jace_the_perfected_mind() -> CardDefinition {
    CardDefinition {
        name: "Jace, the Perfected Mind",
        cost: cost(&[generic(2), u(), phyrexian(Color::Blue)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace],
            ..Default::default()
        },
        keywords: vec![Keyword::Compleated],
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(0),
                    duration: Duration::UntilYourNextUntap,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Mill {
                        who: target_player(),
                        amount: Value::Const(3),
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::MaxGraveyardSize, Value::Const(20)),
                        then: Box::new(draw(3)),
                        else_: Box::new(draw(1)),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                x_cost: true,
                effect: Effect::Mill {
                    who: target_player(),
                    amount: Value::Times(Box::new(Value::Const(3)), Box::new(Value::XFromCost)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Vraska, Betrayal's Sting — {4}{B}{B/P}, Compleated, loyalty 6. 0: draw,
/// lose 1, proliferate. −2: a creature becomes a Treasure. −9: target player
/// is topped up to nine poison counters.
pub fn vraska_betrayals_sting() -> CardDefinition {
    CardDefinition {
        name: "Vraska, Betrayal's Sting",
        cost: cost(&[generic(4), b(), phyrexian(Color::Black)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Vraska],
            ..Default::default()
        },
        keywords: vec![Keyword::Compleated],
        base_loyalty: 6,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::Proliferate,
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::BecomeTreasure {
                    what: target_filtered(SelectionRequirement::Creature),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::AddPoison {
                    who: target_player(),
                    amount: Value::NonNeg(Box::new(Value::Diff(
                        Box::new(Value::Const(9)),
                        Box::new(Value::PoisonCountersOf(PlayerRef::Target(0))),
                    ))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lukka, Bound to Ruin — {2}{R}{R/G/P}{G}, Compleated, loyalty 5. +1: add
/// {R}{G} for creatures. −1: a 3/3 toxic Beast. −4: greatest-power damage
/// divided among creatures and planeswalkers.
pub fn lukka_bound_to_ruin() -> CardDefinition {
    CardDefinition {
        name: "Lukka, Bound to Ruin",
        cost: cost(&[
            generic(2),
            r(),
            phyrexian_hybrid(Color::Red, Color::Green),
            g(),
        ]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Lukka],
            ..Default::default()
        },
        keywords: vec![Keyword::Compleated],
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Restricted(
                        Box::new(crate::effect::ManaPayload::Colors(vec![
                            Color::Red,
                            Color::Green,
                        ])),
                        crate::mana::SpendRestriction::CreatureSpellsOrAbilities,
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(beast_token()),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::DealDamageDivided {
                    retaliate_to_source: false,
                    total: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                    filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    max_targets: 5,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nahiri, the Unforgiving — {1}{R}{R/W/P}{W}, Compleated, loyalty 5.
/// +1: a creature must attack until your next turn / loot. 0: exile a creature
/// or Equipment card from your graveyard for a hasty copy, exiled at end step.
pub fn nahiri_the_unforgiving() -> CardDefinition {
    CardDefinition {
        name: "Nahiri, the Unforgiving",
        cost: cost(&[
            generic(1),
            r(),
            phyrexian_hybrid(Color::Red, Color::White),
            w(),
        ]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nahiri],
            ..Default::default()
        },
        keywords: vec![Keyword::Compleated],
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::MustAttack,
                    duration: Duration::UntilYourNextUntap,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                    draw(1),
                ]),
                ..Default::default()
            },
            // (The "mana value less than Nahiri's loyalty" cap is dropped.)
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: target_filtered(SelectionRequirement::InYourGraveyard.and(
                            SelectionRequirement::Creature.or(
                                SelectionRequirement::HasArtifactSubtype(
                                    ArtifactSubtype::Equipment,
                                ),
                            ),
                        )),
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![Keyword::Haste],
                    },
                    Effect::ExileLastCreatedTokensAtNextEndStep,
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Exile,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nissa, Ascended Animist — {3}{G}{G}{G/P}{G/P}, Compleated, loyalty 7.
/// +1: an X/X Horror where X is her loyalty. −1: naturalize. −7: your team
/// gets +1/+1 per Forest and trample.
pub fn nissa_ascended_animist() -> CardDefinition {
    let forests = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            SelectionRequirement::HasLandType(crate::card::LandType::Forest)
                .and(SelectionRequirement::ControlledByYou),
        )))
    };
    CardDefinition {
        name: "Nissa, Ascended Animist",
        cost: cost(&[
            generic(3),
            g(),
            g(),
            phyrexian(Color::Green),
            phyrexian(Color::Green),
        ]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nissa],
            ..Default::default()
        },
        keywords: vec![Keyword::Compleated],
        base_loyalty: 7,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Phyrexian Horror".into(),
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
                            ..Default::default()
                        },
                        dynamic_pt: Some((
                            Value::LoyaltyOf(Box::new(Selector::This)),
                            Value::LoyaltyOf(Box::new(Selector::This)),
                        )),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        power: forests(),
                        toughness: forests(),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kaya, Intangible Slayer — {3}{W}{W}{B}{B}, hexproof, loyalty 6. +2: drain 3.
/// 0: draw two, opponents scry. −3: exile a creature or enchantment; non-Aura
/// leaves behind a 1/1 flying Spirit copy.
pub fn kaya_intangible_slayer() -> CardDefinition {
    CardDefinition {
        name: "Kaya, Intangible Slayer",
        cost: cost(&[generic(3), w(), w(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Kaya],
            ..Default::default()
        },
        keywords: vec![Keyword::Hexproof],
        base_loyalty: 6,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: drain(3),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    draw(2),
                    Effect::Scry {
                        who: PlayerRef::EachOpponent,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::If {
                        cond: Predicate::Not(Box::new(Predicate::EntityMatches {
                            what: Selector::Target(0),
                            filter: SelectionRequirement::HasEnchantmentSubtype(
                                crate::card::EnchantmentSubtype::Aura,
                            ),
                        })),
                        then: Box::new(Effect::CreateTokenCopyOf {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            source: Selector::Target(0),
                            extra_creature_types: vec![CreatureType::Spirit],
                            extra_card_types: vec![CardType::Creature],
                            override_pt: Some((1, 1)),
                            override_colors: Some(vec![Color::White]),
                            enters_tapped: false,
                            non_legendary: false,
                            legendary: false,
                            extra_keywords: vec![Keyword::Flying],
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                    Effect::Move {
                        what: target_filtered(
                            SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                        ),
                        to: ZoneDest::Exile,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kaito, Dancing Shadow — {2}{U}{B}, loyalty 3. Combat damage from your
/// creatures may bounce the dealer for double loyalty activations. +1: detain.
/// 0: draw. −2: a deathtouch Drone whose exit drains 2.
pub fn kaito_dancing_shadow() -> CardDefinition {
    CardDefinition {
        name: "Kaito, Dancing Shadow",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Kaito],
            ..Default::default()
        },
        base_loyalty: 3,
        // Fires per damaging creature (the printed batch is "one or more").
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Return the creature to hand for double loyalty use?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    },
                    Effect::GrantLoyaltyTwiceThisTurn {
                        what: Selector::This,
                    },
                ])),
            },
        }],
        loyalty_abilities: vec![
            // Printed: "can't attack or block until your next turn" —
            // modeled as detain (also locks abilities).
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Detain {
                    what: target_filtered(SelectionRequirement::Creature),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: draw(1),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Drone".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Drone],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Deathtouch],
                        triggered_abilities: vec![TriggeredAbility {
                            event: EventSpec::new(
                                EventKind::PermanentLeavesBattlefield,
                                EventScope::SelfSource,
                            ),
                            effect: drain(2),
                        }],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Eternal Wanderer — {4}{W}{W}, loyalty 5. +1: flicker until the next end
/// step. 0: a double-strike Samurai. −4: each player keeps one creature.
pub fn the_eternal_wanderer() -> CardDefinition {
    CardDefinition {
        name: "The Eternal Wanderer",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ExileReturnNextEndStep {
                    what: target_filtered(
                        SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Samurai".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Samurai],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::DoubleStrike],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::EachPlayerKeepsOneSacrificeRest {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    filter: SelectionRequirement::Creature,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── ONE wave 6: Sun's Twilight cycle + rares ─────────────────────────────────

/// White Sun's Twilight — {X}{W}{W}. Gain X life, X Mites; X≥5 board-wipes
/// first (the printed order mints before wiping — the Mites survive either way).
pub fn white_suns_twilight() -> CardDefinition {
    CardDefinition {
        name: "White Sun's Twilight",
        cost: cost(&[crate::mana::x(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                then: Box::new(Effect::Destroy {
                    what: Selector::EachPermanent(SelectionRequirement::Creature),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: Box::new(mite_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Blue Sun's Twilight — {X}{U}{U}. Steal a creature with MV ≤ X; X≥5 also
/// mints a copy of it.
pub fn blue_suns_twilight() -> CardDefinition {
    CardDefinition {
        name: "Blue Sun's Twilight",
        cost: cost(&[crate::mana::x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMostXFromCost),
                ),
                to: None,
                duration: Duration::Permanent,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                then: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::Target(0),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Red Sun's Twilight — {X}{R}{R}. Destroy up to five target artifacts (the
/// printed cap is X); X≥5 leaves hasty copies, exiled at the next end step.
pub fn red_suns_twilight() -> CardDefinition {
    CardDefinition {
        name: "Red Sun's Twilight",
        cost: cost(&[crate::mana::x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 5,
            min_targets: 0,
            filter: SelectionRequirement::Artifact,
            effect: Box::new(Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::CreateTokenCopyOf {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            source: Selector::Target(0),
                            extra_creature_types: vec![],
                            extra_card_types: vec![],
                            override_pt: None,
                            override_colors: None,
                            enters_tapped: false,
                            non_legendary: false,
                            legendary: false,
                            extra_keywords: vec![Keyword::Haste],
                        },
                        Effect::ExileLastCreatedTokensAtNextEndStep,
                    ])),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Destroy {
                    what: Selector::Target(0),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Green Sun's Twilight — {X}{G}. Dig X+1 for a creature and/or land to hand;
/// X≥5 deploys them instead.
pub fn green_suns_twilight() -> CardDefinition {
    let dig_filter = || SelectionRequirement::Creature.or(SelectionRequirement::Land);
    let x_plus_one = || Value::Sum(vec![Value::XFromCost, Value::Const(1)]);
    CardDefinition {
        name: "Green Sun's Twilight",
        cost: cost(&[crate::mana::x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
            then: Box::new(Effect::LookTopPutMatchingOntoBattlefield {
                count: x_plus_one(),
                filter: dig_filter(),
                then: None,
                max: Some(2),
                tapped: false,
                exile_rest: false,
            }),
            else_: Box::new(Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: x_plus_one(),
                pick_filter: Some(dig_filter()),
                take: Some(Value::Const(2)),
    ..Default::default()
}))),
        },
        ..Default::default()
    }
}

/// Kinzu of the Bleak Coven — {4}{B} 5/4 flying. Another nontoken creature of
/// yours dying may be exiled for 2 life, leaving a 1/1 toxic 1 copy.
pub fn kinzu_of_the_bleak_coven() -> CardDefinition {
    CardDefinition {
        name: "Kinzu of the Bleak Coven",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Vampire],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::NotToken,
                },
            ),
            effect: Effect::MayPayLife {
                description: "Pay 2 life to exile it for a 1/1 toxic copy?".into(),
                amount: Value::Const(2),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::TriggerSource,
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: Some((1, 1)),
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![Keyword::Toxic(1)],
                    },
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Exile,
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Kethek, Crucible Goliath — {2}{B}{R} 4/4. Your end step: may sacrifice
/// another creature to reveal-until a lesser-MV nonlegendary creature, deploy.
pub fn kethek_crucible_goliath() -> CardDefinition {
    CardDefinition {
        name: "Kethek, Crucible Goliath",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a creature to dig out a lesser one?".into(),
                filter: SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: SelectionRequirement::Creature
                        .and(SelectionRequirement::Not(Box::new(
                            SelectionRequirement::HasSupertype(Supertype::Legendary),
                        )))
                        .and(SelectionRequirement::ManaValueAtMostSacrificedPlus(0))
                        .and(SelectionRequirement::Not(Box::new(
                            SelectionRequirement::ManaValueEqualsSacrificedPlus(0),
                        ))),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                    cap: Value::LibrarySizeOf(PlayerRef::You),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::BottomRandom,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Argentum Masticore — {5} 5/5, first strike, protection from multicolored.
/// Your upkeep: discard a card (destroying a lesser opposing nonland
/// permanent) or sacrifice it.
pub fn argentum_masticore() -> CardDefinition {
    CardDefinition {
        name: "Argentum Masticore",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Masticore],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::FirstStrike, Keyword::ProtectionFromMulticolored],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayDiscard {
                description: "Discard a card or sacrifice Argentum Masticore?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Destroy {
                        what: target_filtered(
                            SelectionRequirement::Permanent
                                .and(SelectionRequirement::Nonland)
                                .and(SelectionRequirement::ControlledByOpponent)
                                .and(SelectionRequirement::ManaValueAtMostDiscardedThisEffect),
                        ),
                    }),
                }),
                else_: Some(Box::new(Effect::SacrificeSource)),
            },
        }],
        ..Default::default()
    }
}

/// Unctus's Retrofitter — {2}{U} 2/3, toxic 1. ETB: an artifact you control
/// becomes a 4/4 artifact creature (approximated as a permanent animation).
pub fn unctuss_retrofitter() -> CardDefinition {
    CardDefinition {
        name: "Unctus's Retrofitter",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Toxic(1)],
        triggered_abilities: vec![etb(Effect::BecomeCreature {
            what: target_filtered(
                SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(4),
            toughness: Value::Const(4),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::Permanent,
        })],
        ..Default::default()
    }
}

/// Vanish into Eternity — {2}{W} Instant. Exile target nonland permanent;
/// costs {3} more if it targets a creature.
pub fn vanish_into_eternity() -> CardDefinition {
    CardDefinition {
        name: "Vanish into Eternity",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        cost_increase_if_targets: Some((SelectionRequirement::Creature, 3)),
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
            ),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Viral Spawning — {2}{G} Sorcery. A 3/3 toxic Beast; Corrupted grants it
/// flashback {2}{G}.
pub fn viral_spawning() -> CardDefinition {
    CardDefinition {
        name: "Viral Spawning",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), g()]))],
        flashback_condition: Some(Predicate::CorruptedActive {
            who: PlayerRef::You,
        }),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(beast_token()),
        },
        ..Default::default()
    }
}

/// Zenith Chronicler — {2} 3/1. A player's first multicolored spell each turn
/// draws each other player a card.
pub fn zenith_chronicler() -> CardDefinition {
    CardDefinition {
        name: "Zenith Chronicler",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Multicolored,
                    },
                    Predicate::ValueEquals(
                        Value::MulticoloredSpellsCastThisTurn(PlayerRef::ControllerOf(Box::new(
                            Selector::TriggerSource,
                        ))),
                        Value::ONE,
                    ),
                ]),
            ),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayerExceptControllerOf(Box::new(
                    Selector::TriggerSource,
                ))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Noxious Assault — {3}{G}{G} Sorcery. Team +2/+2; blocks this turn poison
/// the blocker's controller.
pub fn noxious_assault() -> CardDefinition {
    CardDefinition {
        name: "Noxious Assault",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::BlockersPoisonedThisTurn { amount: 1 },
        ]),
        ..Default::default()
    }
}

/// Contagious Vorrac — {2}{G} 3/3. ETB: dig 4 for a land to hand; finding
/// none, proliferate. (Taking a found land is assumed.)
pub fn contagious_vorrac() -> CardDefinition {
    let dig = |extra: Option<Effect>| {
        let look = Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(SelectionRequirement::Land),
    ..Default::default()
}));
        match extra {
            Some(e) => Effect::Seq(vec![look, e]),
            None => look,
        }
    };
    CardDefinition {
        name: "Contagious Vorrac",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Boar,
                CreatureType::Beast,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CountMatching {
                    sel: Box::new(Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Const(4),
                    }),
                    filter: SelectionRequirement::Land,
                },
                Value::ONE,
            ),
            then: Box::new(dig(None)),
            else_: Box::new(dig(Some(Effect::Proliferate))),
        })],
        ..Default::default()
    }
}

/// Expand the Sphere — {3}{G} Sorcery. Dig 6 for up to two lands onto the
/// battlefield tapped; proliferate once per land short of two.
pub fn expand_the_sphere() -> CardDefinition {
    CardDefinition {
        name: "Expand the Sphere",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Repeat {
                count: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::Const(2)),
                    Box::new(Value::Min(
                        Box::new(Value::Const(2)),
                        Box::new(Value::CountMatching {
                            sel: Box::new(Selector::TopOfLibrary {
                                who: PlayerRef::You,
                                count: Value::Const(6),
                            }),
                            filter: SelectionRequirement::Land,
                        }),
                    )),
                ))),
                body: Box::new(Effect::Proliferate),
            },
            Effect::LookTopPutMatchingOntoBattlefield {
                count: Value::Const(6),
                filter: SelectionRequirement::Land,
                then: None,
                max: Some(2),
                tapped: true,
                exile_rest: false,
            },
        ]),
        ..Default::default()
    }
}

/// Goliath Hatchery — {4}{G}{G}. ETB: two 3/3 toxic Beasts. Corrupted — your
/// upkeep draws cards equal to your best total toxic value.
pub fn goliath_hatchery() -> CardDefinition {
    CardDefinition {
        name: "Goliath Hatchery",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Box::new(beast_token()),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::CorruptedActive {
                    who: PlayerRef::You,
                }),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::GreatestToxicAmongControlled(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

// ── ONE wave 7: mythics + the oil engines ────────────────────────────────────

/// All Will Be One — {3}{R}{R}. Counters you place (on your permanents, or
/// poison on opponents) ping an opposing target for that much.
pub fn all_will_be_one() -> CardDefinition {
    let ping = || Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::OpponentPlayer
                .or(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent))
                .or(SelectionRequirement::Planeswalker
                    .and(SelectionRequirement::ControlledByOpponent)),
        ),
        amount: Value::TriggerEventAmount,
    };
    CardDefinition {
        name: "All Will Be One",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YourControl),
                effect: ping(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PoisonAdded, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::OpponentPlayer,
                    },
                ),
                effect: ping(),
            },
        ],
        ..Default::default()
    }
}

/// Drivnod, Carnage Dominus — {3}{B}{B} 8/3. Death triggers of your permanents
/// fire twice. {B/P}{B/P}, exile three creature cards from your graveyard:
/// indestructible counter.
pub fn drivnod_carnage_dominus() -> CardDefinition {
    CardDefinition {
        name: "Drivnod, Carnage Dominus",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 8,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "If a creature dying causes a triggered ability of a permanent you control to \
                 trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerDeathTriggers,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[phyrexian(Color::Black), phyrexian(Color::Black)]),
            exile_other_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                3,
            )),
            effect: Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ichormoon Gauntlet — {2}{U}. Your planeswalkers gain "[0]: Proliferate" and
/// "[−12]: extra turn"; your noncreature casts add a counter of a kind already
/// on target permanent.
pub fn ichormoon_gauntlet() -> CardDefinition {
    CardDefinition {
        name: "Ichormoon Gauntlet",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[0]: Proliferate\" and \"[−12]: Take an extra \
                 turn after this one.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: vec![
                    crate::effect::LoyaltyAbility {
                        loyalty_cost: 0,
                        effect: Effect::Proliferate,
                        ..Default::default()
                    },
                    crate::effect::LoyaltyAbility {
                        loyalty_cost: -12,
                        effect: Effect::TakeExtraTurn {
                            who: PlayerRef::You,
                            count: Value::ONE,
                        },
                        ..Default::default()
                    },
                ],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_noncreature()),
            effect: Effect::AddCounterOfPresentKind {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::WithAnyCounter),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Mindsplice Apparatus — {3}{U}, flash. Upkeep oil; instants/sorceries cost
/// {1} less per oil counter on it.
pub fn mindsplice_apparatus() -> CardDefinition {
    CardDefinition {
        name: "Mindsplice Apparatus",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast for each oil counter \
                 on this artifact.",
            effect: StaticEffect::CostReductionPerCounterOnSource {
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                kind: CounterType::Oil,
            },
        }],
        ..Default::default()
    }
}

/// Mercurial Spelldancer — {1}{U} 2/1, unblockable; noncreature casts add oil.
/// Combat damage may cash two oil for a copy of your next instant or sorcery.
pub fn mercurial_spelldancer() -> CardDefinition {
    CardDefinition {
        name: "Mercurial Spelldancer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(crate::effect::shortcut::cast_is_noncreature()),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Oil,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Oil,
                        },
                        Value::Const(2),
                    ),
                    then: Box::new(Effect::MayDo {
                        description: "Remove two oil counters to copy your next spell?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::RemoveCounter {
                                what: Selector::This,
                                kind: CounterType::Oil,
                                amount: Value::Const(2),
                            },
                            Effect::OnYourNextInstantSorceryThisTurn {
                                body: Box::new(Effect::CopySpellMayChooseTargets {
                                    what: Selector::TriggerSource,
                                    count: Value::ONE,
                                }),
                            },
                        ])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Churning Reservoir — {R}. Upkeep: oil another of your artifacts/creatures.
/// {2}, {T}: a 1/1 Goblin, only after oil activity this turn.
pub fn churning_reservoir() -> CardDefinition {
    CardDefinition {
        name: "Churning Reservoir",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::NotToken)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::Oil,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            condition: Some(Predicate::OilActivityThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Phyrexian Goblin".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Phyrexian Vindicator — {W}{W}{W}{W} 5/5 flying. Damage aimed at it is
/// prevented and thrown at another target.
pub fn phyrexian_vindicator() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Vindicator",
        cost: cost(&[w(), w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to this creature, prevent that damage and have it \
                 deal that much damage to any other target.",
            effect: StaticEffect::PreventDamageToThisRedirect,
        }],
        ..Default::default()
    }
}

/// Graaz, Unstoppable Juggernaut — {8} 7/5. Your Juggernauts must attack and
/// can't be blocked by Walls; your other creatures are 5/3 Juggernauts too.
pub fn graaz_unstoppable_juggernaut() -> CardDefinition {
    let your_juggernauts = || {
        Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Juggernaut)
                .and(SelectionRequirement::ControlledByYou),
        )
    };
    let your_other_creatures = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
        )
    };
    CardDefinition {
        name: "Graaz, Unstoppable Juggernaut",
        cost: cost(&[generic(8)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Juggernaut],
            ..Default::default()
        },
        power: 7,
        toughness: 5,
        static_abilities: vec![
            StaticAbility {
                description: "Juggernauts you control attack each combat if able.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: your_juggernauts(),
                    keyword: Keyword::MustAttack,
                },
            },
            StaticAbility {
                description: "Juggernauts you control can't be blocked by Walls.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: your_juggernauts(),
                    keyword: Keyword::CantBeBlockedBy(Box::new(
                        SelectionRequirement::HasCreatureType(CreatureType::Wall),
                    )),
                },
            },
            StaticAbility {
                description: "Other creatures you control have base power and toughness 5/3.",
                effect: StaticEffect::SetBasePtForFilter {
                    applies_to: your_other_creatures(),
                    power: 5,
                    toughness: 3,
                },
            },
            StaticAbility {
                description: "Other creatures you control are Juggernauts in addition to their other \
                     creature types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: your_other_creatures(),
                    creature_type: CreatureType::Juggernaut,
                },
            },
        ],
        ..Default::default()
    }
}

/// Encroaching Mycosynth — {3}{U}. Your nonland permanents are artifacts in
/// addition to their other types. (The spell/off-battlefield halves are
/// dropped.)
pub fn encroaching_mycosynth() -> CardDefinition {
    CardDefinition {
        name: "Encroaching Mycosynth",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Nonland permanents you control are artifacts in addition to their other types.",
            effect: StaticEffect::AddCardTypeToMatching {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                card_type: CardType::Artifact,
                    artifact_subtype: None,
            },
        }],
        ..Default::default()
    }
}

/// Venser, Corpse Puppet — {U}{B} 1/3, lifelink, toxic 1. Proliferating mints
/// The Hollow Sentinel (if missing) or grants an artifact creature evasion.
pub fn venser_corpse_puppet() -> CardDefinition {
    CardDefinition {
        name: "Venser, Corpse Puppet",
        cost: cost(&[u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Zombie,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Lifelink, Keyword::Toxic(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Proliferated, EventScope::YourControl),
            effect: Effect::ChooseMode(vec![
                Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                        Selector::EachPermanent(
                            SelectionRequirement::HasName("The Hollow Sentinel".into())
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                    ))),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Box::new(TokenDefinition {
                            name: "The Hollow Sentinel".into(),
                            power: 3,
                            toughness: 3,
                            card_types: vec![CardType::Artifact, CardType::Creature],
                            supertypes: vec![Supertype::Legendary],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(
                            SelectionRequirement::Artifact
                                .and(SelectionRequirement::Creature)
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ]),
        }],
        ..Default::default()
    }
}

/// The Mycosynth Gardens — Land — Sphere. {T}: {C}. {1}, {T}: any color.
/// {X}, {T}: becomes a copy of a nontoken artifact you control with MV X.
pub fn the_mycosynth_gardens() -> CardDefinition {
    CardDefinition {
        name: "The Mycosynth Gardens",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![crate::card::LandType::Sphere],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[crate::mana::x()]),
                tap_cost: true,
                effect: Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(
                        SelectionRequirement::Artifact
                            .and(SelectionRequirement::NotToken)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::ManaValueExactlyXFromCost),
                    ),
                    duration: Duration::Permanent,
                    non_legendary: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mirran Safehouse — {3}. Has all activated abilities of all land cards in
/// all graveyards.
pub fn mirran_safehouse() -> CardDefinition {
    CardDefinition {
        name: "Mirran Safehouse",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Mirran Safehouse has all activated abilities of all land cards in all \
                 graveyards.",
            effect: StaticEffect::HasActivatedAbilitiesOfGraveyardLands,
        }],
        ..Default::default()
    }
}

/// Monument to Perfection — {2}. {3}, {T}: tutor a basic, Sphere, or Locus to
/// hand. {3}: becomes an indestructible 9/9 toxic 9 Construct at nine
/// differently named lands (approximated: any nine land names).
pub fn monument_to_perfection() -> CardDefinition {
    CardDefinition {
        name: "Monument to Perfection",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand
                        .or(SelectionRequirement::HasLandType(
                            crate::card::LandType::Sphere,
                        ))
                        .or(SelectionRequirement::HasLandType(
                            crate::card::LandType::Locus,
                        )),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                condition: Some(Predicate::ValueAtLeast(
                    Value::DistinctNamesControlledMatching(SelectionRequirement::Land),
                    Value::Const(9),
                )),
                effect: Effect::Seq(vec![
                    Effect::LoseAllAbilities {
                        what: Selector::This,
                        duration: Duration::Permanent,
                    },
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(9),
                        toughness: Value::Const(9),
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
                        keywords: vec![Keyword::Indestructible, Keyword::Toxic(9)],
                        duration: Duration::Permanent,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── ONE wave 8: the last four ────────────────────────────────────────────────

/// Capricious Hellraiser — {3}{R}{R}{R} 4/4 flying; {3} cheaper at nine cards
/// in graveyard. ETB: exile three graveyard cards and free-cast a noncreature
/// nonland one of them. (Random pick and the copy-not-original are elided.)
pub fn capricious_hellraiser() -> CardDefinition {
    CardDefinition {
        name: "Capricious Hellraiser",
        cost: cost(&[generic(3), r(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {3} less to cast if you have nine or more cards in \
                          your graveyard.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::ValueAtLeast(
                    Value::GraveyardSizeOf(PlayerRef::You),
                    Value::Const(9),
                ),
                amount: 3,
            },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: Selector::TakeRandom {
                    inner: Box::new(Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                        filter: SelectionRequirement::Any,
                    }),
                    count: Box::new(Value::Const(3)),
                },
                to: ZoneDest::Exile,
            },
            // Cast a *copy* — the exiled original stays put (CR 707.12).
            Effect::CastWithoutPayingImmediate {
                reduce_generic: 0,
                                pay_own_cost: false,
                what: Selector::ExiledThisResolution {
                    filter: SelectionRequirement::Noncreature.and(SelectionRequirement::Nonland),
                },
                source_zone: crate::card::Zone::Exile,
                exile_after: false,
                copy: true,
            },
        ]))],
        ..Default::default()
    }
}

/// Blade of Shared Souls — {2}{U} For Mirrodin! Whenever it becomes attached
/// to a creature, that creature may become a copy of another creature you
/// control (kept while it stays on the battlefield).
pub fn blade_of_shared_souls() -> CardDefinition {
    CardDefinition {
        name: "Blade of Shared Souls",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Rebel".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Rebel],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
                Effect::Attach {
                    what: Selector::This,
                    to: Selector::LastCreatedToken,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameAttached, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Have the equipped creature copy another creature you control?"
                        .into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::ChoosePermanentForSource {
                            filter: SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        },
                        Effect::BecomeCopyOfFor {
                            what: Selector::TriggerSource,
                            source: Selector::ChosenPermanentOfSource,
                            duration: Duration::Permanent,
                            non_legendary: false,
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Rhuk, Hexgold Nabber — {2}{R} 2/2, trample, haste. When another equipped
/// creature of yours attacks, you may move all its Equipment to Rhuk. (The
/// printed dies-half is elided.)
pub fn rhuk_hexgold_nabber() -> CardDefinition {
    CardDefinition {
        name: "Rhuk, Hexgold Nabber",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Rebel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::IsEquipped,
                },
            ),
            effect: Effect::MayDo {
                description: "Attach that creature's Equipment to Rhuk?".into(),
                body: Box::new(Effect::Attach {
                    what: Selector::AttachedToMe(Box::new(Selector::TriggerSource)),
                    to: Selector::This,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Ria Ivor, Bane of Bladehold — {2}{W}{B} 3/4, battle cry. At combat on your
/// turn: the next combat damage target creature would deal to a player is
/// prevented, minting that many Mites.
pub fn ria_ivor_bane_of_bladehold() -> CardDefinition {
    CardDefinition {
        name: "Ria Ivor, Bane of Bladehold",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            crate::effect::shortcut::battle_cry(1),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::PreventNextDamageByTargetMintMites,
            },
        ],
        ..Default::default()
    }
}
