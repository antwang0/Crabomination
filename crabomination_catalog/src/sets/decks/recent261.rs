//! MKM (Murders at Karlov Manor) gap batch — a Selesnya land Aura.
//! Tests in `tests/recent_b/recent261.rs`.

use crate::card::{
    CardDefinition, CardType, EnchantmentSubtype, ExileReturnZone, SelectionRequirement as R,
    StaticAbility, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, ExtraManaKind, Selector, StaticEffect};
use crate::mana::{cost, g, generic, w};

/// Buried in the Garden — {2}{G}{W} Aura — Enchant land. ETB exiles a nonland
/// permanent you don't control until this Aura leaves. Enchanted land taps for
/// an extra mana of any color.
pub fn buried_in_the_garden() -> CardDefinition {
    CardDefinition {
        name: "Buried in the Garden",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Nonland.and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        static_abilities: vec![StaticAbility {
            description: "Whenever enchanted land is tapped for mana, its controller adds an \
                          additional one mana of any color.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: true,
                filter: R::Land,
                extra: ExtraManaKind::AnyColor,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}
