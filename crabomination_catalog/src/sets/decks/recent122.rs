//! A mixed FDN/BLB batch: a vanilla beater, surveil + threshold evasion, a
//! conditional-discount tuck, and a Fact-or-Fiction sphinx.
//! Tests in `tests/recent122.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, Predicate, SelectionRequirement as R,
    StaticAbility, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, LibraryPosition, PlayerRef, StaticEffect, Value, ZoneDest};
use crate::mana::{cost, g, generic, u};

/// Gigantosaurus — {G}{G}{G}{G}{G} 10/10 Dinosaur.
pub fn gigantosaurus() -> CardDefinition {
    CardDefinition {
        name: "Gigantosaurus",
        cost: cost(&[g(), g(), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        ..Default::default()
    }
}

/// Cephalid Inkmage — {2}{U} 2/2 Octopus Wizard. ETB: surveil 3. Threshold —
/// can't be blocked while seven or more cards are in your graveyard.
pub fn cephalid_inkmage() -> CardDefinition {
    CardDefinition {
        name: "Cephalid Inkmage",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(3),
        })],
        static_abilities: vec![StaticAbility {
            description: "Threshold — can't be blocked while seven or more cards are in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ThresholdActive {
                    who: PlayerRef::You,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
            },
        }],
        ..Default::default()
    }
}

/// Dire Downdraft — {3}{U} Instant. Costs {1} less if it targets an attacking or
/// tapped creature. Target creature's owner puts it on the top or bottom of
/// their library (their choice).
pub fn dire_downdraft() -> CardDefinition {
    CardDefinition {
        name: "Dire Downdraft",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((R::IsAttacking.or(R::Tapped), 1)),
        effect: Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::OwnerChoice,
            },
        },
        ..Default::default()
    }
}

/// Curator of Destinies — {4}{U}{U} 5/5 Sphinx. Can't be countered; flying. ETB:
/// Fact or Fiction on the top five cards. (An opponent splits the reveal into a
/// pile to your hand and a pile to your graveyard.)
pub fn curator_of_destinies() -> CardDefinition {
    CardDefinition {
        name: "Curator of Destinies",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::CantBeCountered, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::FactOrFiction {
            count: Value::Const(5),
            to_bottom: false,
        })],
        ..Default::default()
    }
}
