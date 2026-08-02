//! CR 313 / 902 — Vanguard avatars. Each starts in its owner's command zone
//! (`GameState::seat_vanguard`), applies its CR 211/212 hand and life
//! modifiers, and functions from there. Tests in `core_rules/cr_recent_vanguard`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, SelectionRequirement as R,
    StaticAbility, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, StaticEffect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value, ZoneDest,
    shortcut::{draw, target_filtered},
};
use crate::mana::{ManaCost, cost, generic};
use crabomination_base::turn_step::TurnStep;

fn avatar(name: &'static str, hand: i32, life: i32) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Vanguard],
        hand_modifier: hand,
        life_modifier: life,
        ..Default::default()
    }
}

/// "At the beginning of your upkeep, …" from the command zone.
fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

fn cz_ability(mana_cost: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost, effect, from_command_zone: true, ..Default::default() }
}

/// Ashling the Pilgrim Avatar — a repeatable board sweeper for {2} a point.
pub fn ashling_the_pilgrim_avatar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cz_ability(
            cost(&[generic(2)]),
            Effect::Seq(vec![
                Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::ONE,
                    }),
                },
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachPlayer),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::ONE,
                    }),
                },
            ]),
        )],
        ..avatar("Ashling the Pilgrim Avatar", -1, 6)
    }
}

/// Barrin — sacrifice anything to bounce a creature.
pub fn barrin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Permanent, 1)),
            effect: Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            from_command_zone: true,
            ..Default::default()
        }],
        ..avatar("Barrin", 0, 6)
    }
}

/// Crovax — every point your creatures deal is a point of life.
pub fn crovax() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..avatar("Crovax", 2, 0)
    }
}

/// Serra Angel Avatar — two life per spell.
pub fn serra_angel_avatar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..avatar("Serra Angel Avatar", 0, -1)
    }
}

/// Arcbound Overseer Avatar — a counter on a creature and a permanent each
/// upkeep.
pub fn arcbound_overseer_avatar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            upkeep(Effect::MayDo {
                description: "Put a +1/+1 counter on target creature you control?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            }),
            upkeep(Effect::MayDo {
                description: "Put a charge counter on target permanent you control?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                }),
            }),
        ],
        ..avatar("Arcbound Overseer Avatar", 0, 3)
    }
}

/// Squee, Goblin Nabob Avatar — a {1} prevention shield, over and over.
pub fn squee_goblin_nabob_avatar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cz_ability(
            cost(&[generic(1)]),
            Effect::PreventNextDamage {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                amount: Value::ONE,
            },
        )],
        ..avatar("Squee, Goblin Nabob Avatar", 3, -4)
    }
}

/// Chronatog Avatar — no hand limit, and three cards for your next turn.
pub fn chronatog_avatar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            from_command_zone: true,
            effect: Effect::Seq(vec![
                draw(3),
                Effect::SkipTurns { who: PlayerRef::You, count: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..avatar("Chronatog Avatar", -1, 1)
    }
}

/// Maraxus of Keld — {1} for a point of power on anything.
pub fn maraxus_of_keld() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cz_ability(
            cost(&[generic(1)]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        )],
        ..avatar("Maraxus of Keld", -1, 6)
    }
}
