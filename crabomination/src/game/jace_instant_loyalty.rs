//! Jace's Machinations — "Until end of turn, you may activate loyalty
//! abilities of Jace planeswalkers you control on any player's turn any time
//! you could cast an instant" (CR 606.3 override). Stored as the turn number
//! it was granted (`Player::jace_instant_loyalty_turn`), read by the loyalty
//! activation gate for a Jace the activator controls.

use super::GameState;
use crate::card::PlaneswalkerSubtype;

impl GameState {
    pub(crate) fn grant_jace_loyalty_at_instant_speed(&mut self, seat: usize) {
        let turn = self.turn_number;
        if let Some(p) = self.players.get_mut(seat) {
            p.jace_instant_loyalty_turn = Some(turn);
        }
    }

    /// Whether `seat` may activate the loyalty abilities of the planeswalker
    /// at battlefield index `pos` at instant speed through the grant.
    pub(crate) fn jace_loyalty_at_instant_speed(&self, seat: usize, pos: usize) -> bool {
        self.players.get(seat).is_some_and(|p| p.jace_instant_loyalty_turn == Some(self.turn_number))
            && self.battlefield.get(pos).is_some_and(|c| {
                self.computed_permanent(c.id).is_some_and(|cp| {
                    cp.subtypes().planeswalker_subtypes.contains(&PlaneswalkerSubtype::Jace)
                })
            })
    }
}
