//! An eleventh wave — spell-form bending/blight cards (the effects fire from
//! instants/sorceries, not just creature triggers). Tests in
//! `crabomination/src/tests/recent11.rs`.

use crate::card::{
    CardDefinition, CardType, SelectionRequirement, Selector, SpellSubtype, Subtypes, Value,
};
use crate::effect::{Effect, PlayerRef};
use crate::mana::{b, cost, g, generic};

/// Earthbending Lesson — {3}{G} Sorcery — Lesson. Earthbend 4.
pub fn earthbending_lesson() -> CardDefinition {
    CardDefinition {
        name: "Earthbending Lesson",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::Earthbend { n: Value::Const(4) },
        ..Default::default()
    }
}

/// Dai Li Indoctrination — {1}{B} Sorcery — Lesson. Choose one — target
/// opponent reveals their hand and you make them discard a chosen nonland
/// card; or earthbend 2.
pub fn dai_li_indoctrination() -> CardDefinition {
    CardDefinition {
        name: "Dai Li Indoctrination",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::ChooseMode(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: SelectionRequirement::Nonland,
            },
            Effect::Earthbend { n: Value::Const(2) },
        ]),
        ..Default::default()
    }
}
