//! CR 701.38 secret council ballots whose *counts* matter: a vote for any
//! player, yourself included (Círdan the Shipwright — a card per vote
//! received, a free permanent for anyone nobody voted for), and a vote for a
//! permanent (Trap the Trespassers — a stun counter per vote). The ballots
//! are cast one seat at a time, starting with the controller, and revealed
//! together, as `each_player_votes_for_a_player` does.

use crate::card::{CardId, SelectionRequirement};
use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::SecretCouncilPlayerVote`. A headless voter's first option is
    /// itself (a vote you receive is a card you draw), then the others by
    /// seat.
    pub(super) fn secret_council_player_vote(
        &mut self,
        per_vote: &Effect,
        unvoted: &Effect,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let voters = self.seats_in_turn_order_from(ctx.controller);
        let mut cursor = 0usize;
        let mut cast: Vec<(usize, usize)> = Vec::new();
        for &v in &voters {
            let mut ballot = voters.clone();
            ballot.sort_by_key(|&q| (q != v, q));
            let labels: Vec<String> = ballot.iter().map(|q| format!("Player {}", q + 1)).collect();
            let asked = self.vote_controller_this_turn.unwrap_or(v);
            for _ in 0..1 + self.additional_votes_for(v) {
                let Some(pick) = self.ask_seat_option(
                    &mut cursor,
                    asked,
                    "Vote for a player".to_string(),
                    source,
                    labels.clone(),
                    effect,
                ) else {
                    return Ok(());
                };
                cast.push((v, ballot[pick.min(ballot.len() - 1)]));
            }
        }
        self.clear_answer_log();
        for &(v, q) in &cast {
            events.push(GameEvent::Voted { player: v, choice: format!("Player {}", q + 1) });
        }
        events.push(GameEvent::VotingFinished);
        let mut steps = Vec::new();
        for &q in &voters {
            let received = cast.iter().filter(|&&(_, to)| to == q).count();
            let body = if received == 0 {
                unvoted.clone()
            } else {
                Effect::Repeat { count: crate::effect::Value::Const(received as i32), body: Box::new(per_vote.clone()) }
            };
            steps.push(Effect::ForEach { selector: Selector::Player(PlayerRef::Seat(q)), body: Box::new(body) });
        }
        self.run_effect(&Effect::Seq(steps), ctx, events)
    }

    /// `Effect::SecretCouncilPermanentVote`. The candidates are the
    /// permanents matching `filter` for the controller, biggest power first,
    /// so a headless voter backs the largest threat.
    pub(super) fn secret_council_permanent_vote(
        &mut self,
        filter: &SelectionRequirement,
        per_vote: &Effect,
        // `Some(on_none)`: the "up to one … most votes" form — a voter may
        // abstain, `per_vote` runs once per permanent tied for most votes,
        // and `on_none` when nobody got a vote (Vault 11).
        most: Option<&Effect>,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let mut candidates: Vec<(CardId, i32, String)> = self
            .battlefield
            .iter()
            .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
            .map(|c| (c.id, self.effective_power(c), c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            return match most {
                Some(on_none) => self.run_effect(on_none, ctx, events),
                None => Ok(()),
            };
        }
        candidates.sort_by_key(|(id, pw, _)| (std::cmp::Reverse(*pw), *id));
        let mut labels: Vec<String> = candidates.iter().map(|(_, _, n)| n.clone()).collect();
        if most.is_some() {
            labels.push("No creature".to_string());
        }
        let voters = self.seats_in_turn_order_from(ctx.controller);
        let mut cursor = 0usize;
        let mut cast: Vec<(usize, CardId)> = Vec::new();
        for &v in &voters {
            let asked = self.vote_controller_this_turn.unwrap_or(v);
            for _ in 0..1 + self.additional_votes_for(v) {
                let Some(pick) = self.ask_seat_option(
                    &mut cursor,
                    asked,
                    "Vote for a permanent".to_string(),
                    source,
                    labels.clone(),
                    effect,
                ) else {
                    return Ok(());
                };
                if pick >= candidates.len() {
                    continue; // abstained ("up to one")
                }
                cast.push((v, candidates[pick].0));
            }
        }
        self.clear_answer_log();
        for &(v, id) in &cast {
            let name = candidates.iter().find(|c| c.0 == id).map(|c| c.2.clone()).unwrap_or_default();
            events.push(GameEvent::Voted { player: v, choice: name });
        }
        events.push(GameEvent::VotingFinished);
        if let Some(on_none) = most {
            if cast.is_empty() {
                return self.run_effect(on_none, ctx, events);
            }
            let tally = |id: CardId| cast.iter().filter(|(_, c)| *c == id).count();
            let top = cast.iter().map(|(_, id)| tally(*id)).max().unwrap_or(0);
            let mut winners: Vec<CardId> = cast.iter().map(|(_, id)| *id).filter(|id| tally(*id) == top).collect();
            winners.sort();
            winners.dedup();
            for id in winners {
                let sub = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
                self.run_effect(per_vote, &sub, events)?;
            }
            return Ok(());
        }
        for &(_, id) in &cast {
            let sub = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
            self.run_effect(per_vote, &sub, events)?;
        }
        Ok(())
    }

    /// `PlayerRef::OpponentsWhoVotedTheSame` — opponents with at least one
    /// vote matching one of the controller's on the most recent ballot.
    pub(crate) fn opponents_who_voted_the_same(&self, controller: usize) -> Vec<usize> {
        let mine: Vec<usize> =
            self.last_vote.iter().filter(|(seat, _)| *seat == controller).map(|(_, pick)| *pick).collect();
        self.apnap_sort(
            self.opponents_of(controller)
                .into_iter()
                .filter(|i| self.players[*i].is_alive())
                .filter(|i| self.last_vote.iter().any(|(seat, pick)| seat == i && mine.contains(pick)))
                .collect(),
        )
    }
}
