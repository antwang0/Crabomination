//! Dissension (DIS) gap wave 8: War's Toll's land lock, Rakdos Riteknife's
//! blood counters, and Brace for Impact's damage-into-counters shield.
//! Tests in `classic_sets/dis`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, EquipBonus, EquipScale, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect, TriggeredAbility,
};
use crate::mana::{b, cost, generic, r, w};

/// War's Toll — {3}{R} Enchantment. Whenever an opponent taps a land for mana,
/// tap all lands they control; and their creatures attack each combat if able.
///
/// Approximation: the printed attack clause only bites once one of their
/// creatures attacks — here it's an unconditional attack requirement.
pub fn wars_toll() -> CardDefinition {
    CardDefinition {
        name: "War's Toll",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control attack each combat if able.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::MustAttack],
                opponents: true,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                filter: Some(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land,
                }),
                ..EventSpec::new(EventKind::TappedForMana, EventScope::OpponentControl)
            },
            effect: Effect::Tap {
                what: Selector::ControlledBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    filter: R::Land,
                },
            },
        }],
        ..Default::default()
    }
}

/// Rakdos Riteknife — {2} Equipment. Equipped creature gets +1/+0 per blood
/// counter and can tap-and-sacrifice a creature to add one; {B}{R}, sacrifice
/// this: target player sacrifices a permanent per blood counter. Equip {2}.
pub fn rakdos_riteknife() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Riteknife",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 0,
                count_self_counters: Some(CounterType::Blood),
                ..Default::default()
            }),
            ..Default::default()
        }),
        activated_abilities: vec![
            // The printed line grants the equipped creature "{T}, Sacrifice a
            // creature: put a blood counter on Rakdos Riteknife"; modeled as an
            // Equipment ability that taps an equipped creature instead.
            ActivatedAbility {
                tap_other_filter: Some(R::Creature.and(R::IsEquipped)),
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Blood,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b(), r()]),
                sac_cost: true,
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Target(0)),
                    count: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Blood,
                    },
                    filter: R::Permanent,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Brace for Impact — {4}{W} Instant. Prevent all damage that would be dealt to
/// target multicolored creature this turn; it gets a +1/+1 counter per point.
pub fn brace_for_impact() -> CardDefinition {
    CardDefinition {
        name: "Brace for Impact",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventAllDamageThisTurnWithCounters {
            target: target_filtered(R::Creature.and(R::Multicolored)),
        },
        ..Default::default()
    }
}
