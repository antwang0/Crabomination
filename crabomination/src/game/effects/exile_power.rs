//! "Exile all creatures. Each player creates a [token] and puts a number of
//! +1/+1 counters on it equal to the total power of creatures they controlled
//! that were exiled this way." — Oversimplify.

use super::EffectContext;
use crate::card::{CardId, CounterType, SelectionRequirement, TokenDefinition};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};
use std::sync::Arc;

impl GameState {
    /// Powers are read (layer-aware) before anything moves, per controller;
    /// only the permanents that actually left count. Each living
    /// player, in turn order from the controller, then creates the token with
    /// its total (a token with none is still created, and dies as a 0/0).
    pub(super) fn exile_all_then_token_per_player_by_power(
        &mut self,
        filter: &SelectionRequirement,
        definition: &Arc<TokenDefinition>,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let doomed: Vec<(CardId, usize, i32)> = self
            .resolve_selector(&Selector::EachPermanent(filter.clone()), ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .filter_map(|id| {
                let c = self.battlefield_find(id)?;
                let power = self.computed_permanent(id).map_or(c.power(), |cp| cp.power);
                Some((id, c.controller, power))
            })
            .collect();
        self.run_effect(
            &Effect::Exile { what: Selector::ExactObjects(doomed.iter().map(|d| d.0).collect()) },
            ctx,
            events,
        )?;
        let n = self.players.len();
        let mut totals = vec![0i32; n];
        for (id, seat, power) in &doomed {
            // Left the battlefield this way — a token counts too, though it
            // ceased to exist in exile (CR 111.7).
            let exiled = self.battlefield_find(*id).is_none();
            if exiled && *seat < n {
                totals[*seat] += (*power).max(0);
            }
        }
        for i in 0..n {
            let seat = (ctx.controller + i) % n;
            if !self.players[seat].is_alive() {
                continue;
            }
            let token = Arc::new(
                (**definition).clone().entering_with(CounterType::PlusOnePlusOne, Value::Const(totals[seat])),
            );
            self.run_effect(
                &Effect::CreateToken { who: PlayerRef::Seat(seat), count: Value::ONE, definition: token },
                ctx,
                events,
            )?;
        }
        Ok(())
    }
}
