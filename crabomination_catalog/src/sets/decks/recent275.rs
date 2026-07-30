//! BLB/MOM gap batch — an X-dig Sorcery, a counter-Treasure Dwarf, a
//! Phyrexian sacrifice engine, and a Rabbit go-wide pump. All on existing
//! primitives. Tests in `tests/recent_b/recent275.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{mint_treasures, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{b, cost, generic, r, w, x};

/// Stargaze — {X}{B}{B} Sorcery. Look at 2X cards from the top of your library,
/// put X into your hand and the rest into your graveyard, then lose X life.
pub fn stargaze() -> CardDefinition {
    CardDefinition {
        name: "Stargaze",
        cost: cost(&[x(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Sum(vec![Value::XFromCost, Value::XFromCost]),
                rest_to_graveyard: true,
                pick_filter: None,
                take: Some(Value::XFromCost),
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
        ]),
        ..Default::default()
    }
}

/// Axgard Artisan — {1}{R} 2/1 Dwarf Artificer. Whenever one or more +1/+1
/// counters are put on it for the first time each turn, create a Treasure.
pub fn axgard_artisan() -> CardDefinition {
    CardDefinition {
        name: "Axgard Artisan",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::SelfSource,
            )
            .once_per_turn(),
            effect: mint_treasures(1),
        }],
        ..Default::default()
    }
}

/// Bloated Processor — {2}{B} 3/2 Phyrexian. Sacrifice another Phyrexian: put a
/// +1/+1 counter on it. When it dies, incubate X, where X is its power.
pub fn bloated_processor() -> CardDefinition {
    CardDefinition {
        name: "Bloated Processor",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Phyrexian), 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![on_dies(Effect::Incubate {
            who: PlayerRef::You,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Harvestrite Host — {2}{W} 3/3 Rabbit Citizen. Whenever it or another Rabbit
/// you control enters, target creature you control gets +1/+0; draw a card if
/// this is the second such resolution this turn.
pub fn harvestrite_host() -> CardDefinition {
    CardDefinition {
        name: "Harvestrite Host",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Citizen],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Rabbit),
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                // Draw only on the second resolution this turn (shared per-turn
                // resolution counter; exact for a lone Harvestrite Host).
                Effect::NthResolutionThisTurn {
                    branches: vec![
                        Effect::Noop,
                        Effect::Draw {
                            who: Selector::You,
                            amount: Value::ONE,
                        },
                    ],
                },
            ]),
        }],
        ..Default::default()
    }
}
