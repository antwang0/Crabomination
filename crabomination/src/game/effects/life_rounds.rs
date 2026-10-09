//! Plague of Vermin — "starting with you, each player may pay any amount of
//! life; repeat until no one pays; each player creates a token per 1 life
//! they paid." Every ask comes first (the replay-log contract of
//! `ask_seat_amount`: a suspended seat re-runs this from the top), then the
//! life and the tokens, in turn order from the controller.

use super::{EffectContext, GameEvent};
use crate::card::{CardId, TokenDefinition};
use crate::decision::AmountKind;
use crate::effect::{Effect, PlayerRef, Value};
use crate::game::GameState;
use crate::game::types::GameError;

impl GameState {
    pub(super) fn pay_life_rounds_for_tokens(
        &mut self,
        definition: &std::sync::Arc<TokenDefinition>,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let src = ctx.source.unwrap_or(CardId(0));
        let mut order = vec![ctx.controller];
        let mut s = ctx.controller;
        for _ in 1..self.players.len() {
            s = self.next_alive_seat(s);
            if s == ctx.controller {
                break;
            }
            order.push(s);
        }
        let mut left: Vec<i64> = order.iter().map(|&p| i64::from(self.players[p].life)).collect();
        let mut paid = vec![0u32; order.len()];
        let mut cursor = 0usize;
        loop {
            let mut any = false;
            for (i, &p) in order.iter().enumerate() {
                let max = left[i].clamp(0, i64::from(u16::MAX)) as u32;
                if max == 0 {
                    continue;
                }
                let Some(n) = self.ask_seat_amount(
                    &mut cursor,
                    p,
                    "Pay how much life for Rats? (0 to pass)".to_string(),
                    src,
                    max,
                    AmountKind::LifeOfRemaining,
                    effect,
                ) else {
                    return Ok(());
                };
                if n > 0 {
                    left[i] -= i64::from(n);
                    paid[i] += n;
                    any = true;
                }
            }
            if !any {
                break;
            }
        }
        self.clear_answer_log();
        for (i, &p) in order.iter().enumerate() {
            if paid[i] == 0 {
                continue;
            }
            let applied = self.adjust_life_applied(p, -(paid[i] as i32));
            if applied < 0 {
                events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
            }
        }
        for (i, &p) in order.iter().enumerate() {
            if paid[i] == 0 {
                continue;
            }
            let mint = Effect::CreateToken {
                who: PlayerRef::Seat(p),
                count: Value::Const(paid[i] as i32),
                definition: definition.clone(),
            };
            self.run_effect(&mint, ctx, events)?;
        }
        Ok(())
    }
}
