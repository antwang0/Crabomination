//! CR 118.3 / 603.12 / 608.2c — "you may [pay a cost]. If you do, [payoff]."
//! A player can't choose to pay a cost they can't pay, and the payoff hangs
//! on the payment. An `Effect::MayDo` whose body opens with the cost (a
//! `Seq` of discard-then-draw, sacrifice-then-…) ran the rest of the body
//! after a discard from an empty hand: every "you may discard a card. If you
//! do, draw a card" was a free card with nothing to discard, and the bots
//! took it.

use super::{EffectContext, GameState};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::types::Target;

impl GameState {
    /// The `MayDo` body opens with a fixed cost its controller can't pay in
    /// full — a discard of more cards than they hold, a sacrifice of more
    /// matching permanents than they control, counters its source doesn't
    /// have, a source that's gone — so it isn't offered.
    pub(super) fn may_do_cost_unpayable(&self, body: &Effect, ctx: &EffectContext) -> bool {
        let Effect::Seq(steps) = body else { return false };
        let (Some(cost), true) = (steps.first(), steps.len() > 1) else { return false };
        let you = |who: &Selector| matches!(who, Selector::You | Selector::Player(PlayerRef::You));
        let p = ctx.controller;
        match cost {
            Effect::Discard { who, amount: Value::Const(n), .. } if you(who) => {
                self.players.get(p).is_none_or(|pl| (pl.hand.len() as i64) < i64::from(*n))
            }
            Effect::Sacrifice { who, count: Value::Const(n), filter } if you(who) => {
                let have = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == p
                            && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, ctx.source)
                    })
                    .count();
                (have as i64) < i64::from(*n)
            }
            // "Remove N [kind] counters from this" — fewer than N on it.
            Effect::RemoveCounter { what: Selector::This, kind, amount: Value::Const(n) } => ctx
                .source
                .and_then(|id| self.battlefield_find(id))
                .is_none_or(|c| i64::from(c.counter_count(*kind)) < i64::from(*n)),
            // "Sacrifice this" — it's no longer a permanent you control.
            Effect::SacrificeSource => ctx
                .source
                .and_then(|id| self.battlefield_find(id))
                .is_none_or(|c| c.controller != p),
            // "Exile this [from your graveyard]" — it's gone, or already exiled.
            Effect::Exile { what: Selector::This } => ctx
                .source
                .is_none_or(|id| self.find_card_anywhere(id).is_none() || self.exile.iter().any(|c| c.id == id)),
            _ => false,
        }
    }
}
