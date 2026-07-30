//! Forage-matters (BLB). Corpseberry Cultivator exercises the new
//! `EventKind::Foraged` "whenever you forage" trigger. Tests in
//! `tests/recent123.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Predicate, Subtypes, TriggeredAbility,
};
use crate::effect::{Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, generic, hybrid};

/// Corpseberry Cultivator — {1}{B/G}{B/G} 2/3 Squirrel Warlock. At the beginning
/// of combat on your turn, you may forage. Whenever you forage, put a +1/+1
/// counter on it.
pub fn corpseberry_cultivator() -> CardDefinition {
    CardDefinition {
        name: "Corpseberry Cultivator",
        cost: cost(&[
            generic(1),
            hybrid(Color::Black, Color::Green),
            hybrid(Color::Black, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::Forage {
                    then: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Foraged, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}
