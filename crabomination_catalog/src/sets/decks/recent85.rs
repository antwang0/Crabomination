//! Chosen-type keyword-grant batch (`StaticEffect::GrantKeywordToChosenType`):
//! Steely Resolve (shroud), Kindred Boon (indestructible), Cover of Darkness
//! (fear); plus Elvish Clancaller (fixed-type Elf lord + name-tutor). Tests in
//! `tests/recent85.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{cost, g, generic, w, b};

/// Enchantment that names a creature type at ETB and grants it `keyword`.
fn chosen_type_keyword(name: &'static str, mana: &[crate::mana::ManaSymbol],
                       keyword: Keyword) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(mana),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::NameCreatureType { what: Selector::This })],
        static_abilities: vec![StaticAbility {
            description: "Creatures of the chosen type have the granted keyword.",
            effect: StaticEffect::GrantKeywordToChosenType { keyword, opponents: false },
        }],
        ..Default::default()
    }
}

/// Steely Resolve — {2}{G} Enchantment. Choose a creature type. Creatures of the
/// chosen type have shroud.
pub fn steely_resolve() -> CardDefinition {
    chosen_type_keyword("Steely Resolve", &[generic(2), g()], Keyword::Shroud)
}

/// Kindred Boon — {2}{W} Enchantment. Choose a creature type. Creatures you
/// control of the chosen type have indestructible.
pub fn kindred_boon() -> CardDefinition {
    chosen_type_keyword("Kindred Boon", &[generic(2), w()], Keyword::Indestructible)
}

/// Cover of Darkness — {1}{B} Enchantment. Choose a creature type. Creatures of
/// the chosen type have fear.
pub fn cover_of_darkness() -> CardDefinition {
    chosen_type_keyword("Cover of Darkness", &[generic(1), b()], Keyword::Fear)
}

/// Elvish Clancaller — {G}{G} 1/1 Elf. Other Elves you control get +1/+1.
/// {3}{G}{G}, {T}: Search your library for a card named Elvish Clancaller,
/// reveal it, put it into your hand, then shuffle.
pub fn elvish_clancaller() -> CardDefinition {
    let others = || Selector::EachPermanent(
        R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou).and(R::OtherThanSource),
    );
    CardDefinition {
        name: "Elvish Clancaller",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Other Elves you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: others(), power: 1, toughness: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), g()]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasName("Elvish Clancaller".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
