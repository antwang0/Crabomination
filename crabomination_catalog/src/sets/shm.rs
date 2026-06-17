//! Shadowmoor / Eventide — Conspire (CR 702.79). As you cast a Conspire
//! spell you may tap two untapped creatures you control sharing a color with
//! it to copy it once (the copy may choose new targets). Cast via
//! `GameAction::CastSpellConspire`.

use crate::card::{CardDefinition, CardType, Keyword};
use crate::effect::shortcut::{deal, pump_target, target};
use crate::mana::{cost, g, generic, r};

/// Burn Trail — {3}{R} Sorcery. "Burn Trail deals 3 damage to any target.
/// Conspire."
pub fn burn_trail() -> CardDefinition {
    CardDefinition {
        name: "Burn Trail",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: deal(3, target()),
        ..Default::default()
    }
}

/// Barkshell Blessing — {G/W} Instant. "Target creature gets +2/+2 until end
/// of turn. Conspire." (Modeled with the green half of the hybrid pip.)
pub fn barkshell_blessing() -> CardDefinition {
    CardDefinition {
        name: "Barkshell Blessing",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Conspire],
        effect: pump_target(2, 2),
        ..Default::default()
    }
}
