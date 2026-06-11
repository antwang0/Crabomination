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
    pub(crate) bot_matches: u64,
    pub(crate) pair_matches: u64,
    /// Total cumulative match duration (sum). Average = total / count.
    pub(crate) total_duration: Duration,
    /// Shortest observed match duration. `None` until the first match
    /// completes. Surfaces outlier-short games (instant disconnects,
    /// concession-on-turn-1).
    pub(crate) min_duration: Option<Duration>,
    /// Longest observed match duration. `None` until the first match
    /// completes. Surfaces stalls / long grindy games.
    pub(crate) max_duration: Option<Duration>,
    /// Bucketed histogram of match durations. Buckets:
    /// `[0]` = under 30s, `[1]` = 30s-1m, `[2]` = 1-2m, `[3]` = 2-5m,
    /// `[4]` = 5-10m, `[5]` = 10m+. Lets operators see the distribution
    /// shape at a glance without leaving the per-match log line —
    /// e.g. a sudden spike in the `<30s` bucket indicates many bots
    /// are conceding turn 1 (often a regression signal).
    pub(crate) duration_buckets: [u32; 6],
    /// Per-format histogram of completed matches, indexed by the local
    /// server `Format` discriminant (Demo / Cube). Lets operators see
    /// the running cube-vs-demo split in the rolling summary line. Push
    /// (claude/modern_decks batch 162).
    pub(crate) format_buckets: [u64; FORMAT_BUCKET_COUNT],
    /// Cumulative turn count across all matches — divided by total
    /// matches in the summary line. Operators see at a glance whether
    /// games are concession-heavy (low avg turn count) or grindy
    /// (high avg turn count) without sampling individual match logs.
    /// Push (claude/modern_decks batch 172).
    pub(crate) total_turns: u64,
    /// Longest observed final turn count across all completed matches.
    /// Surfaces "grindiest" games for outlier debugging — paired with
    /// `total_turns / total_matches` (the running average) lets operators
    /// distinguish "consistent 8-turn games" from "5-turn average with
    /// one 30-turn outlier". `None` until the first match completes.
    /// Push (claude/modern_decks batch 189).
    pub(crate) max_turns: Option<u32>,
    /// Shortest observed final turn count across all completed matches.
    /// Paired with `max_turns` and the running average, this completes
    /// the turn-count envelope so operators can distinguish a tight
    /// "always 6-8 turn" distribution from a wide "2-turn concession to
    /// 30-turn grind" spread without sampling individual match logs.
    /// `None` until the first match completes. Push
    /// (claude/modern_decks batch 205).
    pub(crate) min_turns: Option<u32>,
    /// Number of matches that ended in a draw (MatchOutcome.winner =
    /// Some(None)). Useful for spotting "stalemate" regressions
    /// (typically a bot-vs-bot loop where neither side can finish).
    /// Push (claude/modern_decks batches 192-194).
    pub(crate) draws: u64,
    /// Number of matches that ended cleanly with a declared winner
    /// (Some(Some(seat))). Pre-game-over exits (channel disconnect,
    /// watchdog) yield None and are excluded from this counter, so
    /// `wins + draws ≤ total_matches`. The delta surfaces "stuck"
    /// matches that never produced an outcome. Push
    /// (claude/modern_decks batches 192-194).
    pub(crate) wins: u64,
    /// Per-seat wins (indexed by seat 0..SEAT_BUCKET_COUNT). Surfaces
    /// turn-order bias in bot-vs-bot ladders: if `seat_wins[0]` is
    /// twice `seat_wins[1]` over a long run, the active-player heuristic
    /// or starting-hand luck is leaking through. Pre-warmup all-zero
    /// rows render as `seat_wins=0/0` in the rolling summary so the
    /// operator can spot empty samples. Push (claude/modern_decks
    /// batch 198).
    pub(crate) seat_wins: [u64; SEAT_BUCKET_COUNT],
    /// Cumulative life delta on wins: for each completed match with a
    /// winner, sum `winner_life - max(other_seat_life, 0)`. Divided by
    /// `wins` in the summary line gives an average win-by-life number,
    /// surfacing whether games are "blowouts" (high delta) or "races"
    /// (low / negative delta — winner ended at 1 with opp at 0).
    /// Saturates positive; clamped at zero when winner life is below
    /// the negative of the opp's. Push (claude/modern_decks batch 202).
    pub(crate) cumulative_win_life_delta: i64,
    /// Σ of (win-life-delta)² across sampled wins, for the population
    /// standard deviation (σ = √(E[x²] − E[x]²)). Paired with the average,
    /// σ distinguishes a consistent "win-by-5" meta from a bimodal
    /// "blowout-or-squeaker" split the average alone hides. Deltas are
    /// clamped ≥ 0, so the squared sum is non-negative; `u128` headroom
    /// keeps it from overflowing over a long run.
    pub(crate) cumulative_win_life_delta_squared: u128,
    /// Number of matches counted in `cumulative_win_life_delta`. Lets
    /// the formatter compute the average without dividing by `wins`
    /// directly (a winner with no available life data — e.g. a forced
    /// concession — is skipped in the cumulative sum but still counted
    /// in `wins`).
    pub(crate) win_life_samples: u64,
    /// Bucketed histogram of win-by-life deltas, parallel to
    /// `duration_buckets`. Buckets: `[0]` = 0 (won at parity / race),
    /// `[1]` = 1-3, `[2]` = 4-6, `[3]` = 7-10, `[4]` = 11-15, `[5]` = 16+.
    /// The average + σ give the centre and spread; this gives the
    /// distribution *shape*, and feeds a median (p50) estimate that's
    /// robust to the blowout outliers the mean is sensitive to.
    pub(crate) win_life_delta_buckets: [u32; 6],
    /// Bucketed histogram of final turn counts, parallel to
    /// `duration_buckets`. Buckets: `[0]` = 1-2 turns, `[1]` = 3-5,
    /// `[2]` = 6-8, `[3]` = 9-12, `[4]` = 13-20, `[5]` = 21+. The
    /// turn-count envelope (`min_turns`/`max_turns`/average) gives the
    /// extremes and centre; this histogram gives the distribution
    /// *shape* — e.g. a fat `[0]` bucket flags a concession regression
    /// even when one long outlier keeps the average high. Mirrors the
    /// duration histogram so operators read both in the same summary
    /// line. Push (claude/modern_decks).
    pub(crate) turn_buckets: [u32; 6],
    /// Number of clean wins where every losing seat ended with life > 0
    /// — i.e. the loser did *not* die to lethal face damage. These are
    /// "alternate" wins (decking out, poison, mill, or a win-the-game
    /// effect). Surfaced next to `wins` so operators can see the
    /// damage-vs-alternate win split: a sudden rise in `deckout_wins`
    /// relative to `wins` flags a stall regression where bots grind to
    /// empty libraries instead of closing on life. Counted only on
    /// `Some(Some(seat))` outcomes with available life data. Push
    /// (claude/modern_decks).
    pub(crate) deckout_wins: u64,
    /// Subset of `deckout_wins` where at least one losing seat was
    /// eliminated specifically by poison (CR 104.3c). Classified from the
    /// outcome's precise `loss_reasons`, not the life-total heuristic, so
    /// poison ladders show a distinct signal from pure deck-out grinds.
    pub(crate) poison_wins: u64,
    /// Subset of `deckout_wins` where at least one losing seat decked out
    /// (drew from an empty library, CR 104.3a). The dredge/mill shells push
    /// this bucket; reading it next to `poison_wins` splits the umbrella
    /// non-damage win count into its two main alternate paths.
    pub(crate) deck_wins: u64,
    /// Subset of `deckout_wins` where at least one losing seat was killed by
    /// 21+ combat damage from a single commander (CR 903.10a). Relevant to
    /// the Commander/Brawl formats; reads alongside `poison_wins`/`deck_wins`
    /// as a third distinct alternate-win path.
    pub(crate) commander_damage_wins: u64,
    /// Running sum of squared final turn counts (`Σ turns²`). Paired with
    /// `total_turns` (`Σ turns`) and the match count it yields the
    /// population standard deviation of game length via
    /// [`turn_count_stddev`](Self::turn_count_stddev) — a single number
    /// that tells operators whether games cluster tightly around the
    /// average (small σ) or swing wildly (large σ), complementing the
    /// min/max envelope and the histogram shape.
    pub(crate) total_turns_squared: u128,
    /// Running sum of squared match durations in **milliseconds**
    /// (`Σ ms²`). The duration analogue of `total_turns_squared`: paired
    /// with `total_duration` (`Σ ms`) and the match count it yields the
    /// population standard deviation of match length via
    /// [`duration_stddev`](Self::duration_stddev), so the rolling summary
    /// reports duration σ next to the turn-count σ. Milliseconds keep the
    /// squares well within `u128` even for very long sessions.
    pub(crate) total_duration_squared_ms: u128,
    /// Matches that completed without a declared outcome (`observe_winner`
    /// got `None` — channel disconnect, watchdog kill, or a stuck loop). The
    /// `wins + draws + inconclusive ≤ total_matches` identity makes this the
    /// explicit count of the "stuck" delta operators previously had to derive
    /// by subtraction; a rising `inconclusive_pct` flags a hang regression.
    pub(crate) inconclusive: u64,
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
