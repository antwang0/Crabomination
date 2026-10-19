//! CR 701.68 — *blight N*: the controller puts N -1/-1 counters on a creature
//! they control (`Effect::Blight`, `Effect::BlightThen`).

use super::EffectContext;
use crate::card::{CardId, CounterType};
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// Blight `n`; the creature that took the counters, if any did.
    pub(super) fn blight(
        &mut self,
        n: &crate::effect::Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<Option<CardId>, GameError> {
        let amt = self.evaluate_value(n, ctx).max(0);
        if amt == 0 {
            return Ok(None);
        }
        // CR 701.68a/b — the controller chooses one creature they
        // control; with none, they can't blight (no-op).
        let candidates: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == ctx.controller && self.computed_is_creature(c))
            .map(|c| c.id)
            .collect();
        if candidates.is_empty() {
            return Ok(None);
        }
        // CR 701.68a — the choice is the controller's own, and it went
        // to the decider rather than to the seat, so a `wants_ui`
        // controller was never asked which of their creatures takes it.
        let chosen = if candidates.len() == 1 {
            candidates[0]
        } else {
            let mut cursor = 0;
            let Some(picked) = self.ask_seat_target_logged(
                &mut cursor,
                ctx.controller,
                format!("blight {amt}: choose a creature you control"),
                ctx.source.unwrap_or(CardId(0)),
                candidates.iter().map(|id| Target::Permanent(*id)).collect(),
                effect,
            ) else {
                return Ok(None);
            };
            self.clear_answer_log();
            match picked {
                Target::Permanent(id) if candidates.contains(&id) => id,
                _ => candidates[0],
            }
        };
        let mut sub = ctx.clone();
        sub.targets = vec![Target::Permanent(chosen)];
        self.run_effect(
            &Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::MinusOneMinusOne,
                amount: crate::effect::Value::Const(amt),
            },
            &sub,
            events,
        )?;
        Ok(Some(chosen))
    }
}
