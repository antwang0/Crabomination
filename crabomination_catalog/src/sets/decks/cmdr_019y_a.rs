//! Commander: 2026-10-17 (`019Y6J4R`) closers, first batch — cards that close
//! one or more EDHREC average decks. Tests in `tests/recent_b/cmdr_019y_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, investigate_by, prowess, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LibraryPosition, PlayerRef, Selector, StaticEffect,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// Gilt-Leaf Winnower — {3}{B}{B} 4/3 Elf Warrior, menace. ETB: you may destroy
/// target non-Elf creature whose power and toughness aren't equal.
pub fn gilt_leaf_winnower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Destroy target non-Elf creature whose power and toughness aren't equal?"
                .into(),
            body: Box::new(Effect::Destroy {
                what: target_filtered(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Elf).negate())
                        .and(R::PowerToughnessUnequal),
                ),
            }),
        })],
        ..creature(
            "Gilt-Leaf Winnower",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            4,
            3,
        )
    }
}

/// Vicious Battlerager — {3}{B} 1/5 Dwarf Barbarian. ETB: take the initiative.
/// Whenever it becomes blocked by a creature, that creature's controller loses
/// 5 life (once per blocker).
pub fn vicious_battlerager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::TakeInitiative { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::ForEach {
                    selector: Selector::BlockingCreatures,
                    body: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                            Selector::TriggerSource,
                        ))),
                        amount: Value::Const(5),
                    }),
                },
            },
        ],
        ..creature(
            "Vicious Battlerager",
            cost(&[generic(3), b()]),
            vec![CreatureType::Dwarf, CreatureType::Barbarian],
            1,
            5,
        )
    }
}

/// Underdark Explorer — {4}{B} 5/3 Lizard Warrior, menace. ETB: you take the
/// initiative.
pub fn underdark_explorer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::TakeInitiative { who: PlayerRef::You })],
        ..creature(
            "Underdark Explorer",
            cost(&[generic(4), b()]),
            vec![CreatureType::Lizard, CreatureType::Warrior],
            5,
            3,
        )
    }
}

/// Safana, Calimport Cutthroat — {2}{B} legendary 3/2 Human Rogue, menace,
/// choose a Background. At your end step, if you have the initiative, create a
/// Treasure — three instead once you've completed a dungeon (CR 603.4).
pub fn safana_calimport_cutthroat() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace, Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::HasInitiative { who: PlayerRef::You }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::IfAtLeast {
                    value: Box::new(Value::DungeonsCompleted),
                    threshold: 1,
                    then: Box::new(Value::Const(3)),
                    else_: Box::new(Value::ONE),
                },
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        }],
        ..creature(
            "Safana, Calimport Cutthroat",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            2,
        )
    }
}

/// Fury Charm — {1}{R} instant. Choose one: destroy target artifact; target
/// creature gets +1/+1 and trample until end of turn; or remove two time
/// counters from target permanent or suspended card (the last one casts a
/// suspended card, CR 702.62a).
pub fn fury_charm() -> CardDefinition {
    spell(
        "Fury Charm",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::RemoveCounter {
                what: target_filtered(R::OnBattlefield.or(R::InExile.and(R::IsSuspended))),
                kind: CounterType::Time,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Pearl Lake Ancient — {5}{U}{U} 6/7 Leviathan. Flash; can't be countered;
/// prowess. Return three lands you control to their owner's hand: return this
/// creature to its owner's hand.
pub fn pearl_lake_ancient() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::CantBeCountered, Keyword::Prowess],
        triggered_abilities: vec![prowess()],
        activated_abilities: vec![ActivatedAbility {
            bounce_other_filter: Some((R::Land, 3)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ..Default::default()
        }],
        ..creature(
            "Pearl Lake Ancient",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Leviathan],
            6,
            7,
        )
    }
}

/// "Take an extra turn after this one. At the beginning of that turn's end
/// step, you lose the game." (Warrior's Oath; Alchemist's Gambit's uncleaved text.)
fn doomed_extra_turn() -> Vec<Effect> {
    vec![
        Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
        Effect::DelayUntil {
            kind: DelayedTriggerKind::YourNextTurnsEndStep,
            body: Box::new(Effect::LoseGame { who: PlayerRef::You }),
        },
    ]
}

/// Warrior's Oath — {R}{R} sorcery. Take an extra turn after this one; at the
/// beginning of that turn's end step, you lose the game.
pub fn warriors_oath() -> CardDefinition {
    spell("Warrior's Oath", cost(&[r(), r()]), CardType::Sorcery, Effect::Seq(doomed_extra_turn()))
}

/// Alchemist's Gambit — {1}{R}{R} sorcery, cleave {4}{U}{U}{R}. Take an extra
/// turn after this one; damage can't be prevented during it; [at its end step,
/// you lose the game]. Exile it (CR 702.148: cleave drops the bracket).
pub fn alchemists_gambit() -> CardDefinition {
    // CR 615.12 — set at the extra turn's upkeep; nothing deals damage in an
    // untap step, so that covers "during that turn".
    let unpreventable = Effect::DelayUntil {
        kind: DelayedTriggerKind::YourNextUpkeep,
        body: Box::new(Effect::DamageCantBePreventedThisTurn),
    };
    let mut full = doomed_extra_turn();
    full.insert(1, unpreventable.clone());
    CardDefinition {
        exile_on_resolve: true,
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), u(), u(), r()]),
            effect_override: Some(Effect::Seq(vec![
                Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
                unpreventable,
            ])),
            ..Default::default()
        }),
        ..spell("Alchemist's Gambit", cost(&[generic(1), r(), r()]), CardType::Sorcery, Effect::Seq(full))
    }
}

/// Tamiyo Meets the Story Circle — {1}{U} Saga. I: until your next turn, a
/// creature attacking you or a planeswalker you control gets -2/-0. II: discard
/// any number of cards, then investigate twice for each. III: shuffle up to
/// three target cards from your graveyard into your library.
pub fn tamiyo_meets_the_story_circle() -> CardDefinition {
    CardDefinition {
        name: "Tamiyo Meets the Story Circle",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::CreatureAttacksYouUntilYourNextTurn,
                    body: Box::new(Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(-2),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    }),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::DiscardAnyNumber { who: Selector::You, filter: R::Any, max: None },
                    investigate_by(
                        PlayerRef::You,
                        Value::Times(
                            Box::new(Value::CardsDiscardedThisEffect),
                            Box::new(Value::Const(2)),
                        ),
                    ),
                ]),
            ),
            (
                3,
                Effect::ApplyToTargets {
                    max_targets: 3,
                    min_targets: 0,
                    filter: R::InGraveyard.and(R::OwnedByYou),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Shuffled },
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// The keyword-family Eater of Virtue's last paragraph lists. "Protection" and
/// "hexproof" cover their "from [quality]" forms too (CR 702.16, 702.11d).
fn virtues() -> Vec<Keyword> {
    let mut k = vec![
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Haste,
        Keyword::Hexproof,
        Keyword::Indestructible,
        Keyword::Lifelink,
        Keyword::Menace,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Vigilance,
        Keyword::ProtectionFromCreatures,
        Keyword::ProtectionFromEverything,
        Keyword::ProtectionFromMulticolored,
        Keyword::ProtectionFromMonocolored,
        Keyword::ProtectionFromInstants,
    ];
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        k.push(Keyword::Protection(c));
        k.push(Keyword::HexproofFromColor(c));
    }
    k
}

/// Eater of Virtue — {1} legendary Equipment, equip {1}. +2/+0. When equipped
/// creature dies, exile it (linked). Equipped creature has each listed keyword
/// a card exiled with it has.
pub fn eater_of_virtue() -> CardDefinition {
    let static_abilities = virtues()
        .into_iter()
        .map(|kw| StaticAbility {
            description: "As long as a card exiled with Eater of Virtue has this keyword, equipped creature has it.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::ValueAtLeast(
                    Value::CardsExiledWithSourceMatching(R::HasKeyword(kw.clone())),
                    Value::ONE,
                ),
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    keyword: kw,
                }),
            },
        })
        .collect();
    CardDefinition {
        name: "Eater of Virtue",
        cost: cost(&[generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus { power: 2, ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::ExileWithSource {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::TriggerSource),
                    filter: R::InGraveyard,
                },
            },
        }],
        static_abilities,
        ..Default::default()
    }
}
