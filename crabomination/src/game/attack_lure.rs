//! CR 508.1d — "during [player]'s next turn, creatures that player controls
//! attack [this permanent] if able" (Gideon Jura's +2). A per-seat lure, set by
//! `Effect::LureCreaturesToSourceNextTurn`, read by the attack declaration and
//! the bot's planner, and cleared as that player's next turn ends. The
//! one-creature form (Gideon, Battle-Forged's +2) rides the same clock.

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

    /// The permanent `creature` must attack this turn, if a one-creature
    /// lure (Gideon, Battle-Forged) binds it now — the same conditions as
    /// [`attack_lure_of`](Self::attack_lure_of), read per creature.
    pub(crate) fn creature_lure_of(&self, p: usize, creature: CardId) -> Option<CardId> {
        let pl = self.players.get(p)?;
        if pl.creature_attack_lures.is_empty() || self.active_player_idx != p {
            return None;
        }
        pl.creature_attack_lures
            .iter()
            .filter(|&&(c, _, set_on)| c == creature && self.turn_number > set_on)
            .find_map(|&(_, pw, _)| self.lure_walker_attackable(p, pw))
    }

    /// Whether any one-creature lure binds a creature of `p`'s this turn.
    pub(crate) fn any_creature_lure(&self, p: usize) -> bool {
        self.players.get(p).is_some_and(|pl| {
            pl.creature_attack_lures.iter().any(|&(c, _, _)| self.creature_lure_of(p, c).is_some())
        })
    }

    fn lure_walker_attackable(&self, p: usize, pw: CardId) -> Option<CardId> {
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
        if let Some(pl) = self.players.get_mut(ended_active) {
            pl.creature_attack_lures.retain(|&(_, _, set_on)| ended_turn <= set_on);
        }
    }
}
