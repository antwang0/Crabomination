//! OTJ gap batch — Doc Aurlock (graveyard/exile/plot cost reductions via the new
//! `StaticEffect::{ExileCastCostReduction, PlotCostReduction}`).
//! Tests in `recent_b/recent288`.

use crate::card::{
    CardDefinition, CardType, CreatureType, StaticAbility, Subtypes, Supertype,
};
use crate::effect::StaticEffect;
use crate::mana::{cost, g, u};

/// Doc Aurlock, Grizzled Genius — {G}{U} Legendary Creature — Bear Druid 2/3.
/// Spells you cast from your graveyard or from exile cost {2} less; plotting
/// cards from your hand costs {2} less.
pub fn doc_aurlock_grizzled_genius() -> CardDefinition {
    let reduce = |effect: StaticEffect, description: &'static str| StaticAbility {
        description,
        effect,
    };
    CardDefinition {
        name: "Doc Aurlock, Grizzled Genius",
        cost: cost(&[g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![
            reduce(
                StaticEffect::GraveyardCastCostReduction { amount: 2 },
                "Spells you cast from your graveyard cost {2} less to cast.",
            ),
            reduce(
                StaticEffect::ExileCastCostReduction { amount: 2 },
                "Spells you cast from exile cost {2} less to cast.",
            ),
            reduce(
                StaticEffect::PlotCostReduction { amount: 2 },
                "Plotting cards from your hand costs {2} less.",
            ),
        ],
        ..Default::default()
    }
}
