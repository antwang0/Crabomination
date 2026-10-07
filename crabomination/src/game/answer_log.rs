//! The resolution answer log's seat tags. A multi-seat arm suspended for a
//! `wants_ui` seat re-runs from the top and replays the log by position; a
//! seat that leaves the game in between (CR 800.4a) drops out of the arm's
//! seat walk, so its answers must be skipped, not read as the next seat's.

use super::GameState;
use crate::decision::DecisionAnswer;

impl GameState {
    /// Append `a` to the replay log, tagged with the seat that gave it.
    pub(crate) fn log_answer(&mut self, a: DecisionAnswer, seat: Option<usize>) {
        let tag = seat.map_or(u8::MAX, |s| s.min(u8::MAX as usize - 1) as u8);
        // A log planted without tags (tests) stays untagged in front.
        let s = &mut self.scratch;
        while s.resolution_answer_seats.len() < s.resolution_answer_log.len() {
            s.resolution_answer_seats.push(u8::MAX);
        }
        s.resolution_answer_log.push(a);
        s.resolution_answer_seats.push(tag);
    }

    /// Does the log hold an answer someone still in the game gave (or an
    /// untagged one)? A departed seat's answers are left unread on purpose —
    /// the re-run skips that seat (CR 800.4a) — so they are not a leak.
    #[cfg(all(debug_assertions, not(test)))]
    pub(crate) fn answer_log_holds_live_answers(&self) -> bool {
        let s = &self.scratch;
        (0..s.resolution_answer_log.len()).any(|i| match s.resolution_answer_seats.get(i) {
            Some(&t) if t != u8::MAX => self.players.get(t as usize).is_none_or(|p| p.is_alive()),
            _ => true,
        })
    }

    /// Before `seat` replays the entry at `cursor`: step past every entry a
    /// different, departed seat gave (CR 800.4a).
    pub(crate) fn skip_departed_answers(&self, cursor: &mut usize, seat: usize) {
        let s = &self.scratch;
        while let Some(&tag) = s.resolution_answer_seats.get(*cursor) {
            let t = tag as usize;
            if tag == u8::MAX || t == seat || self.players.get(t).is_none_or(|p| p.is_alive()) {
                break;
            }
            *cursor += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog;
    use crate::decision::DecisionAnswer;
    use crate::game::types::GameAction;
    use crate::game::*;

    /// An answer whose resumed action fails rolls back and drops the decision;
    /// the replay log it would have resumed from goes with it, or the next
    /// resolution's first ask replays a stale answer (Divergent Equation's
    /// unpayable X, with a planted earlier answer).
    #[test]
    fn a_dropped_decision_takes_its_replay_log() {
        let mut g = two_player_game();
        g.players[0].wants_ui = true;
        g.players[0].manual_mana = true;
        g.add_card_to_battlefield(0, catalog::island());
        let id = g.add_card_to_hand(0, catalog::divergent_equation());
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("the cast suspends on the X pick");
        g.log_answer(DecisionAnswer::Bool(true), Some(0));
        assert!(g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Amount(3))).is_err());
        assert!(g.pending_decision.is_none());
        assert!(g.scratch.resolution_answer_log.is_empty(), "no answer outlives the dropped decision");
    }
}
