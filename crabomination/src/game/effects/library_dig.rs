//! Lim-Dûl's Vault — dig through the library five cards at a time for 1 life
//! each, then shuffle the rest under the last five.

use super::EffectContext;
use crate::game::GameState;
use crate::game::types::GameEvent;

impl GameState {
    /// `Effect::LookTopFiveDigForLife`. The controller pays 1 life to bottom
    /// the window and look at the next five while the window is unhelpful — no
    /// spell, or no land when they have fewer than four — and their life stays
    /// above 10; never more windows than the library holds. Then the library
    /// below the final window is shuffled and the window stays on top.
    pub(super) fn look_top_five_dig_for_life(&mut self, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let p = ctx.controller;
        let len = self.players[p].library.len();
        if len == 0 {
            return;
        }
        let window = len.min(5);
        let lands_out = self.battlefield.iter().filter(|c| c.controller == p && c.definition.is_land()).count();
        let helpful = |g: &GameState| {
            let top = || g.players[p].library.iter().take(window);
            top().any(|c| !c.definition.is_land()) && (lands_out >= 4 || top().any(|c| c.definition.is_land()))
        };
        let mut digs = 0;
        while digs < len / 5
            && self.players[p].library.len() > window
            && !helpful(self)
            && self.players[p].life > 10
        {
            if !self.replace_life_payment(p, 1, events) {
                let applied = self.adjust_life_applied(p, -1);
                if applied < 0 {
                    events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                }
            }
            for _ in 0..window {
                let card = self.players[p].library.remove(0);
                self.players[p].library.push(card);
            }
            digs += 1;
        }
        let window = window.min(self.players[p].library.len());
        let kept: Vec<_> = (0..window).map(|_| self.players[p].library.remove(0)).collect();
        self.shuffle_library(p, events);
        for (i, card) in kept.into_iter().enumerate() {
            self.players[p].library.insert(i, card);
        }
    }
}
