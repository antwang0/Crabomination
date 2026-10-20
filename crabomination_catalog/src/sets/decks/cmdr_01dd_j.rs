//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Sygg, Wanderwine
//! Wisdom's Merfolk and Tam, Mindful First-Year's vivid. Tests in
//! `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, mint_token, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, hybrid, u};

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

fn merfolk_token() -> TokenDefinition {
    TokenDefinition {
        name: "Merfolk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        ..Default::default()
    }
}

/// "Vivid — … the number of colors among permanents you control."
fn vivid() -> Value {
    Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou)))
}

fn vivid_discount() -> StaticAbility {
    StaticAbility {
        description: "Vivid — This spell costs {1} less to cast for each color among permanents you control.",
        effect: StaticEffect::SelfCostReducedByValue { amount: vivid() },
    }
}

// ── Sygg, Wanderwine Wisdom ─────────────────────────────────────────────────

/// Merrow Skyswimmer — {3}{W/U}{W/U} 2/2 Merfolk Soldier. Convoke, flying,
/// vigilance. ETB: a 1/1 white and blue Merfolk.
pub fn merrow_skyswimmer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(mint_token(merfolk_token(), 1))],
        ..creature(
            "Merrow Skyswimmer",
            ManaCost::new(vec![generic(3), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Silvergill Peddler — {2}{U} 2/3 Merfolk Citizen. Whenever it becomes
/// tapped, draw a card, then discard a card.
pub fn silvergill_peddler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        ..creature("Silvergill Peddler", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Citizen], 2, 3)
    }
}

/// Wanderwine Farewell — {5}{U}{U} Kindred sorcery — Merfolk, convoke. Return
/// one or two target nonland permanents to their owners' hands; then if you
/// control a Merfolk, a 1/1 white and blue Merfolk per permanent returned.
pub fn wanderwine_farewell() -> CardDefinition {
    CardDefinition {
        name: "Wanderwine Farewell",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        keywords: vec![Keyword::Convoke],
        effect: Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::ApplyToTargets {
                min_targets: 1,
                max_targets: 2,
                filter: R::Permanent.and(R::Not(Box::new(R::Land))),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasCreatureType(CreatureType::Merfolk)),
                    Value::ONE,
                ),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountOf(Box::new(Selector::LastMoved)),
                    definition: std::sync::Arc::new(merfolk_token()),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Tam, Mindful First-Year ─────────────────────────────────────────────────

/// Glister Bairn — {2}{G/U}{G/U}{G/U} 1/4 Ouphe. Vivid — at the beginning of
/// combat on your turn, another target creature you control gets +X/+X.
pub fn glister_bairn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                power: vivid(),
                toughness: vivid(),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Glister Bairn",
            ManaCost::new(vec![
                generic(2),
                hybrid(Color::Green, Color::Blue),
                hybrid(Color::Green, Color::Blue),
                hybrid(Color::Green, Color::Blue),
            ]),
            vec![CreatureType::Ouphe],
            1,
            4,
        )
    }
}

/// Prismabasher — {4}{G}{G} 6/6 Elemental, trample. Vivid — ETB: up to X
/// target creatures you control get +X/+X.
pub fn prismabasher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CapTargetsAt {
            amount: vivid(),
            body: Box::new(Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 5,
                filter: R::Creature.and(R::ControlledByYou),
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: vivid(),
                    toughness: vivid(),
                    duration: Duration::EndOfTurn,
                }),
            }),
        })],
        ..creature("Prismabasher", cost(&[generic(4), g(), g()]), vec![CreatureType::Elemental], 6, 6)
    }
}

/// Shinestriker — {4}{U}{U} 3/3 Elemental, flying. Vivid — ETB: draw X.
pub fn shinestriker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: vivid() })],
        ..creature("Shinestriker", cost(&[generic(4), u(), u()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Wildvine Pummeler — {6}{G} 6/5 Giant Berserker, reach, trample. Vivid —
/// {1} less per color among permanents you control.
pub fn wildvine_pummeler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        static_abilities: vec![vivid_discount()],
        ..creature("Wildvine Pummeler", cost(&[generic(6), g()]), vec![CreatureType::Giant, CreatureType::Berserker], 6, 5)
    }
}

/// Rime Chill — {6}{U} instant. Vivid — {1} less per color among permanents
/// you control. Tap up to two target creatures and put a stun counter on
/// each. Draw a card.
pub fn rime_chill() -> CardDefinition {
    CardDefinition {
        name: "Rime Chill",
        cost: cost(&[generic(6), u()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![vivid_discount()],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::Target(0) },
                    Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                ])),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}
