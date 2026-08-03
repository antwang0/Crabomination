//! Zendikar Quest enchantment cycle (the TODO.md deferred remainder).
//! Tests in `tests/quests.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, SelectionRequirement, Selector, TriggeredAbility, Value,
};
use crate::effect::{Effect, PlayerRef, Predicate};
use crate::mana::{b, cost, r, u};

fn quest_counter_on_self() -> Effect {
    Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Quest,
        amount: Value::ONE,
    }
}

/// Quest for Pure Flame — {R}. Your sources damaging an opponent accrue quest
/// counters; remove four + sac: your sources deal double damage this turn.
pub fn quest_for_pure_flame() -> CardDefinition {
    CardDefinition {
        name: "Quest for Pure Flame",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PlayerDamaged,
                EventScope::YourSourceDamagedOpponent,
            ),
            effect: quest_counter_on_self(),
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Quest, 4)),
            sac_cost: true,
            effect: Effect::DoubleYourSourcesDamageThisTurn,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Quest for Ula's Temple — {U}. Upkeep: reveal top, creature → quest counter;
/// each end step with 3+ counters: deploy a sea monster from hand.
pub fn quest_for_ulas_temple() -> CardDefinition {
    let sea_monster = SelectionRequirement::Creature.and(
        SelectionRequirement::HasCreatureType(CreatureType::Kraken)
            .or(SelectionRequirement::HasCreatureType(
                CreatureType::Leviathan,
            ))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Octopus))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Serpent)),
    );
    CardDefinition {
        name: "Quest for Ula's Temple",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::RevealTopThenIf {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature,
                    then: Box::new(quest_counter_on_self()),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Quest,
                    },
                    Value::Const(3),
                )),
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: sea_monster,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                return_eot: false,
                then: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Quest for the Nihil Stone — {B}. Opponent discards accrue quest counters;
/// at an empty-handed opponent's upkeep with 2+ counters they lose 5 life.
pub fn quest_for_the_nihil_stone() -> CardDefinition {
    CardDefinition {
        name: "Quest for the Nihil Stone",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
                effect: quest_counter_on_self(),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::OpponentControl,
                )
                .with_filter(Predicate::All(vec![
                    Predicate::ValueAtMost(
                        Value::HandSizeOf(PlayerRef::ActivePlayer),
                        Value::Const(0),
                    ),
                    Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Quest,
                        },
                        Value::Const(2),
                    ),
                ])),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(5),
                },
            },
        ],
        ..Default::default()
    }
}
