//! "Remove a time counter from a permanent you control **or suspended card
//! you own**" (Rift Elemental) — the exile half of a
//! `remove_counter_among_filter` cost flagged
//! `remove_counter_among_or_suspended`. The suspended cards' counters count
//! toward the pre-flight total, and the payment drains them first: a time
//! counter off a suspended card brings it closer to being cast, and the last
//! one casts it (CR 702.62a, `remove_suspend_time_counter`).

use super::GameState;
use super::types::GameEvent;
use crate::card::CounterType;

impl GameState {
    /// Time counters on suspended cards `p` owns that a drain of `kinds`
    /// (`None` = any kind) could take.
    pub(crate) fn suspended_drainable_counters(&self, p: usize, kinds: Option<&[CounterType]>) -> u32 {
        if kinds.is_some_and(|ks| !ks.contains(&CounterType::Time)) {
            return 0;
        }
        self.exile
            .iter()
            .filter(|c| c.owner == p && c.is_suspended())
            .map(|c| c.counter_count(CounterType::Time))
            .sum()
    }

    /// Drain up to `left` time counters off `p`'s suspended cards, one at a
    /// time (each removal may cast its card). Returns how many came off.
    pub(crate) fn drain_suspended_counters(
        &mut self,
        p: usize,
        kinds: Option<&[CounterType]>,
        mut left: u32,
        events: &mut Vec<GameEvent>,
    ) -> u32 {
        if self.suspended_drainable_counters(p, kinds) == 0 {
            return 0;
        }
        let start = left;
        // Fewest counters first: the card nearest to being cast.
        let mut cards: Vec<(crate::card::CardId, u32)> = self
            .exile
            .iter()
            .filter(|c| c.owner == p && c.is_suspended())
            .map(|c| (c.id, c.counter_count(CounterType::Time)))
            .collect();
        cards.sort_by_key(|&(_, n)| n);
        for (id, n) in cards {
            for _ in 0..n {
                if left == 0 {
                    return start;
                }
                events.extend(self.remove_suspend_time_counter(id));
                left -= 1;
            }
        }
        start - left
    }
}
