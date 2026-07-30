//! CR 702.158 Space Sculptor. Tests in `core_rules/cr_recent36`.

use crate::card::{
    CardDefinition, CardType, CounterType, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Subtypes,
    Supertype, Value,
};
use crate::effect::{Effect, Selector};
use crate::mana::{cost, generic, u, w};

/// Space Beleren — {2}{W}{U} legendary planeswalker, loyalty 3. Space sculptor
/// divides the battlefield into sectors; his abilities lock blocks to a sector,
/// grow one, or wipe one.
pub fn space_beleren() -> CardDefinition {
    CardDefinition {
        name: "Space Beleren",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace],
            ..Default::default()
        },
        keywords: vec![Keyword::SpaceSculptor],
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::SectorBlockLockThisTurn,
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::ChooseSector {
                    body: Box::new(Effect::AddCounter {
                        what: Selector::CreaturesInChosenSector,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::ChooseSector {
                    body: Box::new(Effect::Destroy {
                        what: Selector::CreaturesInChosenSector,
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
