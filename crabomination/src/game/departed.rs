//! CR 800.4f/g — where an ask goes when the seat it names has left the game.
//!
//! Multiplayer is the only place this arises: in a duel a departed opponent
//! ends the game, so nothing is ever asked of one. In a pod it is ordinary —
//! a tribute creature (CR 702.104) entering under one seat asks *an opponent*
//! whether to pay tribute, and that opponent may have been dead for ten
//! turns. Four-seat pods over the target decks hit it in 4.4 % of games, all
//! through [`GameState::ask_seat_bool`](crate::game::GameState::ask_seat_bool)
//! and its siblings.
//!
//! The rules split the answer by whether the ask is a *cost*:
//!
//! * **CR 800.4f** — "If an object requires a player who has left the game to
//!   pay a cost or choose whether to pay a cost, that cost is not paid." So a
//!   Rhystic tax owed by a departed seat is simply not paid, and resolution
//!   continues into the "unless" half. It is not asked of anyone.
//! * **CR 800.4g** — "If an object requires a player who has left the game to
//!   make a choice other than whether to pay a cost, the controller of the
//!   object chooses another player to make that choice. If the original choice
//!   was to be made by an opponent of the controller of the object, that
//!   player chooses another opponent if possible."
//!
//! CR 800.4h ("if a *rule* requires" — the next player in turn order chooses)
//! needs no arm here: every rule-required ask the engine suspends on names the
//! asking seat's own cards (a mulligan, a cleanup discard, a combat-damage
//! assignment order), and CR 800.4a has already taken those out of the game.
//! There is nothing left to choose, which is why `objects_leave_with_player`
//! drops one of those rather than re-seating it.

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
    /// whether to pay one".
    ///
    /// A live seat answers its own questions, which is every ask in a duel and
    /// nearly every ask in a pod — one `is_alive` read before anything else.
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
        // CR 800.4g. The object's controller is the one who re-seats the
        // choice; with the source gone too (an ability resolving off a
        // permanent that has left), fall back to the seat that still has to
        // finish resolving, which the caller identifies by holding priority.
        let controller = self
            .find_card_anywhere(source)
            .map(|c| c.controller)
            .filter(|&c| self.players.get(c).is_some_and(|p| p.is_alive()))
            .unwrap_or(self.priority.player_with_priority);
        if self.same_team(controller, seat) {
            // The departed chooser was the controller or a teammate: the
            // controller may name any other player, and naming themselves is
            // the one pick that needs no policy.
            return AskRoute::Seat(controller);
        }
        // "…that player chooses another opponent if possible" — the ranked
        // pick, so the re-seating is the same question the rest of the engine
        // answers with `default_hostile_opponent` and a fixed seed reproduces
        // it. No opponent left means the controller chooses themselves, which
        // the first sentence of 800.4g allows.
        AskRoute::Seat(self.default_hostile_opponent(controller).unwrap_or(controller))
    }

    /// Whether an interactive seat should be handed a modal — `wants_ui`,
    /// **and still in the game** (CR 800.4).
    ///
    /// The `is_alive` half is what the 102 hand-written `players[x].wants_ui`
    /// reads this replaces were each missing. A seat that has left has no
    /// player behind it, so a decision addressed to one is a table waiting on
    /// somebody who cannot answer — the recurring shape in this engine's
    /// multiplayer bugs, and the reason CR 800.4a/f/g/h exist at all. One
    /// predicate so a new ask site cannot reintroduce it, and so the answer
    /// matches [`route_ask`](Self::route_ask)'s.
    ///
    /// Identical to a bare `wants_ui` read in a duel: a seat leaving there
    /// ends the game.
    pub(crate) fn seat_prompts(&self, seat: usize) -> bool {
        self.players.get(seat).is_some_and(|p| p.wants_ui && p.is_alive())
    }

    /// [`route_ask`](Self::route_ask) for a question that is *not* about
    /// paying a cost, which is every ask but the yes/no ones — CR 800.4f
    /// cannot apply, so there is always a seat to ask.
    pub(crate) fn route_ask_choice(&self, seat: usize, source: crate::card::CardId) -> usize {
        match self.route_ask(seat, source, false) {
            AskRoute::Seat(q) => q,
            AskRoute::CostNotPaid => unreachable!("route_ask(.., false) never withholds the ask"),
        }
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

    /// CR 800.4 generally: no modal the engine poses may name a seat that has
    /// left. `seat_prompts` is the one predicate behind every suspend, so this
    /// is the shape check rather than a per-card one — the 102 hand-written
    /// `players[x].wants_ui` reads it replaced were each a place this could
    /// recur, and each of them was missing the `is_alive` half.
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
