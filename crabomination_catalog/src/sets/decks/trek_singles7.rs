//! Star Trek (TRK/TRC) — the seventh batch of EDHREC Star Trek closers: the
//! missing cards of Will Riker's, Voq's and Dr. McCoy's single-commander average
//! decks (scan of 2026-11-01). Tests in `tests/recent_b/trek_singles7.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, SpellSubtype, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, r};

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Erica Ortegas, Plucky Pilot — {3}{R} legendary 3/3 Human Pilot, haste.
/// Becoming tapped, you may put a land from hand onto the battlefield; if you
/// do, you return a land you control (your pick, then) at the next end step.
pub fn erica_ortegas_plucky_pilot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: Some(Box::new(Effect::AtNextEndStep {
                    body: Box::new(Effect::ChooseOneAmong {
                        what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                        chooser: PlayerRef::You,
                        chosen: Box::new(Effect::Move {
                            what: Selector::SeparatedPile { chosen: true },
                            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                        }),
                        other: Box::new(Effect::Noop),
                    }),
                })),
            },
        }],
        ..legend(
            "Erica Ortegas, Plucky Pilot",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Pilot],
            3,
            3,
        )
    }
}

/// Will Riker, First in the Field — {3}{R} legendary 4/3 Human Officer. At
/// the beginning of combat on your turn, reveal until two lands; they enter,
/// the rest go to the bottom in a random order, and you sacrifice those lands
/// at your next end step.
pub fn will_riker_first_in_the_field() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::RevealUntilLandsToBattlefield { count: Value::Const(2), tapped: false },
                Effect::SacrificeAtNextEndStep { what: Selector::LastMoved },
            ]),
        }],
        ..legend(
            "Will Riker, First in the Field",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Officer],
            4,
            3,
        )
    }
}

/// Armus, Skin of Evil — {5}{B}{B} legendary 6/6 Ooze Horror, menace. Your
/// end step: target opponent discards two. {2}{B}, discard this card: target
/// creature gets -4/-4 until end of turn.
pub fn armus_skin_of_evil() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Discard {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::Const(2),
                random: false,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-4),
                toughness: Value::Const(-4),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Armus, Skin of Evil",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Ooze, CreatureType::Horror],
            6,
            6,
        )
    }
}

/// Hela's Arcanum — {3}{B}{B} instant — Arcane. Up to five {P} of modes,
/// repeats allowed: {P} +1/+0 and deathtouch; {P}{P} "when this dies, return
/// it tapped" this turn; {P}{P}{P} destroy each non-Zombie creature with
/// power 3 or less.
pub fn helas_arcanum() -> CardDefinition {
    CardDefinition {
        name: "Hela's Arcanum",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Arcane], ..Default::default() },
        effect: Effect::ChooseModesByPoints {
            points: vec![1, 2, 3],
            budget: 5,
            modes: vec![
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
                ]),
                Effect::WhenTargetDiesThisTurn {
                    body: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: true },
                    }),
                    slot: 0,
                    filter: Some(R::Creature),
                },
                Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Zombie)))).and(R::PowerAtMost(3)),
                    ),
                },
            ],
        },
        ..Default::default()
    }
}

/// Dr. Leonard "Bones" McCoy — {1}{G}{G} legendary 3/2 Human Doctor, flash.
/// Entering, choose one: return a non-Doctor permanent card from your
/// graveyard to hand; or choose up to one other creature — when it dies this
/// turn, it returns under its owner's control with a +1/+1 counter at the
/// beginning of the next end step.
pub fn dr_leonard_bones_mccoy() -> CardDefinition {
    let back_at_end_step = Effect::AtNextEndStep {
        body: Box::new(Effect::Seq(vec![
            Effect::SpellEntersWithCounters {
                what: Selector::TriggerSource,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
            },
        ])),
    };
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::Move {
                    what: target_filtered(
                        R::PermanentCard
                            .and(R::InYourGraveyard)
                            .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Doctor)))),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 1,
                    filter: R::Creature.and(R::OtherThanSource),
                    effect: Box::new(Effect::WhenTargetDiesThisTurn {
                        body: Box::new(back_at_end_step),
                        slot: 0,
                        filter: None,
                    }),
                },
            ]),
        }],
        ..legend(
            "Dr. Leonard \"Bones\" McCoy",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            3,
            2,
        )
    }
}
