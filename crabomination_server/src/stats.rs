//! Rolling match statistics and the human-readable log summaries.

use std::time::Duration;

use crate::config::Format;
use crabomination::server::LossReason;

/// Process-wide running counters of completed matches. Lets the server
/// emit a rolling summary line ("served 42 matches: 31 bot, 11 pair;
/// avg duration 4m13s") on each match completion alongside the per-match
/// line. Updated by `run_bot_match` / `run_pair_match`; read in those
/// same logging sites.
///
/// The struct holds raw totals; the formatted summary lives in
/// `format_match_stats`. Wrapped in a `Mutex` so concurrent match
/// threads can serialize their updates without an `Arc` allocation per
/// thread (the SlotManager pattern uses `Arc<Mutex<…>>` because slot
/// state has to outlive multiple owning threads; `MATCH_STATS` is a
/// process-global `OnceLock` so a plain `Mutex<MatchStats>` suffices).
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct MatchStats {
    bot_matches: u64,
    pair_matches: u64,
    /// Total cumulative match duration (sum). Average = total / count.
    total_duration: Duration,
    /// Shortest observed match duration. `None` until the first match
    /// completes. Surfaces outlier-short games (instant disconnects,
    /// concession-on-turn-1).
    min_duration: Option<Duration>,
    /// Longest observed match duration. `None` until the first match
    /// completes. Surfaces stalls / long grindy games.
    max_duration: Option<Duration>,
    /// Bucketed histogram of match durations. Buckets:
    /// `[0]` = under 30s, `[1]` = 30s-1m, `[2]` = 1-2m, `[3]` = 2-5m,
    /// `[4]` = 5-10m, `[5]` = 10m+. Lets operators see the distribution
    /// shape at a glance without leaving the per-match log line —
    /// e.g. a sudden spike in the `<30s` bucket indicates many bots
    /// are conceding turn 1 (often a regression signal).
    duration_buckets: [u32; 6],
    /// Per-format histogram of completed matches, indexed by the local
    /// server `Format` discriminant (Demo / Cube). Lets operators see
    /// the running cube-vs-demo split in the rolling summary line. Push
    /// (claude/modern_decks batch 162).
    format_buckets: [u64; FORMAT_BUCKET_COUNT],
    /// Cumulative turn count across all matches — divided by total
    /// matches in the summary line. Operators see at a glance whether
    /// games are concession-heavy (low avg turn count) or grindy
    /// (high avg turn count) without sampling individual match logs.
    /// Push (claude/modern_decks batch 172).
    total_turns: u64,
    /// Longest observed final turn count across all completed matches.
    /// Surfaces "grindiest" games for outlier debugging — paired with
    /// `total_turns / total_matches` (the running average) lets operators
    /// distinguish "consistent 8-turn games" from "5-turn average with
    /// one 30-turn outlier". `None` until the first match completes.
    /// Push (claude/modern_decks batch 189).
    max_turns: Option<u32>,
    /// Shortest observed final turn count across all completed matches.
    /// Paired with `max_turns` and the running average, this completes
    /// the turn-count envelope so operators can distinguish a tight
    /// "always 6-8 turn" distribution from a wide "2-turn concession to
    /// 30-turn grind" spread without sampling individual match logs.
    /// `None` until the first match completes. Push
    /// (claude/modern_decks batch 205).
    min_turns: Option<u32>,
    /// Number of matches that ended in a draw (MatchOutcome.winner =
    /// Some(None)). Useful for spotting "stalemate" regressions
    /// (typically a bot-vs-bot loop where neither side can finish).
    /// Push (claude/modern_decks batches 192-194).
    draws: u64,
    /// Number of matches that ended cleanly with a declared winner
    /// (Some(Some(seat))). Pre-game-over exits (channel disconnect,
    /// watchdog) yield None and are excluded from this counter, so
    /// `wins + draws ≤ total_matches`. The delta surfaces "stuck"
    /// matches that never produced an outcome. Push
    /// (claude/modern_decks batches 192-194).
    wins: u64,
    /// Per-seat wins (indexed by seat 0..SEAT_BUCKET_COUNT). Surfaces
    /// turn-order bias in bot-vs-bot ladders: if `seat_wins[0]` is
    /// twice `seat_wins[1]` over a long run, the active-player heuristic
    /// or starting-hand luck is leaking through. Pre-warmup all-zero
    /// rows render as `seat_wins=0/0` in the rolling summary so the
    /// operator can spot empty samples. Push (claude/modern_decks
    /// batch 198).
    seat_wins: [u64; SEAT_BUCKET_COUNT],
    /// Cumulative life delta on wins: for each completed match with a
    /// winner, sum `winner_life - max(other_seat_life, 0)`. Divided by
    /// `wins` in the summary line gives an average win-by-life number,
    /// surfacing whether games are "blowouts" (high delta) or "races"
    /// (low / negative delta — winner ended at 1 with opp at 0).
    /// Saturates positive; clamped at zero when winner life is below
    /// the negative of the opp's. Push (claude/modern_decks batch 202).
    cumulative_win_life_delta: i64,
    /// Σ of (win-life-delta)² across sampled wins, for the population
    /// standard deviation (σ = √(E[x²] − E[x]²)). Paired with the average,
    /// σ distinguishes a consistent "win-by-5" meta from a bimodal
    /// "blowout-or-squeaker" split the average alone hides. Deltas are
    /// clamped ≥ 0, so the squared sum is non-negative; `u128` headroom
    /// keeps it from overflowing over a long run.
    cumulative_win_life_delta_squared: u128,
    /// Number of matches counted in `cumulative_win_life_delta`. Lets
    /// the formatter compute the average without dividing by `wins`
    /// directly (a winner with no available life data — e.g. a forced
    /// concession — is skipped in the cumulative sum but still counted
    /// in `wins`).
    win_life_samples: u64,
    /// Bucketed histogram of win-by-life deltas, parallel to
    /// `duration_buckets`. Buckets: `[0]` = 0 (won at parity / race),
    /// `[1]` = 1-3, `[2]` = 4-6, `[3]` = 7-10, `[4]` = 11-15, `[5]` = 16+.
    /// The average + σ give the centre and spread; this gives the
    /// distribution *shape*, and feeds a median (p50) estimate that's
    /// robust to the blowout outliers the mean is sensitive to.
    win_life_delta_buckets: [u32; 6],
    /// Bucketed histogram of final turn counts, parallel to
    /// `duration_buckets`. Buckets: `[0]` = 1-2 turns, `[1]` = 3-5,
    /// `[2]` = 6-8, `[3]` = 9-12, `[4]` = 13-20, `[5]` = 21+. The
    /// turn-count envelope (`min_turns`/`max_turns`/average) gives the
    /// extremes and centre; this histogram gives the distribution
    /// *shape* — e.g. a fat `[0]` bucket flags a concession regression
    /// even when one long outlier keeps the average high. Mirrors the
    /// duration histogram so operators read both in the same summary
    /// line. Push (claude/modern_decks).
    turn_buckets: [u32; 6],
    /// Number of clean wins where every losing seat ended with life > 0
    /// — i.e. the loser did *not* die to lethal face damage. These are
    /// "alternate" wins (decking out, poison, mill, or a win-the-game
    /// effect). Surfaced next to `wins` so operators can see the
    /// damage-vs-alternate win split: a sudden rise in `deckout_wins`
    /// relative to `wins` flags a stall regression where bots grind to
    /// empty libraries instead of closing on life. Counted only on
    /// `Some(Some(seat))` outcomes with available life data. Push
    /// (claude/modern_decks).
    deckout_wins: u64,
    /// Subset of `deckout_wins` where at least one losing seat was
    /// eliminated specifically by poison (CR 104.3c). Classified from the
    /// outcome's precise `loss_reasons`, not the life-total heuristic, so
    /// poison ladders show a distinct signal from pure deck-out grinds.
    poison_wins: u64,
    /// Subset of `deckout_wins` where at least one losing seat decked out
    /// (drew from an empty library, CR 104.3a). The dredge/mill shells push
    /// this bucket; reading it next to `poison_wins` splits the umbrella
    /// non-damage win count into its two main alternate paths.
    deck_wins: u64,
    /// Subset of `deckout_wins` where at least one losing seat was killed by
    /// 21+ combat damage from a single commander (CR 903.10a). Relevant to
    /// the Commander/Brawl formats; reads alongside `poison_wins`/`deck_wins`
    /// as a third distinct alternate-win path.
    commander_damage_wins: u64,
    /// Running sum of squared final turn counts (`Σ turns²`). Paired with
    /// `total_turns` (`Σ turns`) and the match count it yields the
    /// population standard deviation of game length via
    /// [`turn_count_stddev`](Self::turn_count_stddev) — a single number
    /// that tells operators whether games cluster tightly around the
    /// average (small σ) or swing wildly (large σ), complementing the
    /// min/max envelope and the histogram shape.
    total_turns_squared: u128,
    /// Running sum of squared match durations in **milliseconds**
    /// (`Σ ms²`). The duration analogue of `total_turns_squared`: paired
    /// with `total_duration` (`Σ ms`) and the match count it yields the
    /// population standard deviation of match length via
    /// [`duration_stddev`](Self::duration_stddev), so the rolling summary
    /// reports duration σ next to the turn-count σ. Milliseconds keep the
    /// squares well within `u128` even for very long sessions.
    total_duration_squared_ms: u128,
    /// Matches that completed without a declared outcome (`observe_winner`
    /// got `None` — channel disconnect, watchdog kill, or a stuck loop). The
    /// `wins + draws + inconclusive ≤ total_matches` identity makes this the
    /// explicit count of the "stuck" delta operators previously had to derive
    /// by subtraction; a rising `inconclusive_pct` flags a hang regression.
    inconclusive: u64,
}

/// Cap on per-seat win tracking. Covers 1v1 (seats 0, 1) plus headroom
/// for 4-player Commander pods. Wins for seats ≥ this cap fall into
/// the last bucket so the array doesn't overflow on exotic formats.
pub(crate) const SEAT_BUCKET_COUNT: usize = 4;

/// Number of buckets in `MatchStats.format_buckets`. Sized to cover the
/// the four current `Format` variants (Demo / Cube / Sos / Commander).
/// New formats slot into the next free index via `format_index`; bump
/// this count when adding a fifth.
pub(crate) const FORMAT_BUCKET_COUNT: usize = 4;

/// Map a local server `Format` (Demo / Cube / Sos / Commander) to its bucket index in
/// `MatchStats.format_buckets`. Stable ordering — new formats append.
pub(crate) fn format_index(f: Format) -> usize {
    match f {
        Format::Demo => 0,
        Format::Cube => 1,
        Format::Sos => 2,
        Format::Commander => 3,
    }
}

/// Reverse map for the format-bucket index. Returns `None` for the
/// trailing reserved slots so the formatter can skip empty buckets.
pub(crate) fn format_label_for_bucket(i: usize) -> Option<&'static str> {
    match i {
        0 => Some(Format::Demo.label()),
        1 => Some(Format::Cube.label()),
        2 => Some(Format::Sos.label()),
        3 => Some(Format::Commander.label()),
        _ => None,
    }
}

impl MatchStats {
    pub(crate) fn record_bot(&mut self, d: Duration, f: Format) {
        self.bot_matches += 1;
        self.observe_duration(d);
        self.observe_format(f);
    }
    pub(crate) fn record_pair(&mut self, d: Duration, f: Format) {
        self.pair_matches += 1;
        self.observe_duration(d);
        self.observe_format(f);
    }
    /// Bump the cumulative turn counter — called at match completion
    /// from the record paths if the caller has a final turn number.
    /// Defensive against double-counting since this is invoked exactly
    /// once per `record_*` (the caller passes the final turn).
    pub(crate) fn observe_turns(&mut self, turns: u32) {
        self.total_turns = self.total_turns.saturating_add(turns as u64);
        self.total_turns_squared = self
            .total_turns_squared
            .saturating_add((turns as u128) * (turns as u128));
        self.max_turns = Some(match self.max_turns {
            None => turns,
            Some(m) => m.max(turns),
        });
        self.min_turns = Some(match self.min_turns {
            None => turns,
            Some(m) => m.min(turns),
        });
        let idx = Self::turn_bucket_index(turns);
        self.turn_buckets[idx] = self.turn_buckets[idx].saturating_add(1);
    }

    /// Map a final turn count to its `turn_buckets` index.
    /// `[0]` = 1-2, `[1]` = 3-5, `[2]` = 6-8, `[3]` = 9-12, `[4]` =
    /// 13-20, `[5]` = 21+.
    pub(crate) fn turn_bucket_index(turns: u32) -> usize {
        match turns {
            0..=2 => 0,
            3..=5 => 1,
            6..=8 => 2,
            9..=12 => 3,
            13..=20 => 4,
            _ => 5,
        }
    }

    /// Human-readable label for each `turn_buckets` index.
    pub(crate) fn turn_bucket_label(i: usize) -> &'static str {
        match i {
            0 => "1-2",
            1 => "3-5",
            2 => "6-8",
            3 => "9-12",
            4 => "13-20",
            _ => "21+",
        }
    }
    /// Bump the win/draw counters based on the MatchOutcome.winner
    /// shape. `None` (pre-game-over exit — channel disconnect or
    /// watchdog) is silently dropped: callers can compute "stuck"
    /// matches as `total_matches - wins - draws`. `Some(None)` is a
    /// draw; `Some(Some(_))` is a clean win.
    pub(crate) fn observe_winner(&mut self, w: Option<Option<usize>>) {
        match w {
            Some(None) => self.draws = self.draws.saturating_add(1),
            Some(Some(seat)) => {
                self.wins = self.wins.saturating_add(1);
                let idx = seat.min(SEAT_BUCKET_COUNT - 1);
                self.seat_wins[idx] = self.seat_wins[idx].saturating_add(1);
            }
            None => self.inconclusive = self.inconclusive.saturating_add(1),
        }
    }
    /// Percentage of completed matches that produced no declared outcome
    /// (stuck / disconnected). Surfaced next to `decisive_pct` so a hang
    /// regression is visible directly rather than by subtraction.
    pub(crate) fn inconclusive_pct(&self) -> u64 {
        let total = self.total_matches();
        if total == 0 { return 0; }
        self.inconclusive.saturating_mul(100) / total
    }
    /// Share of decisive (non-draw) wins taken by the player on the play
    /// (seat 0), as a percentage. A value far from 50 over a long bot ladder
    /// flags turn-order bias in the active-player heuristic — the
    /// `seat_wins` histogram's stated purpose, surfaced without mental math.
    /// Returns 50 (neutral) when no seated wins have been recorded.
    pub(crate) fn first_seat_win_pct(&self) -> u64 {
        let seated: u64 = self.seat_wins.iter().sum();
        if seated == 0 { return 50; }
        self.seat_wins[0].saturating_mul(100) / seated
    }
    /// Accumulate the win-by-life delta for one match. `final_life`
    /// is the per-seat life array; `winner` is the winning seat. The
    /// delta is `winner_life - max_opponent_life` clamped to ≥0 so
    /// the cumulative sum can't go negative even if both ended at
    /// negative life (rare double-loss scenario). Skipped silently
    /// when the winning seat is out of range or no life data is
    /// available. Push (claude/modern_decks batch 202).
    pub(crate) fn observe_win_life_delta(&mut self, winner: usize, final_life: &[i32]) {
        let Some(&winner_life) = final_life.get(winner) else { return };
        let max_opp = final_life
            .iter()
            .enumerate()
            .filter_map(|(i, &l)| (i != winner).then_some(l))
            .max()
            .unwrap_or(0);
        let delta = (winner_life - max_opp).max(0) as i64;
        self.cumulative_win_life_delta =
            self.cumulative_win_life_delta.saturating_add(delta);
        self.cumulative_win_life_delta_squared = self
            .cumulative_win_life_delta_squared
            .saturating_add((delta as u128) * (delta as u128));
        self.win_life_samples = self.win_life_samples.saturating_add(1);
        let b = Self::win_life_delta_bucket_index(delta);
        self.win_life_delta_buckets[b] = self.win_life_delta_buckets[b].saturating_add(1);
    }
    /// Partition a (clamped ≥0) win-by-life delta into one of six buckets.
    pub(crate) fn win_life_delta_bucket_index(delta: i64) -> usize {
        match delta {
            0 => 0,
            1..=3 => 1,
            4..=6 => 2,
            7..=10 => 3,
            11..=15 => 4,
            _ => 5,
        }
    }
    /// Representative upper bound of win-life-delta bucket `i` (the open
    /// final bucket reports its lower edge, 16). Mirrors the duration /
    /// turn `*_upper_bound` helpers.
    pub(crate) fn win_life_delta_bucket_upper_bound(i: usize) -> i64 {
        match i {
            0 => 0,
            1 => 3,
            2 => 6,
            3 => 10,
            4 => 15,
            _ => 16,
        }
    }
    /// Estimate the median (p50) win-by-life delta from the histogram —
    /// robust to the blowout outliers that inflate the mean. Returns the
    /// upper edge of the bucket holding the median sample, or 0 with no
    /// samples.
    pub(crate) fn win_life_delta_median(&self) -> i64 {
        if self.win_life_samples == 0 {
            return 0;
        }
        let target = self.win_life_samples.div_ceil(2);
        let mut acc = 0u64;
        for (i, &n) in self.win_life_delta_buckets.iter().enumerate() {
            acc = acc.saturating_add(n as u64);
            if acc >= target {
                return Self::win_life_delta_bucket_upper_bound(i);
            }
        }
        Self::win_life_delta_bucket_upper_bound(5)
    }
    /// Classify one clean win as a damage win or an "alternate" win
    /// (deckout / poison / mill / win-the-game). Prefers the outcome's
    /// precise per-seat `loss_reasons`; if any losing seat died to
    /// something other than lethal face damage, the win is "alternate"
    /// (`deckout_wins`), and poison / deck-out losses additionally bump
    /// the `poison_wins` / `deck_wins` sub-buckets. Falls back to the
    /// life-total heuristic when reason data is unavailable.
    /// Push (claude/modern_decks).
    pub(crate) fn observe_win_kind(
        &mut self,
        winner: usize,
        final_life: &[i32],
        loss_reasons: &[Option<LossReason>],
    ) {
        // Precise path: classify from the per-seat loss reasons.
        let reasons: Vec<LossReason> = loss_reasons
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != winner)
            .filter_map(|(_, r)| *r)
            .collect();
        if !reasons.is_empty() {
            let any_alternate = reasons.iter().any(|r| *r != LossReason::LifeDepleted);
            if any_alternate {
                self.deckout_wins = self.deckout_wins.saturating_add(1);
            }
            if reasons.contains(&LossReason::Poison) {
                self.poison_wins = self.poison_wins.saturating_add(1);
            }
            if reasons.contains(&LossReason::Decked) {
                self.deck_wins = self.deck_wins.saturating_add(1);
            }
            if reasons.contains(&LossReason::CommanderDamage) {
                self.commander_damage_wins = self.commander_damage_wins.saturating_add(1);
            }
            return;
        }
        // Fallback: no reason data → infer from life totals (every losing
        // seat above 0 means the win wasn't lethal face damage).
        let mut losers = final_life
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != winner)
            .map(|(_, &l)| l)
            .peekable();
        if losers.peek().is_none() {
            return;
        }
        if losers.all(|l| l > 0) {
            self.deckout_wins = self.deckout_wins.saturating_add(1);
        }
    }
    /// Average win-by-life delta across all sampled wins. Returns 0
    /// when no win-life samples have been recorded yet.
    pub(crate) fn avg_win_life_delta(&self) -> i64 {
        if self.win_life_samples == 0 {
            0
        } else {
            self.cumulative_win_life_delta / (self.win_life_samples as i64)
        }
    }
    /// Population standard deviation of the win-by-life delta (σ = √(E[x²] −
    /// E[x]²)). Returns 0.0 with no samples. A tight σ next to the average
    /// means a consistent win margin; a large σ flags a "blowout-or-squeaker"
    /// split the average hides.
    pub(crate) fn win_life_delta_stddev(&self) -> f32 {
        if self.win_life_samples == 0 {
            return 0.0;
        }
        let n = self.win_life_samples as f64;
        let mean = self.cumulative_win_life_delta as f64 / n;
        let mean_sq = self.cumulative_win_life_delta_squared as f64 / n;
        (mean_sq - mean * mean).max(0.0).sqrt() as f32
    }
    /// Percent of *resolved* matches (wins + draws) that ended decisively
    /// (i.e. had a winner). Returns 0 when nothing has resolved yet. A
    /// sudden drop signals stalemate regressions (mutual lock, no win
    /// condition reachable). Excludes unresolved/watchdog'd matches from
    /// the denominator so disconnects don't deflate the rate.
    pub(crate) fn decisive_pct(&self) -> u64 {
        let resolved = self.wins + self.draws;
        if resolved == 0 {
            0
        } else {
            self.wins.saturating_mul(100) / resolved
        }
    }
    /// Percent of wins that closed via something other than lethal face
    /// damage (deckout / poison / mill / win-the-game). Returns 0 when no
    /// wins have been recorded. A rising share flags a stall regression
    /// where bots grind to empty libraries instead of closing on life.
    pub(crate) fn deckout_pct(&self) -> u64 {
        if self.wins == 0 {
            0
        } else {
            self.deckout_wins.saturating_mul(100) / self.wins
        }
    }
    /// Percent of wins in which a losing seat died to poison (CR 104.3c).
    /// A sub-split of `deckout_pct`; 0 when no wins recorded.
    pub(crate) fn poison_pct(&self) -> u64 {
        if self.wins == 0 { 0 } else { self.poison_wins.saturating_mul(100) / self.wins }
    }
    /// Percent of wins in which a losing seat decked out (CR 104.3a).
    /// A sub-split of `deckout_pct`; 0 when no wins recorded.
    pub(crate) fn deck_pct(&self) -> u64 {
        if self.wins == 0 { 0 } else { self.deck_wins.saturating_mul(100) / self.wins }
    }
    /// Percent of wins via 21+ commander damage (CR 903.10a).
    /// A sub-split of `deckout_pct`; 0 when no wins recorded.
    pub(crate) fn commander_damage_pct(&self) -> u64 {
        if self.wins == 0 { 0 } else { self.commander_damage_wins.saturating_mul(100) / self.wins }
    }
    /// Average turn count across all completed matches. Returns 0
    /// pre-warmup. Used by `format_match_stats` for the operator
    /// rolling-summary line.
    pub(crate) fn avg_turns(&self) -> u64 {
        let n = self.total_matches();
        self.total_turns.checked_div(n).unwrap_or(0)
    }
    /// Increment the per-format match count. Used by both `record_bot`
    /// and `record_pair` so the per-format histogram covers every
    /// completed match regardless of source.
    pub(crate) fn observe_format(&mut self, f: Format) {
        let idx = format_index(f).min(FORMAT_BUCKET_COUNT - 1);
        self.format_buckets[idx] = self.format_buckets[idx].saturating_add(1);
    }
    /// Shared bookkeeping for both record paths — accumulates the
    /// total + tracks the new min/max envelope. Pulled out of the
    /// recorders so the min/max maintenance is canonical at one site.
    pub(crate) fn observe_duration(&mut self, d: Duration) {
        self.total_duration += d;
        self.min_duration = Some(match self.min_duration {
            None => d,
            Some(m) => m.min(d),
        });
        self.max_duration = Some(match self.max_duration {
            None => d,
            Some(m) => m.max(d),
        });
        let idx = Self::bucket_index(d);
        self.duration_buckets[idx] = self.duration_buckets[idx].saturating_add(1);
        let ms = d.as_millis();
        self.total_duration_squared_ms = self.total_duration_squared_ms.saturating_add(ms * ms);
    }
    /// Estimate the `p`th-percentile match duration from the histogram.
    /// `p` is a fraction in `[0.0, 1.0]`. Returns the upper edge of the
    /// bucket containing the `p`-th sample (rounded up), so the estimate
    /// is conservative — an actual median match may be shorter, but
    /// reporting `≤ this` gives operators a useful upper bound on the
    /// typical match length. Returns `Duration::ZERO` if no matches have
    /// been recorded.
    ///
    /// The bucketing is coarse (6 buckets) so this is a *quantile-class*
    /// rather than a true percentile — but enough for spotting drift in
    /// match-length distribution shape over time. Used by
    /// `format_match_stats` to surface `p50` and `p95` in the rolling
    /// summary line.
    pub(crate) fn percentile(&self, p: f32) -> Duration {
        let total = self.total_matches();
        if total == 0 {
            return Duration::ZERO;
        }
        let p = p.clamp(0.0, 1.0);
        // Target rank — 1-indexed so p=1.0 selects the last sample.
        let target = (total as f32 * p).ceil().max(1.0) as u64;
        let mut acc = 0u64;
        for (i, &n) in self.duration_buckets.iter().enumerate() {
            acc = acc.saturating_add(n as u64);
            if acc >= target {
                // Return the upper bound of bucket `i`.
                return Self::bucket_upper_bound(i);
            }
        }
        // Shouldn't reach here when total > 0, but fall back to the
        // open-ended bucket's nominal upper bound.
        Self::bucket_upper_bound(5)
    }
    /// Turn-count analogue of [`percentile`](Self::percentile): the
    /// upper turn-count bound of the bucket containing the `p`-th
    /// percentile match, walking `turn_buckets`. Returns 0 when no
    /// matches have completed. Lets operators read the game-length
    /// distribution centre (p50) and tail (p95) directly instead of
    /// eyeballing the histogram columns.
    pub(crate) fn turn_percentile(&self, p: f32) -> u32 {
        let total = self.total_matches();
        if total == 0 {
            return 0;
        }
        let p = p.clamp(0.0, 1.0);
        let target = (total as f32 * p).ceil().max(1.0) as u64;
        let mut acc = 0u64;
        for (i, &n) in self.turn_buckets.iter().enumerate() {
            acc = acc.saturating_add(n as u64);
            if acc >= target {
                return Self::turn_bucket_upper_bound(i);
            }
        }
        Self::turn_bucket_upper_bound(5)
    }
    /// Population standard deviation of final turn counts, computed from
    /// the running `Σ turns` and `Σ turns²` accumulators (σ = √(E[x²] −
    /// E[x]²)). Returns 0.0 when no matches have completed. A small σ next
    /// to the average means consistent game lengths; a large σ flags a
    /// bimodal "fast concession vs. long grind" split the average alone
    /// hides.
    pub(crate) fn turn_count_stddev(&self) -> f32 {
        let n = self.total_matches();
        if n == 0 {
            return 0.0;
        }
        let n = n as f64;
        let mean = self.total_turns as f64 / n;
        let mean_sq = self.total_turns_squared as f64 / n;
        (mean_sq - mean * mean).max(0.0).sqrt() as f32
    }
    /// Population standard deviation of match durations, computed from the
    /// running `Σ ms` (`total_duration`) and `Σ ms²`
    /// (`total_duration_squared_ms`) accumulators (σ = √(E[x²] − E[x]²)),
    /// returned as a [`Duration`]. Returns `Duration::ZERO` when no matches
    /// have completed. The duration analogue of
    /// [`turn_count_stddev`](Self::turn_count_stddev): a tight σ next to the
    /// average means consistent match lengths; a large σ flags a "fast
    /// concession vs. long grind" split the average alone hides.
    pub(crate) fn duration_stddev(&self) -> Duration {
        let n = self.total_matches();
        if n == 0 {
            return Duration::ZERO;
        }
        let n = n as f64;
        let mean = self.total_duration.as_millis() as f64 / n;
        let mean_sq = self.total_duration_squared_ms as f64 / n;
        let var = (mean_sq - mean * mean).max(0.0);
        Duration::from_millis(var.sqrt() as u64)
    }
    /// Upper edge (inclusive estimate) of turn bucket `i`. Matches the
    /// cut points in [`turn_bucket_index`](Self::turn_bucket_index); the
    /// open-ended `21+` bucket reports its lower edge (21) since it has
    /// no finite upper bound.
    pub(crate) fn turn_bucket_upper_bound(i: usize) -> u32 {
        match i {
            0 => 2,
            1 => 5,
            2 => 8,
            3 => 12,
            4 => 20,
            _ => 21,
        }
    }
    /// Upper edge (inclusive estimate) of bucket `i` for percentile
    /// reporting. Matches the cut points in `bucket_index`.
    pub(crate) fn bucket_upper_bound(i: usize) -> Duration {
        match i {
            0 => Duration::from_secs(30),
            1 => Duration::from_secs(60),
            2 => Duration::from_secs(120),
            3 => Duration::from_secs(300),
            4 => Duration::from_secs(600),
            _ => Duration::from_secs(3600),
        }
    }
    /// Map a duration onto its histogram bucket index. Buckets are
    /// power-of-rounded thresholds: 30s / 1m / 2m / 5m / 10m / 10m+.
    /// Anything strictly less than 30s lands in bucket 0; bucket 5
    /// is the open-ended `10m+` catch-all.
    pub(crate) fn bucket_index(d: Duration) -> usize {
        let s = d.as_secs();
        if s < 30 {
            0
        } else if s < 60 {
            1
        } else if s < 120 {
            2
        } else if s < 300 {
            3
        } else if s < 600 {
            4
        } else {
            5
        }
    }
    /// Human-readable labels for the histogram buckets, parallel to
    /// `duration_buckets`. Pulled out so the formatter and unit tests
    /// can share the same labels.
    pub(crate) fn bucket_label(i: usize) -> &'static str {
        match i {
            0 => "<30s",
            1 => "30s-1m",
            2 => "1-2m",
            3 => "2-5m",
            4 => "5-10m",
            _ => "10m+",
        }
    }
    pub(crate) fn total_matches(&self) -> u64 {
        self.bot_matches + self.pair_matches
    }
    pub(crate) fn avg_duration(&self) -> Duration {
        let n = self.total_matches();
        if n == 0 {
            Duration::ZERO
        } else {
            // saturating: the wrap-protection guard for the absurd "u64
            // overflow" case in match counts (would need centuries of
            // continuous play to hit).
            Duration::from_secs(self.total_duration.as_secs().saturating_div(n))
        }
    }
}

pub(crate) static MATCH_STATS: std::sync::OnceLock<std::sync::Mutex<MatchStats>> = std::sync::OnceLock::new();

pub(crate) fn match_stats() -> &'static std::sync::Mutex<MatchStats> {
    MATCH_STATS.get_or_init(|| std::sync::Mutex::new(MatchStats::default()))
}

/// Format the running stats as a one-line summary appended to each
/// match-completion log: `served N matches: K bot, P pair; avg
/// duration X`. Read after the per-match update so the new match is
/// included in the rollup.
pub(crate) fn format_match_stats(s: &MatchStats) -> String {
    let n = s.total_matches();
    let mut out = format!(
        "served {} match{}: {} bot, {} pair; avg duration {}; avg turns {}",
        n,
        if n == 1 { "" } else { "es" },
        s.bot_matches,
        s.pair_matches,
        format_duration(s.avg_duration()),
        s.avg_turns(),
    );
    match (s.min_turns, s.max_turns) {
        (Some(mn), Some(mx)) if mn != mx => {
            out.push_str(&format!(" (turns {mn}-{mx})"));
        }
        (Some(_), Some(mx)) => {
            // Only one distinct value observed so far — show it as max.
            out.push_str(&format!(" (max turns {mx})"));
        }
        _ => {}
    }
    // Win/draw split: only render once at least one win or draw is
    // recorded so pre-warmup logs stay tight. The delta vs total
    // matches surfaces "stuck" matches (channel disconnect /
    // watchdog) — `total - wins - draws` is the unresolved count.
    if s.wins + s.draws > 0 {
        out.push_str(&format!(
            " wins={} draws={} decisive={}%",
            s.wins, s.draws, s.decisive_pct()
        ));
        // Alternate-win split: how many of those wins closed via
        // something other than lethal face damage (deckout / poison /
        // mill / win-the-game). Only rendered when at least one such
        // win has been seen so the common all-damage case stays tight.
        if s.deckout_wins > 0 {
            out.push_str(&format!(" alt_wins={} ({}%)", s.deckout_wins, s.deckout_pct()));
            // Split the alternate-win share into its two main paths when seen.
            if s.poison_wins > 0 {
                out.push_str(&format!(" poison={} ({}%)", s.poison_wins, s.poison_pct()));
            }
            if s.deck_wins > 0 {
                out.push_str(&format!(" deck={} ({}%)", s.deck_wins, s.deck_pct()));
            }
            if s.commander_damage_wins > 0 {
                out.push_str(&format!(
                    " cmdr_dmg={} ({}%)",
                    s.commander_damage_wins,
                    s.commander_damage_pct()
                ));
            }
        }
        // Stuck/disconnected matches: prefer the explicit `inconclusive`
        // counter (and its percentage) over the subtraction fallback so a
        // hang regression reads directly off the summary line.
        if s.inconclusive > 0 {
            out.push_str(&format!(
                " unresolved={} ({}%)",
                s.inconclusive,
                s.inconclusive_pct()
            ));
        }
        // Per-seat win histogram: " seat_wins=12/8/0/0" (only render
        // up to the highest non-zero seat so 1v1 doesn't surface
        // padding zeros for the 4-player tail).
        let last_nonzero = s
            .seat_wins
            .iter()
            .rposition(|&n| n > 0)
            .unwrap_or(0);
        let parts: Vec<String> = s.seat_wins[..=last_nonzero]
            .iter()
            .map(|w| w.to_string())
            .collect();
        out.push_str(&format!(" seat_wins={}", parts.join("/")));
        // First-player win share among decisive wins — turn-order-bias gauge.
        // Only meaningful once both seats have had a chance to win.
        if last_nonzero >= 1 {
            out.push_str(&format!(" (p0={}%)", s.first_seat_win_pct()));
        }
        // Average winning-seat life delta — "blowout" check. A high value
        // (12+) means the winner cruised; near-zero values mean games
        // ended in a race. Push (claude/modern_decks batch 202).
        if s.win_life_samples > 0 {
            out.push_str(&format!(
                " avg_win_life_lead={} (σ={:.1}, p50={})",
                s.avg_win_life_delta(),
                s.win_life_delta_stddev(),
                s.win_life_delta_median()
            ));
        }
    }
    if let (Some(mn), Some(mx)) = (s.min_duration, s.max_duration) {
        out.push_str(&format!(
            " (min {}, max {})",
            format_duration(mn),
            format_duration(mx),
        ));
    }
    // Append percentile estimates from the histogram so operators see
    // the distribution shape without manual bucket math. Skip on the
    // first match to avoid degenerate `p50=p95=<30s` noise from a single
    // sample.
    if n >= 2 {
        out.push_str(&format!(
            " p50≤{}, p95≤{}, σ={} (turns p50≤{}, p95≤{}, σ={:.1})",
            format_duration(s.percentile(0.50)),
            format_duration(s.percentile(0.95)),
            format_duration(s.duration_stddev()),
            s.turn_percentile(0.50),
            s.turn_percentile(0.95),
            s.turn_count_stddev(),
        ));
    }
    // Append histogram only when at least one bucket has hits — keeps
    // the rolling log line tight pre-warmup. Format:
    // " | <30s:3 30s-1m:5 1-2m:7 2-5m:2 5-10m:0 10m+:0" (zero buckets
    // included for stability so log greppers can rely on the column).
    if s.total_matches() > 0 {
        out.push_str(" |");
        for (i, count) in s.duration_buckets.iter().enumerate() {
            out.push_str(&format!(" {}:{}", MatchStats::bucket_label(i), count));
        }
        // Per-format breakdown — only render buckets with a label and a
        // hit, so demo-only deployments don't get a "cube:0" trailer.
        // Format: " | format=demo:7 cube:3". Push (claude/modern_decks
        // batch 162).
        let format_chunks: Vec<String> = s
            .format_buckets
            .iter()
            .enumerate()
            .filter_map(|(i, &count)| {
                if count == 0 {
                    return None;
                }
                format_label_for_bucket(i).map(|label| format!("{label}:{count}"))
            })
            .collect();
        if !format_chunks.is_empty() {
            out.push_str(" | format=");
            out.push_str(&format_chunks.join(" "));
        }
        // Turn-count histogram — same shape as the duration histogram so
        // operators can spot distribution drift in game length. Format:
        // " | turns=1-2:3 3-5:5 6-8:7 9-12:2 13-20:0 21+:0".
        out.push_str(" | turns=");
        let turn_chunks: Vec<String> = s
            .turn_buckets
            .iter()
            .enumerate()
            .map(|(i, &count)| format!("{}:{}", MatchStats::turn_bucket_label(i), count))
            .collect();
        out.push_str(&turn_chunks.join(" "));
    }
    out
}

/// Render a `Duration` as a short human-readable string for logs:
/// `1h2m3s` / `5m12s` / `38s` / `420ms`. Sub-millisecond durations
/// fall through to `<1ms`. Used by the per-match completion log so
/// operators can spot stuck matches at a glance.
pub(crate) fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let millis = d.subsec_millis();
    if total_secs == 0 {
        if millis == 0 {
            return "<1ms".to_string();
        }
        return format!("{millis}ms");
    }
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{h}h{m}m{s}s")
    } else if m > 0 {
        format!("{m}m{s}s")
    } else {
        format!("{s}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Process-global env mutex for the test module. `cargo test` runs
    /// tests in parallel by default, but env vars are process-wide — without
    /// this lock two env-var tests can stomp on each other's `set_var`
    /// before `Format::from_env` reads. Locking here is sufficient: every
    /// env-touching test funnels through `with_env`.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Hop in / out of an env var without leaking state across tests. The
    /// env API is unsafe in newer Rust; we wrap it in this helper so the
    /// scope is clearly bounded.
    pub(crate) fn with_env<F: FnOnce() -> R, R>(key: &str, value: Option<&str>, f: F) -> R {
        // Hold the lock for the entire setup → call → teardown window so a
        // parallel test can't observe (or overwrite) our env mutation.
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let prev = env::var(key).ok();
        // SAFETY: ENV_LOCK serializes every env-touching test in this module;
        // no other thread is reading or writing the env during this call.
        unsafe {
            match value {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
        let out = f();
        unsafe {
            match prev {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
        out
    }

    #[test]
    pub(crate) fn format_from_env_defaults_to_demo() {
        let f = with_env("CRAB_FORMAT", None, Format::from_env);
        assert_eq!(f.label(), "demo");
    }

    #[test]
    pub(crate) fn format_from_env_picks_cube_when_set() {
        let f = with_env("CRAB_FORMAT", Some("cube"), Format::from_env);
        assert_eq!(f.label(), "cube");
    }

    #[test]
    pub(crate) fn format_from_env_picks_commander_and_edh_alias() {
        assert_eq!(with_env("CRAB_FORMAT", Some("commander"), Format::from_env).label(), "commander");
        assert_eq!(with_env("CRAB_FORMAT", Some("edh"), Format::from_env).label(), "commander");
        // Commander maps to its own stats bucket (index 3).
        assert_eq!(format_index(Format::Commander), 3);
        assert_eq!(format_label_for_bucket(3), Some("commander"));
    }

    #[test]
    pub(crate) fn format_from_env_unknown_value_falls_back_to_demo() {
        let f = with_env("CRAB_FORMAT", Some("Cube"), Format::from_env);
        assert_eq!(f.label(), "demo",
            "case-sensitive: 'Cube' is not a recognized value");
    }

    #[test]
    pub(crate) fn pairing_timeout_defaults_when_unset() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", None, pairing_timeout_from_env);
        assert_eq!(t, DEFAULT_PAIRING_TIMEOUT);
    }

    #[test]
    pub(crate) fn pairing_timeout_parses_valid_integer() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("42"), pairing_timeout_from_env);
        assert_eq!(t, Duration::from_secs(42));
    }

    #[test]
    pub(crate) fn pairing_timeout_rejects_zero() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("0"), pairing_timeout_from_env);
        assert_eq!(t, DEFAULT_PAIRING_TIMEOUT,
            "zero would mean drop seat 0 instantly — treated as misconfig");
    }

    #[test]
    pub(crate) fn pairing_timeout_rejects_garbage() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("five"), pairing_timeout_from_env);
        assert_eq!(t, DEFAULT_PAIRING_TIMEOUT);
    }

    /// `accept_with_deadline` returns `None` after the deadline if no client
    /// connects. This is the core mechanism that prevents seat 0 from
    /// waiting forever for an opponent that never shows up.
    #[test]
    pub(crate) fn accept_with_deadline_returns_none_on_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let start = Instant::now();
        let got = accept_with_deadline(&listener, Duration::from_millis(250))
            .expect("set_nonblocking should succeed");
        let elapsed = start.elapsed();
        assert!(got.is_none(), "no client connected, should time out");
        assert!(
            elapsed >= Duration::from_millis(200),
            "should have waited close to the deadline, got {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(2),
            "should not overshoot deadline by much, got {elapsed:?}"
        );
    }

    pub(crate) fn ip(s: &str) -> IpAddr {
        s.parse().expect("parse ip")
    }

    #[test]
    pub(crate) fn slot_manager_admits_within_caps() {
        let s = SlotManager::new(2, 2);
        let _a = s.try_acquire(ip("10.0.0.1")).expect("first slot");
        let _b = s.try_acquire(ip("10.0.0.2")).expect("second slot");
    }

    #[test]
    pub(crate) fn slot_manager_rejects_at_global_cap() {
        let s = SlotManager::new(2, 0);
        let _a = s.try_acquire(ip("10.0.0.1")).expect("first");
        let _b = s.try_acquire(ip("10.0.0.2")).expect("second");
        assert!(matches!(
            s.try_acquire(ip("10.0.0.3")),
            Err(SlotRefusal::GlobalCapReached)
        ));
    }

    #[test]
    pub(crate) fn slot_manager_rejects_at_per_ip_cap() {
        let s = SlotManager::new(0, 2);
        let addr = ip("10.0.0.1");
        let _a = s.try_acquire(addr).expect("first");
        let _b = s.try_acquire(addr).expect("second");
        assert!(matches!(
            s.try_acquire(addr),
            Err(SlotRefusal::PerIpCapReached)
        ));
        // A different IP is still admitted.
        let _c = s.try_acquire(ip("10.0.0.2")).expect("other ip");
    }

    #[test]
    pub(crate) fn slot_manager_releases_on_guard_drop() {
        // global=2 so the per-IP limit is the one that triggers (otherwise
        // global=1 would reject before we even checked per-IP, masking the
        // behavior we want to test).
        let s = SlotManager::new(2, 1);
        let g = s.try_acquire(ip("10.0.0.1")).expect("first");
        assert!(matches!(
            s.try_acquire(ip("10.0.0.1")),
            Err(SlotRefusal::PerIpCapReached)
        ));
        drop(g);
        // After release, a new connection from the same IP fits again.
        let _ = s.try_acquire(ip("10.0.0.1")).expect("re-admitted");
    }

    #[test]
    pub(crate) fn slot_manager_zero_caps_mean_unlimited() {
        let s = SlotManager::new(0, 0);
        let addr = ip("10.0.0.1");
        let mut guards = Vec::new();
        for _ in 0..1000 {
            guards.push(s.try_acquire(addr).expect("unlimited"));
        }
    }

    #[test]
    pub(crate) fn slot_manager_global_cap_checked_before_per_ip() {
        // Global cap dominates: even from a fresh IP, we refuse when
        // global is full. Refusal reason indicates which limit hit.
        let s = SlotManager::new(1, 100);
        let _a = s.try_acquire(ip("10.0.0.1")).expect("first");
        assert!(matches!(
            s.try_acquire(ip("10.0.0.2")),
            Err(SlotRefusal::GlobalCapReached)
        ));
    }

    #[test]
    pub(crate) fn usize_from_env_defaults_when_unset() {
        let v = with_env("CRAB_MAX_CONNS", None, || usize_from_env("CRAB_MAX_CONNS", 42));
        assert_eq!(v, 42);
    }

    #[test]
    pub(crate) fn usize_from_env_parses_zero() {
        // Unlike pairing_timeout_from_env, here 0 is meaningful: it means
        // "unlimited". Make sure the parser preserves it.
        let v = with_env("CRAB_MAX_CONNS", Some("0"), || usize_from_env("CRAB_MAX_CONNS", 42));
        assert_eq!(v, 0);
    }

    #[test]
    pub(crate) fn usize_from_env_rejects_garbage() {
        let v = with_env("CRAB_MAX_CONNS", Some("nope"), || {
            usize_from_env("CRAB_MAX_CONNS", 42)
        });
        assert_eq!(v, 42);
    }

    #[test]
    pub(crate) fn format_duration_renders_short_values() {
        assert_eq!(format_duration(Duration::from_micros(500)), "<1ms");
        assert_eq!(format_duration(Duration::from_millis(420)), "420ms");
        assert_eq!(format_duration(Duration::from_secs(0)), "<1ms");
    }

    #[test]
    pub(crate) fn format_duration_renders_seconds_minutes_hours() {
        assert_eq!(format_duration(Duration::from_secs(38)), "38s");
        assert_eq!(format_duration(Duration::from_secs(5 * 60 + 12)), "5m12s");
        assert_eq!(
            format_duration(Duration::from_secs(3600 + 2 * 60 + 3)),
            "1h2m3s"
        );
    }

    /// `accept_with_deadline` returns the connection if a client arrives
    /// before the deadline elapses. Also verifies the listener is restored
    /// to blocking mode on success so subsequent accepts behave normally.
    #[test]
    pub(crate) fn accept_with_deadline_returns_connection_when_client_arrives() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");

        let connector = thread::spawn(move || {
            // Small delay so the server is already polling.
            thread::sleep(Duration::from_millis(50));
            TcpStream::connect(addr).expect("connect")
        });

        let got = accept_with_deadline(&listener, Duration::from_secs(2))
            .expect("set_nonblocking should succeed")
            .expect("client should arrive in time");
        let _client = connector.join().expect("connector thread");
        let _accepted = got;

        // Listener should be back in blocking mode; a follow-up `accept`
        // should block (not return WouldBlock immediately).
        let second_connector = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            TcpStream::connect(addr).expect("connect")
        });
        let (_s, _peer) = listener.accept().expect("blocking accept should work");
        let _c = second_connector.join().expect("second connector");
    }

    // ── MatchStats tests ────────────────────────────────────────────────────

    #[test]
    pub(crate) fn match_stats_starts_zero() {
        let s = MatchStats::default();
        assert_eq!(s.total_matches(), 0);
        assert_eq!(s.bot_matches, 0);
        assert_eq!(s.pair_matches, 0);
        assert_eq!(s.total_duration, Duration::ZERO);
        assert_eq!(s.avg_duration(), Duration::ZERO);
    }

    #[test]
    pub(crate) fn match_stats_record_bot_and_pair() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_pair(Duration::from_secs(120), Format::Demo);
        s.record_bot(Duration::from_secs(30), Format::Demo);
        assert_eq!(s.total_matches(), 3);
        assert_eq!(s.bot_matches, 2);
        assert_eq!(s.pair_matches, 1);
        assert_eq!(s.total_duration, Duration::from_secs(210));
        assert_eq!(s.avg_duration(), Duration::from_secs(70));
    }

    #[test]
    pub(crate) fn turn_count_stddev_measures_spread() {
        let mut s = MatchStats::default();
        assert_eq!(s.turn_count_stddev(), 0.0, "no matches → 0");
        // Turn counts [4, 4, 10]: mean 6, variance (4+4+16)/3 = 8, σ ≈ 2.828.
        for t in [4u32, 4, 10] {
            s.record_bot(Duration::from_secs(1), Format::Demo);
            s.observe_turns(t);
        }
        assert!((s.turn_count_stddev() - 8f32.sqrt()).abs() < 0.001,
            "population stddev of [4,4,10] is √8");
    }

    #[test]
    pub(crate) fn duration_stddev_measures_spread() {
        let mut s = MatchStats::default();
        assert_eq!(s.duration_stddev(), Duration::ZERO, "no matches → 0");
        // Durations [1s, 1s, 7s]: mean 3s, variance (4+4+16)/3 = 8 s²,
        // σ = √8 s ≈ 2828 ms.
        for ms in [1000u64, 1000, 7000] {
            s.record_bot(Duration::from_millis(ms), Format::Demo);
        }
        let sigma = s.duration_stddev().as_millis();
        assert!((2825..=2831).contains(&sigma), "σ of [1s,1s,7s] ≈ 2828ms, got {sigma}");
    }

    #[test]
    pub(crate) fn observe_winner_tracks_wins_and_draws() {
        let mut s = MatchStats::default();
        s.observe_winner(Some(Some(0))); // seat 0 wins
        s.observe_winner(Some(None));     // draw
        s.observe_winner(Some(Some(1))); // seat 1 wins
        s.observe_winner(None);           // unresolved — counted as inconclusive
        assert_eq!(s.wins, 2);
        assert_eq!(s.draws, 1);
        assert_eq!(s.inconclusive, 1);
        assert_eq!(s.seat_wins[0], 1);
        assert_eq!(s.seat_wins[1], 1);
        // 2 decisive of 3 resolved → 66%. Unresolved is excluded.
        assert_eq!(s.decisive_pct(), 66);
    }

    #[test]
    pub(crate) fn first_seat_win_pct_gauges_turn_order_bias() {
        let mut s = MatchStats::default();
        assert_eq!(s.first_seat_win_pct(), 50, "neutral with no data");
        for _ in 0..3 { s.observe_winner(Some(Some(0))); }
        s.observe_winner(Some(Some(1)));
        s.observe_winner(Some(None)); // draws don't count toward seat share
        // 3 of 4 seated wins on the play → 75%.
        assert_eq!(s.first_seat_win_pct(), 75);
    }

    #[test]
    pub(crate) fn inconclusive_pct_is_share_of_all_matches() {
        // 1 stuck match out of 4 total → 25%.
        let mut s = MatchStats { bot_matches: 4, ..Default::default() };
        s.observe_winner(Some(Some(0)));
        s.observe_winner(Some(Some(1)));
        s.observe_winner(Some(None));
        s.observe_winner(None); // inconclusive
        assert_eq!(s.inconclusive, 1);
        assert_eq!(s.inconclusive_pct(), 25);
    }

    #[test]
    pub(crate) fn decisive_pct_zero_before_any_resolution() {
        let s = MatchStats::default();
        assert_eq!(s.decisive_pct(), 0);
    }

    #[test]
    pub(crate) fn deckout_pct_is_share_of_wins() {
        // 1 of 4 wins closed via an alternate condition → 25%.
        let s = MatchStats { wins: 4, deckout_wins: 1, ..Default::default() };
        assert_eq!(s.deckout_pct(), 25);
        // No wins yet → 0, not a divide-by-zero.
        assert_eq!(MatchStats::default().deckout_pct(), 0);
    }

    #[test]
    pub(crate) fn observe_winner_per_seat_clamps_at_seat_bucket_count() {
        let mut s = MatchStats::default();
        // Exotic 8-player format: seat 7 wins. Must not panic, must
        // collapse into the last bucket.
        s.observe_winner(Some(Some(7)));
        assert_eq!(s.seat_wins[SEAT_BUCKET_COUNT - 1], 1);
        assert_eq!(s.wins, 1);
    }

    #[test]
    pub(crate) fn observe_win_kind_counts_alternate_wins() {
        let mut s = MatchStats::default();
        // Loser still alive (life 7) → alternate win (deckout/poison/etc.).
        s.observe_win_kind(0, &[3, 7], &[]);
        assert_eq!(s.deckout_wins, 1, "loser alive → alternate win counted");
        // Loser dead to face damage (life 0) → NOT an alternate win.
        s.observe_win_kind(0, &[5, 0], &[]);
        assert_eq!(s.deckout_wins, 1, "damage win must not count");
        // Loser at negative life → damage win, still no bump.
        s.observe_win_kind(1, &[-3, 9], &[]);
        assert_eq!(s.deckout_wins, 1);
        // Another alternate win → bumps to 2.
        s.observe_win_kind(1, &[2, 4], &[]);
        assert_eq!(s.deckout_wins, 2);
    }

    #[test]
    pub(crate) fn observe_win_kind_classifies_from_loss_reasons() {
        let mut s = MatchStats::default();
        // Seat 0 wins; seat 1 decked out → alternate + deck bucket.
        s.observe_win_kind(0, &[5, 9], &[None, Some(LossReason::Decked)]);
        assert_eq!(s.deckout_wins, 1);
        assert_eq!(s.deck_wins, 1);
        assert_eq!(s.poison_wins, 0);
        // Seat 1 wins; seat 0 poisoned out → alternate + poison bucket.
        s.observe_win_kind(1, &[3, 8], &[Some(LossReason::Poison), None]);
        assert_eq!(s.deckout_wins, 2);
        assert_eq!(s.poison_wins, 1);
        // Precise life-depletion loss is NOT an alternate win.
        s.observe_win_kind(0, &[6, -1], &[None, Some(LossReason::LifeDepleted)]);
        assert_eq!(s.deckout_wins, 2, "life-damage loss stays out of alt bucket");
    }

    #[test]
    pub(crate) fn observe_win_kind_ignores_missing_loser_data() {
        let mut s = MatchStats::default();
        // Single-element array: no opponent seat to classify against.
        s.observe_win_kind(0, &[10], &[]);
        assert_eq!(s.deckout_wins, 0, "no loser data → no classification");
        // Empty array likewise.
        s.observe_win_kind(0, &[], &[]);
        assert_eq!(s.deckout_wins, 0);
    }

    #[test]
    pub(crate) fn observe_win_life_delta_accumulates_winners_lead() {
        let mut s = MatchStats::default();
        // Winner: seat 0 at 12 life vs seat 1 at 4 life → delta 8.
        s.observe_win_life_delta(0, &[12, 4]);
        // Winner: seat 1 at 1 life vs seat 0 at 0 → delta 1.
        s.observe_win_life_delta(1, &[0, 1]);
        // Winner: seat 0 at -1 life vs seat 1 at -5 → delta 4
        // (winner lost less life than opp; positive lead remains).
        s.observe_win_life_delta(0, &[-1, -5]);
        assert_eq!(s.win_life_samples, 3);
        // Average = (8 + 1 + 4) / 3 = 4.
        assert_eq!(s.avg_win_life_delta(), 4);
        // σ = √(E[x²] − E[x]²) = √(81/3 − (13/3)²) = √(74/9) ≈ 2.867.
        assert!((s.win_life_delta_stddev() - (74f32 / 9.0).sqrt()).abs() < 1e-3);
    }

    #[test]
    pub(crate) fn win_life_delta_stddev_zero_without_samples() {
        let s = MatchStats::default();
        assert_eq!(s.win_life_delta_stddev(), 0.0);
    }

    #[test]
    pub(crate) fn observe_win_life_delta_clamps_negative_lead_to_zero() {
        let mut s = MatchStats::default();
        // Pathological: winner ended at less life than opp (shouldn't
        // happen with normal SBAs but covers forced-draw exits).
        s.observe_win_life_delta(0, &[1, 5]);
        assert_eq!(s.cumulative_win_life_delta, 0,
            "negative lead clamps to 0");
        assert_eq!(s.win_life_samples, 1);
    }

    #[test]
    pub(crate) fn observe_win_life_delta_handles_seat_out_of_range() {
        let mut s = MatchStats::default();
        s.observe_win_life_delta(5, &[20, 0]); // seat 5 doesn't exist
        assert_eq!(s.win_life_samples, 0, "out-of-range silently skipped");
    }

    #[test]
    pub(crate) fn format_match_stats_includes_avg_win_life_lead_when_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        s.observe_win_life_delta(0, &[18, 0]);
        let line = format_match_stats(&s);
        assert!(line.contains("avg_win_life_lead=18"), "got: {line}");
    }

    #[test]
    pub(crate) fn win_life_delta_median_is_robust_to_blowout_outliers() {
        let mut s = MatchStats::default();
        // Three squeakers (delta 1) and one blowout (delta 40). The mean is
        // pulled up by the blowout; the median stays in the squeaker bucket.
        for d in [1, 1, 1, 40] {
            s.observe_win_life_delta(0, &[d, 0]);
        }
        assert_eq!(s.avg_win_life_delta(), (1 + 1 + 1 + 40) / 4); // 10
        assert_eq!(s.win_life_delta_median(), 3, "median lands in the 1-3 bucket");
        assert!(format_match_stats_with_win(&s).contains("p50=3"));
    }

    pub(crate) fn format_match_stats_with_win(s: &MatchStats) -> String {
        let mut s = *s;
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        format_match_stats(&s)
    }

    #[test]
    pub(crate) fn format_match_stats_omits_avg_win_life_lead_when_no_samples() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        // No observe_win_life_delta call → samples = 0.
        let line = format_match_stats(&s);
        assert!(!line.contains("avg_win_life_lead"), "got: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_includes_seat_wins_when_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_bot(Duration::from_secs(70), Format::Demo);
        s.record_bot(Duration::from_secs(80), Format::Demo);
        s.observe_winner(Some(Some(0)));
        s.observe_winner(Some(Some(0)));
        s.observe_winner(Some(Some(1)));
        let line = format_match_stats(&s);
        assert!(line.contains("seat_wins=2/1"), "got: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_includes_win_draw_when_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        let line = format_match_stats(&s);
        assert!(line.contains("wins=1"), "got: {line}");
        assert!(line.contains("draws=0"), "got: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_omits_win_draw_pre_warmup() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        // No observe_winner — pre-warmup.
        let line = format_match_stats(&s);
        assert!(!line.contains("wins="), "got: {line}");
        assert!(!line.contains("draws="), "got: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_renders_unresolved_when_some_matches_lack_outcome() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_bot(Duration::from_secs(70), Format::Demo);
        s.observe_winner(Some(Some(0)));
        s.observe_winner(None); // second match ended with no declared outcome
        let line = format_match_stats(&s);
        assert!(line.contains("wins=1"), "got: {line}");
        assert!(line.contains("unresolved=1 (50%)"), "got: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_renders_summary() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("served 1 match"));
        assert!(line.contains("1 bot"));
        assert!(line.contains("0 pair"));
    }

    #[test]
    pub(crate) fn format_match_stats_pluralizes_at_two() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_pair(Duration::from_secs(120), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("served 2 matches"));
        assert!(line.contains("1 bot"));
        assert!(line.contains("1 pair"));
    }

    #[test]
    pub(crate) fn format_match_stats_zero_matches() {
        let s = MatchStats::default();
        let line = format_match_stats(&s);
        assert!(line.contains("served 0 matches"));
    }

    #[test]
    pub(crate) fn match_stats_tracks_min_and_max_duration() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_pair(Duration::from_secs(300), Format::Demo);
        s.record_bot(Duration::from_secs(30), Format::Demo);
        s.record_pair(Duration::from_secs(120), Format::Demo);
        assert_eq!(s.min_duration, Some(Duration::from_secs(30)));
        assert_eq!(s.max_duration, Some(Duration::from_secs(300)));
    }

    #[test]
    pub(crate) fn match_stats_avg_turns_averages_observed_turns() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_turns(10);
        s.record_pair(Duration::from_secs(60), Format::Demo);
        s.observe_turns(20);
        // 30 turns / 2 matches = 15
        assert_eq!(s.avg_turns(), 15);
    }

    #[test]
    pub(crate) fn match_stats_avg_turns_zero_before_any_record() {
        let s = MatchStats::default();
        assert_eq!(s.avg_turns(), 0);
    }

    #[test]
    pub(crate) fn match_stats_max_turns_tracks_longest_match() {
        let mut s = MatchStats::default();
        assert_eq!(s.max_turns, None, "unset before any record");
        s.observe_turns(5);
        s.observe_turns(20);
        s.observe_turns(8);
        assert_eq!(s.max_turns, Some(20), "tracks the longest observed turn count");
    }

    #[test]
    pub(crate) fn match_stats_max_turns_rendered_in_summary_line() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_turns(42);
        let line = format_match_stats(&s);
        assert!(line.contains("max turns 42"), "expected max-turns in summary: {line}");
    }

    #[test]
    pub(crate) fn match_stats_min_max_unset_before_any_record() {
        let s = MatchStats::default();
        assert_eq!(s.min_duration, None);
        assert_eq!(s.max_duration, None);
    }

    #[test]
    pub(crate) fn format_match_stats_includes_min_max_when_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(30), Format::Demo);
        s.record_pair(Duration::from_secs(300), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("min "), "should include min: {line}");
        assert!(line.contains("max "), "should include max: {line}");
        assert!(line.contains("30s"), "min 30s: {line}");
        assert!(line.contains("5m"), "max 5m: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_omits_min_max_when_zero_matches() {
        let s = MatchStats::default();
        let line = format_match_stats(&s);
        // No min/max parenthetical when no matches have been recorded.
        assert!(!line.contains("min "));
        assert!(!line.contains("max "));
    }

    // ── duration-histogram tests ────────────────────────────────────────────

    #[test]
    pub(crate) fn bucket_index_partitions_durations_into_six_buckets() {
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(0)), 0);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(15)), 0);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(29)), 0);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(30)), 1);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(59)), 1);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(60)), 2);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(119)), 2);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(120)), 3);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(299)), 3);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(300)), 4);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(599)), 4);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(600)), 5);
        assert_eq!(MatchStats::bucket_index(Duration::from_secs(3600)), 5);
    }

    #[test]
    pub(crate) fn turn_bucket_index_partitions_turn_counts_into_six_buckets() {
        assert_eq!(MatchStats::turn_bucket_index(1), 0);
        assert_eq!(MatchStats::turn_bucket_index(2), 0);
        assert_eq!(MatchStats::turn_bucket_index(3), 1);
        assert_eq!(MatchStats::turn_bucket_index(5), 1);
        assert_eq!(MatchStats::turn_bucket_index(6), 2);
        assert_eq!(MatchStats::turn_bucket_index(8), 2);
        assert_eq!(MatchStats::turn_bucket_index(9), 3);
        assert_eq!(MatchStats::turn_bucket_index(12), 3);
        assert_eq!(MatchStats::turn_bucket_index(13), 4);
        assert_eq!(MatchStats::turn_bucket_index(20), 4);
        assert_eq!(MatchStats::turn_bucket_index(21), 5);
        assert_eq!(MatchStats::turn_bucket_index(99), 5);
    }

    #[test]
    pub(crate) fn observe_turns_increments_turn_histogram_and_renders() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_turns(2); // bucket 0
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_turns(7); // bucket 2
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_turns(25); // bucket 5
        assert_eq!(s.turn_buckets, [1, 0, 1, 0, 0, 1]);
        let line = format_match_stats(&s);
        assert!(line.contains("turns=1-2:1"), "histogram in line: {line}");
        assert!(line.contains("21+:1"), "long-game bucket in line: {line}");
    }

    #[test]
    pub(crate) fn match_stats_observe_duration_increments_correct_bucket() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo); // bucket 0
        s.record_bot(Duration::from_secs(45), Format::Demo); // bucket 1
        s.record_bot(Duration::from_secs(100), Format::Demo); // bucket 2
        s.record_bot(Duration::from_secs(180), Format::Demo); // bucket 3
        s.record_bot(Duration::from_secs(500), Format::Demo); // bucket 4
        s.record_bot(Duration::from_secs(700), Format::Demo); // bucket 5
        assert_eq!(s.duration_buckets, [1, 1, 1, 1, 1, 1]);
        // Two more matches in the <30s bucket
        s.record_pair(Duration::from_secs(5), Format::Demo);
        s.record_pair(Duration::from_secs(20), Format::Demo);
        assert_eq!(s.duration_buckets[0], 3);
    }

    #[test]
    pub(crate) fn format_match_stats_includes_histogram_when_matches_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo);
        s.record_pair(Duration::from_secs(700), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("<30s:1"), "1 in <30s bucket: {line}");
        assert!(line.contains("10m+:1"), "1 in 10m+ bucket: {line}");
        assert!(line.contains("30s-1m:0"), "stable column with 0 count: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_omits_histogram_when_zero_matches() {
        let s = MatchStats::default();
        let line = format_match_stats(&s);
        assert!(!line.contains("|"), "no histogram section when 0 matches: {line}");
    }

    #[test]
    pub(crate) fn percentile_zero_when_no_matches() {
        let s = MatchStats::default();
        assert_eq!(s.percentile(0.5), Duration::ZERO);
    }

    #[test]
    pub(crate) fn percentile_lands_in_correct_bucket() {
        let mut s = MatchStats::default();
        // 9 short, 1 long: p50 should land in the short bucket, p95 in
        // the long bucket.
        for _ in 0..9 {
            s.record_bot(Duration::from_secs(10), Format::Demo);
        }
        s.record_bot(Duration::from_secs(2000), Format::Demo);
        assert_eq!(s.percentile(0.5), Duration::from_secs(30),
            "p50 lands in <30s bucket (upper bound 30s)");
        assert_eq!(s.percentile(0.95), Duration::from_secs(3600),
            "p95 lands in 10m+ bucket (upper bound 3600s)");
    }

    #[test]
    pub(crate) fn percentile_p100_returns_max_bucket_upper_bound() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(5), Format::Demo);
        assert_eq!(s.percentile(1.0), Duration::from_secs(30),
            "p100 with one sample = upper bound of its bucket");
    }

    #[test]
    pub(crate) fn turn_percentile_zero_when_no_matches() {
        let s = MatchStats::default();
        assert_eq!(s.turn_percentile(0.5), 0);
    }

    #[test]
    pub(crate) fn turn_percentile_lands_in_correct_bucket() {
        let mut s = MatchStats::default();
        // 9 short games (2 turns → bucket 0) + 1 grindy game (30 turns →
        // bucket 5). p50 lands in the short bucket; p95 in the open-ended.
        for _ in 0..9 {
            s.record_bot(Duration::from_secs(10), Format::Demo);
            s.observe_turns(2);
        }
        s.record_bot(Duration::from_secs(2000), Format::Demo);
        s.observe_turns(30);
        assert_eq!(s.turn_percentile(0.5), 2, "p50 lands in the 1-2 bucket");
        assert_eq!(s.turn_percentile(0.95), 21, "p95 lands in the 21+ bucket");
    }

    #[test]
    pub(crate) fn format_match_stats_includes_turn_percentiles() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo);
        s.observe_turns(7);
        s.record_pair(Duration::from_secs(15), Format::Demo);
        s.observe_turns(9);
        let line = format_match_stats(&s);
        assert!(line.contains("turns p50≤"), "turn percentile present: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_adds_percentile_when_at_least_two_matches() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo);
        s.record_pair(Duration::from_secs(15), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("p50≤"), "p50 estimate present: {line}");
        assert!(line.contains("p95≤"), "p95 estimate present: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_omits_percentile_at_single_sample() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo);
        let line = format_match_stats(&s);
        assert!(!line.contains("p50"), "no p50 when only 1 sample: {line}");
    }

    #[test]
    pub(crate) fn match_stats_tracks_per_format_counts() {
        // Push (claude/modern_decks batch 162) — the new per-format
        // histogram surfaces a `format=demo:N cube:M` breakdown in the
        // rolling stats line so operators see cube-vs-demo split.
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_bot(Duration::from_secs(120), Format::Cube);
        s.record_pair(Duration::from_secs(30), Format::Cube);
        assert_eq!(s.format_buckets[format_index(Format::Demo)], 1);
        assert_eq!(s.format_buckets[format_index(Format::Cube)], 2);
        let line = format_match_stats(&s);
        assert!(line.contains("format=demo:1 cube:2"),
            "expected per-format breakdown in rolling line: {line}");
    }

    #[test]
    pub(crate) fn match_stats_format_breakdown_omitted_when_only_one_format_used() {
        // When only one format has hits, the other label should be
        // silently dropped instead of rendering ":0".
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_bot(Duration::from_secs(30), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("format=demo:2"), "demo bucket present: {line}");
        assert!(!line.contains("cube:0"),
            "cube:0 should not appear when no cube matches played: {line}");
    }

    #[test]
    pub(crate) fn observe_turns_tracks_min_and_max_envelope() {
        // Push (claude/modern_decks batch 205) — the new min_turns
        // completes the turn-count envelope alongside the existing
        // max_turns + running average.
        let mut s = MatchStats::default();
        assert_eq!(s.min_turns, None, "no samples yet");
        s.observe_turns(8);
        s.observe_turns(3);
        s.observe_turns(20);
        assert_eq!(s.min_turns, Some(3), "shortest game tracked");
        assert_eq!(s.max_turns, Some(20), "longest game tracked");
    }

    #[test]
    pub(crate) fn format_match_stats_renders_turn_envelope_when_spread() {
        // When min != max, the rolling line shows "(turns MIN-MAX)".
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(30), Format::Demo);
        s.observe_turns(5);
        s.record_bot(Duration::from_secs(120), Format::Demo);
        s.observe_turns(15);
        let line = format_match_stats(&s);
        assert!(line.contains("(turns 5-15)"),
            "expected turn envelope in rolling line: {line}");
    }

    #[test]
    pub(crate) fn format_match_stats_collapses_turn_envelope_to_max_when_single_value() {
        // A single distinct turn count renders as "(max turns N)" to
        // avoid the degenerate "(turns N-N)".
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(30), Format::Demo);
        s.observe_turns(7);
        let line = format_match_stats(&s);
        assert!(line.contains("(max turns 7)"), "single value → max turns: {line}");
        assert!(!line.contains("(turns 7-7)"), "no degenerate range: {line}");
    }
}

