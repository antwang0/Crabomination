//! MKM (Murders at Karlov Manor) gap batch — an Azorius Detective.
//! Tests in `tests/recent_b/recent257.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, Supertype,
};
use crate::effect::shortcut::{etb, investigate};
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, generic, u, w, x};

/// Alquist Proft, Master Sleuth — {1}{W}{U} Legendary Creature — Human Detective.
/// Vigilance. ETB: investigate. {X}{W}{U}{U}, {T}, Sacrifice a Clue: draw X
/// cards and gain X life.
pub fn alquist_proft_master_sleuth() -> CardDefinition {
    CardDefinition {
        name: "Alquist Proft, Master Sleuth",
        cost: cost(&[generic(1), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(investigate(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), w(), u(), u()]),
            tap_cost: true,
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Clue), 1)),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::XFromCost,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::XFromCost,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
