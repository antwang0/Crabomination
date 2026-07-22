//! Return to Ravnica (RTR) gap wave 12: an Izzet spell-eating Elemental
//! (exile-a-spell-you-control activation cost). Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect,
    SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::Selector;
use crate::mana::{cost, hybrid, Color};

/// Nivmagus Elemental — {U/R} 1/2 Elemental. Exile an instant or sorcery
/// spell you control: put two +1/+1 counters on this creature.
pub fn nivmagus_elemental() -> CardDefinition {
    CardDefinition {
        name: "Nivmagus Elemental",
        cost: cost(&[hybrid(Color::Blue, Color::Red)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            exile_spell_cost: Some(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
