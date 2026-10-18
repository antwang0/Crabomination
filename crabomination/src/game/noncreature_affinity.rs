//! "The next noncreature spell you cast this turn has affinity for artifacts"
//! (Don & Raph, Hard Science). Stamped with the caster's noncreature-spell
//! tally at grant (`Player::pending_affinity_next_noncreature`); it applies
//! while that tally is unchanged — to the next noncreature spell, creature
//! spells in between leaving it armed — and lapses once the tally ticks up
//! on cast. Counted as the spell is cast (CR 702.41a). Cleared at cleanup.

use super::GameState;
use crate::card::{CardInstance, CardType};

impl GameState {
    pub(crate) fn grant_next_noncreature_spell_affinity(&mut self, seat: usize) {
        if let Some(p) = self.players.get_mut(seat) {
            let at = p.noncreature_spells_cast_this_game_turn;
            p.pending_affinity_next_noncreature.push(at);
        }
    }

    /// The generic discount the grant gives `card`, cast by `caster`.
    pub(crate) fn noncreature_affinity_reduction(&self, caster: usize, card: &CardInstance) -> u32 {
        let pl = &self.players[caster];
        if pl.pending_affinity_next_noncreature.is_empty() || card.definition.card_types.contains(&CardType::Creature) {
            return 0;
        }
        if !pl.pending_affinity_next_noncreature.contains(&pl.noncreature_spells_cast_this_game_turn) {
            return 0;
        }
        self.battlefield
            .iter()
            .filter(|c| c.controller == caster && self.computed_has_card_type(c, CardType::Artifact))
            .count() as u32
    }
}
