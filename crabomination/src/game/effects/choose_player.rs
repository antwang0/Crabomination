//! "Choose a player" / "choose an opponent" as the controller's own pick
//! (CR 614.12 for the as-enters form; not a target, CR 115.10). The seats are
//! offered as a `ChooseOption` ballot whose first entry is the headless
//! answer, so a bot or `AutoDecider` seat picks exactly what the engine used
//! to pick for it, and a prompting or scripted seat picks any seat.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, PendingEffectState};

impl GameState {
    /// The seats `chooser` may name, headless pick first: the most hostile
    /// opponent, then the other opponents in turn order, then (unless
    /// `opponent`) the chooser and its teammates.
    pub(crate) fn player_ballot(&self, chooser: usize, opponent: bool) -> Vec<usize> {
        let order = self.seats_in_turn_order_from(chooser);
        let hostile = self.default_hostile_opponent(chooser);
        let mut seats: Vec<usize> = hostile.into_iter().collect();
        let foe = |q: usize| q != chooser && !self.same_team(q, chooser);
        for &q in &order {
            if foe(q) && self.players[q].is_alive() && Some(q) != hostile {
                seats.push(q);
            }
        }
        if !opponent {
            seats.extend(order.iter().copied().filter(|&q| !foe(q) && self.players[q].is_alive()));
        }
        seats
    }

    /// `Effect::ChoosePlayerForSource` — stamp the controller's pick on the
    /// source's `chosen_player`. A prompting seat suspends; the answer is
    /// applied by `ChosenPlayerPending`.
    pub(super) fn choose_player_for_source(&mut self, opponent: bool, ctx: &EffectContext) -> Result<(), GameError> {
        let Some(source) = ctx.source else { return Ok(()) };
        let seats = self.player_ballot(ctx.controller, opponent);
        if seats.is_empty() {
            return Ok(());
        }
        let decision = ballot_decision(source, &seats, if opponent { "Choose an opponent" } else { "Choose a player" });
        let pending = PendingEffectState::ChosenPlayerPending { target_id: source, seats: seats.clone() };
        let answer = if seats.len() == 1 {
            DecisionAnswer::Amount(0)
        } else if self.seat_prompts(ctx.controller) {
            self.suspend_signal = Some(Box::new((decision, pending, Effect::Noop)));
            return Ok(());
        } else if matches!(self.decider.kind(), crate::decision::DeciderKind::Auto) {
            DecisionAnswer::Amount(0)
        } else {
            self.decider.decide(&decision)
        };
        self.apply_chosen_player_answer(source, &seats, &answer)
    }

    /// `ChosenPlayerPending`'s apply: the `Amount(i)` answer indexes `seats`.
    pub(crate) fn apply_chosen_player_answer(
        &mut self,
        source: CardId,
        seats: &[usize],
        answer: &DecisionAnswer,
    ) -> Result<(), GameError> {
        let DecisionAnswer::Amount(i) = answer else { return Err(GameError::DecisionAnswerMismatch) };
        let Some(&pick) = seats.get(*i as usize).or(seats.first()) else { return Ok(()) };
        if let Some(c) = self.battlefield_find_mut(source) {
            c.chosen_player = Some(pick);
        }
        Ok(())
    }

    /// `Effect::ChooseOpponentThen` — the controller names an opponent for
    /// `then`. The headless pick is the opponent with the fewest creatures
    /// (the gift goes where it helps least), turn order breaking ties.
    pub(super) fn choose_opponent_then(
        &mut self,
        then: &Effect,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let mut opps: Vec<usize> = self
            .seats_in_turn_order_from(me)
            .into_iter()
            .filter(|&p| p != me && !self.same_team(p, me) && self.players[p].is_alive())
            .collect();
        // Stable: turn order breaks ties.
        opps.sort_by_key(|&p| self.battlefield.iter().filter(|c| c.controller == p && c.definition.is_creature()).count());
        let i = match take_opt_scratch!(self.stashed_resolution_answer) {
            Some(DecisionAnswer::Amount(n)) => n as usize,
            _ if opps.len() <= 1 => 0,
            _ => {
                let decision = ballot_decision(ctx.source.unwrap_or(CardId(0)), &opps, "Choose an opponent");
                if self.seat_suspends(me) {
                    let max = opps.len() as u32 - 1;
                    self.suspend_signal =
                        Some(Box::new((decision, PendingEffectState::AmountAnswerPending { max }, effect.clone())));
                    return Ok(());
                }
                match self.decider.kind() {
                    crate::decision::DeciderKind::Auto => 0,
                    _ => match self.decider.decide(&decision) {
                        DecisionAnswer::Amount(n) => n as usize,
                        _ => 0,
                    },
                }
            }
        };
        let Some(&pick) = opps.get(i).or(opps.first()) else { return Ok(()) };
        let prev = self.scratch.chosen_opponent_scratch.replace(pick);
        let r = self.run_effect(then, ctx, events);
        self.scratch.chosen_opponent_scratch = prev;
        r
    }
}

fn ballot_decision(source: CardId, seats: &[usize], prompt: &str) -> Decision {
    Decision::ChooseOption {
        source,
        prompt: prompt.to_string(),
        options: seats.iter().map(|q| format!("Player {}", q + 1)).collect(),
    }
}

impl GameState {
    /// `Effect::GainProtectionFromPlayer` (CR 702.16). `UntilNextTurn` lasts
    /// until the protected player's next turn; every other duration is read
    /// as end of turn for a player.
    pub(super) fn gain_protection_from_player(
        &mut self,
        what: &crate::effect::Selector,
        from: &crate::effect::PlayerRef,
        duration: crate::effect::Duration,
        ctx: &EffectContext,
    ) {
        use crate::game::effects::EntityRef;
        let Some(q) = self.resolve_player(from, ctx) else { return };
        let bit = 1u64.checked_shl(q as u32).unwrap_or(0);
        let kw = crate::card::Keyword::ProtectionFromMatching(Box::new(
            crate::card::SelectionRequirement::ControlledBySeat(q as u8),
        ));
        for ent in self.resolve_selector(what, ctx) {
            match ent {
                EntityRef::Player(p) if duration == crate::effect::Duration::UntilNextTurn => {
                    self.players[p].protected_from_seats_until_next_turn |= bit;
                }
                EntityRef::Player(p) => self.players[p].protected_from_seats_eot |= bit,
                EntityRef::Permanent(cid) => self.grant_keyword_for(cid, kw.clone(), duration, ctx),
                _ => {}
            }
        }
    }
}
