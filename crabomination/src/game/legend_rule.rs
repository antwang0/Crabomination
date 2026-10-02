//! CR 704.5j — "If a player controls two or more legendary permanents with the
//! same name, that player chooses one of them, and the rest are put into their
//! owners' graveyards."
//!
//! The choice is the controller's. A seat that prompts (a client, a pod seat)
//! has its group left in place by the sweep and queued
//! (`GameState::deferred_legend_choices`); the oldest group still live is
//! posed once the action settles ([`GameState::pose_legend_choice`]), and the
//! answer's resume keeps the chosen one and finishes the sweep. Any other seat
//! is answered inside the sweep as before. Unlike the commander return
//! (`commander_return.rs`, which rescans), the group is queued because finding
//! it takes the sweep's grouping and exemptions.

use super::GameState;
use super::types::{GameError, GameEvent, PendingDecision, ResumeContext};
use crate::card::CardId;
use crate::decision::{Decision, DecisionAnswer};

impl GameState {
    /// Queue `player`'s same-name group for a prompting seat, once: a group
    /// already waiting takes the current duplicates (a third copy joins it).
    pub(super) fn defer_legend_choice(&mut self, player: usize, name: String, duplicates: Vec<(CardId, String)>) {
        if let Some(at) = self.deferred_legend_choices.iter().position(|(p, n, _)| *p == player && *n == name) {
            if self.deferred_legend_choices[at].2 != duplicates {
                self.deferred_legend_choices[at].2 = duplicates;
            }
            return;
        }
        self.deferred_legend_choices.push((player, name, duplicates));
    }

    /// Pose the oldest queued group that still has two or more members under
    /// its player. Cheap when the queue is empty, which it always is for a
    /// headless game.
    pub(crate) fn pose_legend_choice(&mut self) {
        if self.deferred_legend_choices.is_empty()
            || self.pending_decision.is_some()
            || self.suspend_signal.is_some()
            || self.is_game_over()
        {
            return;
        }
        while !self.deferred_legend_choices.is_empty() {
            let (player, name, duplicates) = self.deferred_legend_choices.remove(0);
            let live = self.live_legend_duplicates(player, &duplicates);
            if live.len() > 1 && self.players.get(player).is_some_and(|p| p.is_alive()) {
                self.pending_decision = Some(Box::new(PendingDecision {
                    decision: Decision::ChooseLegendToKeep { player, name: name.clone(), duplicates: live.clone() },
                    resume: ResumeContext::LegendRule { player, name, duplicates: live },
                }));
                return;
            }
        }
    }

    /// The queued duplicates still on the battlefield under `player`.
    fn live_legend_duplicates(&self, player: usize, duplicates: &[(CardId, String)]) -> Vec<(CardId, String)> {
        duplicates
            .iter()
            .filter(|(id, _)| self.battlefield_find(*id).is_some_and(|c| c.controller == player))
            .cloned()
            .collect()
    }

    /// Apply the controller's answer to a posed CR 704.5j choice, then finish
    /// the sweep it was part of. An answer outside the live group keeps the
    /// headless default.
    pub(crate) fn resume_legend_choice(
        &mut self,
        player: usize,
        duplicates: Vec<(CardId, String)>,
        answer: &DecisionAnswer,
    ) -> Result<Vec<GameEvent>, GameError> {
        let DecisionAnswer::KeptLegend(kept) = *answer else {
            return Err(GameError::DecisionAnswerMismatch);
        };
        let live = self.live_legend_duplicates(player, &duplicates);
        let kept = if live.iter().any(|(id, _)| *id == kept) { kept } else { self.legend_keep_default(&live) };
        let mut evs = Vec::new();
        let victims: Vec<CardId> = live.iter().map(|(id, _)| *id).filter(|id| *id != kept).collect();
        self.put_legend_rule_victims(victims, &mut evs);
        self.check_state_based_actions_into(&mut evs);
        if self.pending_decision.is_none() {
            self.dispatch_triggers_for_events(&evs);
        }
        Ok(evs)
    }

    /// The headless keep: the copy carrying the most accumulated board state
    /// (counters + attachments), newest on a tie — "keep newest" alone
    /// sacrificed the aura'd or countered copy to a fresh vanilla one.
    pub(crate) fn legend_keep_default(&self, duplicates: &[(CardId, String)]) -> CardId {
        duplicates
            .iter()
            .map(|(id, _)| {
                let counters: u32 = self.battlefield_find(*id).map(|c| c.counters.values().sum()).unwrap_or(0);
                let attached = self.battlefield.iter().filter(|c| c.attached_to == Some(*id)).count() as u32;
                (counters + attached, id.0, *id)
            })
            .max()
            .map(|(_, _, id)| id)
            .unwrap_or(CardId(0))
    }

    /// The legends not kept go to their owners' graveyards. Only a creature
    /// dies (CR 700.4); the snapshot lets "another of yours" death triggers
    /// read the card.
    pub(super) fn put_legend_rule_victims(&mut self, victims: Vec<CardId>, events: &mut Vec<GameEvent>) {
        for id in victims {
            if let Some(c) = self.battlefield.find_by_id(id) {
                if self.computed_is_creature(c) {
                    events.push(GameEvent::CreatureDied { card_id: id });
                }
                self.died_card_snapshots.insert(id, self.lki_clone(c));
            }
            self.remove_from_battlefield_to_graveyard_raw(id);
        }
    }
}
