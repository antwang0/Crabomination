//! TDM gap: Dragonfire Blade (Equipment granting +2/+2 and hexproof from
//! monocolored). Tests in `tests/recent200.rs`.

use crate::card::{ArtifactSubtype, CardDefinition, CardType, EquipBonus, Keyword, Subtypes};
use crate::mana::{cost, generic};

/// Dragonfire Blade — {1} Equipment. Equipped creature gets +2/+2 and has
/// hexproof from monocolored. Equip {4}. (The "costs {1} less per color of the
/// equip target" reduction is approximated as a flat {4}.)
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
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::HexproofFromMonocolored],
            ..Default::default()
        }),
        ..Default::default()
    }
}
