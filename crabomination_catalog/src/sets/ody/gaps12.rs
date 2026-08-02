//! Odyssey (ODY) gap-closing wave 12 — the last five. Tests in
//! `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Predicate,
    SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value,
    shortcut::target_filtered,
};
use crate::mana::{ManaCost, cost, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Karmic Justice — {2}{W}. Killing your noncreature permanents costs them
/// one of theirs.
pub fn karmic_justice() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentDestroyedByEffect,
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Noncreature,
            }),
            // The printed clause scopes the retaliation to the destroying
            // opponent; the target filter is any opponent's permanent, which
            // is exact in a two-player game.
            effect: Effect::Destroy {
                what: target_filtered(R::Permanent.and(R::ControlledByOpponent)),
            },
        }],
        ..enchantment("Karmic Justice", cost(&[generic(2), w()]))
    }
}

/// Liquid Fire — {4}{R}{R}. Split five damage between a creature and its
/// controller.
pub fn liquid_fire() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Sorcery],
        // The printed split is chosen as an additional cost; the engine picks
        // it at resolution instead.
        effect: Effect::PlayerChoosesNumber {
            who: Selector::You,
            prompt: "Liquid Fire: how much damage to the creature?".to_string(),
            max: Value::Const(5),
            then: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::ChosenNumber,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Diff(Box::new(Value::Const(5)), Box::new(Value::ChosenNumber)),
                },
            ])),
        },
        ..creature("Liquid Fire", cost(&[generic(4), r(), r()]), vec![], 0, 0)
    }
}

/// The state trigger both of Bomb Squad's abilities check: a creature at four
/// fuse counters loses them, burns its controller for 4, and dies.
fn detonate() -> Effect {
    Effect::ForEach {
        selector: Selector::EachPermanent(
            R::Creature.and(R::WithCounterAtLeast(CounterType::Fuse, 4)),
        ),
        body: Box::new(Effect::Seq(vec![
            Effect::RemoveCounter {
                what: Selector::TriggerSource,
                kind: CounterType::Fuse,
                amount: Value::Const(100),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::Const(4),
            },
            Effect::Destroy { what: Selector::TriggerSource },
        ])),
    }
}

/// Bomb Squad — {3}{R} 1/1 Dwarf that mines creatures with fuse counters.
pub fn bomb_squad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::Fuse,
                    amount: Value::ONE,
                },
                detonate(),
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::WithCounter(CounterType::Fuse)),
                    ),
                    kind: CounterType::Fuse,
                    amount: Value::ONE,
                },
                detonate(),
            ]),
        }],
        ..creature("Bomb Squad", cost(&[generic(3), r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// Impulsive Maneuvers — {2}{R}{R}. Every attack is a coin flip.
pub fn impulsive_maneuvers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer),
            effect: Effect::FlipCoin {
                count: Value::ONE,
                // The printed clauses scope to the attacker's *next* combat
                // damage; the doubling here runs for the rest of the turn.
                on_heads: Box::new(Effect::DoubleDamageFromSourceThisTurn {
                    what: Selector::TriggerSource,
                }),
                on_tails: Box::new(Effect::PreventNextEventFromChosenSourceAnywhere {
                    what: Some(Selector::TriggerSource),
                }),
            },
        }],
        ..enchantment("Impulsive Maneuvers", cost(&[generic(2), r(), r()]))
    }
}

/// Shifty Doppelganger — {2}{U} 1/1 that swaps itself for something big for
/// a turn.
pub fn shifty_doppelganger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    count: Value::ONE,
                    tapped: false,
                    haste: true,
                    sacrifice_eot: true,
                    return_eot: false,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::ReturnSelf),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Shifty Doppelganger",
            cost(&[generic(2), u()]),
            vec![CreatureType::Shapeshifter],
            1,
            1,
        )
    }
}
