//! CR 613.4b — a base P/T set to a count that keeps counting (Svogthos, the
//! Restless Tomb). The stored effect carries the `Value`s; each gather swaps
//! in the numbers they read now, so the layer pass sees a plain 7b set at the
//! effect's own timestamp.

use super::GameState;
use crate::game::effects::EffectContext;
use crate::game::layers::{AffectedPermanents, ContinuousEffect, Modification};

impl GameState {
    pub(crate) fn settle_live_pt(&self, effects: &mut [ContinuousEffect]) {
        for e in effects.iter_mut() {
            let Modification::SetPowerToughnessLive(pt) = &e.modification else { continue };
            let controller = match &e.affected {
                AffectedPermanents::Specific(ids) => ids.first().and_then(|&id| self.battlefield_find(id)),
                _ => None,
            }
            .map_or(0, |c| c.controller);
            let mut ctx = EffectContext::for_spell(controller, None, 0, 0);
            ctx.source = Some(e.source);
            let (p, t) = (self.evaluate_value(&pt.0, &ctx), self.evaluate_value(&pt.1, &ctx));
            e.modification = Modification::SetPowerToughness(p, t);
        }
    }
}
