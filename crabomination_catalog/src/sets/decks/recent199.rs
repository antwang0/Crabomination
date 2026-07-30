//! DSK gap batch on existing primitives: Growing Dread (manifest dread +
//! face-up payoff) and Entity Tracker (Eerie enchantment-enters draw). Tests in
//! `tests/recent199.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, u};

/// Growing Dread — {G}{U} Enchantment with Flash. ETB manifest dread; whenever
/// you turn a permanent face up, put a +1/+1 counter on it.
pub fn growing_dread() -> CardDefinition {
    CardDefinition {
        name: "Growing Dread",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::ManifestDread {
                who: PlayerRef::You,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Entity Tracker — {2}{U} 2/3 Human Scout with Flash. Eerie — whenever an
/// enchantment you control enters, draw a card. (The "fully unlock a Room" half
/// is approximated — Rooms aren't wired to the Eerie trigger.)
pub fn entity_tracker() -> CardDefinition {
    CardDefinition {
        name: "Entity Tracker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}
