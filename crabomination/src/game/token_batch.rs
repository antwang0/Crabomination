//! "Create N tokens" through the token multipliers (CR 111.1 / 614.16:
//! Doubling Season, Anointed Procession, Parallel Lives, Ojer Taq's tripler).
//! The per-token replacements (Divine Visitation, Jinnie Fay, the steals) and
//! the bookkeeping live in the mint funnel, `mint_token_with_counters`; the
//! multiplier lives here, sized once per batch by `scaled_token_count`. A site
//! that attacks, blocks or attaches with the new tokens iterates the ids.

use super::GameState;
use super::types::GameEvent;
use crate::card::{CardDefinition, CardId};

impl GameState {
    /// Mint `base` copies of `def` for `controller`, scaled by the
    /// controller's token multipliers, and return the new tokens' ids (a
    /// stolen or replaced mint still reports the id the funnel returned).
    pub(crate) fn mint_tokens_scaled(
        &mut self,
        def: std::sync::Arc<CardDefinition>,
        controller: usize,
        base: u32,
        tapped: bool,
        events: &mut Vec<GameEvent>,
    ) -> Vec<CardId> {
        let n = self.scaled_token_count(controller, base, def.is_creature());
        (0..n)
            .map(|_| self.mint_token_onto_battlefield(def.clone(), controller, tapped, events))
            .collect()
    }
}
