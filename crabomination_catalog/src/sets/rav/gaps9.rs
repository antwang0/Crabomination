//! Ravnica (RAV) gap wave 9: Golgari graveyard value (dredge, sac-for-counter)
//! and a pair of edict-style black cards. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
    Value,
};
use crate::game::TurnStep;
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic};

/// Necroplasm — {1}{B}{B} 1/1 Ooze. Upkeep: put a +1/+1 counter on it. End
/// step: destroy each creature with mana value equal to its counter count.
/// Dredge 2.
pub fn necroplasm() -> CardDefinition {
    CardDefinition {
        name: "Necroplasm",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Dredge(2)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::DestroyEachCreatureWithManaValue {
                    value: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Shambling Shell — {1}{B}{G} 3/1 Plant Zombie. Sacrifice this creature: put a
/// +1/+1 counter on target creature. Dredge 3.
pub fn shambling_shell() -> CardDefinition {
    CardDefinition {
        name: "Shambling Shell",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Zombie],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Dredge(3)],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Woebringer Demon — {3}{B}{B} 4/4 Demon with flying. At the beginning of each
/// player's upkeep, that player sacrifices a creature of their choice; if they
/// can't, sacrifice this creature.
pub fn woebringer_demon() -> CardDefinition {
    CardDefinition {
        name: "Woebringer Demon",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CreatureCountControlledBy(PlayerRef::ActivePlayer),
                    Value::ONE,
                ),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    count: Value::ONE,
                    filter: R::Creature,
                }),
                else_: Box::new(Effect::SacrificeSource),
            },
        }],
        ..Default::default()
    }
}

/// Perilous Forays — {3}{G}{G} Enchantment. {1}, Sacrifice a creature: Search
/// your library for a land card with a basic land type, put it onto the
/// battlefield tapped, then shuffle.
pub fn perilous_forays() -> CardDefinition {
    let basic_type = R::HasLandType(LandType::Plains)
        .or(R::HasLandType(LandType::Island))
        .or(R::HasLandType(LandType::Swamp))
        .or(R::HasLandType(LandType::Mountain))
        .or(R::HasLandType(LandType::Forest));
    CardDefinition {
        name: "Perilous Forays",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Land.and(basic_type),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
