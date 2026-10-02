//! CR 601.2d / 608.2d — distributing counters: the split is the controller's
//! `Decision::DivideDamage`, answered once (a `wants_ui` seat suspends and the
//! re-run reads the stashed answer); a malformed answer falls back to an even
//! split. Shared by the targeted `DistributeCounters` and the untargeted
//! `DistributeCountersAmong`.

use crate::card::{CardId, CounterType};
use crate::decision::Decision;
use crate::effect::{Effect, Value};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, PendingEffectState, Target};
use crate::game::{GameError, GameState};

impl GameState {
    pub(crate) fn distribute_counters_over(
        &mut self,
        targets: Vec<Target>,
        total: &Value,
        counter: CounterType,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let amt = self.evaluate_value(total, ctx).max(0) as u32;
        if amt == 0 || targets.is_empty() {
            return Ok(());
        }
        let decision = Decision::DivideDamage {
            source: ctx.source.unwrap_or(CardId(0)),
            total: amt,
            targets: targets.clone(),
            noun: super::counter_noun(counter).into(),
        };
        let answer = match take_opt_scratch!(self.stashed_resolution_answer) {
            Some(a) => a,
            None if self.seat_prompts(ctx.controller) => {
                self.suspend_signal = Some(Box::new((
                    decision,
                    PendingEffectState::DivisionAnswerPending,
                    effect.clone(),
                )));
                return Ok(());
            }
            None => self.decider.decide(&decision),
        };
        let mut division = match answer {
            crate::decision::DecisionAnswer::DamageDivision(v) => v,
            _ => vec![],
        };
        if division.len() != targets.len() || division.iter().sum::<u32>() != amt {
            division = crate::decision::even_damage_split(amt, targets.len());
        }
        for (t, n) in targets.iter().zip(division) {
            if n == 0 {
                continue;
            }
            let Target::Permanent(id) = t else { continue };
            let id = *id;
            // CR 614.16 — counter replacement effects (Doubling Season,
            // Hardened Scales) scale the placement.
            let n = self
                .battlefield_find(id)
                .map(|c| (c.controller, self.computed_is_creature(c)))
                .map(|(ctrl, cre)| self.scaled_counter_count(ctrl, counter, n, cre))
                .unwrap_or(n);
            if let Some(c) = self.battlefield_find_mut(id) {
                c.add_counters(counter, n);
                events.push(GameEvent::CounterAdded {
                    card_id: id,
                    counter_type: counter,
                    count: n,
                    placer: self.resolution_causer,
                });
            }
            self.turn.permanents_gained_counter_this_turn.insert(id);
        }
        self.check_state_based_actions_mid_resolution(events);
        Ok(())
    }
}
