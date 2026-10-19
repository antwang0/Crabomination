//! Commander: 2026-10-19 (`01DJHkWk`) — the Erebor Goblin pair and Tiger
//! Shark's deck. Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::shortcut::{connive, etb, on_attack};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, u};

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

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..def
    }
}

fn eot_pump(what: Selector, n: i32) -> Effect {
    Effect::PumpPT {
        what,
        power: Value::Const(n),
        toughness: Value::Const(n),
        duration: Duration::EndOfTurn,
    }
}

/// Bolg, Erebor's Reckoning — {4}{B}{R} legendary 6/6 trample Goblin Soldier.
/// At the beginning of each combat, your other Goblins and Orcs get +2/+2 and
/// your opponents' creatures get -1/-1 until end of turn.
pub fn bolg_erebors_reckoning() -> CardDefinition {
    let kin = R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc));
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Seq(vec![
                eot_pump(
                    Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(kin)
                            .and(R::OtherThanSource),
                    ),
                    2,
                ),
                eot_pump(
                    Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    -1,
                ),
            ]),
        }],
        ..legend(creature(
            "Bolg, Erebor's Reckoning",
            cost(&[generic(4), b(), r()]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            6,
            6,
        ))
    }
}

/// Great Goblin, Foul-Hearted — {3}{B}{R} legendary 3/3 Goblin Noble.
/// Entering or attacking, amass Goblins 3. Armies you control have trample.
pub fn great_goblin_foul_hearted() -> CardDefinition {
    let amass = || Effect::Amass {
        who: PlayerRef::You,
        count: Value::Const(3),
        extra_type: Some(CreatureType::Goblin),
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Armies you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasCreatureType(CreatureType::Army)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![etb(amass()), on_attack(amass())],
        ..legend(creature(
            "Great Goblin, Foul-Hearted",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Goblin, CreatureType::Noble],
            3,
            3,
        ))
    }
}

/// Tiger Shark, Abyssal Hunter — {2}{U/B}{U/B} legendary 2/3 Shark Human
/// Villain. Entering or attacking, he connives. {4}{U/B}: can't be blocked
/// this turn.
pub fn tiger_shark_abyssal_hunter() -> CardDefinition {
    let ub = || hybrid(Color::Blue, Color::Black);
    CardDefinition {
        triggered_abilities: vec![etb(connive(1)), on_attack(connive(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), ub()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(creature(
            "Tiger Shark, Abyssal Hunter",
            cost(&[generic(2), ub(), ub()]),
            vec![
                CreatureType::Shark,
                CreatureType::Human,
                CreatureType::Villain,
            ],
            2,
            3,
        ))
    }
}

/// Spined Megalodon — {5}{U}{U} 5/7 hexproof Shark; scry 1 when it attacks.
pub fn spined_megalodon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![on_attack(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Spined Megalodon",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Shark],
            5,
            7,
        )
    }
}
