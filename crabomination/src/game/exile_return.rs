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
        Some(self.exile_remove_at(pos))
    }

    /// Lift card `id` out of exile. CR 400.7 — what leaves exile is a new
    /// object: an exiler's "until this leaves" return link (`exiled_by`) does
    /// not ride along into the next zone, or into a later exile. `exiled_with`
    /// stays for the cast paths that read it as the card leaves (Aminatou's
    /// Augury, Share the Spoils); `move_card_to` drops both.
    pub(crate) fn take_from_exile(&mut self, id: CardId) -> Option<CardInstance> {
        let pos = self.exile.iter().position(|c| c.id == id)?;
        Some(self.exile_remove_at(pos))
    }

    /// [`take_from_exile`](Self::take_from_exile) by position.
    pub(crate) fn exile_remove_at(&mut self, pos: usize) -> CardInstance {
        let mut card = self.exile.remove(pos);
        card.exiled_by = None;
        card
    }
}
