//! The Last Airbender (TLA) **Blight** cards (CR 701.68). The keyword action
//! ships as `Effect::Blight`; this file adds the Ward—Blight variant
//! (`WardCost::Blight`) and Auntie Ool's -1/-1-counter payoff. Tests in
//! `crabomination/src/tests/blight.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, Selector, Subtypes, Supertype, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::PlayerRef;
use crate::mana::{b, cost, g, generic, r};

/// Auntie Ool, Cursewretch — {1}{B}{R}{G} 4/4 legendary Goblin Warlock.
/// Ward—Blight 2. Whenever one or more -1/-1 counters are put on a creature,
/// draw a card if you control it; otherwise its controller loses 1 life.
pub fn auntie_ool_cursewretch() -> CardDefinition {
    CardDefinition {
        name: "Auntie Ool, Cursewretch",
        cost: cost(&[generic(1), b(), r(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Ward(WardCost::Blight(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::MinusOneMinusOne),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature,
            }),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::ControlledByYou,
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}
