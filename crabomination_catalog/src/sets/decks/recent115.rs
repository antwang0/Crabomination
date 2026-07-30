//! Mana-persistence staples. Engine work: `StaticEffect::ManaPoolsNeverEmpty`
//! (Upwelling — CR 500.4 exception), `StaticEffect::UnspentColorManaPersists`
//! (Omnath keeps green — CR 106.4 exception), and
//! `DynamicPt::BasePlusUnspentColorMana` (Omnath grows with unspent green).
//! Tests in `tests/recent115.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, DynamicPt, StaticAbility, Subtypes, Supertype,
};
use crate::effect::StaticEffect;
use crate::mana::{Color, cost, g, generic};

/// Upwelling — {3}{G} enchantment. Players don't lose unspent mana as steps
/// and phases end.
pub fn upwelling() -> CardDefinition {
    CardDefinition {
        name: "Upwelling",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Players don't lose unspent mana as steps and phases end.",
            effect: StaticEffect::ManaPoolsNeverEmpty,
        }],
        ..Default::default()
    }
}

/// Omnath, Locus of Mana — {2}{G} legendary 1/1 Elemental. You don't lose
/// unspent green mana as steps and phases end; Omnath gets +1/+1 for each
/// unspent green mana you have.
pub fn omnath_locus_of_mana() -> CardDefinition {
    CardDefinition {
        name: "Omnath, Locus of Mana",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        dynamic_pt: Some(DynamicPt::BasePlusUnspentColorMana {
            base_p: 1,
            base_t: 1,
            color: Color::Green,
        }),
        static_abilities: vec![StaticAbility {
            description: "You don't lose unspent green mana as steps and phases end.",
            effect: StaticEffect::UnspentColorManaPersists(Color::Green),
        }],
        ..Default::default()
    }
}
