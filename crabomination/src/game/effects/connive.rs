//! CR 701.50 — connive N on each selected permanent: its controller draws
//! N, discards N, and it gets a +1/+1 counter per nonland card discarded
//! *this way*. Per-permanent, so "each of X target creatures connive"
//! (Change of Plans) counts every creature's own discards — the
//! `shortcut::connive` Seq reads the whole resolution's discard list.

use crate::card::{CardId, CounterType};
use crate::effect::{Effect, Selector, Value};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::{GameError, GameState};

impl GameState {
    pub(crate) fn connive(
        &mut self,
        what: &Selector,
        amount: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let n = self.evaluate_value(amount, ctx).max(0);
        if n == 0 {
            return Ok(());
        }
        let ids: Vec<CardId> =
            self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_permanent_id()).collect();
        for id in ids {
            // CR 701.50c — a conniver that left still connives; its last
            // controller does the drawing and discarding.
            let controller = self.battlefield_find(id).map_or(ctx.controller, |c| c.controller);
            let sub = EffectContext { controller, targets: vec![Target::Permanent(id)], ..ctx.clone() };
            let before = self.scratch.discarded_card_ids_this_resolution.len();
            self.run_effect(&Effect::Draw { who: Selector::You, amount: Value::Const(n) }, &sub, events)?;
            self.run_effect(&Effect::Discard { who: Selector::You, amount: Value::Const(n), random: false }, &sub, events)?;
            let nonland = self.scratch.discarded_card_ids_this_resolution[before..]
                .iter()
                .filter(|cid| self.find_card_anywhere(**cid).is_some_and(|c| !c.definition.is_land()))
                .count() as i32;
            if nonland > 0 && self.battlefield_find(id).is_some() {
                self.run_effect(
                    &Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(nonland),
                    },
                    &sub,
                    events,
                )?;
            }
            events.push(GameEvent::Connived { card_id: id, controller });
        }
        Ok(())
    }
}
