//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), eleventh file. Tests in `tests/recent_b/cmdr_rank3k_k.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{enrage, sneak, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, r, w};

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
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

/// "Each player may discard their hand and draw seven cards."
fn each_player_may_wheel() -> Effect {
    Effect::EachPlayerDoes {
        who: PlayerRef::EachPlayer,
        body: Box::new(Effect::MayDo {
            description: "Discard your hand and draw seven cards?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(7),
                },
            ])),
        }),
    }
}

// ── Raphael, Ninja Destroyer ─────────────────────────────────────────────────

/// Raphael, Ninja Destroyer — {2}{R}{R} legendary 4/4 Mutant Ninja Turtle; must
/// be blocked if able. Enrage — whenever he's dealt damage, add that much {R},
/// kept until end of turn.
pub fn raphael_ninja_destroyer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBeBlocked],
        triggered_abilities: vec![enrage(Effect::Repeat {
            count: Value::TriggerEventAmount,
            body: Box::new(Effect::AddManaKeptThisTurn {
                who: PlayerRef::You,
                colors: vec![Color::Red],
            }),
        })],
        ..legend(
            "Raphael, Ninja Destroyer",
            cost(&[generic(2), r(), r()]),
            vec![
                CreatureType::Mutant,
                CreatureType::Ninja,
                CreatureType::Turtle,
            ],
            4,
            4,
        )
    }
}

/// Raphael's Technique — {4}{R}{R} instant; sneak {2}{R}. Each player may
/// discard their hand and draw seven cards.
pub fn raphaels_technique() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(2), r()]))],
        alternative_cost: Some(sneak(cost(&[generic(2), r()]))),
        ..spell(
            "Raphael's Technique",
            cost(&[generic(4), r(), r()]),
            CardType::Instant,
            each_player_may_wheel(),
        )
    }
}

// ── Rinoa, Angel Wing ────────────────────────────────────────────────────────

/// Rinoa, Angel Wing — {2}{W} legendary 2/4 Human Rebel Warlock. Beginning of
/// combat on your turn: your flyers get +1/+1 and vigilance until end of turn.
/// Whenever one or more attacking creatures you control die, you may return
/// one of them tapped under its owner's control with a flying counter; once
/// each turn (spent only when the return is taken).
pub fn rinoa_angel_wing() -> CardDefinition {
    let flyers = || {
        Selector::EachPermanent(
            R::Creature
                .and(R::ControlledByYou)
                .and(R::HasKeyword(Keyword::Flying)),
        )
    };
    // "Whenever one or more attacking creatures you control die" (CR 603.2c).
    let mut died = EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
        Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: R::IsAttacking,
        },
    );
    died.once_per_batch = true;
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::YourControl,
                ),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: flyers(),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: flyers(),
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            crate::effect::shortcut::may_once_each_turn(
                died,
                "Return the creature tapped with a flying counter?",
                Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                            tapped: true,
                        },
                    },
                    Effect::AddKeywordCounter {
                        what: Selector::LastMoved,
                        keyword: Keyword::Flying,
                        amount: Value::ONE,
                    },
                ]),
            ),
        ],
        ..legend(
            "Rinoa, Angel Wing",
            cost(&[generic(2), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Rebel,
                CreatureType::Warlock,
            ],
            2,
            4,
        )
    }
}

/// Fated Clash — {3}{W}{W} sorcery, castable as though it had flash while a
/// creature is attacking and a creature is blocking. Target creature you
/// control and target creature an opponent controls gain indestructible until
/// end of turn; then destroy all creatures.
pub fn fated_clash() -> CardDefinition {
    let exists =
        |filter: R| Predicate::SelectorExists(Selector::EachPermanent(R::Creature.and(filter)));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may cast this spell as though it had flash if a creature is attacking and a creature is blocking.",
            effect: StaticEffect::SelfFlashIf {
                condition: Predicate::All(vec![exists(R::IsAttacking), exists(R::IsBlocking)]),
            },
        }],
        ..spell(
            "Fated Clash",
            cost(&[generic(3), w(), w()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Destroy {
                    what: Selector::EachPermanent(R::Creature),
                },
            ]),
        )
    }
}
