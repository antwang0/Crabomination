//! CR 800.4f/g — where an ask goes when the seat it names has left the game.
//!
//! Multiplayer only (in a duel a departure ends the game): e.g. a tribute
//! creature (CR 702.104) asks an opponent who may be long dead. Asks go
//! through [`GameState::ask_seat_bool`](crate::game::GameState::ask_seat_bool)
//! and its siblings.
//!
//! * **CR 800.4f** — a departed player's cost (or choice whether to pay one)
//!   is not paid: a Rhystic tax goes to the "unless" half, asking no one.
//! * **CR 800.4g** — any other choice: the object's controller picks another
//!   player to make it, another opponent if the original was an opponent.
//!
//! CR 800.4h (rule-required asks) needs no arm: every such ask names the
//! asking seat's own cards, which CR 800.4a already removed, so
//! `objects_leave_with_player` drops it.

use crate::game::GameState;

/// Who answers an ask nominally routed to a seat, once CR 800.4f/g have had
/// their say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AskRoute {
    /// Ask this seat. The named seat itself, except after CR 800.4g.
    Seat(usize),
    /// CR 800.4f — nobody is asked and the cost is not paid, so the caller
    /// takes the declining answer.
    CostNotPaid,
}

impl GameState {
    /// Route an ask that names `seat` per CR 800.4f/g. `source` is the object
    /// making the demand (its controller is the one CR 800.4g hands the choice
    /// to); `is_cost` states whether the question is "pay a cost / choose
    /// whether to pay one". A live seat answers its own questions.
    pub(crate) fn route_ask(
        &self,
        seat: usize,
        source: crate::card::CardId,
        is_cost: bool,
    ) -> AskRoute {
        if self.players.get(seat).is_none_or(|p| p.is_alive()) {
            return AskRoute::Seat(seat);
        }
        if is_cost {
            // CR 800.4f.
            return AskRoute::CostNotPaid;
        }
        AskRoute::Seat(self.reseat_ask(seat, source))
    }

    /// CR 800.4g's re-seating, as a total function: who a departed seat's
    /// choice goes to. Shared by [`route_ask`](Self::route_ask) and
    /// [`route_ask_choice`](Self::route_ask_choice).
    fn reseat_ask(&self, seat: usize, source: crate::card::CardId) -> usize {
        // The object's controller re-seats the choice; with the source gone,
        // fall back to the priority holder. Every candidate must be in the
        // game (two seats can be gone at once — a six-seat pod once asked a
        // dead seat); last resort is the next live seat in turn order.
        let alive = |c: &usize| self.players.get(*c).is_some_and(|p| p.is_alive());
        let controller = self
            .find_card_anywhere(source)
            .map(|c| c.controller)
            .filter(alive)
            .or(Some(self.priority.player_with_priority).filter(alive))
            .unwrap_or_else(|| self.next_alive_seat(seat));
        if self.same_team(controller, seat) {
            // The departed chooser was the controller or a teammate: the
            // controller may name any other player, and naming themselves is
            // the one pick that needs no policy.
            return controller;
        }
        // "…another opponent if possible" — the deterministic ranked pick.
        // With no opponent left the controller chooses (800.4g allows it).
        self.default_hostile_opponent(controller).filter(alive).unwrap_or(controller)
    }

    /// Whether an interactive seat should be handed a modal — `wants_ui`,
    /// **and still in the game** (CR 800.4).
    ///
    /// A modal posed to a departed seat waits forever. Use this one
    /// predicate (not a bare `wants_ui` read) at every ask site so the answer
    /// matches [`route_ask`](Self::route_ask)'s. Same as `wants_ui` in a duel.
    pub(crate) fn seat_prompts(&self, seat: usize) -> bool {
        self.players.get(seat).is_some_and(|p| p.wants_ui && p.is_alive())
    }

    /// CR 800.4a / 800.4g — a resolution paused on a per-seat ask whose seat
    /// has since left goes on without it: the ask takes a placeholder logged
    /// under the departed seat, and the re-run skips it — a vote is not cast,
    /// a choice moves to another player. Bounded by the seat count; a failed
    /// resume drops the decision as the leave pass would have.
    pub(crate) fn resume_past_departed_asker(&mut self, events: &mut Vec<crate::game::GameEvent>) {
        for _ in 0..self.players.len() {
            if self.game_over.is_some() || self.resolution_depth != 0 || self.suspend_signal.is_some() {
                return;
            }
            let Some(filler) = self
                .pending_decision
                .as_ref()
                .filter(|d| !self.players.get(d.acting_player()).is_some_and(|q| q.is_alive()))
                .and_then(|d| d.departed_filler())
            else {
                return;
            };
            // The paused spell's card, should the resume fail: it must land
            // somewhere, as on the leave pass's own drop.
            let paused = match self.pending_decision.as_ref().map(|d| &d.resume) {
                Some(crate::game::types::ResumeContext::Spell { card, .. }) if !card.is_token => {
                    Some((**card).clone())
                }
                _ => None,
            };
            match self.submit_decision(filler) {
                Ok(mut evs) => events.append(&mut evs),
                Err(_) => {
                    self.pending_decision = None;
                    self.clear_answer_log();
                    if let Some(mut card) = paused
                        && self.find_card_anywhere(card.id).is_none()
                    {
                        card.controller = card.owner;
                        if let Some(card) = self.bottom_instead_of_graveyard(card) {
                            self.route_to_graveyard(card, events);
                        }
                    }
                    return;
                }
            }
        }
    }

    /// [`route_ask`](Self::route_ask) for a non-cost question: CR 800.4f
    /// cannot apply, so there is always a seat to ask. Built on
    /// [`reseat_ask`](Self::reseat_ask) rather than unwrapping `route_ask`,
    /// which would need an `unreachable!` (`audit_panics` flags those).
    pub(crate) fn route_ask_choice(&self, seat: usize, source: crate::card::CardId) -> usize {
        if self.players.get(seat).is_none_or(|p| p.is_alive()) {
            return seat;
        }
        self.reseat_ask(seat, source)
    }
}

#[cfg(test)]
mod tests {
    use super::AskRoute;
    use crate::catalog;
    use crate::game::*;

    /// CR 800.4g — a tribute ask owed by a seat that has left is re-seated on
    /// another *opponent* of the object's controller, not on the controller.
    #[test]
    fn cr_800_4g_a_departed_opponents_choice_moves_to_another_opponent() {
        let mut g = multi_player_game(4);
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert_eq!(g.route_ask(1, src, false), AskRoute::Seat(1), "a live seat answers");
        g.players[1].life = 0;
        g.check_state_based_actions();
        assert!(!g.players[1].is_alive());
        // Seat 0 controls the object, so the re-seat must land on 2 or 3.
        let AskRoute::Seat(q) = g.route_ask(1, src, false) else {
            panic!("a non-cost choice is re-seated, not dropped")
        };
        assert!(q == 2 || q == 3, "another opponent of seat 0, got {q}");
    }

    /// CR 800.4f — a *cost* question is not re-seated: the cost is not paid.
    #[test]
    fn cr_800_4f_a_departed_seats_cost_is_simply_not_paid() {
        let mut g = multi_player_game(4);
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[1].life = 0;
        g.check_state_based_actions();
        assert_eq!(g.route_ask(1, src, true), AskRoute::CostNotPaid);
    }

    /// CR 800.4g's first sentence, with no opponent left to hand it to: the
    /// controller chooses, and "another player" is themselves.
    #[test]
    fn cr_800_4g_with_no_opponent_left_the_controller_chooses() {
        let mut g = multi_player_game(2);
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[1].life = 0;
        g.check_state_based_actions();
        assert_eq!(g.route_ask(1, src, false), AskRoute::Seat(0));
    }

    /// CR 800.4a — an ask suspended for a seat that then leaves in the same
    /// action is dropped with the seat, not installed later as a decision it
    /// owes (a six-seat pod asked a departed seat for a graveyard pick after
    /// it lost to the priority pass that suspended the ask).
    #[test]
    fn cr_800_4a_a_departed_seats_suspended_ask_leaves_with_it() {
        use crate::decision::{Decision, PickValue};
        use crate::game::types::PendingEffectState;
        let mut g = multi_player_game(4);
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ask = |player| {
            Some(Box::new((
                Decision::ChooseCards {
                    source: src,
                    prompt: String::new(),
                    candidates: vec![],
                    min: 1,
                    max: 1,
                    eligible: None,
                    value: PickValue::Gain,
                },
                PendingEffectState::CardsAnswerPending { player },
                crate::effect::Effect::Noop,
            )))
        };
        g.suspend_signal = ask(3);
        g.players[1].life = 0;
        g.check_state_based_actions();
        assert!(g.suspend_signal.is_some(), "another seat's ask stays");
        g.suspend_signal = ask(2);
        // An earlier seat's answer in the same resolution's replay log.
        g.log_answer(crate::decision::DecisionAnswer::Bool(true), Some(3));
        g.players[2].life = 0;
        g.check_state_based_actions();
        assert!(g.suspend_signal.is_none(), "the departed seat's ask is gone");
        // ...and its continuation's replay log with it: the next resolution's
        // first ask would replay that answer as its own (a strict concession
        // pod, seats 79/64/128 seed 160002).
        assert!(g.scratch.resolution_answer_log.is_empty(), "no answer outlives the dropped continuation");
    }

    /// CR 800.4: no modal may name a departed seat. `seat_prompts` is behind
    /// every suspend, so this is a shape check, not a per-card one.
    #[test]
    fn cr_800_4_a_departed_seat_is_never_handed_a_modal() {
        let mut g = multi_player_game(4);
        for p in g.players.iter_mut() {
            p.wants_ui = true;
        }
        g.players[2].life = 0;
        g.check_state_based_actions();
        assert!(g.seat_prompts(0) && g.seat_prompts(1) && g.seat_prompts(3));
        assert!(!g.seat_prompts(2), "seat 2 has left the game");
        assert!(!g.seat_prompts(99), "and an out-of-range index is not a seat");
        assert!(!g.seat_suspends(2), "so it cannot suspend either");
    }

    /// A choice the departed seat owed about the *controller's own* side goes
    /// back to the controller (CR 800.4g's first sentence), not to an
    /// opponent: `same_team` decides which sentence applies.
    #[test]
    fn cr_800_4g_a_teammates_choice_goes_to_the_controller() {
        let mut g = multi_player_game(4);
        g.assign_teams(vec![vec![0, 1], vec![2, 3]]).expect("two teams of two");
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[1].life = 0;
        g.check_state_based_actions();
        assert_eq!(g.route_ask(1, src, false), AskRoute::Seat(0));
    }
}
