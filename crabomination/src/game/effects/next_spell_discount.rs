//! `Effect::GrantNextMatchingSpellDiscountThisTurn` — "the next [filter]
//! spell you cast this turn costs {N} less to cast" (Invasion of the Giants
//! III). CR 601.2f: the reduction applies while the cost is determined; the
//! cast that took it consumes the grant, and a cast that doesn't match leaves
//! it waiting. Unspent grants lapse at end of turn (`stack.rs`).

use crate::card::{CardInstance, SelectionRequirement};
use crate::game::GameState;

impl GameState {
    pub(super) fn grant_next_matching_spell_discount(&mut self, seat: usize, filter: &SelectionRequirement, amount: u32) {
        if amount > 0 {
            self.players[seat].next_matching_spell_discounts.push((filter.clone(), amount));
        }
    }

    /// The generic mana `seat`'s waiting grants take off `card`.
    pub(crate) fn next_matching_spell_discount(&self, seat: usize, card: &CardInstance) -> u32 {
        self.players[seat]
            .next_matching_spell_discounts
            .iter()
            .filter(|(f, _)| self.evaluate_requirement_on_card(f, card, seat))
            .fold(0u32, |acc, (_, n)| acc.saturating_add(*n))
    }

    /// `card` was cast: every grant it matched is spent.
    pub(crate) fn consume_next_matching_spell_discounts(&mut self, seat: usize, card: &CardInstance) {
        if self.players[seat].next_matching_spell_discounts.is_empty() {
            return;
        }
        let keep: Vec<bool> = self.players[seat]
            .next_matching_spell_discounts
            .iter()
            .map(|(f, _)| !self.evaluate_requirement_on_card(f, card, seat))
            .collect();
        let mut keep = keep.into_iter();
        self.players[seat].next_matching_spell_discounts.retain(|_| keep.next().unwrap_or(true));
    }
}

impl GameState {
    /// The index of `seat`'s "next [filter] spell is free" grant that `card`
    /// spends, if any (Apex Observatory, World War Hulk).
    pub(crate) fn free_next_spell(&self, seat: usize, card: &CardInstance) -> Option<usize> {
        let grants = &self.players.get(seat)?.free_next_spells;
        if grants.is_empty() {
            return None;
        }
        grants.iter().position(|f| self.evaluate_requirement_on_card(f, card, seat))
    }
}
