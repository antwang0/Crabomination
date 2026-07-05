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
