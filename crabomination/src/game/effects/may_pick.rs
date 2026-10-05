//! "You may put a [card] from among them onto the battlefield" — a looked-at
//! or revealed pile the resolving player picks up to one card from (Lonis,
//! Aethermage's Touch, Arthur's "look at the top N"). Each handler used to
//! take its own best card and never decline.

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::Effect;
use crate::game::GameState;

impl GameState {
    /// Ask `seat` for up to one of `candidates`, `auto` being what a headless
    /// seat takes (the handler's old pick). `None` means the ask suspended and
    /// the arm returns; `Some(pick)` is the answer, `Some(None)` a decline.
    /// The answer log is cleared on an answer, so side effects follow it.
    pub(crate) fn may_pick_one(
        &mut self,
        seat: usize,
        prompt: &str,
        source: CardId,
        candidates: Vec<(CardId, String)>,
        auto: Option<CardId>,
        effect: &Effect,
    ) -> Option<Option<CardId>> {
        if candidates.is_empty() {
            self.clear_answer_log();
            return Some(None);
        }
        let mut cursor = 0;
        let picked = self.ask_seat_cards_logged(
            &mut cursor,
            seat,
            prompt.into(),
            source,
            candidates,
            0,
            1,
            PickValue::Gain,
            effect,
            auto.into_iter().collect(),
        )?;
        self.clear_answer_log();
        Some(picked.first().copied())
    }
}
