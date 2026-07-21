//! Ravnica (RAV) gap wave 10: a doesn't-untap Aura (reusing `PreventUntap`),
//! Savra's Golgari sacrifice payoffs, and Searing Meditation's lifegain burn.
//! Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Stasis Cell — {4}{U} Aura. Enchant creature; it doesn't untap during its
/// controller's untap step. {3}{U}: Attach this Aura to target creature.
pub fn stasis_cell() -> CardDefinition {
    CardDefinition {
        name: "Stasis Cell",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Savra, Queen of the Golgari — {2}{B}{G} 2/2 Elf Shaman. Whenever you
/// sacrifice a black creature, you may pay 2 life; if you do, each other player
/// sacrifices a creature of their choice. Whenever you sacrifice a green
/// creature, you may gain 2 life.
pub fn savra_queen_of_the_golgari() -> CardDefinition {
    let sac_of_color = |color| {
        EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasColor(color),
            },
        )
    };
    CardDefinition {
        name: "Savra, Queen of the Golgari",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: sac_of_color(crate::mana::Color::Black),
                effect: Effect::MayPayLife {
                    description: "Each other player sacrifices a creature?".into(),
                    amount: Value::Const(2),
                    body: Box::new(Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        count: Value::ONE,
                        filter: R::Creature,
                    }),
                    else_: None,
                },
            },
            TriggeredAbility {
                event: sac_of_color(crate::mana::Color::Green),
                effect: Effect::MayDo {
                    description: "Gain 2 life?".into(),
                    body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Searing Meditation — {1}{R}{W} Enchantment. Whenever you gain life, you may
/// pay {2}. If you do, it deals 2 damage to any target.
pub fn searing_meditation() -> CardDefinition {
    CardDefinition {
        name: "Searing Meditation",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {2} to deal 2 damage to any target?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::DealDamage { amount: Value::Const(2), to: target_any() }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
