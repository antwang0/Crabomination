//! TDM gap: Dragonfire Blade (Equipment granting +2/+2 and hexproof from
//! monocolored). Tests in `tests/recent200.rs`.

use crate::card::{ArtifactSubtype, CardDefinition, CardType, EquipBonus, Keyword, Subtypes};
use crate::mana::{cost, generic};

/// Dragonfire Blade — {1} Equipment. Equipped creature gets +2/+2 and has
/// hexproof from monocolored. Equip {4}; it costs {1} less for each color of
/// the creature it targets (`EquipCostReducedPerTargetColor`).
pub fn dragonfire_blade() -> CardDefinition {
    CardDefinition {
        name: "Dragonfire Blade",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        static_abilities: vec![crate::card::StaticAbility {
            description: "This ability costs {1} less to activate for each color of the creature it targets.",
            effect: crate::card::StaticEffect::EquipCostReducedPerTargetColor,
        }],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::HexproofFromMonocolored],
            ..Default::default()
        }),
        ..Default::default()
    }
}
