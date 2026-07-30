//! A final BLB batch of simple creatures reusing existing primitives: a
//! firebreathing pump, Prowess + ETB surveil, threshold evasion, and Offspring
//! + ETB flying grant. Tests in `tests/recent119.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value};
use crate::mana::{b, cost, generic, u};

/// Ravine Raider — {B} 1/1 Lizard Rogue with menace. {1}{B}: gets +1/+1 until
/// end of turn.
pub fn ravine_raider() -> CardDefinition {
    CardDefinition {
        name: "Ravine Raider",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lightshell Duo — {3}{U} 3/4 Rat Otter with prowess. ETB: surveil 2.
pub fn lightshell_duo() -> CardDefinition {
    CardDefinition {
        name: "Lightshell Duo",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Otter],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Nightwhorl Hermit — {2}{U} 1/4 Rat Rogue with vigilance. Threshold — gets
/// +1/+0 and can't be blocked while seven or more cards are in your graveyard.
pub fn nightwhorl_hermit() -> CardDefinition {
    CardDefinition {
        name: "Nightwhorl Hermit",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Threshold — gets +1/+0 and can't be blocked while seven or more cards are in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ThresholdActive {
                    who: PlayerRef::You,
                },
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
            },
        }],
        ..Default::default()
    }
}

/// Finch Formation — {2}{U} 2/2 Bird Scout with flying and Offspring {3}. ETB:
/// target creature you control gains flying until end of turn.
pub fn finch_formation() -> CardDefinition {
    CardDefinition {
        name: "Finch Formation",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Offspring(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}
