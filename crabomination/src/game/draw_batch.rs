//! "Draw N cards" as ONE instruction (CR 121.2): the replacements that read
//! the whole draw — Alms Collector's "an opponent would draw two or more"
//! (CR 614.1a) — see it once, then each card goes through the per-card draw
//! funnel (`draw_one_or_deck`: dredge, Notion Thief, the empty-library loss).
//! A site that wrote "draw that many" as a loop of single draws hid the
//! batch from Alms Collector (Windfall, Memory Jar, cumulative upkeep).

use super::GameState;
use super::types::GameEvent;

impl GameState {
    /// `p` draws `n` cards as one instruction. Returns false when a draw
    /// ended the game or the loop (as `draw_one_or_deck` does).
    pub(crate) fn draw_n(&mut self, p: usize, n: usize, events: &mut Vec<GameEvent>) -> bool {
        // CR 614.1a — Alms Collector: an opponent's draw of two or more
        // becomes one card each.
        let (n, alms) = match (n >= 2).then(|| self.alms_collector_for(p)).flatten() {
            Some(seat) => (1, Some(seat)),
            None => (n, None),
        };
        if let Some(seat) = alms
            && !self.draw_one_or_deck(seat, events)
        {
            return false;
        }
        // CR 121.2b — a per-turn draw cap truncates the draw.
        let n = match self.draw_cap_for(p) {
            Some(cap) => {
                let remaining = (cap as usize).saturating_sub(self.players[p].cards_drawn_this_turn as usize);
                n.min(remaining)
            }
            None => n,
        };
        for _ in 0..n {
            if !self.draw_one_or_deck(p, events) {
                return false;
            }
        }
        true
    }
}
