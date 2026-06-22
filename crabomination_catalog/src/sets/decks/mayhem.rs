//! **Mayhem** cards (CR 702.187). A static ability in the graveyard: "As long
//! as you discarded this card this turn, you may cast it from your graveyard by
//! paying its mayhem cost rather than its mana cost." Wired through the
//! flashback machinery (`GameAction::CastMayhem` → `cast_flashback`), gated on
//! `Player.discarded_this_turn`; the spell is exiled if it would leave the
//! stack. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement, Subtypes, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::Duration;
use crate::mana::{b, cost, generic, r};

/// Electro's Bolt — {2}{R} Sorcery. Deal 4 damage to target creature.
/// Mayhem {1}{R}.
pub fn electros_bolt() -> CardDefinition {
    CardDefinition {
        name: "Electro's Bolt",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Mayhem(cost(&[generic(1), r()]))],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Sadistic Slash — {3}{B} Instant. Target creature gets -5/-5 until end of
/// turn. Mayhem {1}{B}.
pub fn sadistic_slash() -> CardDefinition {
    CardDefinition {
        name: "Sadistic Slash",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Mayhem(cost(&[generic(1), b()]))],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-5),
            toughness: Value::Const(-5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Raging Goblinoids — {4}{R} 5/4 Goblin Berserker with Haste. Mayhem {2}{R}.
pub fn raging_goblinoids() -> CardDefinition {
    CardDefinition {
        name: "Raging Goblinoids",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Haste, Keyword::Mayhem(cost(&[generic(2), r()]))],
        ..Default::default()
    }
}
