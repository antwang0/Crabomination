//! Commander: the cards the **Arcane Maelstrom** precon (C20, Kalamax, the
//! Stormsire) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kalamax.rs`.
//!
//! Residuals (each also on its card):
//! - **Eon Frolicker** — the protection from that player until your next
//!   turn isn't granted.
//! - **Haldan, Avid Arcanist** / **Pako, Arcane Retriever** — the play
//!   permission is granted as Pako exiles the cards while you control
//!   Haldan (not re-read if Haldan comes or goes later), and creature cards
//!   are playable too.
//! - **Lavabrink Floodgates** — each upkeep's player may only add a doom
//!   counter (removing one isn't offered).

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, hybrid, r, u, x};
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

fn instant(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn cast_matching(filter: R) -> EventSpec {
    EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter })
}

/// Kalamax, the Stormsire — your first instant each turn is copied while
/// Kalamax is tapped; copying an instant grows it.
pub fn kalamax_the_stormsire() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::CastSpellFirstMatchingThisTurn(R::HasCardType(CardType::Instant)),
                    Predicate::EntityMatches { what: Selector::This, filter: R::Tapped },
                ])),
                effect: Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCopied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCardType(CardType::Instant) },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature(
            "Kalamax, the Stormsire",
            cost(&[generic(1), g(), u(), r()]),
            vec![CreatureType::Elemental, CreatureType::Dinosaur],
            4,
            4,
        )
    }
}

/// Commune with Lava — exile the top X; play them until the end of your next
/// turn.
pub fn commune_with_lava() -> CardDefinition {
    instant(
        "Commune with Lava",
        cost(&[x(), r(), r()]),
        Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::XFromCost,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        },
    )
}

/// Curious Herd — a 3/3 Beast per artifact the target opponent controls.
pub fn curious_herd() -> CardDefinition {
    instant(
        "Curious Herd",
        cost(&[generic(3), g()]),
        // "Choose target opponent": a zero-card draw names slot 0's filter.
        Effect::Seq(vec![
            Effect::Draw { who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer }, amount: Value::Const(0) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Artifact })),
                definition: Arc::new(token("Beast", vec![Color::Green], vec![CreatureType::Beast], 3, 3)),
            },
        ]),
    )
}

/// Decoy Gambit — for each opponent, up to one of their creatures goes home
/// unless its controller has you draw a card.
pub fn decoy_gambit() -> CardDefinition {
    instant(
        "Decoy Gambit",
        cost(&[generic(2), u()]),
        Effect::ForEachOpponentTarget {
            body: Box::new(Effect::PlayersMayAccept {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                description: "Let the caster draw a card to keep your creature?".into(),
                on_accept: Box::new(Effect::Noop),
                if_any: Box::new(draw(1)),
                otherwise: Box::new(Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            }),
        },
    )
}

/// Eon Frolicker — flying; cast, it gives target opponent an extra turn, and
/// you and your planeswalkers protection from them until your next turn.
pub fn eon_frolicker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::TriggerSourceEnteredByCast),
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![
                    Effect::TakeExtraTurn { who: PlayerRef::Target(0), count: Value::ONE },
                    Effect::GainProtectionFromPlayer {
                        what: Selector::You,
                        from: PlayerRef::Target(0),
                        duration: Duration::UntilNextTurn,
                    },
                    Effect::GainProtectionFromPlayer {
                        what: Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou)),
                        from: PlayerRef::Target(0),
                        duration: Duration::UntilNextTurn,
                    },
                ])),
            },
        }],
        ..creature("Eon Frolicker", cost(&[generic(2), u(), u()]), vec![CreatureType::Elemental, CreatureType::Otter], 5, 5)
    }
}

/// Evolution Charm — a basic land to hand, a creature card back, or flying.
pub fn evolution_charm() -> CardDefinition {
    instant(
        "Evolution Charm",
        cost(&[generic(1), g()]),
        Effect::ChooseMode(vec![
            Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: crate::effect::Duration::EndOfTurn,
            },
        ]),
    )
}

/// Glademuse — a spell cast off its caster's turn draws them a card.
pub fn glademuse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::Triggerer)))),
            effect: Effect::Draw { who: Selector::Player(PlayerRef::Triggerer), amount: Value::ONE },
        }],
        ..creature("Glademuse", cost(&[generic(2), g()]), vec![CreatureType::Beast], 2, 4)
    }
}

/// Haldan, Avid Arcanist — partner with Pako; the cards Pako exiles are yours
/// to play, with mana of any color.
/// Residual: see the module note — the permission is stamped as Pako exiles.
pub fn haldan_avid_arcanist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Pako, Arcane Retriever".into())],
        ..creature(
            "Haldan, Avid Arcanist",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            4,
        )
    }
}

/// Pako, Arcane Retriever — partner with Haldan; haste; attacking, the top
/// card of each library is exiled with a fetch counter (playable while you
/// control Haldan), and Pako grows per noncreature card.
pub fn pako_arcane_retriever() -> CardDefinition {
    let exile = |pay_any_color: bool| Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::EachPlayer,
        count: Value::ONE,
        duration: MayPlayDuration::WhileExiled,
        pay_any_color,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Haldan, Avid Arcanist".into()), Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountOf(Box::new(Selector::EachPermanent(
                        R::HasName("Haldan, Avid Arcanist".into()).and(R::ControlledByYou),
                    ))),
                    Value::ONE,
                ),
                then: Box::new(exile(true)),
                else_: Box::new(Effect::ExileLinked {
                    what: Selector::TopOfLibrary { who: PlayerRef::EachPlayer, count: Value::ONE },
                }),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::LastMoved)),
            },
        ]))],
        ..creature(
            "Pako, Arcane Retriever",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Elemental, CreatureType::Dog],
            3,
            3,
        )
    }
}

/// Lavabrink Floodgates — {T}: {R}{R}; each upkeep its player may add a doom
/// counter; at three it's sacrificed for 6 damage to each creature.
/// Residual: removing a doom counter isn't offered.
pub fn lavabrink_floodgates() -> CardDefinition {
    CardDefinition {
        name: "Lavabrink Floodgates",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red, Color::Red]) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::MayDoBy {
                    who: PlayerRef::ActivePlayer,
                    description: "Put a doom counter on Lavabrink Floodgates?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Doom,
                        amount: Value::ONE,
                    }),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Doom },
                        Value::Const(3),
                    ),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSelected { what: Selector::This },
                        Effect::ForEach {
                            selector: Selector::EachPermanent(R::Creature),
                            body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(6) }),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Lunar Mystic — each instant you cast may pay {1} to draw.
pub fn lunar_mystic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: cast_matching(R::HasCardType(CardType::Instant)),
            effect: Effect::MayPay {
                description: "Pay {1} to draw a card?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(draw(1)),
                else_: None,
            },
        }],
        ..creature("Lunar Mystic", cost(&[generic(2), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Nascent Metamorph — attacking or blocking, it becomes a copy of the first
/// creature card from the top of an opponent's library until end of turn.
pub fn nascent_metamorph() -> CardDefinition {
    let shift = || Effect::RevealUntilCreatureBecomeCopy { who: PlayerRef::Target(0) };
    CardDefinition {
        triggered_abilities: vec![
            on_attack(shift()),
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: shift() },
        ],
        ..creature("Nascent Metamorph", cost(&[generic(1), u()]), vec![CreatureType::Shapeshifter], 1, 1)
    }
}

/// Rashmi, Eternities Crafter — your first spell each turn reveals your top
/// card: a cheaper spell may be cast free, else it goes to hand.
pub fn rashmi_eternities_crafter() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::ValueAtMost(
                Value::SpellsCastThisTurn(PlayerRef::You),
                Value::ONE,
            )),
            effect: Effect::RevealTopCastFreeIfLesserElseHand,
        }],
        ..creature(
            "Rashmi, Eternities Crafter",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            3,
        )
    }
}

/// Ravenous Gigantotherium — devour 3; entering, its power divided among up
/// to that many creatures, each of which hits back.
pub fn ravenous_gigantotherium() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(crate::effect::shortcut::devour(3)),
        triggered_abilities: vec![etb(Effect::DealDamageDivided {
            total: Value::PowerOf(Box::new(Selector::This)),
            filter: R::Creature,
            max_targets: 8,
            retaliate_to_source: true,
        })],
        ..creature("Ravenous Gigantotherium", cost(&[generic(5), g(), g()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Surreal Memoir — an instant card at random from your graveyard to hand;
/// rebound.
pub fn surreal_memoir() -> CardDefinition {
    CardDefinition {
        name: "Surreal Memoir",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Rebound],
        effect: Effect::ReturnRandomFromGraveyard {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Instant),
            count: Value::ONE,
        },
        ..Default::default()
    }
}

/// Twinning Staff — your spell copies come with one more; {7}, {T}: copy
/// your instant or sorcery spell.
pub fn twinning_staff() -> CardDefinition {
    CardDefinition {
        name: "Twinning Staff",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If you would copy a spell one or more times, instead copy it that many times plus an additional time.",
            effect: StaticEffect::SpellCopiesPlusOne,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(7)]),
            effect: Effect::CopySpell {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::ControlledByYou),
                ),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Whiplash Trap — bounce two creatures; {U} instead if an opponent had two
/// creatures enter this turn.
pub fn whiplash_trap() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![crate::card::SpellSubtype::Trap], ..Default::default() },
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[u()]),
            condition: Some(Predicate::AnOpponentHadCreaturesEnterAtLeast(2)),
            ..Default::default()
        }),
        ..instant(
            "Whiplash Trap",
            cost(&[generic(3), u(), u()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::Move {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(1)))),
                },
            ]),
        )
    }
}

/// Wort, the Raidmother — two 1/1 Goblin Warriors on entry; your red or green
/// instants and sorceries have conspire.
pub fn wort_the_raidmother() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Arc::new(token(
                "Goblin Warrior",
                vec![Color::Red, Color::Green],
                vec![CreatureType::Goblin, CreatureType::Warrior],
                1,
                1,
            )),
        })],
        static_abilities: vec![StaticAbility {
            description: "Each red or green instant or sorcery spell you cast has conspire.",
            effect: StaticEffect::GrantConspireToSpells {
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::HasColor(Color::Red).or(R::HasColor(Color::Green))),
            },
        }],
        ..creature(
            "Wort, the Raidmother",
            cost(&[generic(4), hybrid(Color::Red, Color::Green), hybrid(Color::Red, Color::Green)]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            3,
            3,
        )
    }
}
