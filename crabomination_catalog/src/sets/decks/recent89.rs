//! More red burn: Flame Burst, Lightning Blast, Inferno (board+players), and
//! Crater Hellion (Echo + ETB sweep). Tests in `tests/recent89.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::{deal, etb, target};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, generic, r};

/// Flame Burst — {1}{R} Instant. Deals 2 damage to any target.
pub fn flame_burst() -> CardDefinition {
    CardDefinition {
        name: "Flame Burst",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: deal(2, target()),
        ..Default::default()
    }
}

/// Lightning Blast — {3}{R} Instant. Deals 4 damage to any target.
pub fn lightning_blast() -> CardDefinition {
    CardDefinition {
        name: "Lightning Blast",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: deal(4, target()),
        ..Default::default()
    }
}

/// Inferno — {5}{R}{R} Instant. Deals 6 damage to each creature and each player.
pub fn inferno() -> CardDefinition {
    CardDefinition {
        name: "Inferno",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(6),
                }),
            },
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(6),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Crater Hellion — {4}{R}{R} 5/5 Beast. Echo {4}{R}{R}. When it enters, it
/// deals 4 damage to each other creature.
pub fn crater_hellion() -> CardDefinition {
    CardDefinition {
        name: "Crater Hellion",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Echo(cost(&[generic(4), r(), r()]))],
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(4),
            }),
        })],
        ..Default::default()
    }
}
