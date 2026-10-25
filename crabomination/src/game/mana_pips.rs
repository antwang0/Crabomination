//! How many mana a "add X mana" payload makes, as a pip count the per-pip
//! loops can walk. Each pip is one `ManaAdded` event, so an X off a saturated
//! power (Selvala, Heart of the Wilds beside an `i32::MAX` creature) pushed
//! two billion events and took a four-seat pod to 13.6 GB before the OOM
//! kill (`random_pods` seed 10856, decks 434,1376,5670,5938, game 3).

use super::GameState;
use super::effects::EffectContext;
use crate::effect::Value;

impl GameState {
    /// `v` as a pip count, clamped to the saturation band
    /// (`player::SCALE_CEILING`): past it a mana pool is "more than enough",
    /// the same verdict the scale's consumers already read.
    pub(crate) fn mana_pip_count(&self, v: &Value, ctx: &EffectContext) -> u32 {
        self.evaluate_value(v, ctx).clamp(0, crate::player::SCALE_CEILING) as u32
    }
}
