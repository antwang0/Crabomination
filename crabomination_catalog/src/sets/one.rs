//! Phyrexia: All Will Be One — Incubate (CR 701.53). "Incubate N" creates an
//! Incubator double-faced token with N +1/+1 counters; `{2}: Transform` flips
//! it to a 0/0 Phyrexian artifact creature (so it becomes N/N).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{deal, drain, draw, etb, gain_life, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, phyrexian, r, u, w, Color};

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
    Effect::Incubate { who: PlayerRef::You, amount: Value::Const(amount as i32) }
}

/// Eyes of Gitaxias — {2}{U} Sorcery. Incubate 3. Draw a card.
pub fn eyes_of_gitaxias() -> CardDefinition {
    CardDefinition {
        name: "Eyes of Gitaxias",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            incubate(3),
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), crate::card::LandType::Swamp)],
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Elf, CreatureType::Warrior],
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
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::CorruptedActive { who: PlayerRef::You }),
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
                .with_filter(Predicate::CorruptedActive { who: PlayerRef::You }),
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
            cond: Predicate::CorruptedActive { who: PlayerRef::You },
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
            condition: Some(Predicate::CorruptedActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
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
    let tap_target = || Effect::Tap { what: target_filtered(SelectionRequirement::Creature) };
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
                condition: Some(Predicate::CorruptedActive { who: PlayerRef::You }),
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
                condition: Predicate::CorruptedActive { who: PlayerRef::You },
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
            condition: Predicate::CorruptedActive { who: PlayerRef::You },
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Elf, CreatureType::Scout],
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Bird],
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_dies(Effect::Draw { who: Selector::You, amount: Value::ONE })],
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
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
            definition: goblin,
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Rebel],
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
    TokenDefinition {
        name: "Phyrexian Mite".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian, CreatureType::Mite], ..Default::default() },
        keywords: vec![Keyword::Toxic(1), Keyword::CantBlock],
        ..Default::default()
    }
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
            sac_other_filter: Some((SelectionRequirement::Artifact.or(SelectionRequirement::Creature), 2)),
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
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: mite_token() },
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "Corrupted — creatures you control with toxic have lifelink.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::CorruptedActive { who: PlayerRef::You },
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
            Effect::AddPoison { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
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
            cond: Predicate::CorruptedActive { who: PlayerRef::You },
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
            Effect::AddPoison { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
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
            definition: mite_token(),
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
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasToxic,
                }),
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
            effect: deal(2, target_filtered(
                SelectionRequirement::Player.or(SelectionRequirement::Planeswalker),
            )),
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Mandibular Kite",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3), w()]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: germ },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
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
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Menace, duration: Duration::EndOfTurn },
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
                    what: target_filtered(SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment)),
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
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).once_per_turn(),
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
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), r()]),
            sac_cost: true,
            self_counter_cost_reduction: Some(CounterType::Oil),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::CardsInHandMatching { who: PlayerRef::You, filter: SelectionRequirement::Any },
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
            Effect::Tap { what: Selector::This },
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
            definition: mite_token(),
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
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                },
            },
            Effect::AddPoison { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
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
                what: target_filtered(SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker)),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crate::game::effects::treasure_token(),
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
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
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
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
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
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Toxic(2)], ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
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
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
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
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
                    Value::Const(3),
                ),
                power: 3,
                toughness: 0,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::InGraveyard),
                    to: ZoneDest::Exile,
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
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
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
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
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
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
                Effect::Destroy { what: target_filtered(filter()) },
                Effect::Proliferate,
            ])),
            else_: Box::new(Effect::Destroy { what: target_filtered(filter()) }),
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
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
                condition: Predicate::CorruptedActive { who: PlayerRef::You },
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Elf, CreatureType::Warrior],
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
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
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
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: phyrexian_goblin_token(),
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
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
                take: None,
                to_battlefield: false,
            },
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
                    SelectionRequirement::PermanentCard
                        .and(SelectionRequirement::InYourGraveyard),
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
                Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
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
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::AddPoison { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
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
            Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) },
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Rebel], ..Default::default() },
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
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: rebel },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
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
        EquipBonus { power: 2, toughness: 1, keywords: vec![Keyword::Vigilance], ..Default::default() },
    )
}

/// Vulshok Splitter — {3}{R} For Mirrodin! Equipped gets +2/+0. Equip {2}{R}.
pub fn vulshok_splitter() -> CardDefinition {
    for_mirrodin(
        "Vulshok Splitter",
        cost(&[generic(3), r()]),
        cost(&[generic(2), r()]),
        EquipBonus { power: 2, ..Default::default() },
    )
}

/// Sylvok Battle-Chair — {4}{G}{G} For Mirrodin! Equipped gets +4/+4 and
/// trample. Equip {5}{G}{G}.
pub fn sylvok_battle_chair() -> CardDefinition {
    for_mirrodin(
        "Sylvok Battle-Chair",
        cost(&[generic(4), g(), g()]),
        cost(&[generic(5), g(), g()]),
        EquipBonus { power: 4, toughness: 4, keywords: vec![Keyword::Trample], ..Default::default() },
    )
}

/// Hexgold Hoverwings — {3}{W} For Mirrodin! Equipped has flying; your
/// equipped creatures get +1/+0. Equip {2}{W}.
pub fn hexgold_hoverwings() -> CardDefinition {
    let mut def = for_mirrodin(
        "Hexgold Hoverwings",
        cost(&[generic(3), w()]),
        cost(&[generic(2), w()]),
        EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() },
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
        EquipBonus { keywords: vec![Keyword::DoubleStrike], ..Default::default() },
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
                definition: mite_token(),
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
        affinity_filter: Some(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)),
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
        affinity_filter: Some(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)),
        effect: Effect::Seq(vec![
            Effect::LoseKeywordThisTurn {
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
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
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: cat },
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
                        uncast_penalty: None,
                    },
                    Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
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
        subtypes: Subtypes { land_types: vec![LandType::Sphere], ..Default::default() },
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

pub fn the_autonomous_furnace() -> CardDefinition { sphere_land("The Autonomous Furnace", Color::Red) }
pub fn the_dross_pits() -> CardDefinition { sphere_land("The Dross Pits", Color::Black) }
pub fn the_fair_basilica() -> CardDefinition { sphere_land("The Fair Basilica", Color::White) }
pub fn the_hunter_maze() -> CardDefinition { sphere_land("The Hunter Maze", Color::Green) }
pub fn the_surgical_bay() -> CardDefinition { sphere_land("The Surgical Bay", Color::Blue) }

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
            Keyword::CantBeBlockedBy(Box::new(SelectionRequirement::HasKeyword(
                Keyword::Flying,
            ))),
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
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::ManaValueAtMost(1)),
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
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
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
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 0, base_t: 5 }),
        ..Default::default()
    }
}

// ── Oil-counter engine cards + more commons ──────────────────────────────────

/// "Whenever you cast a noncreature spell, put an oil counter on this."
fn oil_on_noncreature_cast() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
    }
}

/// "Whenever another creature or artifact you control is put into a graveyard
/// from the battlefield, put an oil counter on this."
fn oil_on_another_dying() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureOrArtifactDied, EventScope::AnotherOfYours),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
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
            counter_type: CounterType::Oil, base_p: 0, base_t: 0, per_p: 1, per_t: 1,
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
            counter_type: CounterType::Oil, base_p: 0, base_t: 0, per_p: 1, per_t: 1,
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        toughness: 1,
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::BasePlusCountersOnSelf {
            counter_type: CounterType::Oil, base_p: 0, base_t: 1, per_p: 1, per_t: 0,
        }),
        triggered_abilities: vec![
            etb(Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE }),
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
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
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
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
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
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
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
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
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
            Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
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
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(oil_at_least(2)),
                effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::Colorless(Value::ONE) },
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
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
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
                definition: TokenDefinition {
                    name: "Phyrexian Golem".into(),
                    power: 3,
                    toughness: 3,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Urabrask's Forge — {2}{R}. At combat on your turn: add an oil counter,
/// then mint an X/1 trample haste Horror (X = oil), sacrificed at end step.
pub fn urabrasks_forge() -> CardDefinition {
    let oil = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil };
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
                Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
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
                    },
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
            amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
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
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
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
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
                Value::ONE,
            ),
            then: Box::new(Effect::MayDo {
                description: "Remove an oil counter to untap and pump?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
                    Effect::Untap { what: Selector::This, up_to: None },
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
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil },
                    Value::Const(2),
                ),
                then: Box::new(Effect::MayDo {
                    description: "Remove two oil counters: a creature can't block this turn?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::Const(2) },
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
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Elf, CreatureType::Warrior],
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
    let oil = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Oil };
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
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtMost(oil(), Value::Const(0)),
                    then: Box::new(Effect::SacrificePermanent { what: Selector::This }),
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
            etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
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
                Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: phyrexian_goblin_token(),
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
        triggered_abilities: vec![on_dies(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        })],
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
            to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
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
            definition: mite_token(),
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Scry { who: PlayerRef::EachOpponent, amount: Value::ONE },
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
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
            Effect::If {
                cond: Predicate::CorruptedActive { who: PlayerRef::You },
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
            to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
        })],
        ..Default::default()
    }
}
