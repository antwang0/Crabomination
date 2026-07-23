//! Dissension (DIS) gap wave 6. Tests in `classic_sets/dis`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, u, Color};

/// Momir Vig, Simic Visionary — {3}{G}{U} 2/2 Elf Wizard. Casting a green
/// creature spell tutors a creature to the top of your library; casting a blue
/// creature spell reveals the top card and takes it if it's a creature.
pub fn momir_vig_simic_visionary() -> CardDefinition {
    CardDefinition {
        name: "Momir Vig, Simic Visionary",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(R::Creature.and(R::HasColor(Color::Green))),
                ),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(R::Creature.and(R::HasColor(Color::Blue))),
                ),
                effect: Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    filter: R::Creature,
                },
            },
        ],
        ..Default::default()
    }
}

/// Sphinx of the Chimes — {4}{U}{U} 5/6 Sphinx. Flying; discard two nonland
/// cards with the same name to draw four cards.
pub fn sphinx_of_the_chimes() -> CardDefinition {
    CardDefinition {
        name: "Sphinx of the Chimes",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Nonland, 2)),
            discard_cost_same_name: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(4) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elemental Resonance — {2}{G}{G} Aura. Enchant permanent; at the beginning of
/// your first main phase, add mana equal to the enchanted permanent's cost.
pub fn elemental_resonance() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Elemental Resonance",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Permanent },
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::AddManaEqualToPermanentCost {
                permanent: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Vigean Intuition — {3}{G}{U} Instant. Choose a card type, then reveal the
/// top four cards of your library; put those of the chosen type into your hand
/// and the rest into your graveyard.
pub fn vigean_intuition() -> CardDefinition {
    CardDefinition {
        name: "Vigean Intuition",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseTypeRevealTopPartition { count: Value::Const(4) },
        ..Default::default()
    }
}

/// Fertile Imagination — {2}{G}{G} Sorcery. Choose a card type; target opponent
/// reveals their hand; create two 1/1 green Saproling tokens for each card of
/// the chosen type revealed this way.
pub fn fertile_imagination() -> CardDefinition {
    CardDefinition {
        name: "Fertile Imagination",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::FertileImagination { per: Value::Const(2) },
        ..Default::default()
    }
}
