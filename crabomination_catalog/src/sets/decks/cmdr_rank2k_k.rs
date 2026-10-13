//! Commander: two-card gaps of EDHREC's top-2000 commanders (the 2026-10-13
//! rescan), eleventh file — Drizzt, Thranduil and Karrthus. Tests in
//! `tests/recent_b/cmdr_rank2k_k.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, landfall, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn make(who: PlayerRef, count: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who, count, definition: Arc::new(def) }
}

fn elves() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Elf))
}

fn elf_token() -> Effect {
    make(PlayerRef::You, Value::ONE, token("Elf", 1, 1, Color::Green, vec![CreatureType::Elf], vec![]))
}

fn dragons() -> R {
    R::HasCreatureType(CreatureType::Dragon)
}

/// Descent of the Dragons — {4}{R}{R} sorcery: destroy any number of target
/// creatures; for each one destroyed this way, its controller creates a 4/4
/// red flying Dragon.
pub fn descent_of_the_dragons() -> CardDefinition {
    CardDefinition {
        name: "Descent of the Dragons",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            // A survivor (indestructible, regenerated) makes no Dragon.
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(make(
                    PlayerRef::You,
                    Value::CreaturesDestroyedThisResolutionControlledBy(PlayerRef::You),
                    token("Dragon", 4, 4, Color::Red, vec![CreatureType::Dragon], vec![Keyword::Flying]),
                )),
            },
        ]),
        ..Default::default()
    }
}

/// Down in the Valley — {2}{G} Saga. I — a basic land card to hand. II — it
/// gains "Landfall — create a 1/1 green Elf." III, IV — Elves you control get
/// +1/+0 and vigilance until end of turn.
pub fn down_in_the_valley() -> CardDefinition {
    let rally = || {
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(yours(elves())),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(yours(elves())),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    CardDefinition {
        name: "Down in the Valley",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::IsBasicLand),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
            (
                2,
                Effect::GrantTriggeredAbility {
                    what: Selector::This,
                    trigger: Box::new(landfall(elf_token())),
                    duration: Duration::Permanent,
                },
            ),
            (3, rally()),
            (4, rally()),
        ],
        ..Default::default()
    }
}

/// Drizzt Do'Urden — {3}{G}{W} legendary 3/3 double strike Elf Ranger.
/// Entering makes Guenhwyvar, a legendary 4/1 green Cat with trample. Whenever
/// a creature dies with more power than Drizzt, he gets the difference in
/// +1/+1 counters.
pub fn drizzt_dourden() -> CardDefinition {
    let excess = || Value::Diff(Box::new(Value::PowerOf(Box::new(Selector::TriggerSource))), Box::new(Value::PowerOf(Box::new(Selector::This))));
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![
            etb(make(
                PlayerRef::You,
                Value::ONE,
                TokenDefinition {
                    supertypes: vec![Supertype::Legendary],
                    ..token("Guenhwyvar", 4, 1, Color::Green, vec![CreatureType::Cat], vec![Keyword::Trample])
                },
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::ValueAtLeast(excess(), Value::ONE)),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(excess(), Value::ONE),
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: excess(),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legend(
            "Drizzt Do'Urden",
            cost(&[generic(3), g(), w()]),
            vec![CreatureType::Elf, CreatureType::Ranger],
            3,
            3,
        )
    }
}

/// Icingdeath, Frost Tyrant — {2}{W}{W} legendary 4/3 flying, vigilance
/// Dragon. Dying makes Icingdeath, Frost Tongue: a legendary white Equipment
/// token (+2/+0; "whenever equipped creature attacks, tap target creature
/// defending player controls"; equip {2}).
pub fn icingdeath_frost_tyrant() -> CardDefinition {
    let tongue = TokenDefinition {
        name: "Icingdeath, Frost Tongue".into(),
        colors: vec![Color::White],
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            triggered_abilities: vec![on_attack(Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
            })],
            ..Default::default()
        }),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: make(PlayerRef::You, Value::ONE, tongue),
        }],
        ..legend("Icingdeath, Frost Tyrant", cost(&[generic(2), w(), w()]), vec![CreatureType::Dragon], 4, 3)
    }
}

/// Karrthus, Tyrant of Jund — {4}{B}{R}{G} legendary 7/7 flying, haste
/// Dragon. Entering, gain control of all Dragons, then untap all Dragons.
/// Other Dragon creatures you control have haste.
pub fn karrthus_tyrant_of_jund() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Other Dragon creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature.and(dragons())).and(R::OtherThanSource)),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl { what: Selector::EachPermanent(dragons()), to: None, duration: Duration::Permanent },
            Effect::Untap { what: Selector::EachPermanent(dragons()), up_to: None },
        ]))],
        ..legend(
            "Karrthus, Tyrant of Jund",
            cost(&[generic(4), b(), r(), g()]),
            vec![CreatureType::Dragon],
            7,
            7,
        )
    }
}

/// Thranduil the Strategist — {3}{G}{U} legendary 4/4 Elf Noble. Other Elves
/// you control have "{T}: Add {G} or {U}." Landfall — create a 1/1 green Elf.
pub fn thranduil_the_strategist() -> CardDefinition {
    let other_elves = || Selector::EachPermanent(yours(elves()).and(R::OtherThanSource));
    CardDefinition {
        static_abilities: [Color::Green, Color::Blue]
            .into_iter()
            .map(|c| StaticAbility {
                description: "Other Elves you control have \"{T}: Add {G} or {U}.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: other_elves(),
                    ability: crate::sets::tap_add(c),
                    condition: None,
                },
            })
            .collect(),
        triggered_abilities: vec![landfall(elf_token())],
        ..legend(
            "Thranduil the Strategist",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            4,
            4,
        )
    }
}
