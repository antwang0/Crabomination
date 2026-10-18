//! Commander: 2026-10-18 (`019Y6J4R`) closers, twentieth batch. Tests in
//! `tests/recent_b/cmdr_019y_t.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, SelectionRequirement as R, StaticAbility, Subtypes};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, Effect, PlayerRef, StaticEffect, Value};
use crate::mana::{b, cost};

/// Dawnhand Dissident — {B} 1/2 Elf Warlock. {T}, Blight 1: surveil 1.
/// {T}, Blight 2: exile target card from a graveyard (linked to this). During
/// your turn, you may cast creature spells from among cards you own exiled
/// with it by removing three counters from among creatures you control in
/// addition to paying their other costs.
pub fn dawnhand_dissident() -> CardDefinition {
    CardDefinition {
        name: "Dawnhand Dissident",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warlock], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                blight_cost: 1,
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                blight_cost: 2,
                effect: Effect::ExileWithSource { what: target_filtered(R::InGraveyard) },
                ..Default::default()
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "During your turn, you may cast creature spells from among cards you own exiled with this creature by removing three counters from among creatures you control in addition to paying their other costs.",
            effect: StaticEffect::CastLinkedExiledCreaturesForCounters { counters: 3 },
        }],
        ..Default::default()
    }
}
