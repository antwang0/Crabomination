//! CR 732.2 — a shortcut through a finite mandatory drain. Enduring Tenacity
//! and Exquisite Blood trade X life a cycle until the drained seat is dead:
//! no board ever repeats (so the CR 104.4b watch in `stack.rs` never fires)
//! and nothing grows (so `growth_loop.rs` doesn't either), but against a
//! 22,562-life seat the cycles alone ran an eight-seat pod to the action cap
//! (`--pod-decks 3901,5276,901,1843,4744,4931,1048,1498 --seed 1071172`).
//!
//! A trigger chain that outlives the exact watch's period arms this watch: it
//! digests the state *without* life totals and records the life totals each
//! time that digest comes back (a "return"). When the last `m` returns repeat
//! the `m` before them — same resolutions apart, same life moved — the chain
//! is a loop that only moves life, and the controller's proposal (CR 732.2a)
//! is "repeat it until one cycle before a seat would reach 0". The last
//! cycles play out for real, so the death, its SBA and every "when a player
//! loses" trigger happen through the ordinary path.

use super::GameState;

/// The longest cycle the watch can see, in returns of the digest.
const MAX_CYCLE: usize = 8;

/// Resolutions without a return before the watch gives up.
const MAX_GAP: u32 = 4 * GameState::MANDATORY_LOOP_MAX_PERIOD;

#[derive(Clone, Default, Debug)]
pub(crate) struct DrainWatch {
    /// The life-free digest the loop must return to.
    digest: u64,
    /// Resolutions since the last return.
    since: u32,
    /// Each seat's lowest life since the last return.
    low: Vec<i32>,
    /// One row a return, oldest first: (resolutions since the one before,
    /// life totals, lowest totals in between). Row 0 is the arming state.
    returns: Vec<(u32, Vec<i32>, Vec<i32>)>,
}

impl GameState {
    fn drain_digest(&self) -> u64 {
        let head = self.fingerprint_head(true, self.turn_number, false, false);
        self.fingerprint_tail(head, false)
    }

    /// After every top-of-stack resolution. `long_chain` is the exact watch
    /// re-anchoring a trigger chain that ran its whole period without a
    /// repeat — the only thing that arms this watch, so an ordinary game
    /// never writes the cold group here.
    pub(crate) fn note_drain_loop(&mut self, was_trigger: bool, long_chain: bool) {
        if !was_trigger || self.game_over.is_some() {
            if self.drain_watch.is_some() {
                self.drain_watch = None;
            }
            return;
        }
        if self.drain_watch.is_none() {
            // A shared team life total (CR 810.9) is not on `Player::life`.
            if long_chain && self.teams.iter().all(|t| t.shared_life.is_none()) {
                let lives: Vec<i32> = self.players.iter().map(|p| p.life).collect();
                let watch = DrainWatch {
                    digest: self.drain_digest(),
                    low: lives.clone(),
                    returns: vec![(0, lives.clone(), lives)],
                    ..Default::default()
                };
                self.drain_watch = Some(Box::new(watch));
            }
            return;
        }
        let returned = self.drain_watch.as_deref().is_some_and(|w| w.digest == self.drain_digest());
        let lives: Vec<i32> = self.players.iter().map(|p| p.life).collect();
        let Some(w) = self.drain_watch.as_deref_mut() else { return };
        w.since += 1;
        for (low, &l) in w.low.iter_mut().zip(&lives) {
            *low = (*low).min(l);
        }
        if !returned {
            if w.since > MAX_GAP {
                self.drain_watch = None;
            }
            return;
        }
        let low = std::mem::replace(&mut w.low, lives.clone());
        w.returns.push((std::mem::take(&mut w.since), lives, low));
        if w.returns.len() > 2 * MAX_CYCLE + 1 {
            w.returns.remove(0);
        }
        if let Some(m) = repeating_cycle(&w.returns) {
            let watch = self.drain_watch.take().expect("armed");
            self.shortcut_drain(&watch.returns, m);
        }
    }

    /// Repeat the last `m` returns' cycle `k` more times at once, `k` one
    /// short of the first cycle in which a living seat could reach 0 or less.
    /// A loop no live seat dies of (nobody losing, or every loser already at
    /// 0 under Platinum Angel) is not finite and is the other watches'
    /// business.
    fn shortcut_drain(&mut self, returns: &[(u32, Vec<i32>, Vec<i32>)], m: usize) {
        let n = returns.len() - 1;
        let start = &returns[n - m].1;
        let cycle = &returns[n - m + 1..];
        let seats = self.players.len().min(start.len());
        let delta: Vec<i64> = (0..seats).map(|s| i64::from(returns[n].1[s]) - i64::from(start[s])).collect();
        if delta.iter().all(|&d| d == 0) {
            return;
        }
        let k = (0..seats)
            .filter(|&s| delta[s] < 0 && self.players[s].is_alive() && self.players[s].life > 0)
            .map(|s| {
                // The cycle's deepest point below its start: skipping `k`
                // cycles must keep `life + (k - 1) * delta + dip` above 0.
                let dip = cycle.iter().map(|r| i64::from(r.2[s]) - i64::from(start[s])).min().unwrap_or(0).min(0);
                let headroom = i64::from(self.players[s].life) + dip;
                if headroom <= 0 { 0 } else { (headroom - delta[s] - 1) / -delta[s] - 1 }
            })
            .min()
            .unwrap_or(0);
        if k <= 0 {
            return;
        }
        for s in 0..seats {
            // Gross per-turn tallies: each return's net move, summed by sign.
            let (mut gained, mut lost) = (0i64, 0i64);
            for (i, r) in cycle.iter().enumerate() {
                let before = if i == 0 { start[s] } else { cycle[i - 1].1[s] };
                let d = i64::from(r.1[s]) - i64::from(before);
                if d > 0 { gained += d } else { lost -= d }
            }
            let p = &mut self.players[s];
            let total = (delta[s] * k).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
            p.life = p.life.saturating_add(total);
            let tally = |x: i64| u32::try_from(x.saturating_mul(k)).unwrap_or(u32::MAX);
            p.life_gained_this_turn = p.life_gained_this_turn.saturating_add(tally(gained));
            p.life_lost_this_turn = p.life_lost_this_turn.saturating_add(tally(lost));
        }
    }
}

/// The shortest `m` for which the last `m` returns repeat the `m` before them
/// (same spacing, same life moved), if any.
fn repeating_cycle(returns: &[(u32, Vec<i32>, Vec<i32>)]) -> Option<usize> {
    let n = returns.len() - 1;
    let step = |t: usize| -> (u32, Vec<i64>) {
        let (since, now, _) = &returns[t];
        let before = &returns[t - 1].1;
        (*since, now.iter().zip(before).map(|(&a, &b)| i64::from(a) - i64::from(b)).collect())
    };
    (1..=MAX_CYCLE.min(n / 2)).find(|&m| (0..m).all(|i| step(n - i) == step(n - m - i)))
}
