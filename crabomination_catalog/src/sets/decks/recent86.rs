//! Chosen-type cost-reduction batch (`StaticEffect::ChosenTypeSpellCostReduction`)
//! + a discard-fuelled burn engine. Tests in `tests/recent86.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, SelectionRequirement as R, StaticAbility,
    StaticEffect,
};
use crate::effect::shortcut::{deal, etb, target};
use crate::effect::{Effect, Selector};
use crate::mana::{cost, generic, r};

/// Enchantment/artifact that names a creature type at ETB and makes creature
/// spells of that type cost `amount` less.
fn incubator(name: &'static str, mana: &[crate::mana::ManaSymbol], types: Vec<CardType>,
             amount: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(mana),
        card_types: types,
        triggered_abilities: vec![etb(Effect::NameCreatureType { what: Selector::This })],
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast of the chosen type cost less.",
            effect: StaticEffect::ChosenTypeSpellCostReduction { amount },
        }],
        ..Default::default()
    }
}

/// Urza's Incubator — {3} Artifact. Choose a creature type. Creature spells of
/// the chosen type cost {2} less to cast.
pub fn urzas_incubator() -> CardDefinition {
    incubator("Urza's Incubator", &[generic(3)], vec![CardType::Artifact], 2)
}

/// Herald's Horn — {3} Artifact. Choose a creature type. Creatures you control
/// of the chosen type cost {1} less to cast. (The upkeep "look at the top card,
/// reveal a chosen-type creature to hand" rider is approximated as dropped.)
pub fn heralds_horn() -> CardDefinition {
    incubator("Herald's Horn", &[generic(3)], vec![CardType::Artifact], 1)
}

/// Seismic Assault — {R}{R}{R} Enchantment. Discard a land card: Seismic Assault
/// deals 2 damage to any target.
pub fn seismic_assault() -> CardDefinition {
    CardDefinition {
        name: "Seismic Assault",
        cost: cost(&[r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Land, 1)),
            effect: deal(2, target()),
            ..Default::default()
        }],
        ..Default::default()
    }
}
