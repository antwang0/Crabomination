//! CR 701.68 — "Blight N" as an activation cost (Dawnhand Dissident's "{T},
//! Blight 1: Surveil 1."): put N -1/-1 counters on a creature you control.
//! Unpayable with no creature. The pick is automatic, like Ward—Blight's: a
//! creature that survives the counters first, the weakest of those.

use super::GameState;
use super::types::GameEvent;
use crate::card::{CardId, CounterType};

impl GameState {
    /// Whether `p` controls a creature to blight.
    pub(crate) fn can_pay_blight(&self, p: usize) -> bool {
        self.battlefield.iter().any(|c| c.controller == p && self.computed_is_creature(c))
    }

    /// Put `n` -1/-1 counters on the auto-picked creature `p` controls. A
    /// cost that puts counters is still a placement (CR 614.16).
    pub(crate) fn pay_blight(&mut self, p: usize, n: u32, events: &mut Vec<GameEvent>) -> Option<CardId> {
        let pick = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p && self.computed_is_creature(c))
            .filter_map(|c| self.computed_permanent(c.id).map(|cp| (c.id, cp.power, cp.toughness)))
            .min_by_key(|&(_, power, toughness)| (toughness <= n as i32, power))
            .map(|(id, _, _)| id)?;
        let put = self.scaled_counter_count_on(pick, CounterType::MinusOneMinusOne, n);
        if put > 0
            && let Some(c) = self.battlefield_find_mut(pick)
        {
            c.add_counters(CounterType::MinusOneMinusOne, put);
            events.push(GameEvent::CounterAdded {
                card_id: pick,
                counter_type: CounterType::MinusOneMinusOne,
                count: put,
                placer: self.resolution_causer,
            });
        }
        Some(pick)
    }
}
