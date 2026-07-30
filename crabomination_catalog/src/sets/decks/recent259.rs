//! MKM (Murders at Karlov Manor) gap batch — the empty-library Elemental.
//! Tests in `tests/recent_b/recent259.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, Keyword, StaticAbility, Subtypes};
use crate::effect::{PlayerRef, Predicate, StaticEffect, Value};
use crate::mana::{cost, generic, u};

/// Living Conundrum — {4}{U} Creature — Elemental 2/5. Hexproof. As long as your
/// library is empty, it's a 10/10 with flying and vigilance. (The "skip a draw
/// from an empty library" rider is inert here — deck-out isn't a loss.)
pub fn living_conundrum() -> CardDefinition {
    let library_empty =
        || Predicate::ValueAtMost(Value::LibrarySizeOf(PlayerRef::You), Value::ZERO);
    CardDefinition {
        name: "Living Conundrum",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Hexproof],
        static_abilities: vec![
            StaticAbility {
                description: "As long as there are no cards in your library, this creature \
                              has base power and toughness 10/10.",
                effect: StaticEffect::SetBasePtIf {
                    condition: library_empty(),
                    power: 10,
                    toughness: 10,
                },
            },
            StaticAbility {
                description: "As long as there are no cards in your library, this creature \
                              has flying.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Flying,
                    condition: library_empty(),
                },
            },
            StaticAbility {
                description: "As long as there are no cards in your library, this creature \
                              has vigilance.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Vigilance,
                    condition: library_empty(),
                },
            },
        ],
        ..Default::default()
    }
}
