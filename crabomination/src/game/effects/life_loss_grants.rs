//! "During your turn, if an opponent lost life this turn, you may play cards
//! exiled with this" (Theater of Horrors): the dormant may-play grant wakes.

use crate::card::{MAY_PLAY_DORMANT, MayPlayDuration, MayPlayPermission};
use crate::game::GameState;

impl GameState {
    /// Called as `loser` loses life: when that is an opponent of the active
    /// player, wake the active player's dormant
    /// `HolderTurnsAfterOpponentLostLife` grants.
    pub(crate) fn wake_opponent_life_loss_grants(&mut self, loser: usize) {
        let active = self.active_player_idx;
        if active >= self.players.len() || self.same_team(loser, active) {
            return;
        }
        let dormant = |perm: &MayPlayPermission| {
            perm.player == MAY_PLAY_DORMANT
                && perm.duration == MayPlayDuration::HolderTurnsAfterOpponentLostLife { holder: active }
        };
        if !self.exile.iter().any(|c| c.cold_any(|k| k.may_play_until.as_ref().is_some_and(dormant))) {
            return;
        }
        for c in self.exile.iter_mut() {
            if let Some(perm) = c.may_play_until
                && dormant(&perm)
            {
                c.may_play_until = Some(MayPlayPermission { player: active, ..perm });
            }
        }
    }

    /// The seat a fresh `HolderTurnsAfterOpponentLostLife` grant starts
    /// under: awake when it is already the holder's turn and an opponent has
    /// lost life, else dormant.
    pub(crate) fn opponent_life_loss_grant_seat(&self, holder: usize) -> usize {
        let live = self.active_player_idx == holder
            && self.opponents_of(holder).iter().any(|&o| self.players[o].life_lost_this_turn > 0);
        if live { holder } else { MAY_PLAY_DORMANT }
    }
}
