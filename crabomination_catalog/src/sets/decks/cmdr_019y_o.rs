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
