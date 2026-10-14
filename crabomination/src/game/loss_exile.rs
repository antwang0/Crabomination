//! CR 614 / 104.3 — "If you would lose the game, instead exile [this] and
//! your life total becomes 1" (The Golden Throne). The sibling of Lich's
//! Mirror's `apply_loss_reset`: one application replaces every loss
//! state-based action that fired for the player at once. Also Stunning
//! Reversal's one-shot, turn-scoped version of the same replacement.

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

    /// CR 614 / 104.3 — Stunning Reversal: "the next time you would lose the
    /// game this turn, instead draw seven cards and your life total becomes
    /// 1". Spends one of `p`'s shields armed this turn; false when none is.
    /// Like the Golden Throne, poison and commander damage are not reset.
    pub(crate) fn apply_loss_reversal(&mut self, p: usize, events: &mut Vec<crate::game::GameEvent>) -> bool {
        let turn = self.turn_number;
        let Some(i) = self.loss_reversals_this_turn.iter().position(|&(q, t)| q == p && t == turn) else {
            return false;
        };
        self.loss_reversals_this_turn.remove(i);
        // CR 704.7 — spends an armed deck-out; the seven draws may arm a new one.
        self.players[p].pending_deck_loss = false;
        self.draw_n(p, 7, events);
        let delta = 1 - self.effective_life(p);
        self.adjust_life_emit(p, delta, events);
        true
    }
}
