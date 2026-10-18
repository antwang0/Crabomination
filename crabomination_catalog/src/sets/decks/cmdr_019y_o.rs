//! Commander: 2026-10-18 (`019Y6J4R`) closers, fifteenth batch. Tests in
//! `tests/recent_b/cmdr_019y_o.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    LoyaltyAbility, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{cost, generic, u};

/// Teferi's Talent — {3}{U}{U} Aura, enchant planeswalker. Enchanted
/// planeswalker has "[−12]: You get an emblem with 'You may activate loyalty
/// abilities of planeswalkers you control on any player's turn any time you
/// could cast an instant.'" Whenever you draw a card, put a loyalty counter on
/// enchanted planeswalker.
pub fn teferis_talent() -> CardDefinition {
    CardDefinition {
        name: "Teferi's Talent",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Planeswalker) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted planeswalker has \"[−12]: You get an emblem with 'You may activate loyalty abilities of planeswalkers you control on any player's turn any time you could cast an instant.'\"",
            effect: StaticEffect::AttachedPlaneswalkerHasLoyaltyAbilities {
                abilities: vec![LoyaltyAbility {
                    loyalty_cost: -12,
                    effect: Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Teferi's Talent".into(),
                        triggered: vec![],
                        statics: vec![StaticAbility {
                            description: "You may activate loyalty abilities of planeswalkers you control on any player's turn any time you could cast an instant.",
                            effect: StaticEffect::LoyaltyAbilitiesAtInstantSpeed,
                        }],
                    },
                    ..Default::default()
                }],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::attached_to(Selector::This),
                kind: CounterType::Loyalty,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// The Clone Saga — {3}{U} Saga. I — surveil 3. II — when you next cast a
/// creature spell this turn, copy it, except the copy isn't legendary. III —
/// choose a card name; whenever a creature with the chosen name deals combat
/// damage to a player this turn, draw a card.
pub fn the_clone_saga() -> CardDefinition {
    CardDefinition {
        name: "The Clone Saga",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Surveil { who: PlayerRef::You, amount: Value::Const(3) }),
            (
                2,
                Effect::OnYourNextSpellMatchingThisTurn {
                    filter: R::Creature,
                    body: Box::new(Effect::CopySpellNonLegendary { what: Selector::TriggerSource }),
                },
            ),
            (
                3,
                Effect::NameCardThenNamedCreatureCombatDamageThisTurn {
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ),
        ],
        ..Default::default()
    }
}
