//! Shadowmoor / Eventide — Conspire (CR 702.78). As you cast a Conspire
//! spell you may tap two untapped creatures you control sharing a color with
//! it to copy it once (the copy may choose new targets). Cast via
//! `GameAction::CastSpellConspire`.

use crate::card::{CardDefinition, CardType, Keyword, SelectionRequirement};
use crate::effect::shortcut::{deal, pump_target, target, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, g, generic, hybrid, r, u, Color};

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

/// Memory Sluice — {U/B} Sorcery. "Target player mills four cards. Conspire."
pub fn memory_sluice() -> CardDefinition {
    CardDefinition {
        name: "Memory Sluice",
        cost: cost(&[hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Mill {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Gleeful Sabotage — {1}{G} Sorcery. "Destroy target artifact or enchantment.
/// Conspire."
pub fn gleeful_sabotage() -> CardDefinition {
    CardDefinition {
        name: "Gleeful Sabotage",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Artifact)
                    .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
            ),
        },
        ..Default::default()
    }
}

/// Ghastly Discovery — {2}{U} Sorcery. "Draw two cards, then discard a card.
/// Conspire."
pub fn ghastly_discovery() -> CardDefinition {
    CardDefinition {
        name: "Ghastly Discovery",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Disturbing Plot — {1}{B} Sorcery. "Return target creature card from a
/// graveyard to its owner's hand. Conspire."
pub fn disturbing_plot() -> CardDefinition {
    CardDefinition {
        name: "Disturbing Plot",
        cost: cost(&[generic(1), crate::mana::b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}
