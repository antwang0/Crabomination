//! Commander: 2026-10-19 (`01DJHkWk`) — three-gap closers: Captain America,
//! Steve Rogers (Valkyrior Skyrider, Soldier On, Hawkeye's Shot); Tam, the
//! Possibility (the two Annexes, Entrust the Spark); Baxter Stockman (Mouser
//! Mark III, Nobody, Rock Soldiers); Arnim Zola (Transmogrant Altar, Ceti Eel,
//! Windgrace Acolyte); V'Ger (itself, Boarded Window, Cumber Stone).
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, StaticAbility, StaticEffect,
    Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, u, w};
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
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn artifact_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, mana, types, p, t)
    }
}

// ── Captain America, Steve Rogers ───────────────────────────────────────────

/// Valkyrior Skyrider — {4}{W} 3/4 flying God Warrior Hero. Enters: gain 4.
pub fn valkyrior_skyrider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(4),
        })],
        ..creature(
            "Valkyrior Skyrider",
            cost(&[generic(4), w()]),
            vec![CreatureType::God, CreatureType::Warrior, CreatureType::Hero],
            3,
            4,
        )
    }
}

/// Soldier On — {2}{W} instant. Two +1/+1 counters on target creature; untap
/// it.
pub fn soldier_on() -> CardDefinition {
    CardDefinition {
        name: "Soldier On",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Hawkeye's Shot — {4}{W} sorcery; {2} less if it targets a tapped creature.
/// Exile target creature.
pub fn hawkeyes_shot() -> CardDefinition {
    CardDefinition {
        name: "Hawkeye's Shot",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_target: Some((R::Tapped, 2)),
        effect: Effect::Exile {
            what: target_filtered(R::Creature),
        },
        ..Default::default()
    }
}

// ── Tam, the Possibility ────────────────────────────────────────────────────

/// Fatehold Annex — land; tapped unless you control a planeswalker. {T}: {W}
/// or {U}.
pub fn fatehold_annex() -> CardDefinition {
    super::cmdr_oct17_g::walker_dual("Fatehold Annex", Color::White, Color::Blue)
}

/// Vigorbloom Annex — land; tapped unless you control a planeswalker. {T}:
/// {G} or {W}.
pub fn vigorbloom_annex() -> CardDefinition {
    super::cmdr_oct17_g::walker_dual("Vigorbloom Annex", Color::Green, Color::White)
}

/// Entrust the Spark — {3}{G}{U} sorcery. You may sacrifice a planeswalker.
/// If you do, search your library for a planeswalker card, put it onto the
/// battlefield, then shuffle.
pub fn entrust_the_spark() -> CardDefinition {
    CardDefinition {
        name: "Entrust the Spark",
        cost: cost(&[generic(3), crate::mana::g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::MaySacrifice {
            description: "Sacrifice a planeswalker to search for another?".into(),
            filter: R::Planeswalker.and(R::ControlledByYou),
            count: Value::ONE,
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Planeswalker,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            }),
            else_: None,
        },
        ..Default::default()
    }
}

// ── Baxter Stockman ─────────────────────────────────────────────────────────

/// Mouser Mark III — {1}{U/R} 2/3 artifact creature Robot; can't attack unless
/// you control another artifact.
pub fn mouser_mark_iii() -> CardDefinition {
    let alone = Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
        R::Artifact.and(R::ControlledByYou).and(R::OtherThanSource),
    ))));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature can't attack unless you control another artifact.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::CantAttack,
                condition: alone,
            },
        }],
        ..artifact_creature(
            "Mouser Mark III",
            cost(&[generic(1), hybrid(Color::Blue, Color::Red)]),
            vec![CreatureType::Robot],
            2,
            3,
        )
    }
}

/// Nobody — {1}{U/R}{U/R} 3/2 artifact creature Human Hero. Enters: return up
/// to one other target artifact you control to its owner's hand; scry 1.
pub fn nobody() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Artifact.and(R::ControlledByYou).and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]))],
        ..artifact_creature(
            "Nobody",
            cost(&[generic(1), ur(), ur()]),
            vec![CreatureType::Human, CreatureType::Hero],
            3,
            2,
        )
    }
}

/// Rock Soldiers — {3}{R} 4/3 artifact creature Elemental Soldier. Enters:
/// destroy up to one target noncreature artifact.
pub fn rock_soldiers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Artifact.and(R::Noncreature)),
            }),
        })],
        ..artifact_creature(
            "Rock Soldiers",
            cost(&[generic(3), r()]),
            vec![CreatureType::Elemental, CreatureType::Soldier],
            4,
            3,
        )
    }
}

// ── Arnim Zola, Bio-Fanatic ─────────────────────────────────────────────────

/// Transmogrant Altar — {3} artifact. {B}, {T}, sacrifice a creature: add
/// {C}{C}{C}. {2}, {T}, sacrifice a creature: a 3/3 colorless Zombie artifact
/// creature token (sorcery speed).
pub fn transmogrant_altar() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Transmogrant Altar",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(3)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                sorcery_speed: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(zombie),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ceti Eel — {3}{B} 2/2 Worm. Enters: mill two, then a +1/+1 counter on it
/// for each artifact and/or creature card in your graveyard.
pub fn ceti_eel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Artifact.or(R::Creature),
                },
            },
        ]))],
        ..creature("Ceti Eel", cost(&[generic(3), b()]), vec![CreatureType::Worm], 2, 2)
    }
}

/// Windgrace Acolyte — {4}{B} 3/2 flying Cat Warrior. Enters: mill three and
/// gain 3.
pub fn windgrace_acolyte() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]))],
        ..creature(
            "Windgrace Acolyte",
            cost(&[generic(4), b()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            3,
            2,
        )
    }
}

// ── V'Ger, the Intruder ─────────────────────────────────────────────────────

/// V'Ger, the Intruder — {6}{U} legendary 5/7 flying artifact creature
/// Construct. Whenever it enters or attacks, choose one — look at target
/// opponent's hand, then draw a card; or creatures your opponents control get
/// -1/-0 until your next turn.
pub fn vger_the_intruder() -> CardDefinition {
    let modes = || {
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::LookAtHand {
                    who: target_filtered(R::Player.and(R::OpponentPlayer)),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::UntilNextTurn,
            },
        ])
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(modes()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: modes(),
            },
        ],
        ..artifact_creature(
            "V'Ger, the Intruder",
            cost(&[generic(6), u()]),
            vec![CreatureType::Construct],
            5,
            7,
        )
    }
}

/// Boarded Window — {3} artifact. Creatures attacking you get -1/-0. At the
/// beginning of each end step, if you were dealt 4 or more damage this turn,
/// exile it.
pub fn boarded_window() -> CardDefinition {
    CardDefinition {
        name: "Boarded Window",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creatures attacking you get -1/-0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsAttackingYou)),
                power: -1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::DamageTakenThisTurn(PlayerRef::You), Value::Const(4)),
            ),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Exile,
            },
        }],
        ..Default::default()
    }
}

/// Cumber Stone — {3}{U} artifact. Creatures your opponents control get -1/-0.
pub fn cumber_stone() -> CardDefinition {
    CardDefinition {
        name: "Cumber Stone",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control get -1/-0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                power: -1,
                toughness: 0,
            },
        }],
        ..Default::default()
    }
}
