//! Ravnica (RAV) gap wave 17: Belltower Sphinx, riding the new
//! `PlayerRef::LastDamagerControllerOf`. Tests in `classic_sets/rav`.

use crate::card::{CardDefinition, CardType, CreatureType, Keyword, Subtypes, Value};
use crate::effect::shortcut::enrage;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{cost, generic, u};

/// Belltower Sphinx — {4}{U} 2/5 Sphinx with flying. Whenever a source deals
/// damage to this creature, that source's controller mills that many cards.
pub fn belltower_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Belltower Sphinx",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![enrage(Effect::Mill {
            who: Selector::Player(PlayerRef::LastDamagerControllerOf(Box::new(Selector::This))),
            amount: Value::TriggerEventAmount,
        })],
        ..Default::default()
    }
}
