//! Rolling match statistics and the human-readable log summaries.

use std::time::Duration;

use crate::config::Format;
use crabomination::server::LossReason;

/// Process-wide running totals of completed matches, rendered by
/// `format_match_stats` as the rolling summary line logged after each match.
/// Lives in a process-global `OnceLock<Mutex<_>>` (no per-thread `Arc`).
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct MatchStats {
    pub(crate) bot_matches: u64,
    pub(crate) pair_matches: u64,
    /// Sum of match durations.
    pub(crate) total_duration: Duration,
    /// Shortest match (`None` until the first); surfaces instant disconnects.
    pub(crate) min_duration: Option<Duration>,
    /// Longest match (`None` until the first); surfaces stalls.
    pub(crate) max_duration: Option<Duration>,
    /// Duration histogram: `<30s`, `30s-1m`, `1-2m`, `2-5m`, `5-10m`, `10m+`.
    /// A spike in `<30s` usually means bots conceding turn 1.
    pub(crate) duration_buckets: [u32; 6],
    /// Completed matches per `Format`, indexed by `format_index`.
    pub(crate) format_buckets: [u64; FORMAT_BUCKET_COUNT],
    /// Sum of final turn counts per format (indexed like `format_buckets`),
    /// for the per-format average game length (`demo:7(9t)`).
    pub(crate) format_turn_totals: [u64; FORMAT_BUCKET_COUNT],
    /// Sum of wall-clock seconds per format, so a format slow in real time
    /// (long decision timeouts) reads apart from one merely long in turns.
    pub(crate) format_duration_totals: [u64; FORMAT_BUCKET_COUNT],
    /// Sum of final turn counts; average = this / `total_matches()`.
    pub(crate) total_turns: u64,
    /// Longest final turn count (`None` until the first match).
    pub(crate) max_turns: Option<u32>,
    /// Shortest final turn count (`None` until the first match).
    pub(crate) min_turns: Option<u32>,
    /// Matches that ended in a draw (`winner == Some(None)`), e.g. a bot loop
    /// neither side can finish.
    pub(crate) draws: u64,
    /// Matches with a declared winner (`Some(Some(seat))`). Disconnect /
    /// watchdog exits are excluded, so `wins + draws <= total_matches`.
    pub(crate) wins: u64,
    /// Wins per seat (seats past the cap fold into the last bucket); flags
    /// turn-order bias in bot ladders.
    pub(crate) seat_wins: [u64; SEAT_BUCKET_COUNT],
    /// Sum over wins of `winner_life - max_opponent_life`, clamped at 0; the
    /// average separates blowouts from races.
    pub(crate) cumulative_win_life_delta: i64,
    /// Sum of squared win-life deltas, for the population σ. Deltas are ≥ 0;
    /// `u128` keeps a long run from overflowing.
    pub(crate) cumulative_win_life_delta_squared: u128,
    /// Wins counted in `cumulative_win_life_delta`. Can trail `wins`: a win
    /// with no life data is skipped there.
    pub(crate) win_life_samples: u64,
    /// Win-life-delta histogram: `0`, `1-3`, `4-6`, `7-10`, `11-15`, `16+`.
    /// Gives the shape and an outlier-robust median.
    pub(crate) win_life_delta_buckets: [u32; 6],
    /// Final-turn histogram: `1-2`, `3-5`, `6-8`, `9-12`, `13-20`, `21+`. A fat
    /// first bucket flags a concession regression even when a long outlier keeps
    /// the average high.
    pub(crate) turn_buckets: [u32; 6],
    /// Wins where every loser ended above 0 life ("alternate" wins: decking,
    /// poison, concession, win-the-game effects). A rise relative to `wins` flags
    /// bots grinding to empty libraries instead of closing on life.
    pub(crate) deckout_wins: u64,
    /// Wins via lethal face damage (every loser at ≤ 0 life);
    /// `damage_wins + deckout_wins == wins` when life or reason data exists.
    pub(crate) damage_wins: u64,
    /// Subset of `deckout_wins` with a loser eliminated by poison (CR 104.3c),
    /// classified from `loss_reasons`, not life totals.
    pub(crate) poison_wins: u64,
    /// Subset of `deckout_wins` with a loser decked (CR 104.3a).
    pub(crate) deck_wins: u64,
    /// Subset of `deckout_wins` with a loser killed by 21+ damage from one
    /// commander (CR 903.10a).
    pub(crate) commander_damage_wins: u64,
    /// Subset of `deckout_wins` with a loser who conceded (CR 104.3a), kept
    /// apart from `other_wins` so rage-quits and bot timeouts are visible.
    pub(crate) concede_wins: u64,
    /// Sum of final turns over concession wins, for
    /// [`avg_concede_turn`](Self::avg_concede_turn): a low value means bots give up
    /// too early.
    pub(crate) concede_turn_sum: u64,
    /// Subset of `deckout_wins` with a loser out for any other reason (a "you
    /// lose the game" effect, CR 104.3g).
    pub(crate) other_wins: u64,
    /// Sum of squared final turn counts, for
    /// [`turn_count_stddev`](Self::turn_count_stddev).
    pub(crate) total_turns_squared: u128,
    /// Sum of final turn counts over decisive matches.
    pub(crate) decisive_turn_sum: u64,
    /// Sum of final turn counts over drawn matches.
    pub(crate) draw_turn_sum: u64,
    /// Sum of squared durations in milliseconds, for
    /// [`duration_stddev`](Self::duration_stddev); ms keeps the squares well inside
    /// `u128`.
    pub(crate) total_duration_squared_ms: u128,
    /// Matches with no declared outcome (`observe_winner(None)`: disconnect,
    /// watchdog, stuck loop); a rise flags a hang regression.
    pub(crate) inconclusive: u64,
    /// Sum of the winner's end-of-match permanent count. Next to the decisive
    /// turn average it separates fast face-damage wins from wide attrition wins.
    /// Skipped when `final_board_sizes` is empty (aborted match).
    pub(crate) winner_board_sum: u64,
    /// Sum of squared winner board sizes, for the population σ.
    pub(crate) winner_board_sum_squared: u128,
    /// Wins counted in `winner_board_sum` (can trail `wins`).
    pub(crate) winner_board_samples: u64,
    /// Narrowest / widest winning board (`None` until the first sample); a
    /// `min` of 0 means some win came off an empty board.
    pub(crate) winner_board_min: Option<u32>,
    pub(crate) winner_board_max: Option<u32>,
}

/// Per-seat win buckets: 1v1 plus 4-player Commander pods. Higher seats
/// fold into the last bucket.
pub(crate) const SEAT_BUCKET_COUNT: usize = 4;

/// Bucket count for `format_buckets`, one per `Format`; bump it when adding one.
pub(crate) const FORMAT_BUCKET_COUNT: usize = 4;

/// Bucket index of a `Format` in `MatchStats.format_buckets`. Stable order;
/// new formats append.
pub(crate) fn format_index(f: Format) -> usize {
    match f {
        Format::Demo => 0,
        Format::Cube => 1,
        Format::Sos => 2,
        Format::Commander => 3,
    }
}

/// Label for a format-bucket index; `None` past the last format.
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
    /// Fold a completed match into every counter: match kind, turns, winner and
    /// seat, and for decisive games the life delta, win kind and board. The one
    /// recording path for `run_bot_match` and `run_pair_match`.
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
                if outcome
                    .loss_reasons
                    .iter()
                    .enumerate()
                    .any(|(i, r)| i != w && *r == Some(LossReason::Conceded))
                {
                    self.concede_turn_sum =
                        self.concede_turn_sum.saturating_add(outcome.final_turn as u64);
                }
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
    /// Average draw length as a percent of average decisive length: 200 means
    /// draws grind twice as long. 0 when either sample is empty.
    pub(crate) fn stalemate_grind_pct(&self) -> u64 {
        let decisive = self.avg_decisive_turns();
        let draw = self.avg_draw_turns();
        if decisive == 0 || draw == 0 { return 0; }
        draw.saturating_mul(100) / decisive
    }
    /// Record one match's final turn count. Call exactly once per match.
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
    /// `turn_buckets` index for a final turn count.
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
    /// Count a win (and its seat), a draw (`Some(None)`), or an inconclusive
    /// match (`None`: disconnect or watchdog).
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
    /// Percent of all matches with no declared outcome.
    pub(crate) fn inconclusive_pct(&self) -> u64 {
        let total = self.total_matches();
        if total == 0 { return 0; }
        self.inconclusive.saturating_mul(100) / total
    }
    /// Percent of *all* matches that were draws (`decisive_pct` excludes
    /// inconclusive games from its denominator).
    pub(crate) fn draw_pct(&self) -> u64 {
        let total = self.total_matches();
        if total == 0 { return 0; }
        self.draws.saturating_mul(100) / total
    }
    /// Percent of matches that were bot-vs-bot rather than paired humans: ~100
    /// on a self-play soak. 0 before any match.
    pub(crate) fn bot_match_pct(&self) -> u64 {
        let total = self.total_matches();
        if total == 0 { return 0; }
        self.bot_matches.saturating_mul(100) / total
    }
    /// Seat 0's share of seated wins, as a percent; far from 50 over a long
    /// ladder flags turn-order bias. 50 with no wins.
    pub(crate) fn first_seat_win_pct(&self) -> u64 {
        let seated: u64 = self.seat_wins.iter().sum();
        if seated == 0 { return 50; }
        self.seat_wins[0].saturating_mul(100) / seated
    }
    /// Seat `seat`'s share of seated wins, as a percent; 0 with no wins or an out-of-range seat.
    pub(crate) fn seat_win_share_pct(&self, seat: usize) -> u64 {
        let seated: u64 = self.seat_wins.iter().sum();
        if seated == 0 {
            return 0;
        }
        self.seat_wins.get(seat).copied().unwrap_or(0).saturating_mul(100) / seated
    }
    /// Record one win's life margin: winner life minus the best opponent's,
    /// clamped ≥ 0 (a double loss at negative life can't go below). No-op when the
    /// winner seat has no life data.
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
    /// `win_life_delta_buckets` index for a (≥ 0) delta.
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
    /// Upper bound of win-life-delta bucket `i` (the open last bucket reports 16).
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
    /// `p`-th percentile win-life delta (bucket upper edge; 0 with no samples).
    /// `p` is clamped to [0, 1].
    pub(crate) fn win_life_delta_percentile(&self, p: f32) -> i64 {
        Self::percentile_bucket(&self.win_life_delta_buckets, p)
            .map(Self::win_life_delta_bucket_upper_bound)
            .unwrap_or(0)
    }
    /// The median (p50) win-by-life delta.
    pub(crate) fn win_life_delta_median(&self) -> i64 {
        self.win_life_delta_percentile(0.5)
    }
    /// Interquartile range (p75 − p25) of the win-life delta; a tight IQR next to
    /// a wide σ means a few runaway stomps. Bucket-quantised; 0 with no samples.
    pub(crate) fn win_life_delta_iqr(&self) -> i64 {
        self.win_life_delta_percentile(0.75)
            .saturating_sub(self.win_life_delta_percentile(0.25))
    }
    /// Percent of margin-tracked wins with a margin of 3 or less (buckets 0–1); 0 with no samples.
    pub(crate) fn close_win_pct(&self) -> u64 {
        let total: u64 = self.win_life_delta_buckets.iter().map(|&n| n as u64).sum();
        let close =
            self.win_life_delta_buckets[0] as u64 + self.win_life_delta_buckets[1] as u64;
        close.saturating_mul(100).checked_div(total).unwrap_or(0)
    }
    /// Percent of margin-tracked wins with a margin above 15 (bucket 5), the
    /// upper-tail mirror of [`close_win_pct`](Self::close_win_pct). 0 with no
    /// samples.
    pub(crate) fn blowout_win_pct(&self) -> u64 {
        let total: u64 = self.win_life_delta_buckets.iter().map(|&n| n as u64).sum();
        (self.win_life_delta_buckets[5] as u64).saturating_mul(100).checked_div(total).unwrap_or(0)
    }
    /// Classify one win as damage or alternate. Uses the losers' `loss_reasons`
    /// when present (any non-life loss makes it alternate, and each reason bumps its
    /// sub-bucket); otherwise falls back to life totals.
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
    /// Record the winner's board size; no-op without a snapshot for that seat.
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

    /// Average winning board size; 0 with no samples.
    pub(crate) fn avg_winner_board(&self) -> u64 {
        self.winner_board_sum.checked_div(self.winner_board_samples).unwrap_or(0)
    }

    /// Population σ of winning board sizes; 0.0 with no samples.
    pub(crate) fn winner_board_stddev(&self) -> f32 {
        if self.winner_board_samples == 0 {
            return 0.0;
        }
        let n = self.winner_board_samples as f64;
        let mean = self.winner_board_sum as f64 / n;
        let mean_sq = self.winner_board_sum_squared as f64 / n;
        (mean_sq - mean * mean).max(0.0).sqrt() as f32
    }

    /// Coefficient of variation (σ / mean, percent) of winning board size, so
    /// board consistency compares across burn and go-wide formats. 0 with no
    /// samples or a zero mean.
    pub(crate) fn winner_board_cv_pct(&self) -> u64 {
        let mean = self.avg_winner_board();
        if self.winner_board_samples == 0 || mean == 0 {
            return 0;
        }
        (self.winner_board_stddev() as f64 * 100.0 / mean as f64).round() as u64
    }

    /// Average win-life delta; 0 with no samples.
    pub(crate) fn avg_win_life_delta(&self) -> i64 {
        if self.win_life_samples == 0 {
            0
        } else {
            self.cumulative_win_life_delta / (self.win_life_samples as i64)
        }
    }

    /// Population σ of the win-life delta; 0.0 with no samples.
    pub(crate) fn win_life_delta_stddev(&self) -> f32 {
        if self.win_life_samples == 0 {
            return 0.0;
        }
        let n = self.win_life_samples as f64;
        let mean = self.cumulative_win_life_delta as f64 / n;
        let mean_sq = self.cumulative_win_life_delta_squared as f64 / n;
        (mean_sq - mean * mean).max(0.0).sqrt() as f32
    }
    /// Coefficient of variation (σ / mean, percent) of the win-life delta. 0 with
    /// no samples or a non-positive mean.
    pub(crate) fn win_life_delta_cv_pct(&self) -> u64 {
        let mean = self.avg_win_life_delta();
        if self.win_life_samples == 0 || mean <= 0 {
            return 0;
        }
        (self.win_life_delta_stddev() as f64 * 100.0 / mean as f64).round() as u64
    }
    /// Percent of resolved matches (wins + draws) with a winner; 0 if none. Drops
    /// on stalemate regressions; disconnects don't count.
    pub(crate) fn decisive_pct(&self) -> u64 {
        let resolved = self.wins + self.draws;
        self.wins.saturating_mul(100).checked_div(resolved).unwrap_or(0)
    }
    /// Matches with neither a win nor a draw (`total - wins - draws`,
    /// saturating); a rise flags a hang or crash regression.
    pub(crate) fn unresolved(&self) -> u64 {
        self.total_matches().saturating_sub(self.wins + self.draws)
    }
    /// `unresolved()` as a percent of all matches. Unlike
    /// `inconclusive_pct` it derives from the match counter, so it also counts a
    /// match that never reported a winner at all. 0 before any match.
    pub(crate) fn unresolved_pct(&self) -> u64 {
        let total = self.total_matches();
        if total == 0 { return 0; }
        self.unresolved().saturating_mul(100) / total
    }
    /// Percent of wins that were alternate (not lethal face damage); 0 with no wins.
    pub(crate) fn deckout_pct(&self) -> u64 {
        self.deckout_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins via lethal face damage; 0 with no wins.
    pub(crate) fn damage_pct(&self) -> u64 {
        self.damage_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins with a poison loss (CR 104.3c); 0 with no wins.
    pub(crate) fn poison_pct(&self) -> u64 {
        self.poison_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Poison's share of alternate wins (not of all wins); 0 with no alternate wins.
    pub(crate) fn poison_of_alt_pct(&self) -> u64 {
        self.poison_wins.saturating_mul(100).checked_div(self.deckout_wins).unwrap_or(0)
    }
    /// Percent of wins with a deck-out loss (CR 104.3a); 0 with no wins.
    pub(crate) fn deck_pct(&self) -> u64 {
        self.deck_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins with an "other" loss (CR 104.3g); 0 with no wins.
    pub(crate) fn other_pct(&self) -> u64 {
        self.other_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Average final turn of concession wins; 0 with none. A low value flags an
    /// evaluation that quits winnable games.
    pub(crate) fn avg_concede_turn(&self) -> u64 {
        self.concede_turn_sum.checked_div(self.concede_wins).unwrap_or(0)
    }
    /// Average concede turn as a percent of average decisive length: below 100
    /// means seats concede early. 0 when either sample is empty.
    pub(crate) fn concede_earliness_pct(&self) -> u64 {
        let concede = self.avg_concede_turn();
        let decisive = self.avg_decisive_turns();
        if concede == 0 || decisive == 0 { return 0; }
        concede.saturating_mul(100) / decisive
    }
    /// Percent of wins via 21+ commander damage (CR 903.10a); 0 with no wins.
    pub(crate) fn commander_damage_pct(&self) -> u64 {
        self.commander_damage_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Percent of wins with a concession (CR 104.3a); 0 with no wins.
    pub(crate) fn concede_pct(&self) -> u64 {
        self.concede_wins.saturating_mul(100).checked_div(self.wins).unwrap_or(0)
    }
    /// Average final turn over all matches; 0 before any match.
    pub(crate) fn avg_turns(&self) -> u64 {
        let n = self.total_matches();
        self.total_turns.checked_div(n).unwrap_or(0)
    }
    /// Count one match against its format.
    pub(crate) fn observe_format(&mut self, f: Format) {
        let idx = format_index(f).min(FORMAT_BUCKET_COUNT - 1);
        self.format_buckets[idx] = self.format_buckets[idx].saturating_add(1);
    }
    /// Add one match's final turn count to its format's total.
    pub(crate) fn observe_format_turns(&mut self, f: Format, turns: u32) {
        let idx = format_index(f).min(FORMAT_BUCKET_COUNT - 1);
        self.format_turn_totals[idx] = self.format_turn_totals[idx].saturating_add(turns as u64);
    }
    /// Average final turn for format bucket `i`, or `None` without turn data (so
    /// the summary prints `label:count`, not a misleading `(0t)`).
    pub(crate) fn format_avg_turns(&self, i: usize) -> Option<u64> {
        let total = *self.format_turn_totals.get(i)?;
        if total == 0 {
            return None;
        }
        total.checked_div(*self.format_buckets.get(i)?)
    }
    /// Add one match's wall-clock seconds to its format's total.
    pub(crate) fn observe_format_duration(&mut self, f: Format, d: Duration) {
        let idx = format_index(f).min(FORMAT_BUCKET_COUNT - 1);
        self.format_duration_totals[idx] =
            self.format_duration_totals[idx].saturating_add(d.as_secs());
    }
    /// Average wall-clock seconds for format bucket `i`, or `None` with no matches.
    pub(crate) fn format_avg_duration_secs(&self, i: usize) -> Option<u64> {
        let count = *self.format_buckets.get(i)?;
        if count == 0 {
            return None;
        }
        self.format_duration_totals.get(i)?.checked_div(count)
    }
    /// Record a duration: total, min/max, histogram and squared sum.
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
    /// Histogram bucket holding the `p`-th percentile sample, or `None` when
    /// empty. The rank is 1-indexed and ceil-rounded (`p=1.0` is the last sample,
    /// any `p>0` at least the first), counted against the histogram's own total.
    /// Shared by every percentile readout.
    fn percentile_bucket(buckets: &[u32], p: f32) -> Option<usize> {
        let total: u64 = buckets.iter().map(|&n| n as u64).sum();
        if total == 0 {
            return None;
        }
        let p = p.clamp(0.0, 1.0);
        // Rank in f64: `total as f32` loses precision past 2^24 samples. The 1e-6
        // nudge absorbs f32 `p` promotion error (0.3 -> 0.30000001) so the ceil lands
        // on the intended rank.
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

    /// `p`-th percentile match duration, as the upper edge of its bucket: a
    /// conservative "≤ this" from six coarse buckets. `Duration::ZERO` with no
    /// matches.
    pub(crate) fn percentile(&self, p: f32) -> Duration {
        Self::percentile_bucket(&self.duration_buckets, p)
            .map(Self::bucket_upper_bound)
            .unwrap_or(Duration::ZERO)
    }
    /// Turn-count analogue of [`percentile`](Self::percentile): upper bound of
    /// the `turn_buckets` bucket holding the `p`-th match; 0 with no matches.
    pub(crate) fn turn_percentile(&self, p: f32) -> u32 {
        Self::percentile_bucket(&self.turn_buckets, p)
            .map(Self::turn_bucket_upper_bound)
            .unwrap_or(0)
    }
    /// Interquartile range (p75 − p25) of final turn counts; unlike
    /// [`turn_count_stddev`](Self::turn_count_stddev) it ignores the long tail.
    /// Bucket-quantised; 0 with no matches.
    pub(crate) fn turn_count_iqr(&self) -> u32 {
        self.turn_percentile(0.75).saturating_sub(self.turn_percentile(0.25))
    }
    /// Coefficient of variation (σ / mean, percent) of final turn counts; 0 with
    /// no matches or a zero mean.
    pub(crate) fn turn_count_cv_pct(&self) -> u64 {
        let mean = self.avg_turns();
        if self.total_matches() == 0 || mean == 0 {
            return 0;
        }
        (self.turn_count_stddev() as f64 * 100.0 / mean as f64).round() as u64
    }
    /// Label of the most populated turn bucket (the modal game length), which
    /// survives a bimodal fast/grind split where the mean falls between humps.
    /// `None` with no data; ties go to the shorter band.
    pub(crate) fn turn_count_mode_bucket(&self) -> Option<&'static str> {
        // Strictly-greater replaces, so ties keep the first (shorter) bucket.
        let mut best: Option<(usize, u32)> = None;
        for (i, &c) in self.turn_buckets.iter().enumerate() {
            if best.is_none_or(|(_, bc)| c > bc) {
                best = Some((i, c));
            }
        }
        best.filter(|&(_, c)| c > 0).map(|(i, _)| Self::turn_bucket_label(i))
    }
    /// Percent of matches that ended by turn 5 (`turn_buckets` 0–1); 0 with no matches.
    pub(crate) fn fast_game_pct(&self) -> u64 {
        let total: u64 = self.turn_buckets.iter().map(|&n| n as u64).sum();
        let fast = self.turn_buckets[0] as u64 + self.turn_buckets[1] as u64;
        fast.saturating_mul(100).checked_div(total).unwrap_or(0)
    }
    /// Percent of matches that ran 13+ turns (`turn_buckets` 4–5), the grind
    /// complement of [`fast_game_pct`](Self::fast_game_pct). 0 with no matches.
    pub(crate) fn slow_game_pct(&self) -> u64 {
        let total: u64 = self.turn_buckets.iter().map(|&n| n as u64).sum();
        let slow = self.turn_buckets[4] as u64 + self.turn_buckets[5] as u64;
        slow.saturating_mul(100).checked_div(total).unwrap_or(0)
    }
    /// Population σ of final turn counts; 0.0 with no matches.
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
    /// Population σ of match durations; `Duration::ZERO` with no matches.
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
    /// Coefficient of variation (σ / mean, percent) of match duration, the
    /// duration analogue of [`turn_count_cv_pct`](Self::turn_count_cv_pct). 0 with
    /// no matches or a zero mean.
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
    /// Upper edge of turn bucket `i` (cut points of
    /// [`turn_bucket_index`](Self::turn_bucket_index)); the open `21+` bucket
    /// reports 21.
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
    /// Upper edge of duration bucket `i` (cut points of `bucket_index`).
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
    /// Duration histogram index: thresholds 30s / 1m / 2m / 5m / 10m, then `10m+`.
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
    /// Label for duration bucket `i`.
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
            // Saturating, though overflow would take centuries of play.
            Duration::from_secs(self.total_duration.as_secs().saturating_div(n))
        }
    }
}

pub(crate) static MATCH_STATS: std::sync::OnceLock<std::sync::Mutex<MatchStats>> = std::sync::OnceLock::new();

pub(crate) fn match_stats() -> &'static std::sync::Mutex<MatchStats> {
    MATCH_STATS.get_or_init(|| std::sync::Mutex::new(MatchStats::default()))
}

/// One-line rolling summary appended to each match-completion log
/// (`served N matches: K bot, P pair; avg duration X; ...`). Call after recording
/// the match so it is included.
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
    // Turn-length median, tail and spread; only after a handful of matches.
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
    // Win/draw split, once anything has resolved.
    if s.wins + s.draws > 0 {
        out.push_str(&format!(
            " wins={} draws={} decisive={}%",
            s.wins, s.draws, s.decisive_pct()
        ));
        // Draw rate over all matches, plus whether draws run longer than wins.
        if s.draws > 0 {
            out.push_str(&format!(" draw_rate={}%", s.draw_pct()));
            out.push_str(&format!(
                " turns(win/draw)={}/{}",
                s.avg_decisive_turns(),
                s.avg_draw_turns()
            ));
        }
        // Matches with no win or draw (disconnect / watchdog).
        let stuck = s.unresolved();
        if stuck > 0 {
            out.push_str(&format!(" stuck={stuck}"));
        }
        // Alternate-win split, only once one has been seen.
        if s.deckout_wins > 0 {
            out.push_str(&format!(" alt_wins={} ({}%)", s.deckout_wins, s.deckout_pct()));
            out.push_str(&format!(" dmg_wins={} ({}%)", s.damage_wins, s.damage_pct()));
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
        // Matches `observe_winner` saw as `None`.
        if s.inconclusive > 0 {
            out.push_str(&format!(
                " unresolved={} ({}%)",
                s.inconclusive,
                s.inconclusive_pct()
            ));
        }
        // Per-seat wins (`seat_wins=12/8`), trimmed after the last nonzero seat.
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
        // Seat 0's share, once a second seat has won.
        if last_nonzero >= 1 {
            out.push_str(&format!(" (p0={}%)", s.first_seat_win_pct()));
        }
        // Winner's life margin: high = blowouts, near zero = races.
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
        // Winning board size and range.
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
    // Duration and turn percentiles; skipped on the first match (degenerate).
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
    // Duration histogram, every bucket printed (including zeros) so log
    // greppers can rely on the columns.
    if s.total_matches() > 0 {
        out.push_str(" |");
        for (i, count) in s.duration_buckets.iter().enumerate() {
            out.push_str(&format!(" {}:{}", MatchStats::bucket_label(i), count));
        }
        // Per-format counts (`format=demo:7(9t,240s)`); empty formats are omitted.
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
        // Turn histogram (`turns=1-2:3 3-5:5 ...`), same shape as the duration one.
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

/// Short log rendering of a `Duration`: `1h2m3s` / `5m12s` / `38s` /
/// `420ms` / `<1ms`.
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
        assert_eq!(s.concede_pct(), 100, "the only win was a concession");
    }

    #[test]
    fn concede_pct_is_share_of_wins() {
        let s = MatchStats { wins: 4, deckout_wins: 2, concede_wins: 1, ..Default::default() };
        assert_eq!(s.concede_pct(), 25, "one of four wins was a concession");
        assert_eq!(MatchStats::default().concede_pct(), 0, "no wins → 0");
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
    fn bot_match_pct_is_bot_share_of_all_matches() {
        assert_eq!(MatchStats::default().bot_match_pct(), 0);
        let s = MatchStats { bot_matches: 3, pair_matches: 1, ..Default::default() };
        assert_eq!(s.bot_match_pct(), 75);
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

    #[test]
    fn stalemate_grind_is_draw_over_decisive_length() {
        // Decisive games average 10 turns, draws average 25 → 250%.
        let s = MatchStats {
            wins: 2, decisive_turn_sum: 20,
            draws: 1, draw_turn_sum: 25,
            ..Default::default()
        };
        assert_eq!(s.avg_decisive_turns(), 10);
        assert_eq!(s.avg_draw_turns(), 25);
        assert_eq!(s.stalemate_grind_pct(), 250);
        // Either sample empty → 0 (no ratio defined).
        let only_wins = MatchStats { wins: 1, decisive_turn_sum: 8, ..Default::default() };
        assert_eq!(only_wins.stalemate_grind_pct(), 0);
        assert_eq!(MatchStats::default().stalemate_grind_pct(), 0);
    }

    #[test]
    fn concede_earliness_is_concede_over_decisive_length() {
        // Decisive games average 20 turns; concessions land at turn 12 → 60%.
        let s = MatchStats {
            wins: 4, decisive_turn_sum: 80,
            concede_wins: 2, concede_turn_sum: 24,
            ..Default::default()
        };
        assert_eq!(s.avg_decisive_turns(), 20);
        assert_eq!(s.avg_concede_turn(), 12);
        assert_eq!(s.concede_earliness_pct(), 60);
        // No concessions (or no wins) → 0, never a divide-by-zero.
        let no_concede = MatchStats { wins: 1, decisive_turn_sum: 10, ..Default::default() };
        assert_eq!(no_concede.concede_earliness_pct(), 0);
        assert_eq!(MatchStats::default().concede_earliness_pct(), 0);
    }

    #[test]
    fn unresolved_pct_is_rate_over_all_matches() {
        // 4 matches: 2 wins + 1 draw = 3 resolved → 1 unresolved = 25%.
        let s = MatchStats { bot_matches: 4, wins: 2, draws: 1, ..Default::default() };
        assert_eq!(s.unresolved(), 1);
        assert_eq!(s.unresolved_pct(), 25);
        // No matches → 0 rather than dividing by zero.
        assert_eq!(MatchStats::default().unresolved_pct(), 0);
    }
}
