//! Commander Legends Encore batch (CR 702.141) — the first cards on the new
//! `shortcut::encore` graveyard ability. Tests in `tests/recent103.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, SelectionRequirement, Subtypes};
use crate::effect::shortcut::{encore, etb, mint_treasures, on_dies, target_filtered};
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, generic, r, u, w};

/// Impulsive Pilferer — {R} 1/1 Goblin Pirate. Dies: mint a Treasure.
/// Encore {3}{R}.
pub fn impulsive_pilferer() -> CardDefinition {
    CardDefinition {
        name: "Impulsive Pilferer",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(mint_treasures(1))],
        activated_abilities: vec![encore(cost(&[generic(3), r()]))],
        ..Default::default()
    }
}

/// Kinsbaile Courier — {2}{W} 2/1 Kithkin Soldier. ETB: +1/+1 counter on
/// target creature. Encore {2}{W}.
pub fn kinsbaile_courier() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Kinsbaile Courier",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![encore(cost(&[generic(2), w()]))],
        ..Default::default()
    }
}

/// Trove Tracker — {2}{U} 2/2 Human Pirate. Dies: draw a card.
/// Encore {5}{U}{U}.
pub fn trove_tracker() -> CardDefinition {
    CardDefinition {
        name: "Trove Tracker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        activated_abilities: vec![encore(cost(&[generic(5), u(), u()]))],
        ..Default::default()
    }
}
