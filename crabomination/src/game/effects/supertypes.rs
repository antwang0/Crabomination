//! CR 205.4 — effects that add or remove a supertype for a duration (layer 4).

use crate::card::{CardId, Supertype};
use crate::effect::Duration;
use crate::game::GameState;
use crate::game::layers::{AffectedPermanents, ContinuousEffect, Layer, Modification};

impl GameState {
    /// `Effect::BecomeLegendary` — each of `ids` is legendary for `duration`
    /// (Cacophony Unleashed's "becomes a legendary 6/6 … until end of turn").
    pub(super) fn become_legendary(&mut self, ids: &[CardId], source: CardId, duration: Duration, controller: usize) {
        self.change_supertype(ids, source, Supertype::Legendary, true, duration, controller);
    }

    /// `Effect::ChangeSupertype` — each of `ids` gains or loses `supertype`
    /// for `duration` (Thermal Flux's snow toggle).
    pub(super) fn change_supertype(
        &mut self,
        ids: &[CardId],
        source: CardId,
        supertype: Supertype,
        gain: bool,
        duration: Duration,
        controller: usize,
    ) {
        let duration = self.effect_duration_for(duration, controller);
        let modification =
            if gain { Modification::AddSupertype(supertype) } else { Modification::RemoveSupertype(supertype) };
        for &cid in ids {
            let timestamp = self.next_timestamp();
            self.add_continuous_effect(ContinuousEffect {
                timestamp,
                source,
                affected: AffectedPermanents::just(cid),
                layer: Layer::L4Type,
                sublayer: None,
                duration: duration.clone(),
                modification: modification.clone(),
            });
        }
    }
}
