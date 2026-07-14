//! DSK gap: Duskmourn's Domination (Control-Magic Aura that also shrinks and
//! silences its host). Tests in `tests/recent201.rs`.

use crate::card::{CardDefinition, CardType, EnchantmentSubtype, EquipBonus, SelectionRequirement as R, Subtypes};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, Selector};
use crate::mana::{cost, generic, u};

/// Duskmourn's Domination — {4}{U}{U} Aura. Enchant creature; you control it and
/// it gets -3/-0 and loses all abilities.
pub fn duskmourns_domination() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Duskmourn's Domination",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains { what: enchanted() })],
        equipped_bonus: Some(EquipBonus {
            power: -3,
            toughness: 0,
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}
