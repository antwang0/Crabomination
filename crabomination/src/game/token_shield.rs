//! The Master, Multiplied — "Triggered abilities you control can't cause you
//! to sacrifice or exile creature tokens you control"
//! (`StaticEffect::YourTriggersCantRemoveYourCreatureTokens`).
//!
//! The trigger resolution in `stack.rs` stamps `token_shield_seat` for a
//! controller who has the static; `can_be_sacrificed` and the exile hop in
//! `move_card_to` read it. The end-of-combat cleanup of attacking token
//! copies (myriad) is engine-driven, so it asks `shields_own_creature_token`
//! directly.

use super::GameState;
use crate::card::CardId;
use crate::effect::StaticEffect;

impl GameState {
    /// `seat` controls a permanent with the static.
    pub(crate) fn has_token_shield(&self, seat: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::YourTriggersCantRemoveYourCreatureTokens))
        })
    }

    /// `id` is a creature token whose controller has the static: one of
    /// their triggered abilities can't sacrifice or exile it.
    pub(crate) fn shields_own_creature_token(&self, id: CardId) -> bool {
        self.battlefield_find(id).is_some_and(|c| {
            c.is_token && self.computed_is_creature(c) && self.has_token_shield(c.controller)
        })
    }

    /// The triggered ability resolving now is its controller's and `id` is a
    /// creature token of theirs it can't sacrifice or exile.
    pub(crate) fn trigger_shields_token(&self, id: CardId) -> bool {
        let Some(seat) = self.scratch.token_shield_seat else { return false };
        self.battlefield_find(id).is_some_and(|c| c.controller == seat) && self.shields_own_creature_token(id)
    }
}
