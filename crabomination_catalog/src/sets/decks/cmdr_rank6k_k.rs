//! Commander: three-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), eleventh file: Rose Noble's changelings. Tests in
//! `tests/recent_b/cmdr_rank6k_k.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{cost, generic, u};

fn shapeshifter(name: &'static str, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Changeling],
        ..Default::default()
    }
}

/// Littjara Kinseekers — {3}{U} 2/4 Shapeshifter, changeling. ETB, if you
/// control three or more creatures that share a creature type: a +1/+1
/// counter on it, then scry 1 (CR 603.4 intervening if).
pub fn littjara_kinseekers() -> CardDefinition {
    let mut trigger = etb(Effect::Seq(vec![
        Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
    ]));
    trigger.event = trigger
        .event
        .with_filter(Predicate::ValueAtLeast(Value::GreatestSharedCreatureTypeCount, Value::Const(3)));
    CardDefinition { triggered_abilities: vec![trigger], ..shapeshifter("Littjara Kinseekers", 2, 4) }
}

/// Turtleshell Changeling — {3}{U} 1/4 Shapeshifter, changeling. {1}{U}:
/// switch its power and toughness until end of turn.
pub fn turtleshell_changeling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::SwitchPT { what: Selector::This, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..shapeshifter("Turtleshell Changeling", 1, 4)
    }
}

/// Ego Erasure — {2}{U} Kindred instant — Shapeshifter, changeling.
/// Creatures target player controls get -2/-0 until end of turn. (Residual:
/// "and lose all creature types until end of turn" is not modelled.)
pub fn ego_erasure() -> CardDefinition {
    CardDefinition {
        name: "Ego Erasure",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Changeling],
        effect: Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::AsPlayer {
                who: PlayerRef::Target(0),
                body: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(-2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                }),
            }),
        },
        ..Default::default()
    }
}
