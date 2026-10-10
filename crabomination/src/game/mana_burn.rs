//! Yurlok of Scorch Thrash — "a player losing unspent mana causes that player
//! to lose that much life" (mana burn as a static, CR 500.4 / 106.4). Applied
//! by `empty_mana_pools_slow` from each pool's total before and after the
//! empty, so mana a converter keeps (Horizon Stone, Kruphix) isn't lost and
//! burns nothing.

use super::GameState;
use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, Selector, Value};

impl GameState {
    /// Each seat whose pool shrank loses the difference, as a life loss from
    /// `src` (so "whenever a player loses life" listeners see it); the
    /// resulting triggers wait for the next priority (CR 117.5).
    pub(crate) fn burn_unspent_mana(&mut self, src: CardId, ctrl: usize, before: &[u32]) {
        let lost: Vec<(usize, u32)> = before
            .iter()
            .enumerate()
            .filter_map(|(p, &b)| {
                let now = self.players.get(p)?.mana_pool.total();
                (b > now && self.players[p].is_alive()).then_some((p, b - now))
            })
            .collect();
        if lost.is_empty() {
            return;
        }
        let ctx = crate::game::effects::EffectContext::for_ability(src, ctrl, None);
        let mut events = Vec::new();
        for (p, n) in lost {
            let loss = Effect::LoseLife {
                who: Selector::Player(PlayerRef::Seat(p)),
                amount: Value::Const(n.min(i32::MAX as u32) as i32),
            };
            if let Ok(mut evs) = self.resolve_effect_driven(&loss, &ctx) {
                events.append(&mut evs);
            }
        }
        self.dispatch_triggers_for_events(&events);
    }
}
