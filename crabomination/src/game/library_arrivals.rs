//! "Whenever one or more cards are put into a library from anywhere" (Wan Shi
//! Tong, All-Knowing). Library writes are ~120 sites, most of them reorders, so
//! the piles keep the tally themselves (`CardPile::admit`: a card arrives when
//! another zone held it last) and the dispatch turns it into one
//! `GameEvent::CardsPutIntoLibrary` a batch.

use super::GameState;
use crate::effect::{EventKind, KeywordAct};

impl GameState {
    /// Drain every library's arrival tally: `(first receiving seat, total)`,
    /// or `None` when nothing arrived or no permanent listens — a tally with
    /// no listener is dropped, so the event never reaches a non-listening
    /// game's batch.
    pub(crate) fn take_library_arrivals(&mut self) -> Option<(usize, u32)> {
        let mut first = None;
        let mut total = 0u32;
        for p in 0..self.players.len() {
            if self.players[p].library.arrivals() != 0 {
                let n = self.players[p].library.take_arrivals();
                first.get_or_insert(p);
                total += u32::from(n);
            }
        }
        let first = first?;
        let listens = self.battlefield.iter().any(|c| {
            c.definition
                .triggered_abilities
                .iter()
                .any(|t| t.event.kind == EventKind::Performed(KeywordAct::CardsPutIntoLibrary))
        });
        listens.then_some((first, total))
    }
}
