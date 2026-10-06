//! CR 903.9a — "If a commander is in a graveyard or in exile and that object
//! was put into that zone since the last time state-based actions were
//! checked, its owner may put it into the command zone."
//!
//! The "may" is the owner's. A seat that prompts (a client, a pod seat) is
//! asked through a pending `Decision::CommanderRedirect` posed once the action
//! that put it there settles ([`GameState::pose_commander_return`]); any other
//! seat is answered by the decider inside the sweep, as before. A declined
//! commander is remembered in `commander_return_declined` until it turns up in
//! some other zone.
//!
//! CR 903.9b's hand / library replacement is posed the same way to a
//! prompting owner: the event can't suspend, so the card lands and the offer
//! (`commander_redirect_offers`) is asked once the action settles. The rest
//! of that resolution sees it in the hand or library (Chaos Warp's reveal).

use super::GameState;
use super::types::{GameError, GameEvent, PendingDecision, ResumeContext};
use crate::card::{CardId, Zone};
use crate::decision::{Decision, DecisionAnswer};

impl GameState {
    /// The SBA sweep's CR 903.9a pass. A prompting owner's commanders are left
    /// for [`pose_commander_return`](Self::pose_commander_return).
    pub(super) fn commander_zone_return_sba(&mut self) {
        for owner in 0..self.players.len() {
            for i in 0..self.players[owner].commanders.len() {
                let id = self.players[owner].commanders[i];
                let Some(zone) = self.commander_return_zone(owner, id) else {
                    if self.commander_return_declined.contains(&id) {
                        self.commander_return_declined.retain(|d| *d != id);
                    }
                    continue;
                };
                if self.commander_return_declined.contains(&id) || self.seat_suspends(owner) {
                    continue;
                }
                let answer = self.decider.decide(&Decision::CommanderRedirect { commander: id, would_be: zone });
                if matches!(answer, DecisionAnswer::Bool(true)) {
                    self.return_commander(owner, id, zone);
                } else {
                    self.commander_return_declined.push(id);
                }
            }
        }
    }

    /// Where commander `id` waits on a CR 903.9a choice: its owner's graveyard
    /// or exile, unless already declined there.
    fn commander_return_zone(&self, owner: usize, id: CardId) -> Option<Zone> {
        if self.players[owner].graveyard.iter().any(|c| c.id == id) {
            Some(Zone::Graveyard)
        } else if self.exile.iter().any(|c| c.id == id) {
            Some(Zone::Exile)
        } else {
            None
        }
    }

    /// [`commander_return_zone`](Self::commander_return_zone), or the hand or
    /// library a CR 903.9b offer left the commander in (still there).
    fn pending_return_zone(&self, owner: usize, id: CardId) -> Option<Zone> {
        self.commander_return_zone(owner, id).or_else(|| {
            if !self.commander_redirect_offers.contains(&id) {
                None
            } else if self.players[owner].hand.iter().any(|c| c.id == id) {
                Some(Zone::Hand)
            } else if self.players[owner].library.iter().any(|c| c.id == id) {
                Some(Zone::Library)
            } else {
                None
            }
        })
    }

    /// Pose a prompting owner's CR 903.9a choice once an action has settled —
    /// before anyone acts again, since every action is refused while it is
    /// pending. One commander at a time; the answer's action poses the next.
    pub(crate) fn pose_commander_return(&mut self) {
        if self.pending_decision.is_some() || self.suspend_signal.is_some() || self.is_game_over() {
            return;
        }
        // A 903.9b offer whose card has moved on since lapses.
        if !self.commander_redirect_offers.is_empty() {
            let offers = std::mem::take(&mut self.commander_redirect_offers);
            self.commander_redirect_offers = offers
                .into_iter()
                .filter(|&id| {
                    self.players.iter().position(|p| p.commanders.contains(&id)).is_some_and(|o| {
                        self.players[o].hand.iter().chain(self.players[o].library.iter()).any(|c| c.id == id)
                    })
                })
                .collect();
        }
        for owner in 0..self.players.len() {
            if self.players[owner].commanders.is_empty() || !self.seat_suspends(owner) {
                continue;
            }
            for i in 0..self.players[owner].commanders.len() {
                let commander = self.players[owner].commanders[i];
                if self.commander_return_declined.contains(&commander) {
                    continue;
                }
                if let Some(would_be) = self.pending_return_zone(owner, commander) {
                    self.pending_decision = Some(Box::new(PendingDecision {
                        decision: Decision::CommanderRedirect { commander, would_be },
                        resume: ResumeContext::CommanderReturn { owner, commander },
                    }));
                    return;
                }
            }
        }
    }

    /// Apply the owner's answer to a posed CR 903.9a choice, then finish the
    /// sweep it was part of. A commander that has moved on since is left alone.
    pub(crate) fn resume_commander_return(
        &mut self,
        owner: usize,
        commander: CardId,
        answer: &DecisionAnswer,
    ) -> Result<Vec<GameEvent>, GameError> {
        let DecisionAnswer::Bool(yes) = *answer else {
            return Err(GameError::DecisionAnswerMismatch);
        };
        let mut evs = Vec::new();
        if let Some(zone) = self.pending_return_zone(owner, commander) {
            if matches!(zone, Zone::Hand | Zone::Library) {
                // CR 903.9b — a one-time offer, answered either way.
                self.commander_redirect_offers.retain(|d| *d != commander);
                if yes {
                    self.return_commander(owner, commander, zone);
                }
            } else if yes {
                self.return_commander(owner, commander, zone);
            } else if !self.commander_return_declined.contains(&commander) {
                self.commander_return_declined.push(commander);
            }
        }
        self.check_state_based_actions_into(&mut evs);
        if self.pending_decision.is_none() {
            self.dispatch_triggers_for_events(&evs);
        }
        Ok(evs)
    }

    /// Move commander `id` from `zone` to its owner's command zone.
    fn return_commander(&mut self, owner: usize, id: CardId, zone: Zone) {
        let card = match zone {
            Zone::Graveyard => Self::take_card(&mut self.players[owner].graveyard, id),
            Zone::Hand => Self::take_card(&mut self.players[owner].hand, id),
            Zone::Library => Self::take_card(&mut self.players[owner].library, id),
            _ => Self::take_card(&mut self.exile, id),
        };
        let Some(mut card) = card else { return };
        if zone == Zone::Graveyard {
            let mut ev = Vec::new();
            self.note_left_graveyard(owner, id, &mut ev);
            self.scratch.pending_cost_events.extend(ev);
        }
        // CR 400.7 — a new object in the command zone.
        card.drop_counters_for_zone_change(Zone::Command);
        card.exiled_with = None;
        card.controller = owner;
        self.players[owner].command.push(card);
        self.offboard_keyword_grants = true;
        self.note_commander_to_command_zone(id, owner);
    }
}

impl GameState {
    /// CR 903.9b — a commander a direct push would put into its owner's hand
    /// or library (a countered spell's "instead" zone: Spell Crumple, Memory
    /// Lapse, Remand; Glimpse of Tomorrow's shuffle) may go to the command
    /// zone. `Some(card)` back when it does not, for the caller to place as
    /// it would have.
    pub(crate) fn commander_zone_redirect(
        &mut self,
        card: crate::card::CardInstance,
        from: Zone,
        to: Zone,
    ) -> Option<crate::card::CardInstance> {
        let owner = card.owner;
        if card.is_token || !self.players.get(owner).is_some_and(|p| p.commanders.contains(&card.id)) {
            return Some(card);
        }
        if self.resolve_zone_change(card.id, from, to) != Zone::Command {
            return Some(card);
        }
        let mut card = card;
        let id = card.id;
        card.drop_counters_for_zone_change(Zone::Command);
        card.exiled_with = None;
        card.controller = owner;
        self.players[owner].command.push(card);
        self.offboard_keyword_grants = true;
        self.note_commander_to_command_zone(id, owner);
        None
    }

    /// [`commander_zone_redirect`](Self::commander_zone_redirect) over a batch
    /// a mass move lifted (a hand or graveyard shuffled into a library): the
    /// commanders that go home leave `cards`.
    pub(crate) fn commander_zone_redirect_all(
        &mut self,
        cards: &mut Vec<crate::card::CardInstance>,
        from: Zone,
        to: Zone,
    ) {
        let is_commander =
            |g: &Self, c: &crate::card::CardInstance| g.players.get(c.owner).is_some_and(|p| p.commanders.contains(&c.id));
        if !cards.iter().any(|c| is_commander(self, c)) {
            return;
        }
        for c in std::mem::take(cards) {
            if let Some(c) = self.commander_zone_redirect(c, from, to) {
                cards.push(c);
            }
        }
    }
}
