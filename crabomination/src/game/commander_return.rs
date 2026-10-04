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

    /// Pose a prompting owner's CR 903.9a choice once an action has settled —
    /// before anyone acts again, since every action is refused while it is
    /// pending. One commander at a time; the answer's action poses the next.
    pub(crate) fn pose_commander_return(&mut self) {
        if self.pending_decision.is_some() || self.suspend_signal.is_some() || self.is_game_over() {
            return;
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
                if let Some(would_be) = self.commander_return_zone(owner, commander) {
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
        if let Some(zone) = self.commander_return_zone(owner, commander) {
            if yes {
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
        let card = if zone == Zone::Graveyard {
            Self::take_card(&mut self.players[owner].graveyard, id)
        } else {
            Self::take_card(&mut self.exile, id)
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
