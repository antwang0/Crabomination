//! Ravnica (RAV) gap wave 12. Reuses existing primitives
//! (`R::DamagedBySourceThisTurn`). Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, Selector};
use crate::mana::{cost, generic, g};

/// Trophy Hunter — {2}{G} 2/3 Human Archer. {1}{G}: This creature deals 1
/// damage to target creature with flying. Whenever a creature with flying dealt
/// damage by this creature this turn dies, put a +1/+1 counter on this creature.
pub fn trophy_hunter() -> CardDefinition {
    CardDefinition {
        name: "Trophy Hunter",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Flying).and(R::DamagedBySourceThisTurn),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}
