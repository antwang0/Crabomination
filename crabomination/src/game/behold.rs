//! CR 701.4 — behold: "choose a [quality] permanent you control or reveal a
//! [quality] card from your hand". Knowledge-only; nothing moves.

use super::GameState;
use crate::card::{CardId, SelectionRequirement};

impl GameState {
    /// Can `p` behold a `filter` object? `except` is the spell being cast,
    /// which can't reveal itself from hand.
    pub(crate) fn can_behold(&self, filter: &SelectionRequirement, p: usize, except: Option<CardId>) -> bool {
        self.battlefield.iter().any(|c| c.controller == p && self.evaluate_requirement_static_on(filter, c, p, None))
            || self.players.get(p).is_some_and(|pl| {
                pl.hand.iter().any(|c| Some(c.id) != except && self.evaluate_requirement_on_card(filter, c, p))
            })
    }
}
