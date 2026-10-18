//! CR 400.7 / 611.2c — a continuous effect a permanent applied to itself
//! ("this Vehicle becomes an artifact creature until end of turn", a self-
//! pump) affects that object only. Once it leaves the battlefield, the card
//! that comes back is a new object, and the effect doesn't follow it. Without
//! this sweep, Conqueror's Galleon returned as a land that was still a 0/0
//! creature from its crew and died to SBA.

use super::GameState;
use crate::card::CardId;
use crate::game::layers::AffectedPermanents;

impl GameState {
    /// Drop every self-scoped (`AffectedPermanents::Source`) effect sourced
    /// from `id`. Called as `id` leaves the battlefield.
    pub(crate) fn drop_self_scoped_effects(&mut self, id: CardId) {
        let mine = |e: &crate::game::layers::ContinuousEffect| {
            e.source == id && matches!(e.affected, AffectedPermanents::Source)
        };
        if self.continuous_effects.iter().any(mine) {
            self.continuous_effects.retain(|e| !mine(e));
        }
    }
}
