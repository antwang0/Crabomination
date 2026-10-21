//! `Effect::SetArtifactSubtypes` — "it isn't an Equipment" (Dancing Sword):
//! an indefinite layer-4 rewrite of each permanent's artifact subtypes
//! (CR 205.1b, 613.1d).

use crate::card::{ArtifactSubtype, CardId};
use crate::effect::Selector;
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::layers::{AffectedPermanents, ContinuousEffect, EffectDuration, Layer, Modification};

impl GameState {
    pub(super) fn set_artifact_subtypes(&mut self, what: &Selector, subtypes: &[ArtifactSubtype], ctx: &EffectContext) {
        let source = ctx.source.unwrap_or(CardId(0));
        for ent in self.resolve_selector(what, ctx) {
            let Some(cid) = ent.as_permanent_id() else { continue };
            let timestamp = self.next_timestamp();
            self.add_continuous_effect(ContinuousEffect {
                timestamp,
                source,
                affected: AffectedPermanents::just(cid),
                layer: Layer::L4Type,
                sublayer: None,
                duration: EffectDuration::Indefinite,
                modification: Modification::SetArtifactSubtypes(subtypes.to_vec()),
            });
        }
    }
}
