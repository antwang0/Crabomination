//! Return to Ravnica (RTR) gap wave 12: an Izzet spell-eating Elemental
//! (exile-a-spell-you-control activation cost) and a hand-size-scaling Aura.
//! Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec,
    SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::Selector;
use crate::effect::shortcut::target_filtered;
use crate::game::TurnStep;
use crate::mana::{Color, cost, generic, hybrid, u, w};

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
            exile_spell_cost: Some(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            ),
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

/// Righteous Authority — {3}{W}{U} Aura. Enchanted creature gets +1/+1 for each
/// card in its controller's hand, and that player draws an extra card at the
/// beginning of their draw step.
pub fn righteous_authority() -> CardDefinition {
    CardDefinition {
        name: "Righteous Authority",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_host_controller_hand: true,
                ..Default::default()
            }),
            // Fires on the enchanted creature's controller's draw step (the
            // bonus's triggers are granted to the host).
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::YourControl,
                ),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
