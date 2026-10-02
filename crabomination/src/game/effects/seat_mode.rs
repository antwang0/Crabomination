//! CR 608.2d — a resolution-time "choose one of these" asked of the resolving
//! ability's controller through a `Decision::ChooseMode`, so a `wants_ui`
//! controller answers it in the client's mode picker (and a pod seat through
//! the bot policy) instead of the game-wide decider answering for them.
//!
//! Single-slot stash-and-rerun: the answer comes back as
//! `stashed_resolution_answer`, so this must be the arm's only ask and every
//! mutation must come after it. A non-prompting seat (and any seat under a
//! scripted decider) is answered by the decider exactly as before, which is
//! why a headless two-player game is unchanged.

use crate::card::CardId;
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::game::types::PendingEffectState;
use crate::game::GameState;

impl GameState {
    /// Ask the controller of the resolving `effect` to pick one of
    /// `mode_texts`. `Some(index)` (clamped into range) when answered,
    /// `None` when the ask suspended — the caller returns `Ok(())` and the
    /// re-run reads the stashed answer.
    pub(crate) fn ask_controller_mode(
        &mut self,
        controller: usize,
        source: CardId,
        mode_texts: Vec<String>,
        effect: &Effect,
    ) -> Option<usize> {
        let num_modes = mode_texts.len();
        let last = num_modes.saturating_sub(1);
        match take_opt_scratch!(self.stashed_resolution_answer) {
            Some(DecisionAnswer::Mode(i)) => return Some(i.min(last)),
            Some(_) => return Some(0),
            None => {}
        }
        let decision = Decision::ChooseMode { source, num_modes, mode_texts };
        if num_modes > 1 && self.seat_suspends(controller) {
            self.suspend_signal = Some(Box::new((
                decision,
                PendingEffectState::ModeAnswerPending { num_modes },
                effect.clone(),
            )));
            return None;
        }
        Some(match self.decider.decide(&decision) {
            DecisionAnswer::Mode(i) => i.min(last),
            _ => 0,
        })
    }
}
