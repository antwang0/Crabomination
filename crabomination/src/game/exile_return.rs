//! "Exile this, then return it to the battlefield" (the FIN Dominant flips,
//! Saga back faces, blink-in-place): the exile half is a real zone change.
//! Taking the card straight off the battlefield skipped the leave-the-
//! battlefield work — linked exiles coming back and control/static effects
//! ending (CR 610.3, `on_left_battlefield`), LTB triggers with LKI, combat
//! removal, The Ozolith — and re-placed a token that should have ceased to
//! exist (CR 111.8).

use super::GameState;
use super::types::GameEvent;
use crate::card::{CardId, CardInstance};

impl GameState {
    /// Exile battlefield permanent `id` through the full exile path, then lift
    /// it back out of exile for the caller to return. `None` when it isn't on
    /// the battlefield or didn't land in exile (a token, a replaced move).
    pub(crate) fn exile_and_take_back(&mut self, id: CardId, events: &mut Vec<GameEvent>) -> Option<CardInstance> {
        self.battlefield.find_by_id(id)?;
        self.remove_from_battlefield_to_exile(id);
        events.push(GameEvent::PermanentExiled { card_id: id });
        let pos = self.exile.iter().position(|c| c.id == id)?;
        Some(self.exile.remove(pos))
    }
}
