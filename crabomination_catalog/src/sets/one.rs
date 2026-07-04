//! Phyrexia: All Will Be One — Incubate (CR 701.53). "Incubate N" creates an
//! Incubator double-faced token with N +1/+1 counters; `{2}: Transform` flips
//! it to a 0/0 Phyrexian artifact creature (so it becomes N/N).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, StaticAbility, StaticEffect, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, gain_life, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w};

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
