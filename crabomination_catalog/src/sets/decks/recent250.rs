//! MKM (Murders at Karlov Manor) gap batch — control + protection Auras.
//! Tests in `tests/recent_b/recent250.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus, Keyword,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, Selector};
use crate::mana::{b, cost, g, generic, u};

/// The creature this Aura is attached to.
fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// Coerced to Kill — {3}{U}{B} Aura. Enchant creature. You control enchanted
/// creature. It has base power and toughness 1/1, has deathtouch, and is an
/// Assassin in addition to its other types.
pub fn coerced_to_kill() -> CardDefinition {
    CardDefinition {
        name: "Coerced to Kill",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: enchanted(),
        })],
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((1, 1)),
            keywords: vec![Keyword::Deathtouch],
            add_creature_types: vec![CreatureType::Assassin],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Airtight Alibi — {2}{G} Aura, flash. Enchant creature. ETB untap enchanted
/// creature, it gains hexproof until end of turn, and it's no longer suspected.
/// Enchanted creature gets +2/+2. (The "can't become suspected" rider is dropped.)
pub fn airtight_alibi() -> CardDefinition {
    CardDefinition {
        name: "Airtight Alibi",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Untap {
                what: enchanted(),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: enchanted(),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::ClearSuspected { what: enchanted() },
        ]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            ..Default::default()
        }),
        ..Default::default()
    }
}
