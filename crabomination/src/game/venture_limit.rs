//! "Each opponent can't venture into the dungeon more than once each turn"
//! (Keen-Eared Sentry, `StaticEffect::OpponentsVentureOncePerTurn`). A
//! venture (CR 701.49) — including the one taking the initiative makes
//! (CR 725.2) — is refused for a player who already ventured this turn while
//! an opponent controls such a static. `Player.ventured_this_turn` is only
//! written while one is on the battlefield, so no other game pays for it.

use crate::effect::StaticEffect;
use crate::game::GameState;

impl GameState {
    /// True when an opponent of `seat` controls an active
    /// `OpponentsVentureOncePerTurn` static.
    fn venture_limited_by_an_opponent(&self, seat: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller != seat
                && (self.teams.is_empty() || self.team_of(c.controller) != self.team_of(seat))
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(self.active_static(&sa.effect, c), Some(StaticEffect::OpponentsVentureOncePerTurn))
                })
        })
    }

    /// Whether `seat` may venture now. A permitted venture under a limiter is
    /// recorded, so the next one this turn is refused.
    pub(crate) fn begin_venture(&mut self, seat: usize) -> bool {
        if !self.venture_limited_by_an_opponent(seat) {
            return true;
        }
        if self.players[seat].ventured_this_turn {
            return false;
        }
        self.players[seat].ventured_this_turn = true;
        true
    }
}
