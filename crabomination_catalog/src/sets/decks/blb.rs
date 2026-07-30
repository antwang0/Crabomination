//! Bloomburrow (BLB) gap batch. Tests in `tests/recent_b/blb.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, EnchantmentSubtype, EquipBonus,
    Keyword, SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, generic, u};

/// Sugar Coat — {2}{U} Aura. Flash. Enchant creature or Food. The enchanted
/// permanent becomes a colorless Food artifact with "{2}, {T}, Sacrifice this
/// artifact: You gain 3 life" and loses all other card types and abilities.
pub fn sugar_coat() -> CardDefinition {
    CardDefinition {
        name: "Sugar Coat",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Food))),
        },
        equipped_bonus: Some(EquipBonus {
            set_card_types: Some(vec![CardType::Artifact]),
            set_artifact_types: Some(vec![ArtifactSubtype::Food]),
            set_colors: Some(vec![]),
            remove_abilities: true,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
