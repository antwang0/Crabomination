//! "…and the rest on the bottom of your library in any order" — the player
//! orders the cards an effect just bottomed (`Effect::OrderLibraryBottom`),
//! posed as a `Decision::Scry` in `ScryMode::OrderBottom`.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::{Decision, ScryMode};
use crate::effect::{Effect, PlayerRef, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, PendingEffectState};

impl GameState {
    pub(super) fn order_library_bottom(
        &mut self,
        who: &PlayerRef,
        count: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let lib = &self.players[p].library;
        let n = (self.evaluate_value(count, ctx).max(0) as usize).min(lib.len());
        // One card has one order.
        if n < 2 {
            return Ok(());
        }
        let cards: Vec<(CardId, String)> =
            lib[lib.len() - n..].iter().map(|c| (c.id, c.definition.name.to_string())).collect();
        let decision = Decision::Scry { player: p, cards, mode: ScryMode::OrderBottom };
        let pending = PendingEffectState::BottomOrderPeeked { count: n, player: p };
        if self.seat_prompts(p) {
            self.suspend_signal = Some(Box::new((decision, pending, Effect::Noop)));
            return Ok(());
        }
        let answer = self.decider.decide(&decision);
        events.append(&mut self.apply_pending_effect_answer(pending, &answer)?);
        Ok(())
    }
}
