//! A Bloomburrow Mouse wave built on the **Valiant** keyword (CR 702.176 —
//! `shortcut::valiant`, a once-per-turn `BecameTarget` trigger; the implicit
//! source==target guard pins it to the creature and `YourControl` refines on
//! your own casts). Tests in `crabomination/src/tests/recent156.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, SelectionRequirement as R,
    Selector, Subtypes, Value,
};
use crate::effect::shortcut::{target_filtered, valiant};
use crate::effect::{Duration, Effect, ZoneRef};
use crate::mana::{cost, generic, r, w};

fn mouse_soldier() -> Subtypes {
    Subtypes {
        creature_types: vec![CreatureType::Mouse, CreatureType::Soldier],
        ..Default::default()
    }
}

/// Seedglaive Mentor — {1}{R}{W} 3/2 Mouse Soldier with vigilance and haste.
/// Valiant: put a +1/+1 counter on it.
pub fn seedglaive_mentor() -> CardDefinition {
    CardDefinition {
        name: "Seedglaive Mentor",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: mouse_soldier(),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![valiant(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Mouse Trapper — {2}{W} 3/2 Mouse Soldier with flash. Valiant: tap target
/// creature an opponent controls.
pub fn mouse_trapper() -> CardDefinition {
    CardDefinition {
        name: "Mouse Trapper",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: mouse_soldier(),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![valiant(Effect::Tap {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Flowerfoot Swordmaster — {W} 1/2 Mouse Soldier with Offspring {2}. Valiant:
/// Mice you control get +1/+0 until end of turn.
pub fn flowerfoot_swordmaster() -> CardDefinition {
    CardDefinition {
        name: "Flowerfoot Swordmaster",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: mouse_soldier(),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![valiant(Effect::PumpPT {
            what: Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::HasCreatureType(CreatureType::Mouse).and(R::ControlledByYou),
            },
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Whiskerquill Scribe — {1}{R} 2/2 Mouse Citizen. Valiant: you may discard a
/// card; if you do, draw a card.
pub fn whiskerquill_scribe() -> CardDefinition {
    CardDefinition {
        name: "Whiskerquill Scribe",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![valiant(Effect::MayDiscard {
            description: "Discard a card to draw a card?".into(),
            count: Value::ONE,
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            else_: None,
        })],
        ..Default::default()
    }
}
