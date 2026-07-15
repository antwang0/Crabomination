//! Foundations (FDN) gap — Vizier of the Menagerie. Tests in
//! `tests/recent222.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, SelectionRequirement as R, Subtypes};
use crate::effect::{StaticAbility, StaticEffect};
use crate::mana::{cost, g, generic};

/// Vizier of the Menagerie — {3}{G} 3/4 Snake Cleric. Look at / cast creature
/// spells from the top of your library. (The "spend mana of any type to cast
/// creature spells" clause is not modeled.)
pub fn vizier_of_the_menagerie() -> CardDefinition {
    CardDefinition {
        name: "Vizier of the Menagerie",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast creature spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Creature },
            },
        ],
        ..Default::default()
    }
}
