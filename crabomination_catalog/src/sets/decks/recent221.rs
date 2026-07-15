//! FDN + DFT (Aetherdrift) gap batch — a chosen-color lifegain artifact and the
//! Aetherdrift vanilla "Tyrant" legends. Tests in `tests/recent221.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, Subtypes, Supertype, TriggeredAbility};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, EventKind, EventScope, EventSpec, Predicate, Selector, Value};
use crate::mana::{cost, b, g, generic, r, w};

/// Diamond Mare — {2} Artifact Creature — Horse 1/3. As it enters, choose a
/// color; whenever you cast a spell of the chosen color, gain 1 life.
pub fn diamond_mare() -> CardDefinition {
    CardDefinition {
        name: "Diamond Mare",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horse], ..Default::default() },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::ChooseColorForSelf),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellSharesChosenColorOfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

fn tyrant(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

/// Kalakscion, Hunger Tyrant — {1}{B}{B} Legendary Crocodile 7/2 (vanilla).
pub fn kalakscion_hunger_tyrant() -> CardDefinition {
    tyrant("Kalakscion, Hunger Tyrant", cost(&[generic(1), b(), b()]), vec![CreatureType::Crocodile], 7, 2)
}

/// Tyrox, Saurid Tyrant — {1}{R} Legendary Dinosaur Warrior 4/1 (vanilla).
pub fn tyrox_saurid_tyrant() -> CardDefinition {
    tyrant("Tyrox, Saurid Tyrant", cost(&[generic(1), r()]), vec![CreatureType::Dinosaur, CreatureType::Warrior], 4, 1)
}

/// Terrian, World Tyrant — {2}{G}{G}{G} Legendary Dinosaur Ooze 9/7 (vanilla).
pub fn terrian_world_tyrant() -> CardDefinition {
    tyrant("Terrian, World Tyrant", cost(&[generic(2), g(), g(), g()]), vec![CreatureType::Dinosaur, CreatureType::Ooze], 9, 7)
}

/// Sundial, Dawn Tyrant — {1}{W} Legendary Artifact Creature — Construct 3/3
/// (vanilla).
pub fn sundial_dawn_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Sundial, Dawn Tyrant",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}
