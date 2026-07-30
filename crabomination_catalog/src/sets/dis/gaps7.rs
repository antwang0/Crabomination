//! Dissension (DIS) gap wave 7. Tests in `classic_sets/dis`.

use crate::card::{CardDefinition, CardType, CreatureType, Subtypes};
use crate::mana::{cost, generic};

/// Bronze Bombshell — {4} 4/1 Construct. CR 603.8 state trigger: when a player
/// other than its owner controls it, that player sacrifices it and it deals 7
/// damage to them.
pub fn bronze_bombshell() -> CardDefinition {
    CardDefinition {
        name: "Bronze Bombshell",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        sacrifice_and_burn_when_stolen: Some(7),
        ..Default::default()
    }
}
