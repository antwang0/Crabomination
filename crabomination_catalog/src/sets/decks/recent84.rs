//! Kindred / chosen-type-matters batch: Kindred Discovery (draw when a
//! chosen-type creature enters or attacks) and Door of Destinies (charge-
//! counter-scaled tribal anthem). Both exercise the new
//! `Predicate::TriggerObjectIsChosenType` event gate. Tests in
//! `tests/recent84.rs`.

use crate::card::{CardDefinition, CardType, CounterType, StaticAbility, StaticEffect};
use crate::effect::shortcut::{draw, etb};
use crate::effect::{Effect, EventKind, EventScope, EventSpec, Predicate, Selector, TriggeredAbility, Value};
use crate::mana::{cost, generic, u};

/// ETB "choose a creature type" trigger.
fn choose_type() -> TriggeredAbility {
    etb(Effect::NameCreatureType { what: Selector::This })
}

/// Kindred Discovery — {3}{U}{U} Enchantment. Choose a creature type. Whenever
/// a creature of the chosen type you control enters or attacks, draw a card.
pub fn kindred_discovery() -> CardDefinition {
    let of_chosen_type = || Predicate::TriggerObjectIsChosenType;
    CardDefinition {
        name: "Kindred Discovery",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            choose_type(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(of_chosen_type()),
                effect: draw(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(of_chosen_type()),
                effect: draw(1),
            },
        ],
        ..Default::default()
    }
}

/// Door of Destinies — {4} Artifact. Choose a creature type. Whenever you cast
/// a spell of the chosen type, put a charge counter on this. Creatures you
/// control of the chosen type get +1/+1 for each charge counter on this.
pub fn door_of_destinies() -> CardDefinition {
    CardDefinition {
        name: "Door of Destinies",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            choose_type(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::TriggerObjectIsChosenType),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(1),
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1 for each charge counter on this.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false,
                per_counter: Some(CounterType::Charge),
            },
        }],
        ..Default::default()
    }
}
