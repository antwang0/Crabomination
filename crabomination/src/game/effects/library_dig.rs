//! Lim-Dûl's Vault — dig through the library five cards at a time for 1 life
//! each, then shuffle the rest under the last five.

use super::EffectContext;
use crate::game::GameState;
use crate::game::types::GameEvent;

/// Lim-Dûl's Vault's window size.
pub(crate) const VAULT_WINDOW: usize = 5;

impl GameState {
    /// Is the five-card window at `offset` into `seat`'s library worth keeping
    /// — a spell, and a land too while `seat` has fewer than four? The
    /// headless digger's rule, shared with the bot's `DigForLife` answer.
    pub(crate) fn vault_window_helpful(&self, seat: usize, offset: usize) -> bool {
        let lands_out = self.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count();
        let top = || self.players[seat].library.iter().skip(offset).take(VAULT_WINDOW);
        top().any(|c| !c.definition.is_land()) && (lands_out >= 4 || top().any(|c| c.definition.is_land()))
    }

    /// `Effect::LookTopFiveDigForLife`. Before each dig the controller is
    /// asked whether to pay 1 life, bottom the window and look at the next
    /// five (`OptionalKind::DigForLife`; a headless Auto seat digs while the
    /// window is unhelpful and its life stays above 10); never more windows
    /// than the library holds. The asks read the window at its offset, so the
    /// digs happen after the last (a re-run replays them). Then the library
    /// below the final window is shuffled and the window stays on top.
    pub(super) fn look_top_five_dig_for_life(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &crate::effect::Effect,
    ) {
        let p = ctx.controller;
        let len = self.players[p].library.len();
        if len == 0 {
            return;
        }
        let window = len.min(VAULT_WINDOW);
        let ask = self.seat_prompts(p) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        let source = ctx.source.unwrap_or(crate::card::CardId(0));
        let life = self.players[p].life;
        let mut cursor = 0;
        let mut digs = 0usize;
        while digs < len / VAULT_WINDOW && len > window && life - digs as i32 >= 1 {
            let offset = digs * window;
            let more = if ask {
                let Some(b) = self.ask_seat_bool(
                    &mut cursor,
                    p,
                    "Pay 1 life to put these five on the bottom and look at the next five?".into(),
                    source,
                    effect,
                    crate::decision::OptionalKind::DigForLife { offset: offset as u32 },
                ) else {
                    return;
                };
                b
            } else {
                !self.vault_window_helpful(p, offset) && life - digs as i32 > 10
            };
            if !more {
                break;
            }
            digs += 1;
        }
        self.clear_answer_log();
        for _ in 0..digs {
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
        }
        let window = window.min(self.players[p].library.len());
        let kept: Vec<_> = (0..window).map(|_| self.players[p].library.remove(0)).collect();
        self.shuffle_library(p, events);
        for (i, card) in kept.into_iter().enumerate() {
            self.players[p].library.insert(i, card);
        }
    }
}
