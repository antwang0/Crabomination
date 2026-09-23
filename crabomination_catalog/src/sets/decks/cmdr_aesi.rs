//! Commander: the cards the **Reap the Tides** precon (CMR, Aesi, Tyrant of
//! Gyre Strait) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::catalog::sets::{dual_land_untyped, enters_tapped, tap_add};
use crate::effect::shortcut::{emerge, etb, on_cast, target_filtered, token_copy_of};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, cost, g, generic, hybrid, u, x};

fn gu() -> crate::mana::ManaSymbol {
    hybrid(Color::Green, Color::Blue)
}

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
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

/// Elder Deep-Fiend — emerge, and a cast trigger that taps up to four.
pub fn elder_deep_fiend() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        alternative_cost: Some(emerge(cost(&[generic(5), u(), u()]))),
        triggered_abilities: vec![on_cast(Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Permanent,
            effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
        })],
        ..creature(
            "Elder Deep-Fiend",
            cost(&[generic(8)]),
            vec![CreatureType::Eldrazi, CreatureType::Octopus],
            5,
            6,
        )
    }
}

/// Memorial to Genius — a tapped Island that cashes in for two cards.
pub fn memorial_to_genius() -> CardDefinition {
    CardDefinition {
        name: "Memorial to Genius",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(4), u()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Murkfiend Liege — two Simic anthems, and Seedborn Muse for the same
/// creatures.
pub fn murkfiend_liege() -> CardDefinition {
    let other_yours = |c: Color| {
        Selector::EachPermanent(
            R::Creature.and(R::HasColor(c)).and(R::ControlledByYou).and(R::OtherThanSource),
        )
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other green creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: other_yours(Color::Green), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Other blue creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: other_yours(Color::Blue), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Untap all green and/or blue creatures you control during each other player's untap step.",
                effect: StaticEffect::UntapYoursEachUntapStepFiltered(
                    R::Creature.and(R::HasColor(Color::Green).or(R::HasColor(Color::Blue))),
                ),
            },
        ],
        ..creature(
            "Murkfiend Liege",
            cost(&[generic(2), gu(), gu(), gu()]),
            vec![CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Nezahal, Primal Tide — uncounterable, no hand size, a card per opposing
/// noncreature spell, and a discard-three blink that returns it tapped.
pub fn nezahal_primal_tide() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::CantBeCountered],
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Creature).negate(),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 3)),
            effect: Effect::ExileReturnToOwnerNextEndStep { what: Selector::This, tapped: true },
            ..Default::default()
        }],
        ..creature(
            "Nezahal, Primal Tide",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Elder, CreatureType::Dinosaur],
            7,
            7,
        )
    }
}

/// Slinn Voda, the Rising Deep — kicked, it bounces every creature but the
/// sea creatures.
pub fn slinn_voda_the_rising_deep() -> CardDefinition {
    let sea = [
        CreatureType::Merfolk,
        CreatureType::Kraken,
        CreatureType::Leviathan,
        CreatureType::Octopus,
        CreatureType::Serpent,
    ];
    let spared = sea.into_iter().map(R::HasCreatureType).reduce(R::or).expect("five types");
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Kicker(cost(&[generic(1), u()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: Selector::EachPermanent(R::Creature.and(spared.negate())),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Slinn Voda, the Rising Deep",
            cost(&[generic(6), u(), u()]),
            vec![CreatureType::Leviathan],
            8,
            8,
        )
    }
}

/// Sphinx of Uthuun — a flying Fact or Fiction on a body.
pub fn sphinx_of_uthuun() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::FactOrFiction { count: Value::Const(5), to_bottom: false })],
        ..creature("Sphinx of Uthuun", cost(&[generic(5), u(), u()]), vec![CreatureType::Sphinx], 5, 6)
    }
}

/// Spitting Image — a token copy of target creature, again by retrace.
pub fn spitting_image() -> CardDefinition {
    CardDefinition {
        name: "Spitting Image",
        cost: cost(&[generic(4), gu(), gu()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Retrace],
        effect: token_copy_of(PlayerRef::You, Value::ONE, target_filtered(R::Creature)),
        ..Default::default()
    }
}

/// Stumpsquall Hydra — X counters, split between it and your commanders.
/// ⚠ Modelled as all X on the Hydra, then any number moved onto commanders
/// you control (the headless seat spreads them evenly); an opponent's
/// commander, which the printed "any number of commanders" allows, is never
/// offered.
pub fn stumpsquall_hydra() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
            Effect::DistributeCountersFromSource {
                kind: CounterType::PlusOnePlusOne,
                filter: R::Creature.and(R::IsCommander).and(R::ControlledByYou),
            },
        ]))],
        ..creature(
            "Stumpsquall Hydra",
            cost(&[x(), g(), g(), g()]),
            vec![CreatureType::Hydra],
            1,
            1,
        )
    }
}

/// Trench Behemoth — bounce a land to untap it with hexproof; landfall
/// forces an opposing creature to attack.
pub fn trench_behemoth() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            return_permanent_cost: Some((R::Land, 1)),
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::This, up_to: None },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        // "During its controller's next combat phase": until this seat's next
        // turn, which every opponent's turn comes before.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
            ),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                keyword: Keyword::MustAttack,
                duration: Duration::UntilNextTurn,
            },
        }],
        ..creature("Trench Behemoth", cost(&[generic(5), u(), u()]), vec![CreatureType::Kraken], 7, 7)
    }
}

/// A Vivid land: tapped with two charge counters; its color, or any color
/// for a counter.
fn vivid(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        enters_with_counters: Some((CounterType::Charge, Value::Const(2))),
        activated_abilities: vec![
            tap_add(color),
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

pub fn vivid_creek() -> CardDefinition {
    vivid("Vivid Creek", Color::Blue)
}

pub fn vivid_grove() -> CardDefinition {
    vivid("Vivid Grove", Color::Green)
}

/// Woodland Stream — the Simic tapped dual.
pub fn woodland_stream() -> CardDefinition {
    let mut d = dual_land_untyped("Woodland Stream", Color::Green, Color::Blue, vec![]);
    d.static_abilities.push(enters_tapped());
    d
}
