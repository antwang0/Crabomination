//! "You may cast it from your graveyard as an Adventure until the end of your
//! next turn" (Hildibrand Manderville, CR 715.3): a per-player grant, not a
//! `may_play_until` — that permission casts the whole card, and a creature
//! cast is exactly what this one withholds.

use super::GameState;
use crate::card::CardId;

impl GameState {
    /// Record the grant for `card`, which must be in `player`'s graveyard.
    pub(crate) fn grant_adventure_from_graveyard(&mut self, player: usize, card: CardId) {
        let turn = self.turn_number;
        let Some(pl) = self.players.get_mut(player) else { return };
        if pl.graveyard.iter().any(|c| c.id == card && c.definition.has_adventure().is_some()) {
            pl.adventure_graveyard_grants.retain(|&(c, _)| c != card);
            pl.adventure_graveyard_grants.push((card, turn));
        }
    }

    /// `player` may cast `card`'s Adventure from their graveyard now.
    pub(crate) fn adventure_grant_live(&self, player: usize, card: CardId) -> bool {
        self.players.get(player).is_some_and(|pl| {
            pl.adventure_graveyard_grants.iter().any(|&(c, _)| c == card)
                && pl.graveyard.iter().any(|c| c.id == card)
        })
    }

    /// Lapse the grants made before the turn that just ended, when it was
    /// their holder's.
    pub fn expire_adventure_grants(&mut self, ended_active: usize, ended_turn: u32) {
        if self
            .players
            .get(ended_active)
            .is_some_and(|pl| pl.adventure_graveyard_grants.iter().any(|&(_, on)| ended_turn > on))
        {
            self.players[ended_active].adventure_graveyard_grants.retain(|&(_, on)| ended_turn <= on);
        }
    }
}
