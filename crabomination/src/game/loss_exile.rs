//! CR 614 / 104.3 — "If you would lose the game, instead exile [this] and
//! your life total becomes 1" (The Golden Throne). The sibling of Lich's
//! Mirror's `apply_loss_reset`: one application replaces every loss
//! state-based action that fired for the player at once.

use super::GameState;
use crate::effect::StaticEffect;

impl GameState {
    /// Exile the first permanent `p` controls carrying
    /// `ReplaceControllerLossWithExileSelf` and set `p`'s life to 1. Returns
    /// true when the replacement applied. An armed CR 104.3c deck-out is
    /// spent with it; poison and commander damage are not reset, so those
    /// losses come back at the next check with the shield gone.
    pub(crate) fn apply_loss_exile_self(&mut self, p: usize) -> bool {
        let Some(id) = self
            .battlefield
            .iter()
            .find(|c| {
                c.controller == p
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::ReplaceControllerLossWithExileSelf))
            })
            .map(|c| c.id)
        else {
            return false;
        };
        self.remove_from_battlefield_to_exile(id);
        self.players[p].pending_deck_loss = false;
        self.players[p].life = 1;
        true
    }
}
