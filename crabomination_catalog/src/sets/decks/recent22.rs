//! A twenty-second wave — Avatar: The Last Airbender (TLA) Firebending
//! creatures, exercising the new `Keyword::Firebending(n)` (CR 702.189): an
//! attack-triggered mana ability that adds N {R} surviving until end of combat.
//! Tests in `crabomination/src/tests/recent22.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement, Selector, Subtypes, Supertype, Value,
};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{cost, generic, r};

/// Jeong Jeong the Deserter — {2}{R} 2/3 legendary Human Rebel Ally with
/// firebending 1. Exhaust — {3}: put a +1/+1 counter on it. (The "next Lesson
/// you cast this turn is copied" rider is dropped.)
pub fn jeong_jeong_the_deserter() -> CardDefinition {
    CardDefinition {
        name: "Jeong Jeong the Deserter",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Firebending(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ran and Shaw — {3}{R}{R} 4/4 legendary Dragon with flying and firebending 2.
/// (The cast-ETB "copy if 3+ Dragons/Lessons in your graveyard" rider is
/// dropped.)
pub fn ran_and_shaw() -> CardDefinition {
    CardDefinition {
        name: "Ran and Shaw",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Firebending(2)],
        ..Default::default()
    }
}

/// Sozin's Comet — {3}{R}{R} Sorcery. Each creature you control gains
/// firebending 5 until end of turn. (Foretell is dropped.)
pub fn sozins_comet() -> CardDefinition {
    CardDefinition {
        name: "Sozin's Comet",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GrantKeyword {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Firebending(5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
