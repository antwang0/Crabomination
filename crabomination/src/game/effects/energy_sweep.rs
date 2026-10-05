//! A headless seat's "pay any amount of {E}" when the payoff is damage to
//! each creature (Territorial Aetherkite): the amount that kills the most
//! opposing mana value net of its own, the smallest such, instead of all of
//! it (which can wipe its own board).

use super::EffectContext;
use crate::effect::{Effect, Selector, Value};
use crate::game::GameState;

impl GameState {
    /// The amount to pay, or `None` when `then` is no each-creature sweep
    /// scaled by the payment (the caller pays everything, as before).
    pub(super) fn headless_energy_sweep(&self, then: &Effect, avail: u32, ctx: &EffectContext) -> Option<u32> {
        let Effect::DealDamage { to: Selector::EachPermanent(filter), amount: Value::EnergyPaidThisEffect } = then
        else {
            return None;
        };
        let me = ctx.controller;
        let hit: Vec<(i32, bool, i64)> = self
            .resolve_selector(&Selector::EachPermanent(filter.clone()), ctx)
            .into_iter()
            .filter_map(|e| {
                let c = self.battlefield_find(e.as_permanent_id()?)?;
                let cp = self.computed_permanent(c.id)?;
                let left = (cp.toughness - c.damage as i32).max(0);
                let value = i64::from(c.definition.cost.cmc()) + 1;
                Some((left, self.same_team(c.controller, me), value))
            })
            .collect();
        let score = |n: u32| -> i64 {
            hit.iter().filter(|h| h.0 <= n as i32).map(|h| if h.1 { -h.2 } else { h.2 }).sum()
        };
        // Strictly better only: ties keep the smaller payment.
        Some((0..=avail).fold((0, score(0)), |best, n| if score(n) > best.1 { (n, score(n)) } else { best }).0)
    }
}
