//! "Whenever you create one or more tokens / surveil for the first time each
//! turn": each dispatch batch bumps its players' per-turn tallies before any
//! filter reads them, so `Predicate::FirstBatchThisTurn` holds exactly on the
//! turn's first batch — one made before the listener arrived included.

use super::types::GameEvent;
use super::{GameState, seat_bit};

impl GameState {
    pub(crate) fn tally_first_each_turn_batches(&mut self, events: &[GameEvent]) {
        let (mut tokens, mut surveils) = (0u64, 0u64);
        for e in events {
            match *e {
                GameEvent::TokenCreated { card_id } => {
                    if let Some(c) = self.battlefield_find(card_id) {
                        tokens |= seat_bit(c.controller);
                    }
                }
                GameEvent::ScriedOrSurveiled { player, surveil: true, .. } => surveils |= seat_bit(player),
                _ => {}
            }
        }
        if tokens | surveils == 0 {
            return;
        }
        for p in 0..self.players.len() {
            let b = seat_bit(p);
            if tokens & b != 0 {
                let pl = &mut self.players[p];
                pl.token_batches_this_turn = pl.token_batches_this_turn.saturating_add(1);
            }
            if surveils & b != 0 {
                let pl = &mut self.players[p];
                pl.surveil_batches_this_turn = pl.surveil_batches_this_turn.saturating_add(1);
            }
        }
    }
}
