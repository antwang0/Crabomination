//! Secrets of Strixhaven — the Special Guests (SPG) sheet, the eleven
//! library-and-copy cards printed alongside the set. Nine of them already
//! live elsewhere in the catalog; `SPECIAL_GUEST_NAMES` is the whole sheet,
//! and the two below are the ones SOS printed first.

use crate::card::{ActivatedAbility, CardDefinition, CardType, StaticAbility, Subtypes};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, StaticEffect, Value};
use crate::mana::{cost, g, generic};

/// The SOS Special Guests sheet, in collector-number order.
pub const SPECIAL_GUEST_NAMES: [&str; 11] = [
    "Archaeomancer",
    "Archmage Emeritus",
    "Murmuring Mystic",
    "Grim Haruspex",
    "Dualcaster Mage",
    "Magus of the Library",
    "Sylvan Library",
    "Adrix and Nev, Twincasters",
    "Codie, Vociferous Codex",
    "Library of Leng",
    "Library of Alexandria",
];

/// Magus of the Library — {G}{G} 1/1. Taps for {C}, or draws a card while
/// your hand is exactly seven cards deep.
pub fn magus_of_the_library() -> CardDefinition {
    CardDefinition {
        name: "Magus of the Library",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                crate::card::CreatureType::Human,
                crate::card::CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Draw { who: crate::effect::Selector::You, amount: Value::ONE },
                condition: Some(Predicate::ValueEquals(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::Const(7),
                )),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Library of Leng — {1} Artifact. No maximum hand size, and a forced
/// discard goes on top of your library instead of into your graveyard.
pub fn library_of_leng() -> CardDefinition {
    CardDefinition {
        name: "Library of Leng",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "You have no maximum hand size.",
                effect: StaticEffect::NoMaximumHandSize,
            },
            StaticAbility {
                description: "If an effect causes you to discard a card, discard it, \
                              but you may put it on top of your library instead of \
                              into your graveyard.",
                effect: StaticEffect::DiscardToLibraryTop,
            },
        ],
        ..Default::default()
    }
}
