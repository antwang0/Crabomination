//! Urza Assembles the Titans III — "You may activate the loyalty abilities of
//! planeswalkers you control twice this turn rather than only once" (CR 606.3
//! override). A player-level grant, stored as the turn it was made
//! (`Player::loyalty_twice_turn`), so a planeswalker that enters later in the
//! turn is covered too; read by `loyalty_twice_each_turn_for`.

use super::GameState;

impl GameState {
    pub(crate) fn grant_loyalty_twice_this_turn(&mut self, seat: usize) {
        let turn = self.turn_number;
        if let Some(p) = self.players.get_mut(seat) {
            p.loyalty_twice_turn = Some(turn);
        }
    }

    pub(crate) fn loyalty_twice_granted_this_turn(&self, seat: usize) -> bool {
        self.players.get(seat).is_some_and(|p| p.loyalty_twice_turn == Some(self.turn_number))
    }
}
