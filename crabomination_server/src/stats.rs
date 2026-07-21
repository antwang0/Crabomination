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
    /// Per-format cumulative final turn count, indexed like
    /// `format_buckets`. Divided by the matching `format_buckets` count to
    /// surface each format's average game length in the summary
    /// (`demo:7(9t) cube:3(14t)`), so operators can tell a slow format apart
    /// from a slow build without sampling per-match logs.
    pub(crate) format_turn_totals: [u64; FORMAT_BUCKET_COUNT],
    /// Per-format cumulative wall-clock duration in seconds, indexed like
    /// `format_buckets`. Divided by the matching count to surface each
    /// format's average *wall-clock* game length alongside its turn count —
    /// so operators can tell a format that's slow in real time (long decision
    /// timeouts) from one that's merely long in turns. Push (modern_decks).
    pub(crate) format_duration_totals: [u64; FORMAT_BUCKET_COUNT],
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
    /// Number of clean wins that *did* close via lethal face damage (every
    /// losing seat at ≤ 0 life). The complement of `deckout_wins`, tracked
    /// explicitly so the win-kind buckets reconcile with `wins`
    /// (`damage_wins + deckout_wins == wins`) rather than leaving the common
    /// case implicit. Push (claude/modern_decks).
    pub(crate) damage_wins: u64,
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
    /// Subset of `deckout_wins` where at least one losing seat conceded
    /// (CR 104.3a). Split out from `other_wins` so a concession-heavy sample
    /// (rage-quits, bot timeouts) is distinguishable from genuine "you lose
    /// the game" effects.
    pub(crate) concede_wins: u64,
    /// Subset of `deckout_wins` where at least one losing seat left for an
    /// "other" reason (a "you lose the game" effect — CR 104.3g) — not life,
    /// poison, deck-out, commander damage, or concession. Surfacing it
    /// completes the alternate-win decomposition so the umbrella `deckout_wins`
    /// doesn't hide an unclassified residue.
    pub(crate) other_wins: u64,
    /// Running sum of squared final turn counts (`Σ turns²`). Paired with
    /// `total_turns` (`Σ turns`) and the match count it yields the
    /// population standard deviation of game length via
    /// [`turn_count_stddev`](Self::turn_count_stddev) — a single number
    /// that tells operators whether games cluster tightly around the
    /// average (small σ) or swing wildly (large σ), complementing the
    /// min/max envelope and the histogram shape.
    pub(crate) total_turns_squared: u128,
    /// Running sum of final turn counts for decisive matches (a declared
    /// winner). Paired with `wins` it yields the average length of *decided*
    /// games; compared against the draw average below it surfaces whether
    /// stalemates are systematically longer grinds than clean wins.
    pub(crate) decisive_turn_sum: u64,
    /// Running sum of final turn counts for drawn matches. Paired with `draws`
    /// it yields the average length of drawn games (see `decisive_turn_sum`).
    pub(crate) draw_turn_sum: u64,
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
    /// Running sum of the **winner's** battlefield permanent count at match
    /// end (`outcome.final_board_sizes[winner]`). Paired with `board_samples`
    /// it yields the average board a seat controls when it wins — a
    /// development proxy that, next to `avg_decisive_turn`, tells a fast
    /// face-damage win (small board, low turn) apart from a grindy attrition
    /// win (wide board, high turn). Skipped when board data is unavailable
    /// (aborted matches leave `final_board_sizes` empty).
    pub(crate) winner_board_sum: u64,
    /// Running `Σ board²` over the same samples, so a population σ falls out of
    /// √(E[x²] − E[x]²) — the winner-board analogue of `total_turns_squared`.
    /// A tight σ next to `avg_winner_board` says wins land at a consistent
    /// board width; a large σ flags an empty-board burn vs. wide-board grind
    /// split the average alone hides.
    pub(crate) winner_board_sum_squared: u128,
    /// Number of wins counted in `winner_board_sum` (a winner with no board
    /// snapshot is still counted in `wins`, so this can trail `wins`).
    pub(crate) winner_board_samples: u64,
    /// Narrowest / widest board a seat held at victory. `None` until the first
    /// sample. The σ tells you *how spread* winning boards are; these name the
    /// actual extremes — a `min` of 0 means at least one win came off an empty
    /// board (pure burn/mill), a large `max` flags the widest attrition grind.
    pub(crate) winner_board_min: Option<u32>,
    pub(crate) winner_board_max: Option<u32>,
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
        self.observe_format_duration(f, d);
    }
    pub(crate) fn record_pair(&mut self, d: Duration, f: Format) {
        self.pair_matches += 1;
        self.observe_duration(d);
        self.observe_format(f);
        self.observe_format_duration(f, d);
    }
    /// Fold a completed match into every counter at once: the match-kind
    /// tally (`record_bot`/`record_pair`), turn counts, winner/seat bias,
    /// and — for decisive games — the win life-delta and win-kind buckets.
    /// Centralizes the recording logic so `run_bot_match` / `run_pair_match`
    /// stay in lockstep and new stats only land in one place.
    pub(crate) fn record_outcome(
        &mut self,
        outcome: &crabomination::server::MatchOutcome,
        format: Format,
        duration: Duration,
        pair: bool,
    ) {
        if pair {
            self.record_pair(duration, format);
        } else {
            self.record_bot(duration, format);
        }
        self.observe_turns(outcome.final_turn);
        self.observe_format_turns(format, outcome.final_turn);
        self.observe_winner(outcome.winner);
        match outcome.winner {
            Some(Some(w)) => {
                self.decisive_turn_sum =
                    self.decisive_turn_sum.saturating_add(outcome.final_turn as u64);
                self.observe_win_life_delta(w, &outcome.final_life_totals);
                self.observe_win_kind(w, &outcome.final_life_totals, &outcome.loss_reasons);
                self.observe_winner_board(w, &outcome.final_board_sizes);
            }
            Some(None) => {
                self.draw_turn_sum = self.draw_turn_sum.saturating_add(outcome.final_turn as u64);
            }
            None => {}
        }
    }

    /// Average final turn count of decisive (won) matches, or 0 with no wins.
    pub(crate) fn avg_decisive_turns(&self) -> u64 {
        self.decisive_turn_sum.checked_div(self.wins).unwrap_or(0)
    }

    /// Average final turn count of drawn matches, or 0 with no draws.
    pub(crate) fn avg_draw_turns(&self) -> u64 {
        self.draw_turn_sum.checked_div(self.draws).unwrap_or(0)
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
    /// Percentage of *all* completed matches that ended in a draw. Distinct
    /// from `decisive_pct` (which is wins/(wins+draws), ignoring inconclusive
    /// games): this is over the full denominator, so a creeping bot-vs-bot
    /// stalemate regression shows up here even as `inconclusive_pct` stays low.
    pub(crate) fn draw_pct(&self) -> u64 {
        let total = self.total_matches();
        if total == 0 { return 0; }
        self.draws.saturating_mul(100) / total
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
    /// Seat `seat`'s share of all decided wins, as a percent. Generalizes
    /// `first_seat_win_pct` to every seat so an N-player turn-order skew
    /// (not just first-vs-rest) is visible. Returns 0 before any win is
    /// recorded (no data) and for an out-of-range seat.
    pub(crate) fn seat_win_share_pct(&self, seat: usize) -> u64 {
        let seated: u64 = self.seat_wins.iter().sum();
        if seated == 0 {
            return 0;
        }
        self.seat_wins.get(seat).copied().unwrap_or(0).saturating_mul(100) / seated
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
    /// Estimate the p-th percentile win-by-life delta from the histogram —
    /// robust to the blowout outliers that inflate the mean. Returns the upper
    /// edge of the bucket holding the target sample, or 0 with no samples.
    /// `p` is clamped to [0, 1]. Mirrors [`turn_percentile`](Self::turn_percentile).
    pub(crate) fn win_life_delta_percentile(&self, p: f32) -> i64 {
        Self::percentile_bucket(&self.win_life_delta_buckets, p)
            .map(Self::win_life_delta_bucket_upper_bound)
            .unwrap_or(0)
    }
    /// The median (p50) win-by-life delta.
    pub(crate) fn win_life_delta_median(&self) -> i64 {
        self.win_life_delta_percentile(0.5)
    }
    /// Interquartile range of the win-by-life delta (p75 − p25), the margin
    /// analogue of [`turn_count_iqr`](Self::turn_count_iqr). Robust to blowout
    /// outliers that inflate σ: a tight IQR next to a wide σ marks a
    /// mostly-consistent win margin with a few runaway stomps. Returns 0 with
    /// no samples. Bucket-quantised like the other percentile readouts.
    pub(crate) fn win_life_delta_iqr(&self) -> i64 {
        self.win_life_delta_percentile(0.75)
            .saturating_sub(self.win_life_delta_percentile(0.25))
    }
    /// Share (percent) of margin-tracked wins that were nail-biters — a final
    /// life margin of 3 or less (delta buckets 0–1). The blowout-insensitive
    /// companion to the mean/σ/IQR margin readouts: a high value means most
    /// wins came down to the wire even when σ is inflated by a few runaway
    /// stomps. Returns 0 with no margin samples.
    pub(crate) fn close_win_pct(&self) -> u64 {
        let total: u64 = self.win_life_delta_buckets.iter().map(|&n| n as u64).sum();
        let close =
            self.win_life_delta_buckets[0] as u64 + self.win_life_delta_buckets[1] as u64;
        close.saturating_mul(100).checked_div(total).unwrap_or(0)
    }
    /// Share (percent) of margin-tracked wins that were blowouts — a final life
    /// margin above 15 (delta bucket 5). The upper-tail mirror of
    /// [`close_win_pct`](Self::close_win_pct): a rising value with a stable
    /// mean flags a widening blowout tail (one deck stomping the field) that
    /// the average alone hides. Returns 0 with no margin samples.
    pub(crate) fn blowout_win_pct(&self) -> u64 {
        let total: u64 = self.win_life_delta_buckets.iter().map(|&n| n as u64).sum();
        (self.win_life_delta_buckets[5] as u64).saturating_mul(100).checked_div(total).unwrap_or(0)
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
            } else {
                self.damage_wins = self.damage_wins.saturating_add(1);
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
            if reasons.contains(&LossReason::Conceded) {
                self.concede_wins = self.concede_wins.saturating_add(1);
            }
            if reasons.contains(&LossReason::Other) {
                self.other_wins = self.other_wins.saturating_add(1);
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
        } else {
            self.damage_wins = self.damage_wins.saturating_add(1);
        }
    }
    /// Record the winner's end-of-match board size. No-op when the winner
    /// seat has no snapshot (aborted match — `final_board_sizes` empty or
    /// too short).
    pub(crate) fn observe_winner_board(&mut self, winner: usize, final_board_sizes: &[usize]) {
        if let Some(&board) = final_board_sizes.get(winner) {
            self.winner_board_sum = self.winner_board_sum.saturating_add(board as u64);
            self.winner_board_sum_squared =
                self.winner_board_sum_squared.saturating_add((board as u128).saturating_mul(board as u128));
            self.winner_board_samples = self.winner_board_samples.saturating_add(1);
            let b = board as u32;
            self.winner_board_min = Some(self.winner_board_min.map_or(b, |m| m.min(b)));
            self.winner_board_max = Some(self.winner_board_max.map_or(b, |m| m.max(b)));
        }
    }

    /// Average board size (permanents controlled) a seat holds when it wins.
    /// Returns 0 when no winner-board samples have been recorded yet.
    pub(crate) fn avg_winner_board(&self) -> u64 {
        self.winner_board_sum.checked_div(self.winner_board_samples).unwrap_or(0)
    }

    /// Population standard deviation of winning board sizes (σ = √(E[x²] −
    /// E[x]²)). Returns 0.0 until a winner-board sample exists.
    pub(crate) fn winner_board_stddev(&self) -> f32 {
        if self.winner_board_samples == 0 {
            return 0.0;
        }
        let n = self.winner_board_samples as f64;
        let mean = self.winner_board_sum as f64 / n;
        let mean_sq = self.winner_board_sum_squared as f64 / n;
        (mean_sq - mean * mean).max(0.0).sqrt() as f32
    }

    /// Coefficient of variation of the winner's board size (σ / mean, as a
    /// percent). The scale-free companion to [`winner_board_stddev`](Self::
    /// winner_board_stddev) and the sibling of [`win_life_delta_cv_pct`](Self::
    /// win_life_delta_cv_pct): a burn format (winners at ~0 board) and a
    /// go-wide format (winners at ~8 board) can share a σ yet differ wildly in
    /// relative spread — the CV normalizes that so board-consistency is
    /// comparable across formats. Returns 0 with no samples or a zero mean.
    pub(crate) fn winner_board_cv_pct(&self) -> u64 {
        let mean = self.avg_winner_board();
        if self.winner_board_samples == 0 || mean == 0 {
            return 0;
        }
        (self.winner_board_stddev() as f64 * 100.0 / mean as f64).round() as u64
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
    /// Coefficient of variation of the win-by-life delta (σ / mean, as a
    /// percent). Scale-free dispersion: unlike raw σ it stays comparable as the
    /// average margin drifts, so a rising CV flags a swingier win-margin
    /// distribution even when the mean shifts. Returns 0 with no samples or a
    /// non-positive mean (the ratio is undefined there).
    pub(crate) fn win_life_delta_cv_pct(&self) -> u64 {
        let mean = self.avg_win_life_delta();
        if self.win_life_samples == 0 || mean <= 0 {
            return 0;
        }
        (self.win_life_delta_stddev() as f64 * 100.0 / mean as f64).round() as u64
    }
    /// Percent of *resolved* matches (wins + draws) that ended decisively
    /// (i.e. had a winner). Returns 0 when nothing has resolved yet. A
    /// sudden drop signals stalemate regressions (mutual lock, no win
    /// condition reachable). Excludes unresolved/watchdog'd matches from
    /// the denominator so disconnects don't deflate the rate.
    pub(crate) fn decisive_pct(&self) -> u64 {
        let resolved = self.wins + self.draws;
        self.wins.saturating_mul(100).checked_div(resolved).unwrap_or(0)
    }
    /// Matches that ended without a recorded winner or draw — a channel
    /// disconnect or the watchdog tearing down a wedged game (`observe_winner`
    /// saw `None`). `total - wins - draws`, saturating so it never underflows
    /// if the winner counters ever outpace the match counter. A nonzero,
    /// rising value flags a hang/crash regression the win/draw split hides.
    pub(crate) fn unresolved(&self) -> u64 {
        self.total_matches().saturating_sub(self.wins + self.draws)
    }
    /// Percent of wins that closed via something other than lethal face
    /// damage (deckout / poison / mill / win-the-game). Returns 0 when no
    /// wins have been recorded. A rising share flags a stall regression
    /// where bots grind to empty libraries instead of closing on life.
    pub(crate) fn deckout_pct(&self) -> u64 {
        self.deckout_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins that closed via lethal face damage — the complement of
    /// `deckout_pct`. Returns 0 when no wins have been recorded.
    pub(crate) fn damage_pct(&self) -> u64 {
        self.damage_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins in which a losing seat died to poison (CR 104.3c).
    /// A sub-split of `deckout_pct`; 0 when no wins recorded.
    pub(crate) fn poison_pct(&self) -> u64 {
        self.poison_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Poison's share *among alternate (non-lethal-damage) wins*, not of all
    /// wins — a sharper toxic-metagame signal than `poison_pct` when most
    /// games still close on life. 0 when no alternate wins recorded.
    pub(crate) fn poison_of_alt_pct(&self) -> u64 {
        self.poison_wins.saturating_mul(100).checked_div(self.deckout_wins).unwrap_or(0)
    }
    /// Percent of wins in which a losing seat decked out (CR 104.3a).
    /// A sub-split of `deckout_pct`; 0 when no wins recorded.
    pub(crate) fn deck_pct(&self) -> u64 {
        self.deck_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins in which a losing seat left for an "other" reason
    /// (concession / "you lose the game" effect). A sub-split of
    /// `deckout_pct`; 0 when no wins recorded.
    pub(crate) fn other_pct(&self) -> u64 {
        self.other_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins via 21+ commander damage (CR 903.10a).
    /// A sub-split of `deckout_pct`; 0 when no wins recorded.
    pub(crate) fn commander_damage_pct(&self) -> u64 {
        self.commander_damage_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
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
    /// Accumulate a completed match's final turn count into its format
    /// bucket. Called once per match alongside `observe_turns`; divided by
    /// `format_buckets` in the summary to surface each format's average
    /// game length.
    pub(crate) fn observe_format_turns(&mut self, f: Format, turns: u32) {
        let idx = format_index(f).min(FORMAT_BUCKET_COUNT - 1);
        self.format_turn_totals[idx] = self.format_turn_totals[idx].saturating_add(turns as u64);
    }
    /// Average final turn count for format bucket `i`, or `None` if no turn
    /// data has been recorded for that format (so callers fall back to the
    /// bare `label:count` rather than rendering a misleading `(0t)`).
    pub(crate) fn format_avg_turns(&self, i: usize) -> Option<u64> {
        let total = *self.format_turn_totals.get(i)?;
        if total == 0 {
            return None;
        }
        total.checked_div(*self.format_buckets.get(i)?)
    }
    /// Accumulate a completed match's wall-clock duration into its format
    /// bucket. Called from the record paths alongside `observe_format`.
    pub(crate) fn observe_format_duration(&mut self, f: Format, d: Duration) {
        let idx = format_index(f).min(FORMAT_BUCKET_COUNT - 1);
        self.format_duration_totals[idx] =
            self.format_duration_totals[idx].saturating_add(d.as_secs());
    }
    /// Average wall-clock seconds for format bucket `i`, or `None` when no
    /// match has completed in that format yet.
    pub(crate) fn format_avg_duration_secs(&self, i: usize) -> Option<u64> {
        let count = *self.format_buckets.get(i)?;
        if count == 0 {
            return None;
        }
        self.format_duration_totals.get(i)?.checked_div(count)
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
    /// Index of the histogram bucket holding the `p`-th percentile sample,
    /// or `None` when the histogram is empty. The rank is 1-indexed and
    /// ceil-rounded so `p=1.0` selects the last sample and any `p>0` selects
    /// at least the first. Ranks against the histogram's own sample count
    /// (not `total_matches()`) so the quantile stays correct even if the two
    /// ever drift. Shared by the duration / turn-count / win-life-delta
    /// percentile readouts so the rank math lives in exactly one place.
    fn percentile_bucket(buckets: &[u32], p: f32) -> Option<usize> {
        let total: u64 = buckets.iter().map(|&n| n as u64).sum();
        if total == 0 {
            return None;
        }
        let p = p.clamp(0.0, 1.0);
        // Rank in f64: `total as f32` loses integer precision past 2^24
        // (~16.7M samples), which would drift the quantile on a long-running
        // benchmark server. f64 is exact for the u64 sample counts we see. The
        // 1e-6 nudge absorbs the representation error of promoting a non-exact
        // f32 `p` (e.g. 0.3 → 0.30000001), so `ceil(total*p)` still lands on
        // the mathematically intended integer rank rather than one above it.
        let target = ((total as f64) * (p as f64) - 1e-6).ceil().max(1.0) as u64;
        let mut acc = 0u64;
        for (i, &n) in buckets.iter().enumerate() {
            acc = acc.saturating_add(n as u64);
            if acc >= target {
                return Some(i);
            }
        }
        // Unreachable when total > 0; fall back to the open-ended bucket.
        Some(buckets.len().saturating_sub(1))
    }

    pub(crate) fn percentile(&self, p: f32) -> Duration {
        Self::percentile_bucket(&self.duration_buckets, p)
            .map(Self::bucket_upper_bound)
            .unwrap_or(Duration::ZERO)
    }
    /// Turn-count analogue of [`percentile`](Self::percentile): the
    /// upper turn-count bound of the bucket containing the `p`-th
    /// percentile match, walking `turn_buckets`. Returns 0 when no
    /// matches have completed. Lets operators read the game-length
    /// distribution centre (p50) and tail (p95) directly instead of
    /// eyeballing the histogram columns.
    pub(crate) fn turn_percentile(&self, p: f32) -> u32 {
        Self::percentile_bucket(&self.turn_buckets, p)
            .map(Self::turn_bucket_upper_bound)
            .unwrap_or(0)
    }
    /// Interquartile range of final turn counts (p75 − p25), a spread measure
    /// that — unlike [`turn_count_stddev`](Self::turn_count_stddev) — ignores
    /// the blowout tail entirely. A tight IQR next to a wide σ is the signature
    /// of a mostly-consistent length distribution with a few runaway grinds.
    /// Returns 0 when no matches have completed. Bucket-quantised like the other
    /// `turn_*` percentile readouts.
    pub(crate) fn turn_count_iqr(&self) -> u32 {
        self.turn_percentile(0.75).saturating_sub(self.turn_percentile(0.25))
    }
    /// Coefficient of variation of final turn counts (σ / mean, as a percent).
    /// The turn-count analogue of [`win_life_delta_cv_pct`](Self::
    /// win_life_delta_cv_pct): a scale-free spread readout that stays comparable
    /// as the average length drifts, so a rising CV flags a swingier
    /// fast-vs-grind length distribution even when the mean shifts. Returns 0
    /// with no matches or a non-positive mean.
    pub(crate) fn turn_count_cv_pct(&self) -> u64 {
        let mean = self.avg_turns();
        if self.total_matches() == 0 || mean == 0 {
            return 0;
        }
        (self.turn_count_stddev() as f64 * 100.0 / mean as f64).round() as u64
    }
    /// Label of the most-populated turn-length bucket — the *modal* game
    /// length. Unlike the mean or the percentiles it names the single most
    /// common length band, which is the readout that survives a bimodal
    /// "fast concession vs. long grind" split (where the mean lands in an
    /// empty valley between the two humps). Returns `None` before any match
    /// has recorded a turn count; ties resolve to the shorter band (first
    /// bucket wins).
    pub(crate) fn turn_count_mode_bucket(&self) -> Option<&'static str> {
        // Fold keeping the FIRST bucket on ties (strictly-greater replaces),
        // so equal-count bands resolve to the shorter game length.
        let mut best: Option<(usize, u32)> = None;
        for (i, &c) in self.turn_buckets.iter().enumerate() {
            if best.is_none_or(|(_, bc)| c > bc) {
                best = Some((i, c));
            }
        }
        best.filter(|&(_, c)| c > 0).map(|(i, _)| Self::turn_bucket_label(i))
    }
    /// Share (percent) of completed matches that ended in five turns or fewer
    /// (`turn_buckets` 0–1). The turn-length analogue of `close_win_pct`: a
    /// high value flags an aggro-dominated / mana-screw-prone ladder where
    /// games routinely end before the midgame, which the mean/mode can hide
    /// behind a long-grind tail. Returns 0 with no completed matches.
    pub(crate) fn fast_game_pct(&self) -> u64 {
        let total: u64 = self.turn_buckets.iter().map(|&n| n as u64).sum();
        let fast = self.turn_buckets[0] as u64 + self.turn_buckets[1] as u64;
        fast.saturating_mul(100).checked_div(total).unwrap_or(0)
    }
    /// Share (percent) of completed matches that ran thirteen turns or longer
    /// (`turn_buckets` 4–5). The grind-tail complement of
    /// [`fast_game_pct`](Self::fast_game_pct): a high value flags a
    /// control-/stall-dominated ladder where games routinely reach the late
    /// game, which the mean/mode can hide behind a fast-concession spike.
    /// Returns 0 with no completed matches.
    pub(crate) fn slow_game_pct(&self) -> u64 {
        let total: u64 = self.turn_buckets.iter().map(|&n| n as u64).sum();
        let slow = self.turn_buckets[4] as u64 + self.turn_buckets[5] as u64;
        slow.saturating_mul(100).checked_div(total).unwrap_or(0)
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
    /// The match-duration analogue of [`turn_count_cv_pct`](Self::
    /// turn_count_cv_pct) and [`win_life_delta_cv_pct`](Self::
    /// win_life_delta_cv_pct): duration σ as a percent of the mean duration, a
    /// scale-free spread readout that stays comparable as the average match
    /// length drifts (a 30 s σ means something very different for 1-minute vs
    /// 10-minute games). Returns 0 with no matches or a non-positive mean.
    pub(crate) fn duration_cv_pct(&self) -> u64 {
        let n = self.total_matches();
        if n == 0 {
            return 0;
        }
        let mean_ms = self.total_duration.as_millis() as f64 / n as f64;
        if mean_ms <= 0.0 {
            return 0;
        }
        (self.duration_stddev().as_millis() as f64 * 100.0 / mean_ms).round() as u64
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
    // Turn-length distribution: median / tail / spread from the histogram +
    // running σ. Surfaces the game-length shape (fast concessions vs. grinds)
    // that the average alone hides. `turn_percentile`/`turn_count_stddev` were
    // computed but never rendered; gate on a handful of matches so short logs
    // stay tight.
    if n >= 5 {
        out.push_str(&format!(
            " turns_p50={} p95={} (σ={:.1}, cv={}%, fast={}%)",
            s.turn_percentile(0.5),
            s.turn_percentile(0.95),
            s.turn_count_stddev(),
            s.turn_count_cv_pct(),
            s.fast_game_pct(),
        ));
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
        // Draw rate over *all* matches — a stalemate-regression gauge that the
        // resolved-only `decisive_pct` can't show. Only when draws exist.
        if s.draws > 0 {
            out.push_str(&format!(" draw_rate={}%", s.draw_pct()));
            // Are stalemates longer grinds than decided games? Show the split
            // averages so a "draws run 3× longer" pattern is visible at a glance.
            out.push_str(&format!(
                " turns(win/draw)={}/{}",
                s.avg_decisive_turns(),
                s.avg_draw_turns()
            ));
        }
        // Unresolved (disconnect / watchdog) matches — surfaced explicitly so a
        // hang regression is visible instead of hiding in `total - wins - draws`.
        let stuck = s.unresolved();
        if stuck > 0 {
            out.push_str(&format!(" stuck={stuck}"));
        }
        // Alternate-win split: how many of those wins closed via
        // something other than lethal face damage (deckout / poison /
        // mill / win-the-game). Only rendered when at least one such
        // win has been seen so the common all-damage case stays tight.
        if s.deckout_wins > 0 {
            out.push_str(&format!(" alt_wins={} ({}%)", s.deckout_wins, s.deckout_pct()));
            // The complementary lethal-damage share, so the split reads
            // directly (dmg + alt == wins) without subtracting in your head.
            out.push_str(&format!(" dmg_wins={} ({}%)", s.damage_wins, s.damage_pct()));
            // Split the alternate-win share into its two main paths when seen.
            if s.poison_wins > 0 {
                out.push_str(&format!(
                    " poison={} ({}%, {}% of alt)",
                    s.poison_wins,
                    s.poison_pct(),
                    s.poison_of_alt_pct()
                ));
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
            if s.other_wins > 0 {
                out.push_str(&format!(" other={} ({}%)", s.other_wins, s.other_pct()));
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
                " avg_win_life_lead={} (σ={:.1}, cv={}%, p50={}, p90={})",
                s.avg_win_life_delta(),
                s.win_life_delta_stddev(),
                s.win_life_delta_cv_pct(),
                s.win_life_delta_median(),
                s.win_life_delta_percentile(0.9)
            ));
        }
        // Board development at victory — pairs with turns(win) to separate a
        // fast face-damage win (small board) from a grindy attrition win.
        if s.winner_board_samples > 0 {
            out.push_str(&format!(" avg_win_board={}", s.avg_winner_board()));
            if let (Some(mn), Some(mx)) = (s.winner_board_min, s.winner_board_max) {
                out.push_str(&format!(" (board {mn}–{mx})"));
            }
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
            " p50≤{}, p95≤{}, σ={} (cv={}%) (turns p50≤{}, p95≤{}, σ={:.1})",
            format_duration(s.percentile(0.50)),
            format_duration(s.percentile(0.95)),
            format_duration(s.duration_stddev()),
            s.duration_cv_pct(),
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
                format_label_for_bucket(i).map(|label| {
                    match (s.format_avg_turns(i), s.format_avg_duration_secs(i)) {
                        (Some(t), Some(secs)) => format!("{label}:{count}({t}t,{secs}s)"),
                        (Some(t), None) => format!("{label}:{count}({t}t)"),
                        _ => format!("{label}:{count}"),
                    }
                })
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
    use super::MatchStats;
    use crabomination::server::LossReason;

    #[test]
    fn winner_board_average_tracks_only_the_winning_seat() {
        let mut s = MatchStats::default();
        // Seat 0 wins with 5 permanents; seat 1's board is ignored.
        s.observe_winner_board(0, &[5, 12]);
        s.observe_winner_board(1, &[3, 7]); // seat 1 wins with 7
        assert_eq!(s.winner_board_samples, 2);
        assert_eq!(s.avg_winner_board(), 6); // (5 + 7) / 2
        assert_eq!((s.winner_board_min, s.winner_board_max), (Some(5), Some(7)),
            "extremes track the winning boards, not the losers' 12/3");
        // σ of {5, 7} about mean 6 is 1.0.
        assert!((s.winner_board_stddev() - 1.0).abs() < 1e-6, "population σ of the two winning boards");
        // A winner with no board snapshot is skipped, not counted as 0.
        s.observe_winner_board(0, &[]);
        assert_eq!(s.winner_board_samples, 2, "empty snapshot skipped");
        assert_eq!(s.avg_winner_board(), 6);
        assert!((s.winner_board_stddev() - 1.0).abs() < 1e-6, "empty snapshot doesn't perturb σ");
    }

    #[test]
    fn poison_of_alt_pct_reads_share_among_alternate_wins() {
        // Three alternate wins (deckout umbrella), one of them poison.
        let s = MatchStats { wins: 4, deckout_wins: 3, poison_wins: 1, ..Default::default() };
        // 1/4 of all wins, but 1/3 of the alternate wins.
        assert_eq!(s.poison_pct(), 25);
        assert_eq!(s.poison_of_alt_pct(), 33);
        // A poison-only win increments both umbrella and sub-bucket.
        let mut s2 = MatchStats { wins: 1, ..Default::default() };
        s2.observe_win_kind(0, &[20, 15], &[None, Some(LossReason::Poison)]);
        assert_eq!(s2.poison_wins, 1);
        assert_eq!(s2.poison_of_alt_pct(), 100);
    }

    #[test]
    fn concede_win_is_its_own_alternate_sub_bucket() {
        // A concession is an alternate win (deckout umbrella) counted under
        // `concede_wins`, kept distinct from the "you lose the game" residue.
        let mut s = MatchStats { wins: 1, ..Default::default() };
        s.observe_win_kind(0, &[20, 12], &[None, Some(LossReason::Conceded)]);
        assert_eq!(s.deckout_wins, 1, "concession is an alternate win");
        assert_eq!(s.concede_wins, 1, "counted as a concession");
        assert_eq!(s.other_wins, 0, "not lumped into the other bucket");
        assert_eq!(s.damage_wins, 0);
    }

    #[test]
    fn percentile_bucket_empty_is_none() {
        assert_eq!(MatchStats::percentile_bucket(&[0; 6], 0.5), None);
    }

    #[test]
    fn seat_win_share_pct_splits_by_seat() {
        // No wins → 0 (no data).
        let s = MatchStats::default();
        assert_eq!(s.seat_win_share_pct(0), 0);
        // 3 wins for seat 0, 1 for seat 1 → 75 / 25.
        let mut s = MatchStats::default();
        s.seat_wins[0] = 3;
        s.seat_wins[1] = 1;
        assert_eq!(s.seat_win_share_pct(0), 75);
        assert_eq!(s.seat_win_share_pct(1), 25);
        // Out-of-range seat → 0, never a panic.
        assert_eq!(s.seat_win_share_pct(99), 0);
    }

    #[test]
    fn close_win_pct_counts_the_two_tightest_buckets() {
        // No margin samples → 0, never a divide-by-zero.
        assert_eq!(MatchStats::default().close_win_pct(), 0);
        // Feed four wins: margins 0 and 2 are nail-biters (buckets 0/1),
        // margins 8 and 20 are blowouts (buckets 3/5). Half are close.
        let mut s = MatchStats::default();
        for (w, life) in [(0, [20, 20]), (0, [20, 18]), (0, [20, 12]), (0, [20, 0])] {
            s.observe_win_life_delta(w, &life);
        }
        assert_eq!(s.close_win_pct(), 50);
        // Only the 20-margin win (bucket 5) is a blowout — 1 of 4.
        assert_eq!(s.blowout_win_pct(), 25);
        assert_eq!(MatchStats::default().blowout_win_pct(), 0);
    }

    #[test]
    fn fast_game_pct_counts_short_matches() {
        // No matches → 0, never a divide-by-zero.
        assert_eq!(MatchStats::default().fast_game_pct(), 0);
        // Four matches: 2 turns and 5 turns are "fast" (buckets 0/1); 10 and
        // 25 turns are not. Half the games ended by turn 5.
        let mut s = MatchStats::default();
        for turns in [2, 5, 10, 25] {
            s.observe_turns(turns);
        }
        assert_eq!(s.fast_game_pct(), 50);
    }

    #[test]
    fn slow_game_pct_counts_grind_matches() {
        assert_eq!(MatchStats::default().slow_game_pct(), 0);
        // 2 and 5 turns are fast; 15 and 25 turns land in buckets 4/5 (≥13t).
        let mut s = MatchStats::default();
        for turns in [2, 5, 15, 25] {
            s.observe_turns(turns);
        }
        assert_eq!(s.slow_game_pct(), 50);
    }

    #[test]
    fn percentile_bucket_ceil_rank_and_extremes() {
        // 10 samples split 3/0/0/0/0/7: p=0.3 lands in bucket 0 (rank 3),
        // p=0.31 crosses into the tail bucket (rank 4 > 3), p=1.0 the last.
        let h = [3u32, 0, 0, 0, 0, 7];
        assert_eq!(MatchStats::percentile_bucket(&h, 0.0), Some(0));
        assert_eq!(MatchStats::percentile_bucket(&h, 0.3), Some(0));
        assert_eq!(MatchStats::percentile_bucket(&h, 0.31), Some(5));
        assert_eq!(MatchStats::percentile_bucket(&h, 1.0), Some(5));
    }

    #[test]
    fn percentile_bucket_precise_past_f32_range() {
        // 2^24 samples in bucket 0 plus one in the tail (total = 2^24 + 1,
        // odd). With f32 rank math `total as f32` rounds 16_777_217 to the
        // nearest-even 16_777_216, so p=1.0 target=16_777_216 lands in
        // bucket 0 — dropping the tail sample. f64 keeps the exact rank.
        let h = [1u32 << 24, 0, 0, 0, 0, 1];
        assert_eq!(MatchStats::percentile_bucket(&h, 1.0), Some(5), "tail sample survives");
    }

    #[test]
    fn public_percentiles_route_through_shared_helper() {
        let mut s = MatchStats::default();
        // One short match, one long one → p50 sits in an early bucket, p100 last.
        s.observe_duration(std::time::Duration::from_secs(10));
        s.observe_duration(std::time::Duration::from_secs(1200));
        assert_eq!(s.percentile(0.0), std::time::Duration::from_secs(30));
        assert_eq!(s.percentile(1.0), std::time::Duration::from_secs(3600));
        // Empty turn histogram still reads 0 rather than panicking.
        assert_eq!(MatchStats::default().turn_percentile(0.9), 0);
    }

    #[test]
    fn turn_count_iqr_is_p75_minus_p25() {
        let mut s = MatchStats::default();
        // Turns 2,4,10,25 → buckets 0,1,3,5. p25 → bucket 0 (upper 2),
        // p75 → bucket 3 (upper 12), so IQR = 12 − 2 = 10.
        for t in [2u32, 4, 10, 25] {
            s.observe_turns(t);
        }
        assert_eq!(s.turn_percentile(0.25), 2);
        assert_eq!(s.turn_percentile(0.75), 12);
        assert_eq!(s.turn_count_iqr(), 10);
        // Empty histogram reads 0 rather than underflowing.
        assert_eq!(MatchStats::default().turn_count_iqr(), 0);
    }

    #[test]
    fn turn_count_cv_is_stddev_over_mean_percent() {
        // Two matches, 10 and 20 turns → mean 15, σ 5 → cv 33%.
        let mut s = MatchStats { bot_matches: 2, ..Default::default() };
        s.observe_turns(10);
        s.observe_turns(20);
        assert_eq!(s.turn_count_stddev().round() as u64, 5);
        assert_eq!(s.turn_count_cv_pct(), 33);
        // No matches → 0 rather than dividing by zero.
        assert_eq!(MatchStats::default().turn_count_cv_pct(), 0);
    }

    #[test]
    fn duration_cv_is_stddev_over_mean_percent() {
        use std::time::Duration;
        // Two matches, 10 s and 20 s → mean 15 s, σ 5 s → cv 33%.
        let mut s = MatchStats { bot_matches: 2, ..Default::default() };
        s.observe_duration(Duration::from_secs(10));
        s.observe_duration(Duration::from_secs(20));
        assert_eq!(s.duration_stddev().as_secs(), 5);
        assert_eq!(s.duration_cv_pct(), 33);
        // No matches → 0 rather than dividing by zero.
        assert_eq!(MatchStats::default().duration_cv_pct(), 0);
    }
}
