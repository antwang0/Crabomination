//! May-play grant bookkeeping: "during your turn, if an opponent lost life
//! this turn, you may play cards exiled with this" (Theater of Horrors), the
//! one-cast groups of "cast a spell from among them", and the grants that
//! last while their source is on the battlefield (Hedonist's Trove).

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
                && matches!(perm.duration, MayPlayDuration::HolderTurnsAfterOpponentLostLife { holder, .. } if holder == active)
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

    /// `holder` sacrificed a nontoken permanent on their turn: wake their
    /// dormant `HolderTurnsAfterNontokenSacrifice` grants (Evendo).
    pub(crate) fn wake_nontoken_sacrifice_grants(&mut self, holder: usize) {
        let dormant = |perm: &MayPlayPermission| {
            perm.player == MAY_PLAY_DORMANT
                && matches!(perm.duration, MayPlayDuration::HolderTurnsAfterNontokenSacrifice { holder: h, .. } if h == holder)
        };
        if !self.exile.iter().any(|c| c.cold_any(|k| k.may_play_until.as_ref().is_some_and(dormant))) {
            return;
        }
        for c in self.exile.iter_mut() {
            if let Some(perm) = c.may_play_until
                && dormant(&perm)
            {
                c.may_play_until = Some(MayPlayPermission { player: holder, ..perm });
            }
        }
    }

    /// The seat a fresh `HolderTurnsAfterNontokenSacrifice` grant starts
    /// under: awake on the holder's turn once they've sacrificed a nontoken
    /// permanent, else dormant.
    pub(crate) fn nontoken_sacrifice_grant_seat(&self, holder: usize) -> usize {
        let live = self.active_player_idx == holder
            && self.players.get(holder).is_some_and(|p| p.nontoken_sacrificed_this_turn > 0);
        if live { holder } else { MAY_PLAY_DORMANT }
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

impl GameState {
    /// Is `perm` a `WhileSourceOnBattlefield` grant of `source`'s?
    fn source_bound(perm: &MayPlayPermission, source: crate::card::CardId) -> bool {
        matches!(
            perm.duration,
            MayPlayDuration::WhileSourceOnBattlefield { source: s, .. }
                | MayPlayDuration::HolderTurnsAfterNontokenSacrifice { source: s, .. }
                | MayPlayDuration::HolderTurnsAfterOpponentLostLife { source: s, .. }
                | MayPlayDuration::HolderTurnsWithFlashWhileSource { source: s, .. } if s == source
        )
    }

    /// `source` left the battlefield: its `WhileSourceOnBattlefield` grants
    /// end with it (CR 611.3a — a static ability's effect stops applying when
    /// its source leaves). The grants live on exiled cards only.
    pub(crate) fn end_source_bound_grants(&mut self, source: crate::card::CardId) {
        let hit = |c: &crate::card::CardInstance| {
            c.cold_any(|k| k.may_play_until.as_ref().is_some_and(|m| Self::source_bound(m, source)))
        };
        // Guarded walk: `iter_mut` unshares the zone (CoW).
        if !self.exile.iter().any(hit) {
            return;
        }
        for c in self.exile.iter_mut() {
            if hit(c) {
                c.may_play_until = None;
                c.granted_alt_cast_cost_eot = None;
            }
        }
    }

    /// "You can't cast more than one spell this way each turn": a spell cast
    /// through one of `source`'s capped grants parks its other spell grants
    /// until the turn ends. Land grants stay live — the cap is on spells.
    pub(crate) fn park_source_bound_spell_grants(&mut self, source: crate::card::CardId) {
        let hit = |c: &crate::card::CardInstance| {
            !c.definition.is_land()
                && c.cold_any(|k| {
                    k.may_play_until.as_ref().is_some_and(|m| {
                        m.player != MAY_PLAY_DORMANT
                            && matches!(
                                m.duration,
                                MayPlayDuration::WhileSourceOnBattlefield { source: s, one_spell_a_turn: true } if s == source
                            )
                    })
                })
        };
        if !self.exile.iter().any(hit) {
            return;
        }
        for c in self.exile.iter_mut() {
            if hit(c)
                && let Some(perm) = c.may_play_until
            {
                c.may_play_until = Some(MayPlayPermission { player: MAY_PLAY_DORMANT, ..perm });
            }
        }
    }

    /// At each turn's end, wake the parked `WhileSourceOnBattlefield` grants
    /// for their source's current controller ("you" of its static ability),
    /// or drop them if the source is gone.
    pub(crate) fn rearm_source_bound_grants(&mut self) {
        let parked = |c: &crate::card::CardInstance| {
            c.cold_any(|k| {
                k.may_play_until.as_ref().is_some_and(|m| {
                    m.player == MAY_PLAY_DORMANT && matches!(m.duration, MayPlayDuration::WhileSourceOnBattlefield { .. })
                })
            })
        };
        if !self.exile.iter().any(parked) {
            return;
        }
        let ids: Vec<_> = self.exile.iter().filter(|c| parked(c)).map(|c| c.id).collect();
        for id in ids {
            let Some(perm) = self.exile.iter().find(|c| c.id == id).and_then(|c| c.may_play_until) else { continue };
            let MayPlayDuration::WhileSourceOnBattlefield { source, .. } = perm.duration else { continue };
            let holder = self.battlefield_find(source).map(|s| s.controller);
            if let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
                match holder {
                    Some(player) => c.may_play_until = Some(MayPlayPermission { player, ..perm }),
                    None => {
                        c.may_play_until = None;
                        c.granted_alt_cast_cost_eot = None;
                    }
                }
            }
        }
    }
}
