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

    /// The living opponents of `me`, headless pick first: the one with the
    /// fewest creatures (a gift goes where it helps least), turn order
    /// breaking ties.
    fn gift_ballot(&self, me: usize) -> Vec<usize> {
        let mut opps: Vec<usize> = self
            .seats_in_turn_order_from(me)
            .into_iter()
            .filter(|&p| p != me && !self.same_team(p, me) && self.players[p].is_alive())
            .collect();
        // Stable: turn order breaks ties.
        opps.sort_by_key(|&p| self.battlefield.iter().filter(|c| c.controller == p && c.definition.is_creature()).count());
        opps
    }

    /// "Choose an opponent" made where no resolution can suspend — a cast's
    /// additional cost (Gift, CR 702.174a / 601.2b), a replacement as a
    /// permanent enters (a Siege's protector, CR 310.11a; Tribute, CR
    /// 702.104a), a state-based action (CR 704.5w-x) or a cast trigger's copy
    /// (Demonstrate, CR 702.144a). A scripted or bot decider answers the
    /// [`gift_ballot`](Self::gift_ballot); a headless or live-UI seat takes
    /// its headless pick.
    pub(crate) fn choose_opponent_at_once(&mut self, chooser: usize, source: CardId, prompt: &str) -> Option<usize> {
        let opps = self.gift_ballot(chooser);
        let i = if opps.len() <= 1 || matches!(self.decider.kind(), crate::decision::DeciderKind::Auto) {
            0
        } else {
            match self.decider.decide(&ballot_decision(source, &opps, prompt)) {
                DecisionAnswer::Amount(n) => n as usize,
                _ => 0,
            }
        };
        opps.get(i).or(opps.first()).copied()
    }

    /// CR 704.5w / 704.5x — a battle with no protector in the game (its
    /// protector left) and nothing attacking it, or a Siege its own controller
    /// protects (a control change), gets a new protector from its
    /// controller's opponents; with none to choose, it goes to its owner's
    /// graveyard.
    pub(crate) fn reseat_battle_protectors(&mut self, events: &mut Vec<GameEvent>) {
        use crate::game::types::AttackTarget;
        let stale: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.is_battle())
            .filter(|c| match c.protected_by {
                Some(pr) if pr == c.controller => true,
                Some(pr) if self.players.get(pr).is_some_and(|pl| pl.is_alive()) => false,
                _ => !self.attacking.iter().any(|a| a.target == AttackTarget::Battle(c.id)),
            })
            .map(|c| (c.id, c.controller))
            .collect();
        for (id, ctrl) in stale {
            match self.choose_opponent_at_once(ctrl, id, "Choose the Siege's protector") {
                Some(q) => {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.protected_by = Some(q);
                    }
                }
                None => events.extend(self.remove_to_graveyard_with_triggers(id)),
            }
        }
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
        let opps = self.gift_ballot(me);
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
        // Bound, not just set: a body that asks parks a continuation that
        // resumes after this arm restored the scratch.
        let bound = Effect::BindScratch {
            scratch: crate::effect::ScratchBinding::ChosenOpponent(pick),
            body: Box::new(then.clone()),
        };
        self.run_effect(&bound, ctx, events)
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
