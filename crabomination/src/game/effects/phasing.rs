//! CR 702.26 — permanents phased out and held by a source: they phase in
//! when it leaves (Out of Time) or on its own trigger (The Pandorica's
//! untap).

use crate::card::CardId;
use crate::game::GameState;
use crate::game::types::GameEvent;

impl GameState {
    /// Phase in every permanent phased out "until" `source` does something.
    pub(crate) fn phase_in_held_by(&mut self, source: CardId, events: &mut Vec<GameEvent>) {
        let mut i = 0;
        let mut phased_in: Vec<CardId> = Vec::new();
        while i < self.phased_out.len() {
            if self.phased_out[i].phased_out_by == Some(source) {
                let mut c = self.phased_out.remove(i);
                c.phased_out_by = None;
                phased_in.push(c.id);
                self.battlefield.push(c);
            } else {
                i += 1;
            }
        }
        for card_id in phased_in {
            events.push(GameEvent::PermanentPhasedIn { card_id });
        }
    }
}
