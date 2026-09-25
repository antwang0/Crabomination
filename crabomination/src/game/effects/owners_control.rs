//! Brooding Saurian — "each player gains control of all nontoken permanents
//! they own".

use crate::game::GameState;

impl GameState {
    /// `Effect::OwnersGainControlOfNontokens`: every stolen nontoken permanent
    /// goes home, and a temporary steal's record goes with it so its end
    /// can't hand the permanent back to the thief.
    pub(super) fn owners_gain_control_of_nontokens(&mut self) {
        let stolen: Vec<(crate::card::CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| !c.is_token && c.controller != c.owner)
            .map(|c| (c.id, c.owner))
            .collect();
        for (id, owner) in stolen {
            self.temporary_control.retain(|t| t.card != id);
            self.change_control(id, owner);
        }
    }
}
