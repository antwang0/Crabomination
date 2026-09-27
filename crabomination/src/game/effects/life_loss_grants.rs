//! May-play grant bookkeeping: "during your turn, if an opponent lost life
//! this turn, you may play cards exiled with this" (Theater of Horrors), and
//! the one-cast groups of "cast a spell from among them".

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
                c.may_play_until = Some(MayPlayPermission { cast_only: false, locks_further_casts: false, player: active, ..perm });
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

impl GameState {
    /// "You may cast a spell from among them": a cast through one grant of
    /// `group` spends the others (`MayPlayPermission::one_cast_group`).
    pub(crate) fn clear_one_cast_group(&mut self, group: crate::card::CardId) {
        let hit = |c: &crate::card::CardInstance| {
            c.may_play_until.is_some_and(|m| m.one_cast_group == Some(group))
        };
        let clear = |c: &mut crate::card::CardInstance| {
            if hit(c) {
                c.may_play_until = None;
                c.granted_alt_cast_cost_eot = None;
            }
        };
        // Guarded walks: a zone's `iter_mut` unshares it (CoW).
        if self.exile.iter().any(hit) {
            self.exile.iter_mut().for_each(clear);
        }
        for seat in 0..self.players.len() {
            let pl = &self.players[seat];
            let (gy, hand, lib) = (pl.graveyard.iter().any(hit), pl.hand.iter().any(hit), pl.library.iter().any(hit));
            if !(gy || hand || lib) {
                continue;
            }
            let pl = &mut *self.players[seat];
            if gy {
                pl.graveyard.iter_mut().for_each(clear);
            }
            if hand {
                pl.hand.iter_mut().for_each(clear);
            }
            if lib {
                pl.library.iter_mut().for_each(clear);
            }
        }
    }
}
