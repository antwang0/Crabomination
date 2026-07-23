//! Gatecrash (GTC) wave 17 — remaining gap cards. Tests in `classic_sets/gtc`.

use crate::card::{
    CardDefinition, CardType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    LandType, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r};

/// Frenzied Tilling — {3}{R}{G} Sorcery. Destroy target land. Search your
/// library for a basic land card, put it onto the battlefield tapped, then
/// shuffle.
pub fn frenzied_tilling() -> CardDefinition {
    CardDefinition {
        name: "Frenzied Tilling",
        cost: cost(&[generic(3), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Land) },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        ]),
        ..Default::default()
    }
}

/// Contaminated Ground — {1}{B} Aura. Enchant land. Enchanted land is a Swamp.
/// Whenever enchanted land becomes tapped, its controller loses 2 life.
pub fn contaminated_ground() -> CardDefinition {
    CardDefinition {
        name: "Contaminated Ground",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land is a Swamp.",
            effect: StaticEffect::LandTypeChanger {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                land_type: LandType::Swamp,
                replace: true,
            },
        }],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
