//! MKM (Murders at Karlov Manor) gap batch — token hate + a Merfolk untapper.
//! Tests in `tests/recent_b/recent251.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Effect};
use crate::mana::{b, cost, g, generic, u};

/// Kraul Whipcracker — {B}{G} Creature — Insect Assassin 3/2, reach. When this
/// creature enters, destroy target token an opponent controls.
pub fn kraul_whipcracker() -> CardDefinition {
    CardDefinition {
        name: "Kraul Whipcracker",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::IsToken.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Forensic Researcher — {2}{U} Creature — Merfolk Detective 1/3. {T}: Untap
/// another target permanent you control. (Its "{T}, Collect evidence 3: Tap
/// target creature you don't control" ability is not modeled — activated
/// abilities can't yet take a collect-evidence cost.)
pub fn forensic_researcher() -> CardDefinition {
    CardDefinition {
        name: "Forensic Researcher",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap {
                what: target_filtered(
                    R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
