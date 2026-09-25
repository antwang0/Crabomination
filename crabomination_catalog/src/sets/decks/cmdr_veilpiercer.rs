//! Commander: the cards the **Miracle Worker** precon (DSC, Aminatou, Veil
//! Piercer) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_veilpiercer.rs`.
//!
//! Residuals (each also on its card):
//! - **Fear of Sleep Paralysis** — only the untap step's stun removal is
//!   stopped; an effect that removes or moves counters still takes a stun.
//! - **Mirrormade** — the copy isn't optional.
//! - **One with the Multiverse** — the free cast is from hand only, not from
//!   the top of the library.
//! - **Phenomenon Investigators** — Doubt's return targets the permanent.
//! - **Secret Arcade** — permanent *spells* aren't enchantments on the stack.
//! - **Spirit-Sister's Call** — the sacrifice shares the chosen card's first
//!   card type (creature, artifact, enchantment, land, planeswalker, battle).

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EntersAsCopy, EventKind, EventScope,
    EventSpec, Keyword, LandType, RoomDoor, RoomDoors, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{eerie, etb, on_you_attack, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, LookPick, PlayerRef, Predicate, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, u, w, x};
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

fn enchantment_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Enchantment, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn room(name: &'static str, left: RoomDoor, right: RoomDoor) -> CardDefinition {
    CardDefinition {
        name,
        cost: left.cost.clone(),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Room], ..Default::default() },
        room: Some(Box::new(RoomDoors { left, right })),
        ..Default::default()
    }
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

/// Aminatou, Veil Piercer — surveil 2 each upkeep; enchantment cards in hand
/// have miracle at their mana cost less {4} (granted as the first card drawn
/// in a turn is drawn, CR 702.94a — the only moment miracle matters).
pub fn aminatou_veil_piercer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            at_your(TurnStep::Upkeep, Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
                    Predicate::ValueEquals(Value::CardsDrawnThisTurn(PlayerRef::You), Value::ONE),
                ])),
                effect: Effect::GrantMiracleReduced { what: Selector::TriggerSource, reduce: 4 },
            },
        ],
        ..creature(
            "Aminatou, Veil Piercer",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            4,
        )
    }
}

/// Cramped Vents // Access Maze — unlocking Vents deals 6 to an opponent's
/// creature and gains the excess; Maze casts a spell a turn for life.
pub fn cramped_vents_access_maze() -> CardDefinition {
    room(
        "Cramped Vents // Access Maze",
        RoomDoor {
            name: "Cramped Vents".to_string(),
            cost: cost(&[generic(3), b()]),
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        amount: Value::Const(6),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::ExcessDamageDealtThisResolution },
                ]),
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Access Maze".to_string(),
            cost: cost(&[generic(5), b(), b()]),
            static_abilities: vec![StaticAbility {
                description: "Once during each of your turns, you may cast a spell from your hand by paying life equal to its mana value rather than paying its mana cost.",
                effect: StaticEffect::LifeAlternativeCostOncePerYourTurn { filter: R::Any },
            }],
            ..Default::default()
        },
    )
}

/// Diabolic Vision — one of the top five to hand, the rest back on top.
pub fn diabolic_vision() -> CardDefinition {
    spell(
        "Diabolic Vision",
        cost(&[u(), b()]),
        CardType::Sorcery,
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_on_top: true,
            ..Default::default()
        })),
    )
}

/// Fear of Sleep Paralysis — flying; eerie taps and stuns up to one creature;
/// opponents' stun counters don't come off. ⚠ Only the untap step's removal
/// is stopped.
pub fn fear_of_sleep_paralysis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: eerie(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
            ])),
        }),
        static_abilities: vec![StaticAbility {
            description: "Stun counters can't be removed from permanents your opponents control.",
            effect: StaticEffect::OpponentsStunCountersStay,
        }],
        ..enchantment_creature(
            "Fear of Sleep Paralysis",
            cost(&[generic(5), u()]),
            vec![CreatureType::Nightmare],
            6,
            6,
        )
    }
}

/// Mirrormade — enters as a copy of an artifact or enchantment. ⚠ The copy
/// isn't optional.
pub fn mirrormade() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy { filter: R::Artifact.or(R::Enchantment), ..Default::default() }),
        ..spell("Mirrormade", cost(&[generic(1), u(), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Moon-Blessed Cleric — may tutor an enchantment to the top on entry.
pub fn moon_blessed_cleric() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for an enchantment and put it on top?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Enchantment,
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            }),
        })],
        ..creature(
            "Moon-Blessed Cleric",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Cleric],
            3,
            2,
        )
    }
}

/// Obscura Storefront — sacrificed on entry for a tapped basic Plains, Island
/// or Swamp and 1 life.
pub fn obscura_storefront() -> CardDefinition {
    CardDefinition {
        name: "Obscura Storefront",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::SacrificeSource,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(
                    R::HasLandType(LandType::Plains)
                        .or(R::HasLandType(LandType::Island))
                        .or(R::HasLandType(LandType::Swamp)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// One with the Multiverse — look at and play from the top of your library;
/// once during each of your turns, a free spell. ⚠ From hand only.
pub fn one_with_the_multiverse() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may play lands and cast spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Any },
            },
            StaticAbility {
                description: "Once during each of your turns, you may cast a spell from your hand or the top of your library without paying its mana cost.",
                effect: StaticEffect::ZeroAlternativeCostOncePerYourTurn { filter: R::Any },
            },
        ],
        ..spell("One with the Multiverse", cost(&[generic(6), u(), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Phenomenon Investigators — Believe: your nontoken deaths make 2/2 Horror
/// enchantment creatures; Doubt: your end step may bounce a nonland
/// permanent of yours to draw. ⚠ Doubt's return targets.
pub fn phenomenon_investigators() -> CardDefinition {
    let horror = Arc::new(TokenDefinition {
        name: "Horror".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        ..Default::default()
    });
    let grant = |trigger: TriggeredAbility| Effect::GrantTriggeredAbility {
        what: Selector::This,
        trigger: Box::new(trigger),
        duration: Duration::Permanent,
    };
    CardDefinition {
        as_enters_effect: Some(Effect::AsEntersChooseMode(vec![
            grant(TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
                ),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: horror },
            }),
            grant(at_your(
                TurnStep::End,
                Effect::MayDo {
                    description: "Return a nonland permanent you own to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: target_filtered(R::Nonland.and(R::OwnedByYou)),
                            to: ZoneDest::Hand(PlayerRef::You),
                        },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                },
            )),
        ])),
        ..creature(
            "Phenomenon Investigators",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Detective],
            3,
            4,
        )
    }
}

/// Redress Fate — every artifact and enchantment card in your graveyard
/// returns. Miracle {3}{W}.
pub fn redress_fate() -> CardDefinition {
    CardDefinition {
        miracle: Some(cost(&[generic(3), w()])),
        ..spell(
            "Redress Fate",
            cost(&[generic(6), w(), w()]),
            CardType::Sorcery,
            Effect::ReturnAllMatchingFromGraveyardToBattlefield {
                who: PlayerRef::You,
                filter: R::Artifact.or(R::Enchantment),
                sacrifice_eot: false,
            },
        )
    }
}

/// Secret Arcade // Dusty Parlor — your nonland permanents are enchantments;
/// each enchantment spell you cast puts its mana value in counters on up to
/// one creature. ⚠ Permanent spells aren't enchantments on the stack.
pub fn secret_arcade_dusty_parlor() -> CardDefinition {
    room(
        "Secret Arcade // Dusty Parlor",
        RoomDoor {
            name: "Secret Arcade".to_string(),
            cost: cost(&[generic(4), w()]),
            static_abilities: vec![StaticAbility {
                description: "Nonland permanents you control and permanent spells you control are enchantments in addition to their other types.",
                effect: StaticEffect::AddCardTypeToMatching {
                    applies_to: Selector::EachPermanent(R::Nonland.and(R::ControlledByYou)),
                    card_type: CardType::Enchantment,
                    artifact_subtype: None,
                },
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Dusty Parlor".to_string(),
            cost: cost(&[generic(2), w()]),
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
                ),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    }),
                },
            }],
            ..Default::default()
        },
    )
}

/// Soaring Lightbringer — flying; other enchantment creatures you control
/// fly; attacking a player sends a Glimmer at them.
pub fn soaring_lightbringer() -> CardDefinition {
    let glimmer = Arc::new(TokenDefinition {
        name: "Glimmer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Glimmer], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other enchantment creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::Enchantment).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Flying,
            },
        }],
        triggered_abilities: vec![on_you_attack(Effect::ForEachOpponent {
            body: Box::new(Effect::If {
                cond: Predicate::ValueAtLeast(Value::CreaturesAttackingPlayer(PlayerRef::Triggerer), Value::ONE),
                then: Box::new(Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: glimmer,
                    cleanup: Default::default(),
                    defender: Some(PlayerRef::Triggerer),
                }),
                else_: Box::new(Effect::Noop),
            }),
        })],
        ..enchantment_creature(
            "Soaring Lightbringer",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Glimmer],
            4,
            5,
        )
    }
}

/// Spirit-Sister's Call — your end step: for a permanent card in your
/// graveyard, you may sacrifice a permanent sharing a card type to return it,
/// exiled if it would leave. ⚠ The shared type is the card's first one.
pub fn spirit_sisters_call() -> CardDefinition {
    let back = || {
        Effect::Seq(vec![
            // The move names the target (slot 0); the type tests read it.
            Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::ExileIfLeavesBattlefield { what: Selector::LastMoved },
        ])
    };
    let types = [
        CardType::Creature,
        CardType::Artifact,
        CardType::Enchantment,
        CardType::Land,
        CardType::Planeswalker,
        CardType::Battle,
    ];
    let chain = types.iter().rev().fold(Effect::Noop, |rest, t| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::HasCardType(t.clone()) },
        then: Box::new(Effect::MaySacrifice {
            description: "Sacrifice a permanent sharing a card type to return it?".into(),
            filter: R::HasCardType(t.clone()).and(R::ControlledByYou),
            count: Value::ONE,
            then: Box::new(back()),
            else_: None,
        }),
        else_: Box::new(rest),
    });
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: chain,
        }],
        ..spell("Spirit-Sister's Call", cost(&[generic(3), w(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// The Master of Keys — flying; enters with X +1/+1 counters and mills 2X;
/// enchantment cards in your graveyard have escape (mana cost + exile three).
pub fn the_master_of_keys() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::XFromCost },
            Effect::Mill {
                who: Selector::You,
                amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
            },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Each enchantment card in your graveyard has escape. The escape cost is equal to the card's mana cost plus exile three other cards from your graveyard.",
            effect: StaticEffect::GraveyardCardsHaveEscapeMatching {
                filter: R::Enchantment,
                exile_count: 3,
                your_turn_only: false,
                once_per_turn: false,
            },
        }],
        ..enchantment_creature(
            "The Master of Keys",
            cost(&[x(), w(), u(), b()]),
            vec![CreatureType::Horror],
            3,
            3,
        )
    }
}

/// Thriving Moor — enters tapped; {T}: {B} or the chosen other color.
pub fn thriving_moor() -> CardDefinition {
    super::cmdr_bumbleflower::thriving("Thriving Moor", Color::Black)
}

