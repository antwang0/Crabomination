//! "Exile it and the top N cards of your library in a face-down pile, shuffle
//! that pile, and put it back on top of your library" (Triumph of Saint
//! Katherine's Praesidium Protectiva): the card lands at a random depth among
//! the top N + 1.

use super::{EffectContext, EntityRef};
use crate::effect::{Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::ShuffleIntoTopPile` — each card `what` names that is still in
    /// its owner's graveyard joins the top `depth` cards of that library, and
    /// the pile is shuffled back on top. A card that has left the graveyard
    /// (CR 400.7) does nothing ("if you do").
    pub(super) fn shuffle_into_top_pile(
        &mut self,
        what: &Selector,
        depth: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let depth = self.evaluate_value(depth, ctx).max(0) as usize;
        for e in self.resolve_selector(what, ctx) {
            let (EntityRef::Card(id) | EntityRef::Permanent(id)) = e else { continue };
            let Some(owner) = (0..self.players.len()).find(|&p| self.players[p].graveyard.iter().any(|c| c.id == id)) else {
                continue;
            };
            let Some(pos) = self.players[owner].graveyard.iter().position(|c| c.id == id) else { continue };
            let card = self.players[owner].graveyard.remove(pos);
            // "Exile it, then shuffle it into the top N" — through exile.
            self.note_exiled_from_graveyard(owner, id, events);
            let n = depth.min(self.players[owner].library.len());
            let mut pile: Vec<_> = self.players[owner].library.drain(..n).collect();
            pile.push(card);
            pile.shuffle(&mut self.rng.draw());
            for c in pile.into_iter().rev() {
                self.players[owner].library.insert(0, c);
            }
        }
        Ok(())
    }
}
