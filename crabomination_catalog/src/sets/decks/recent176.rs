//! Aetherdrift (DFT) gap batch, continued. Cards unblocked by threading the
//! cast's X onto ETB *triggered* abilities (`CardInstance.cast_x_value`) and by
//! the multi-slot up-to-one graveyard return (`Effect::ReturnFilteredSlots`).
//! Tests in `crabomination/src/tests/recent176.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, Keyword, SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, w, x};

/// Dune Drifter — {X}{W}{B} Artifact — Vehicle 3/3, Crew 2. When it enters,
/// return target artifact or creature card with mana value X or less from your
/// graveyard to the battlefield.
pub fn dune_drifter() -> CardDefinition {
    CardDefinition {
        name: "Dune Drifter",
        cost: cost(&[x(), w(), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Artifact
                    .or(R::Creature)
                    .and(R::InYourGraveyard)
                    .and(R::ManaValueAtMostXFromCost),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..Default::default()
    }
}
