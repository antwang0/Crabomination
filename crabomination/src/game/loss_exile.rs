//! CR 614 / 104.3 — "If you would lose the game, instead exile [this] and
//! your life total becomes 1" (The Golden Throne). The sibling of Lich's
//! Mirror's `apply_loss_reset`: one application replaces every loss
//! state-based action that fired for the player at once.

use super::GameState;
use crate::effect::StaticEffect;

impl GameState {
    /// Exile the first permanent `p` controls carrying
    /// `ReplaceControllerLossWithExileSelf` and set `p`'s life to 1. Returns
    /// the exiled permanent when the replacement applied. An armed CR 104.3c deck-out is
    /// spent with it; poison and commander damage are not reset, so those
    /// losses come back at the next check with the shield gone.
    pub(crate) fn apply_loss_exile_self(&mut self, p: usize) -> Option<crate::card::CardId> {
        let id = self
            .battlefield
            .iter()
            .find(|c| {
                c.controller == p
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::ReplaceControllerLossWithExileSelf))
            })
            .map(|c| c.id)?;
        self.remove_from_battlefield_to_exile(id);
        self.players[p].pending_deck_loss = false;
        // CR 119.5 — "your life total becomes 1" is a gain of the difference,
        // through the CR 119 funnel.
        let delta = 1 - self.effective_life(p);
        self.adjust_life_applied(p, delta);
        Some(id)
    }
}
