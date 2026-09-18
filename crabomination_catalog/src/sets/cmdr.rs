//! Commander-format cards — the ones whose text is *about* the format
//! (CR 903): commanders that are not legendary creatures, cards that read the
//! command zone, cards that read your commander.
//!
//! Their own file rather than a set file, because what they share is the rule
//! they exercise, not the set they were printed in.

use crate::card::{
    CardDefinition, CardType, CreatureType, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef};
use crate::mana::{Color, cost, g, generic};

/// Freyalise, Llanowar's Fury — {3}{G}{G} Legendary Planeswalker, loyalty 3.
/// "+2: Create a 1/1 green Elf Druid creature token with '{T}: Add {G}.'
/// −2: Destroy target artifact or enchantment. −6: Draw a card for each green
/// creature you control. Freyalise, Llanowar's Fury can be your commander."
///
/// CR 903.3a — the `can_be_commander` half is what lets a non-creature lead a
/// deck; `format::validate_commander_deck` reads it.
pub fn freyalise_llanowars_fury() -> CardDefinition {
    let elf_druid = TokenDefinition {
        name: "Elf Druid".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        activated_abilities: vec![super::tap_add(Color::Green)],
        ..Default::default()
    };
    CardDefinition {
        name: "Freyalise, Llanowar's Fury",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Freyalise],
            ..Default::default()
        },
        base_loyalty: 3,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(elf_druid),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::Enchantment)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::count(Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::HasColor(Color::Green)),
                    )),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CR 903.3a — the flag is what makes a non-creature eligible.
    #[test]
    fn freyalise_prints_can_be_your_commander() {
        let def = freyalise_llanowars_fury();
        assert!(def.can_be_commander);
        assert!(def.is_legendary() && !def.is_creature());
        assert_eq!(def.base_loyalty, 3);
        assert_eq!(def.loyalty_abilities.len(), 3);
    }
}
