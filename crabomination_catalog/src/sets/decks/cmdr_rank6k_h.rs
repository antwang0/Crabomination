//! Commander: two-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), eighth file. Tests in `tests/recent_b/cmdr_rank6k_h.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, LandType, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::ExtraManaKind;
use crate::mana::{cost, g, generic, u};

/// Utrom Monitor — {4}{U} 3/3 Utrom Scientist artifact creature, affinity for
/// artifacts, flying.
pub fn utrom_monitor() -> CardDefinition {
    CardDefinition {
        name: "Utrom Monitor",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Utrom, CreatureType::Scientist],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(R::Artifact),
        ..Default::default()
    }
}

/// Keeper of Progenitus — {3}{G} 1/3 Elf Druid. Whenever a player taps a
/// Mountain, Forest or Plains for mana, they add one more of a type it made.
pub fn keeper_of_progenitus() -> CardDefinition {
    CardDefinition {
        name: "Keeper of Progenitus",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Whenever a player taps a Mountain, Forest, or Plains for mana, that player adds one mana of any type that land produced.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::HasLandType(LandType::Mountain)
                    .or(R::HasLandType(LandType::Forest))
                    .or(R::HasLandType(LandType::Plains)),
                extra: ExtraManaKind::Mirror,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}
