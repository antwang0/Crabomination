//! Dragon's Maze (DGM) gap cards, wave 4 — the remaining primitive-blocked
//! cards. Tests in `classic_sets/dgm`.

use crate::card::{CardDefinition, CardType, CreatureType, Keyword, StaticAbility, Subtypes};
use crate::effect::StaticEffect;
use crate::mana::{b, cost, generic, u};

/// Notion Thief — {2}{U}{B} 3/1 Human Rogue. Flash. If an opponent would draw a
/// card except the first one they draw in each of their draw steps, instead that
/// player skips that draw and you draw a card.
pub fn notion_thief() -> CardDefinition {
    CardDefinition {
        name: "Notion Thief",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would draw a card except the first each draw step, you draw instead.",
            effect: StaticEffect::OpponentExtraDrawsRedirected,
        }],
        ..Default::default()
    }
}
