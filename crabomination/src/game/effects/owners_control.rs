//! "Each player gains control of all [permanents] they own" — Brooding
//! Saurian (nontoken permanents), Alicia Masters (creatures).

use crate::card::SelectionRequirement;
use crate::game::GameState;

impl GameState {
    /// `Effect::OwnersGainControlOfNontokens`.
    pub(super) fn owners_gain_control_of_nontokens(&mut self) {
        self.owners_gain_control_of(&SelectionRequirement::IsToken.negate());
    }

    /// `Effect::OwnersGainControlOf`: every stolen permanent matching `filter`
    /// goes home, and a temporary steal's record goes with it so its end
    /// can't hand the permanent back to the thief.
    pub(super) fn owners_gain_control_of(&mut self, filter: &SelectionRequirement) {
        let stolen: Vec<(crate::card::CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller != c.owner && self.evaluate_requirement_on_card(filter, c, c.owner))
            .map(|c| (c.id, c.owner))
            .collect();
        for (id, owner) in stolen {
            self.temporary_control.retain(|t| t.card != id);
            self.change_control(id, owner);
        }
    }
}
