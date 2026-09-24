//! CR 508.1d — "during [player]'s next turn, creatures that player controls
//! attack [this permanent] if able" (Gideon Jura's +2). A per-seat lure, set by
//! `Effect::LureCreaturesToSourceNextTurn`, read by the attack declaration and
//! the bot's planner, and cleared as that player's next turn ends.

use super::GameState;
use crate::card::{CardId, CardType};

impl GameState {
    /// The permanent `p`'s creatures must attack this turn, if a lure binds
    /// them now: it's `p`'s first turn after the lure was set, and the lured
    /// permanent is still a planeswalker an opponent of `p` controls. A
    /// requirement that can't be met isn't one, so otherwise `None`.
    pub(crate) fn attack_lure_of(&self, p: usize) -> Option<CardId> {
        let (pw, set_on) = self.players.get(p)?.attack_lure?;
        if self.active_player_idx != p || self.turn_number <= set_on {
            return None;
        }
        let c = self.battlefield_find(pw)?;
        let walker = self
            .computed_permanent(pw)
            .map_or(c.definition.is_planeswalker(), |cp| cp.card_types().contains(&CardType::Planeswalker));
        (walker && c.controller != p && !self.same_team(p, c.controller)).then_some(pw)
    }

    /// End a lure once its turn is over: the turn that just ended was the
    /// lured player's, and later than the one it was set on.
    pub(crate) fn expire_attack_lure(&mut self, ended_active: usize, ended_turn: u32) {
        if self.players.get(ended_active).and_then(|pl| pl.attack_lure).is_some_and(|(_, set_on)| ended_turn > set_on) {
            self.players[ended_active].attack_lure = None;
        }
    }
}
