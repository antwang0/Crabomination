//! Commander: 2026-10-19 (`01DJHkWk`) — closers that needed an engine
//! extension first. Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, SelectionRequirement as R, Subtypes};
use crate::effect::shortcut::{prowl, target_filtered};
use crate::effect::{Effect, Value};
use crate::mana::{b, cost, generic};

/// Noggin Whack — {2}{B}{B} Kindred sorcery — Rogue, prowl {1}{B}. Target
/// player reveals three cards from their hand; you choose two; they discard
/// them.
pub fn noggin_whack() -> CardDefinition {
    CardDefinition {
        name: "Noggin Whack",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rogue],
            ..Default::default()
        },
        alternative_cost: Some(prowl(cost(&[generic(1), b()]), vec![CreatureType::Rogue])),
        effect: Effect::DiscardChosenFromRevealed {
            from: target_filtered(R::Player),
            reveal: Value::Const(3),
            pick: Value::Const(2),
        },
        ..Default::default()
    }
}
