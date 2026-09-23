//! CR 102.2 — "an opponent controls N or more [permanents]" is a question
//! about each opponent on their own, not about the table's total: at a
//! four-player table, three opponents with two nonbasic lands each don't make
//! "an opponent controls four or more nonbasic lands" true.

use super::GameState;
use crate::card::{CardId, SelectionRequirement};
use crate::game::types::Target;

impl GameState {
    /// Whether some opponent of `seat` still in the game controls at least `n`
    /// permanents matching `filter` (read from `seat`'s point of view, with
    /// `source` as the asking object).
    pub(crate) fn an_opponent_controls_at_least(
        &self,
        seat: usize,
        filter: &SelectionRequirement,
        n: u32,
        source: Option<CardId>,
    ) -> bool {
        (0..self.players.len())
            .filter(|&q| q != seat && !self.players[q].eliminated && !self.same_team(q, seat))
            .any(|q| {
                let count = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == q
                            && self.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                seat,
                                source,
                            )
                    })
                    .count();
                count >= n as usize
            })
    }
}
