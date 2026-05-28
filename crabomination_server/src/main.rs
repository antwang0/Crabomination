//! Standalone TCP match server.
//!
//! Listens on `CRAB_BIND` (default `0.0.0.0:7777`). Each pair of incoming TCP
//! connections is paired into a 2-seat match.
//!
//! Environment variables:
//! - `CRAB_BIND` — address to listen on (default `0.0.0.0:7777`).
//! - `CRAB_BOT=1` — instead of pairing two clients, seat each client
//!   against a `RandomBot`.
//! - `CRAB_FORMAT=cube` — build matches with random two-color cube decks
//!   (via `crabomination::cube::build_cube_state`) instead of the default
//!   BRG/Goryo's demo decks. Any other value (or unset) → demo decks.
//! - `CRAB_PAIRING_TIMEOUT_SECS` — how long the first client of an unpaired
//!   match waits for an opponent before being dropped (default 300 = 5 min).
//!   Only applies in pair mode.
//! - `CRAB_MAX_CONNS` — total concurrent connection slots (default 100). A
//!   pair-mode match holds two slots (one per seat). `0` = unlimited.
//! - `CRAB_MAX_CONNS_PER_IP` — concurrent connections from a single remote
//!   IP (default 5). `0` = unlimited. Operates on the raw peer address, so
//!   clients behind a NAT or load balancer share one counter.

use std::collections::HashMap;
use std::env;
use std::io;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crabomination::cube::build_cube_state;
use crabomination::demo::build_demo_state;
use crabomination::game::GameState;
use crabomination::server::{run_match, tcp_seat, RandomBot, SeatOccupant};

/// Format-builder enum that captures the environment configuration once at
/// boot, so each match thread doesn't re-read env vars.
#[derive(Debug, Clone, Copy)]
enum Format {
    Demo,
    Cube,
    Sos,
}

impl Format {
    fn from_env() -> Self {
        match env::var("CRAB_FORMAT").ok().as_deref() {
            Some("cube") => Self::Cube,
            Some("sos") | Some("strixhaven") => Self::Sos,
            Some("demo") | None => Self::Demo,
            Some(other) => {
                eprintln!(
                    "warning: CRAB_FORMAT={other:?} not recognized — \
                     falling back to demo. Valid: \"demo\" | \"cube\" | \"sos\"."
                );
                Self::Demo
            }
        }
    }
    fn build(&self) -> GameState {
        match self {
            Self::Demo => build_demo_state(),
            Self::Cube => build_cube_state(),
            Self::Sos => crabomination::sos_mode::build_sos_state(),
        }
    }
    fn label(&self) -> &'static str {
        match self {
            Self::Demo => "demo",
            Self::Cube => "cube",
            Self::Sos => "sos",
        }
    }
}

/// Default time the first client of a pair waits for an opponent before
/// being dropped. Configurable via `CRAB_PAIRING_TIMEOUT_SECS`.
const DEFAULT_PAIRING_TIMEOUT: Duration = Duration::from_secs(300);

/// Default total concurrent connection slots. A pair match consumes 2.
const DEFAULT_MAX_CONNS: usize = 100;

/// Default concurrent connection slots from any one remote IP.
const DEFAULT_MAX_CONNS_PER_IP: usize = 5;

/// Tracks concurrent connections to enforce global and per-IP caps.
///
/// One [`SlotGuard`] is acquired per accepted connection (so a pair-mode
/// match holds two slots — one per seat, each indexed by the seat's own
/// peer IP). The guard's `Drop` impl releases the counters, so a panicking
/// match thread still frees its slot.
///
/// Per-IP limits operate on the raw remote address, so clients behind a
/// shared NAT or load balancer share one counter. That's the right behavior
/// for a hobby server (the only signal we have is the socket-level peer
/// address); production setups would want X-Forwarded-For unwrapping at a
/// reverse-proxy layer above us.
#[derive(Clone)]
struct SlotManager {
    inner: Arc<Mutex<SlotState>>,
    /// 0 = unlimited.
    global_cap: usize,
    /// 0 = unlimited.
    per_ip_cap: usize,
}

#[derive(Default)]
struct SlotState {
    total: usize,
    per_ip: HashMap<IpAddr, usize>,
}

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
struct MatchStats {
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
    /// Number of matches counted in `cumulative_win_life_delta`. Lets
    /// the formatter compute the average without dividing by `wins`
    /// directly (a winner with no available life data — e.g. a forced
    /// concession — is skipped in the cumulative sum but still counted
    /// in `wins`).
    win_life_samples: u64,
}

/// Cap on per-seat win tracking. Covers 1v1 (seats 0, 1) plus headroom
/// for 4-player Commander pods. Wins for seats ≥ this cap fall into
/// the last bucket so the array doesn't overflow on exotic formats.
const SEAT_BUCKET_COUNT: usize = 4;

/// Number of buckets in `MatchStats.format_buckets`. Sized to cover the
/// current `Format` enum variants (Demo, Cube) plus headroom for new
/// formats added at the top of `main.rs`. New formats slot into the
/// next free index via `format_index`.
const FORMAT_BUCKET_COUNT: usize = 4;

/// Map a local server `Format` (Demo / Cube / Sos) to its bucket index in
/// `MatchStats.format_buckets`. Stable ordering — new formats append.
fn format_index(f: Format) -> usize {
    match f {
        Format::Demo => 0,
        Format::Cube => 1,
        Format::Sos => 2,
    }
}

/// Reverse map for the format-bucket index. Returns `None` for the
/// trailing reserved slots so the formatter can skip empty buckets.
fn format_label_for_bucket(i: usize) -> Option<&'static str> {
    match i {
        0 => Some(Format::Demo.label()),
        1 => Some(Format::Cube.label()),
        2 => Some(Format::Sos.label()),
        _ => None,
    }
}

impl MatchStats {
    fn record_bot(&mut self, d: Duration, f: Format) {
        self.bot_matches += 1;
        self.observe_duration(d);
        self.observe_format(f);
    }
    fn record_pair(&mut self, d: Duration, f: Format) {
        self.pair_matches += 1;
        self.observe_duration(d);
        self.observe_format(f);
    }
    /// Bump the cumulative turn counter — called at match completion
    /// from the record paths if the caller has a final turn number.
    /// Defensive against double-counting since this is invoked exactly
    /// once per `record_*` (the caller passes the final turn).
    fn observe_turns(&mut self, turns: u32) {
        self.total_turns = self.total_turns.saturating_add(turns as u64);
        self.max_turns = Some(match self.max_turns {
            None => turns,
            Some(m) => m.max(turns),
        });
    }
    /// Bump the win/draw counters based on the MatchOutcome.winner
    /// shape. `None` (pre-game-over exit — channel disconnect or
    /// watchdog) is silently dropped: callers can compute "stuck"
    /// matches as `total_matches - wins - draws`. `Some(None)` is a
    /// draw; `Some(Some(_))` is a clean win.
    fn observe_winner(&mut self, w: Option<Option<usize>>) {
        match w {
            Some(None) => self.draws = self.draws.saturating_add(1),
            Some(Some(seat)) => {
                self.wins = self.wins.saturating_add(1);
                let idx = seat.min(SEAT_BUCKET_COUNT - 1);
                self.seat_wins[idx] = self.seat_wins[idx].saturating_add(1);
            }
            None => {}
        }
    }
    /// Accumulate the win-by-life delta for one match. `final_life`
    /// is the per-seat life array; `winner` is the winning seat. The
    /// delta is `winner_life - max_opponent_life` clamped to ≥0 so
    /// the cumulative sum can't go negative even if both ended at
    /// negative life (rare double-loss scenario). Skipped silently
    /// when the winning seat is out of range or no life data is
    /// available. Push (claude/modern_decks batch 202).
    fn observe_win_life_delta(&mut self, winner: usize, final_life: &[i32]) {
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
        self.win_life_samples = self.win_life_samples.saturating_add(1);
    }
    /// Average win-by-life delta across all sampled wins. Returns 0
    /// when no win-life samples have been recorded yet.
    fn avg_win_life_delta(&self) -> i64 {
        if self.win_life_samples == 0 {
            0
        } else {
            self.cumulative_win_life_delta / (self.win_life_samples as i64)
        }
    }
    /// Average turn count across all completed matches. Returns 0
    /// pre-warmup. Used by `format_match_stats` for the operator
    /// rolling-summary line.
    fn avg_turns(&self) -> u64 {
        let n = self.total_matches();
        if n == 0 { 0 } else { self.total_turns / n }
    }
    /// Increment the per-format match count. Used by both `record_bot`
    /// and `record_pair` so the per-format histogram covers every
    /// completed match regardless of source.
    fn observe_format(&mut self, f: Format) {
        let idx = format_index(f).min(FORMAT_BUCKET_COUNT - 1);
        self.format_buckets[idx] = self.format_buckets[idx].saturating_add(1);
    }
    /// Shared bookkeeping for both record paths — accumulates the
    /// total + tracks the new min/max envelope. Pulled out of the
    /// recorders so the min/max maintenance is canonical at one site.
    fn observe_duration(&mut self, d: Duration) {
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
    fn percentile(&self, p: f32) -> Duration {
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
    /// Upper edge (inclusive estimate) of bucket `i` for percentile
    /// reporting. Matches the cut points in `bucket_index`.
    fn bucket_upper_bound(i: usize) -> Duration {
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
    fn bucket_index(d: Duration) -> usize {
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
    fn bucket_label(i: usize) -> &'static str {
        match i {
            0 => "<30s",
            1 => "30s-1m",
            2 => "1-2m",
            3 => "2-5m",
            4 => "5-10m",
            _ => "10m+",
        }
    }
    fn total_matches(&self) -> u64 {
        self.bot_matches + self.pair_matches
    }
    fn avg_duration(&self) -> Duration {
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

static MATCH_STATS: std::sync::OnceLock<std::sync::Mutex<MatchStats>> = std::sync::OnceLock::new();

fn match_stats() -> &'static std::sync::Mutex<MatchStats> {
    MATCH_STATS.get_or_init(|| std::sync::Mutex::new(MatchStats::default()))
}

/// Format the running stats as a one-line summary appended to each
/// match-completion log: `served N matches: K bot, P pair; avg
/// duration X`. Read after the per-match update so the new match is
/// included in the rollup.
fn format_match_stats(s: &MatchStats) -> String {
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
    if let Some(m) = s.max_turns {
        out.push_str(&format!(" (max turns {m})"));
    }
    // Win/draw split: only render once at least one win or draw is
    // recorded so pre-warmup logs stay tight. The delta vs total
    // matches surfaces "stuck" matches (channel disconnect /
    // watchdog) — `total - wins - draws` is the unresolved count.
    if s.wins + s.draws > 0 {
        out.push_str(&format!(" wins={} draws={}", s.wins, s.draws));
        let unresolved = n.saturating_sub(s.wins + s.draws);
        if unresolved > 0 {
            out.push_str(&format!(" unresolved={unresolved}"));
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
        // Average winning-seat life delta — "blowout" check. A high value
        // (12+) means the winner cruised; near-zero values mean games
        // ended in a race. Push (claude/modern_decks batch 202).
        if s.win_life_samples > 0 {
            out.push_str(&format!(" avg_win_life_lead={}", s.avg_win_life_delta()));
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
            " p50≤{}, p95≤{}",
            format_duration(s.percentile(0.50)),
            format_duration(s.percentile(0.95)),
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
    }
    out
}

#[derive(Debug, PartialEq, Eq)]
enum SlotRefusal {
    GlobalCapReached,
    PerIpCapReached,
}

impl SlotManager {
    fn new(global_cap: usize, per_ip_cap: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlotState::default())),
            global_cap,
            per_ip_cap,
        }
    }

    fn try_acquire(&self, addr: IpAddr) -> Result<SlotGuard, SlotRefusal> {
        // Poisoning here means a previous holder panicked while updating
        // counters. The state is still structurally valid (we only do
        // small arithmetic under the lock), so recover via `into_inner`
        // instead of propagating the panic.
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if self.global_cap != 0 && state.total >= self.global_cap {
            return Err(SlotRefusal::GlobalCapReached);
        }
        if self.per_ip_cap != 0 {
            let count = state.per_ip.get(&addr).copied().unwrap_or(0);
            if count >= self.per_ip_cap {
                return Err(SlotRefusal::PerIpCapReached);
            }
        }
        state.total += 1;
        *state.per_ip.entry(addr).or_insert(0) += 1;
        Ok(SlotGuard {
            inner: Arc::clone(&self.inner),
            addr,
        })
    }
}

/// RAII handle that releases a slot when dropped.
struct SlotGuard {
    inner: Arc<Mutex<SlotState>>,
    addr: IpAddr,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        state.total = state.total.saturating_sub(1);
        if let Some(c) = state.per_ip.get_mut(&self.addr) {
            *c = c.saturating_sub(1);
            if *c == 0 {
                state.per_ip.remove(&self.addr);
            }
        }
    }
}

fn main() {
    let bind = env::var("CRAB_BIND").unwrap_or_else(|_| "0.0.0.0:7777".to_string());
    let bot_mode = env::var("CRAB_BOT").ok().as_deref() == Some("1");
    // Format is `Copy`, so each match thread captures a fresh copy via the
    // `move` closure — no Arc needed.
    let format = Format::from_env();
    let pairing_timeout = pairing_timeout_from_env();
    let slots = SlotManager::new(
        usize_from_env("CRAB_MAX_CONNS", DEFAULT_MAX_CONNS),
        usize_from_env("CRAB_MAX_CONNS_PER_IP", DEFAULT_MAX_CONNS_PER_IP),
    );

    let listener = TcpListener::bind(&bind).expect("failed to bind");
    eprintln!(
        "crabomination_server listening on {bind} (bot_mode={bot_mode}, format={}, \
         pairing_timeout={}s, max_conns={}, max_conns_per_ip={})",
        format.label(),
        pairing_timeout.as_secs(),
        slots.global_cap,
        slots.per_ip_cap,
    );

    if bot_mode {
        loop {
            let (stream, peer) = match accept_with_backoff(&listener) {
                Some(p) => p,
                None => continue,
            };
            let guard = match slots.try_acquire(peer.ip()) {
                Ok(g) => g,
                Err(reason) => {
                    eprintln!("refusing {peer}: {reason:?}");
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    continue;
                }
            };
            eprintln!("client {peer} → bot match");
            thread::spawn(move || {
                let _slot = guard;
                run_bot_match(stream, peer, format);
            });
        }
    } else {
        loop {
            let (a_stream, a_peer) = match accept_with_backoff(&listener) {
                Some(p) => p,
                None => continue,
            };
            let a_guard = match slots.try_acquire(a_peer.ip()) {
                Ok(g) => g,
                Err(reason) => {
                    eprintln!("refusing seat 0 {a_peer}: {reason:?}");
                    let _ = a_stream.shutdown(std::net::Shutdown::Both);
                    continue;
                }
            };
            eprintln!(
                "seat 0: {a_peer} (waiting for opponent, timeout={}s)",
                pairing_timeout.as_secs(),
            );
            let (b_stream, b_peer) = match accept_with_deadline(&listener, pairing_timeout) {
                Ok(Some(p)) => p,
                Ok(None) => {
                    // No opponent arrived in time. Close seat 0 cleanly so the
                    // client sees EOF and can retry. Dropping `a_guard`
                    // releases the slot we reserved for them.
                    eprintln!(
                        "dropping unpaired seat 0 ({a_peer}): no opponent within {}s",
                        pairing_timeout.as_secs(),
                    );
                    let _ = a_stream.shutdown(std::net::Shutdown::Both);
                    drop(a_stream);
                    drop(a_guard);
                    continue;
                }
                Err(e) => {
                    eprintln!("accept_with_deadline failed: {e} (dropping {a_peer})");
                    let _ = a_stream.shutdown(std::net::Shutdown::Both);
                    drop(a_stream);
                    drop(a_guard);
                    continue;
                }
            };
            let b_guard = match slots.try_acquire(b_peer.ip()) {
                Ok(g) => g,
                Err(reason) => {
                    eprintln!(
                        "refusing seat 1 {b_peer}: {reason:?} (releasing seat 0 {a_peer})",
                    );
                    let _ = a_stream.shutdown(std::net::Shutdown::Both);
                    let _ = b_stream.shutdown(std::net::Shutdown::Both);
                    drop(a_guard);
                    continue;
                }
            };
            eprintln!("seat 1: {b_peer} → starting match {a_peer} ↔ {b_peer}");
            thread::spawn(move || {
                let _a = a_guard;
                let _b = b_guard;
                run_pair_match(a_stream, a_peer, b_stream, b_peer, format);
            });
        }
    }
}

/// Parse a non-negative integer env var (e.g. connection caps). Falls back
/// to `default` for missing, empty, or non-numeric values. `0` is preserved
/// (callers treat 0 as "unlimited").
fn usize_from_env(key: &str, default: usize) -> usize {
    match env::var(key).ok().as_deref() {
        None | Some("") => default,
        Some(s) => match s.parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                eprintln!(
                    "warning: {key}={s:?} not a non-negative integer — using default {default}",
                );
                default
            }
        },
    }
}

/// Read `CRAB_PAIRING_TIMEOUT_SECS` from the environment. Falls back to
/// `DEFAULT_PAIRING_TIMEOUT` for missing, empty, non-numeric, or zero values
/// (zero would mean "drop seat 0 instantly", almost certainly a misconfig).
fn pairing_timeout_from_env() -> Duration {
    match env::var("CRAB_PAIRING_TIMEOUT_SECS").ok().as_deref() {
        None | Some("") => DEFAULT_PAIRING_TIMEOUT,
        Some(s) => match s.parse::<u64>() {
            Ok(0) => {
                eprintln!(
                    "warning: CRAB_PAIRING_TIMEOUT_SECS=0 ignored — using default {}s",
                    DEFAULT_PAIRING_TIMEOUT.as_secs(),
                );
                DEFAULT_PAIRING_TIMEOUT
            }
            Ok(n) => Duration::from_secs(n),
            Err(_) => {
                eprintln!(
                    "warning: CRAB_PAIRING_TIMEOUT_SECS={s:?} not a non-negative integer — \
                     using default {}s",
                    DEFAULT_PAIRING_TIMEOUT.as_secs(),
                );
                DEFAULT_PAIRING_TIMEOUT
            }
        },
    }
}

/// Wrap `listener.accept` with logging + a small back-off when the OS reports
/// a transient error (file-descriptor exhaustion, transient EAGAIN). Without
/// the back-off, accept failures form a CPU-spin loop. Returns `None` on
/// failure (caller should `continue`).
fn accept_with_backoff(listener: &TcpListener) -> Option<(TcpStream, std::net::SocketAddr)> {
    match listener.accept() {
        Ok(pair) => Some(pair),
        Err(e) => {
            eprintln!("accept failed: {e}");
            // Avoid CPU spin if accept is failing in a tight loop.
            thread::sleep(Duration::from_millis(100));
            None
        }
    }
}

/// Wait for the next incoming connection up to `timeout`. Switches the
/// listener to nonblocking mode for the duration of the wait, polls at a
/// 100ms cadence, then restores blocking mode. Returns `Ok(None)` if the
/// deadline elapses without a client arriving. Errors from `set_nonblocking`
/// or fatal accept failures are surfaced via `Err`; transient accept errors
/// (EAGAIN, FD exhaustion) are logged and retried until the deadline.
///
/// Used to bound the wait for seat 1 after seat 0 has connected, so that a
/// no-show opponent does not leave seat 0 hanging forever.
fn accept_with_deadline(
    listener: &TcpListener,
    timeout: Duration,
) -> io::Result<Option<(TcpStream, std::net::SocketAddr)>> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + timeout;
    let result = loop {
        match listener.accept() {
            Ok(pair) => break Ok(Some(pair)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    break Ok(None);
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("accept failed: {e}");
                if Instant::now() >= deadline {
                    break Ok(None);
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    };
    // Always restore blocking mode, even on error, so the next iteration of
    // the outer accept loop behaves as expected.
    let _ = listener.set_nonblocking(false);
    result
}

fn run_bot_match(stream: TcpStream, peer: std::net::SocketAddr, format: Format) {
    let seat = match tcp_seat(stream) {
        Ok(s) => s,
        Err(e) => { eprintln!("tcp_seat failed for {peer}: {e}"); return; }
    };
    let started = Instant::now();
    let outcome = run_match(
        format.build(),
        vec![
            SeatOccupant::Human(seat),
            SeatOccupant::Bot(Box::new(RandomBot::new())),
        ],
    );
    let duration = started.elapsed();
    let stats_snapshot = {
        let mut s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
        s.record_bot(duration, format);
        s.observe_turns(outcome.final_turn);
        s.observe_winner(outcome.winner);
        if let Some(Some(w)) = outcome.winner {
            s.observe_win_life_delta(w, &outcome.final_life_totals);
        }
        *s
    };
    eprintln!(
        "bot match ended ({}, format={}, duration={}, turns={}) — {}",
        peer,
        format.label(),
        format_duration(duration),
        outcome.final_turn,
        format_match_stats(&stats_snapshot),
    );
}

fn run_pair_match(
    a: TcpStream,
    a_peer: std::net::SocketAddr,
    b: TcpStream,
    b_peer: std::net::SocketAddr,
    format: Format,
) {
    // Wrap both streams before starting; if either fails, drop the other so
    // the surviving client sees a clean EOF instead of hanging.
    let a_seat = match tcp_seat(a) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("seat 0 wrap failed for {a_peer}: {e} (dropping {b_peer})");
            let _ = b.shutdown(std::net::Shutdown::Both);
            return;
        }
    };
    let b_seat = match tcp_seat(b) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("seat 1 wrap failed for {b_peer}: {e} (dropping {a_peer})");
            // a_seat owns the inner stream; drop is enough to close it.
            drop(a_seat);
            return;
        }
    };
    let started = Instant::now();
    let outcome = run_match(
        format.build(),
        vec![SeatOccupant::Human(a_seat), SeatOccupant::Human(b_seat)],
    );
    let duration = started.elapsed();
    let stats_snapshot = {
        let mut s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
        s.record_pair(duration, format);
        s.observe_turns(outcome.final_turn);
        s.observe_winner(outcome.winner);
        if let Some(Some(w)) = outcome.winner {
            s.observe_win_life_delta(w, &outcome.final_life_totals);
        }
        *s
    };
    eprintln!(
        "pair match ended ({} ↔ {}, format={}, duration={}, turns={}) — {}",
        a_peer,
        b_peer,
        format.label(),
        format_duration(duration),
        outcome.final_turn,
        format_match_stats(&stats_snapshot),
    );
}

/// Render a `Duration` as a short human-readable string for logs:
/// `1h2m3s` / `5m12s` / `38s` / `420ms`. Sub-millisecond durations
/// fall through to `<1ms`. Used by the per-match completion log so
/// operators can spot stuck matches at a glance.
fn format_duration(d: Duration) -> String {
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
    fn with_env<F: FnOnce() -> R, R>(key: &str, value: Option<&str>, f: F) -> R {
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
    fn format_from_env_defaults_to_demo() {
        let f = with_env("CRAB_FORMAT", None, Format::from_env);
        assert_eq!(f.label(), "demo");
    }

    #[test]
    fn format_from_env_picks_cube_when_set() {
        let f = with_env("CRAB_FORMAT", Some("cube"), Format::from_env);
        assert_eq!(f.label(), "cube");
    }

    #[test]
    fn format_from_env_unknown_value_falls_back_to_demo() {
        let f = with_env("CRAB_FORMAT", Some("Cube"), Format::from_env);
        assert_eq!(f.label(), "demo",
            "case-sensitive: 'Cube' is not a recognized value");
    }

    #[test]
    fn pairing_timeout_defaults_when_unset() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", None, pairing_timeout_from_env);
        assert_eq!(t, DEFAULT_PAIRING_TIMEOUT);
    }

    #[test]
    fn pairing_timeout_parses_valid_integer() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("42"), pairing_timeout_from_env);
        assert_eq!(t, Duration::from_secs(42));
    }

    #[test]
    fn pairing_timeout_rejects_zero() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("0"), pairing_timeout_from_env);
        assert_eq!(t, DEFAULT_PAIRING_TIMEOUT,
            "zero would mean drop seat 0 instantly — treated as misconfig");
    }

    #[test]
    fn pairing_timeout_rejects_garbage() {
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("five"), pairing_timeout_from_env);
        assert_eq!(t, DEFAULT_PAIRING_TIMEOUT);
    }

    /// `accept_with_deadline` returns `None` after the deadline if no client
    /// connects. This is the core mechanism that prevents seat 0 from
    /// waiting forever for an opponent that never shows up.
    #[test]
    fn accept_with_deadline_returns_none_on_timeout() {
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

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("parse ip")
    }

    #[test]
    fn slot_manager_admits_within_caps() {
        let s = SlotManager::new(2, 2);
        let _a = s.try_acquire(ip("10.0.0.1")).expect("first slot");
        let _b = s.try_acquire(ip("10.0.0.2")).expect("second slot");
    }

    #[test]
    fn slot_manager_rejects_at_global_cap() {
        let s = SlotManager::new(2, 0);
        let _a = s.try_acquire(ip("10.0.0.1")).expect("first");
        let _b = s.try_acquire(ip("10.0.0.2")).expect("second");
        assert!(matches!(
            s.try_acquire(ip("10.0.0.3")),
            Err(SlotRefusal::GlobalCapReached)
        ));
    }

    #[test]
    fn slot_manager_rejects_at_per_ip_cap() {
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
    fn slot_manager_releases_on_guard_drop() {
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
    fn slot_manager_zero_caps_mean_unlimited() {
        let s = SlotManager::new(0, 0);
        let addr = ip("10.0.0.1");
        let mut guards = Vec::new();
        for _ in 0..1000 {
            guards.push(s.try_acquire(addr).expect("unlimited"));
        }
    }

    #[test]
    fn slot_manager_global_cap_checked_before_per_ip() {
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
    fn usize_from_env_defaults_when_unset() {
        let v = with_env("CRAB_MAX_CONNS", None, || usize_from_env("CRAB_MAX_CONNS", 42));
        assert_eq!(v, 42);
    }

    #[test]
    fn usize_from_env_parses_zero() {
        // Unlike pairing_timeout_from_env, here 0 is meaningful: it means
        // "unlimited". Make sure the parser preserves it.
        let v = with_env("CRAB_MAX_CONNS", Some("0"), || usize_from_env("CRAB_MAX_CONNS", 42));
        assert_eq!(v, 0);
    }

    #[test]
    fn usize_from_env_rejects_garbage() {
        let v = with_env("CRAB_MAX_CONNS", Some("nope"), || {
            usize_from_env("CRAB_MAX_CONNS", 42)
        });
        assert_eq!(v, 42);
    }

    #[test]
    fn format_duration_renders_short_values() {
        assert_eq!(format_duration(Duration::from_micros(500)), "<1ms");
        assert_eq!(format_duration(Duration::from_millis(420)), "420ms");
        assert_eq!(format_duration(Duration::from_secs(0)), "<1ms");
    }

    #[test]
    fn format_duration_renders_seconds_minutes_hours() {
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
    fn accept_with_deadline_returns_connection_when_client_arrives() {
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
    fn match_stats_starts_zero() {
        let s = MatchStats::default();
        assert_eq!(s.total_matches(), 0);
        assert_eq!(s.bot_matches, 0);
        assert_eq!(s.pair_matches, 0);
        assert_eq!(s.total_duration, Duration::ZERO);
        assert_eq!(s.avg_duration(), Duration::ZERO);
    }

    #[test]
    fn match_stats_record_bot_and_pair() {
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
    fn observe_winner_tracks_wins_and_draws() {
        let mut s = MatchStats::default();
        s.observe_winner(Some(Some(0))); // seat 0 wins
        s.observe_winner(Some(None));     // draw
        s.observe_winner(Some(Some(1))); // seat 1 wins
        s.observe_winner(None);           // unresolved — silently dropped
        assert_eq!(s.wins, 2);
        assert_eq!(s.draws, 1);
        assert_eq!(s.seat_wins[0], 1);
        assert_eq!(s.seat_wins[1], 1);
    }

    #[test]
    fn observe_winner_per_seat_clamps_at_seat_bucket_count() {
        let mut s = MatchStats::default();
        // Exotic 8-player format: seat 7 wins. Must not panic, must
        // collapse into the last bucket.
        s.observe_winner(Some(Some(7)));
        assert_eq!(s.seat_wins[SEAT_BUCKET_COUNT - 1], 1);
        assert_eq!(s.wins, 1);
    }

    #[test]
    fn observe_win_life_delta_accumulates_winners_lead() {
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
    }

    #[test]
    fn observe_win_life_delta_clamps_negative_lead_to_zero() {
        let mut s = MatchStats::default();
        // Pathological: winner ended at less life than opp (shouldn't
        // happen with normal SBAs but covers forced-draw exits).
        s.observe_win_life_delta(0, &[1, 5]);
        assert_eq!(s.cumulative_win_life_delta, 0,
            "negative lead clamps to 0");
        assert_eq!(s.win_life_samples, 1);
    }

    #[test]
    fn observe_win_life_delta_handles_seat_out_of_range() {
        let mut s = MatchStats::default();
        s.observe_win_life_delta(5, &[20, 0]); // seat 5 doesn't exist
        assert_eq!(s.win_life_samples, 0, "out-of-range silently skipped");
    }

    #[test]
    fn format_match_stats_includes_avg_win_life_lead_when_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        s.observe_win_life_delta(0, &[18, 0]);
        let line = format_match_stats(&s);
        assert!(line.contains("avg_win_life_lead=18"), "got: {line}");
    }

    #[test]
    fn format_match_stats_omits_avg_win_life_lead_when_no_samples() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        // No observe_win_life_delta call → samples = 0.
        let line = format_match_stats(&s);
        assert!(!line.contains("avg_win_life_lead"), "got: {line}");
    }

    #[test]
    fn format_match_stats_includes_seat_wins_when_present() {
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
    fn format_match_stats_includes_win_draw_when_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        let line = format_match_stats(&s);
        assert!(line.contains("wins=1"), "got: {line}");
        assert!(line.contains("draws=0"), "got: {line}");
    }

    #[test]
    fn format_match_stats_omits_win_draw_pre_warmup() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        // No observe_winner — pre-warmup.
        let line = format_match_stats(&s);
        assert!(!line.contains("wins="), "got: {line}");
        assert!(!line.contains("draws="), "got: {line}");
    }

    #[test]
    fn format_match_stats_renders_unresolved_when_some_matches_lack_outcome() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_bot(Duration::from_secs(70), Format::Demo);
        s.observe_winner(Some(Some(0)));
        // The second match had no observed winner.
        let line = format_match_stats(&s);
        assert!(line.contains("wins=1"), "got: {line}");
        assert!(line.contains("unresolved=1"), "got: {line}");
    }

    #[test]
    fn format_match_stats_renders_summary() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("served 1 match"));
        assert!(line.contains("1 bot"));
        assert!(line.contains("0 pair"));
    }

    #[test]
    fn format_match_stats_pluralizes_at_two() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_pair(Duration::from_secs(120), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("served 2 matches"));
        assert!(line.contains("1 bot"));
        assert!(line.contains("1 pair"));
    }

    #[test]
    fn format_match_stats_zero_matches() {
        let s = MatchStats::default();
        let line = format_match_stats(&s);
        assert!(line.contains("served 0 matches"));
    }

    #[test]
    fn match_stats_tracks_min_and_max_duration() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.record_pair(Duration::from_secs(300), Format::Demo);
        s.record_bot(Duration::from_secs(30), Format::Demo);
        s.record_pair(Duration::from_secs(120), Format::Demo);
        assert_eq!(s.min_duration, Some(Duration::from_secs(30)));
        assert_eq!(s.max_duration, Some(Duration::from_secs(300)));
    }

    #[test]
    fn match_stats_avg_turns_averages_observed_turns() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_turns(10);
        s.record_pair(Duration::from_secs(60), Format::Demo);
        s.observe_turns(20);
        // 30 turns / 2 matches = 15
        assert_eq!(s.avg_turns(), 15);
    }

    #[test]
    fn match_stats_avg_turns_zero_before_any_record() {
        let s = MatchStats::default();
        assert_eq!(s.avg_turns(), 0);
    }

    #[test]
    fn match_stats_max_turns_tracks_longest_match() {
        let mut s = MatchStats::default();
        assert_eq!(s.max_turns, None, "unset before any record");
        s.observe_turns(5);
        s.observe_turns(20);
        s.observe_turns(8);
        assert_eq!(s.max_turns, Some(20), "tracks the longest observed turn count");
    }

    #[test]
    fn match_stats_max_turns_rendered_in_summary_line() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_turns(42);
        let line = format_match_stats(&s);
        assert!(line.contains("max turns 42"), "expected max-turns in summary: {line}");
    }

    #[test]
    fn match_stats_min_max_unset_before_any_record() {
        let s = MatchStats::default();
        assert_eq!(s.min_duration, None);
        assert_eq!(s.max_duration, None);
    }

    #[test]
    fn format_match_stats_includes_min_max_when_present() {
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
    fn format_match_stats_omits_min_max_when_zero_matches() {
        let s = MatchStats::default();
        let line = format_match_stats(&s);
        // No min/max parenthetical when no matches have been recorded.
        assert!(!line.contains("min "));
        assert!(!line.contains("max "));
    }

    // ── duration-histogram tests ────────────────────────────────────────────

    #[test]
    fn bucket_index_partitions_durations_into_six_buckets() {
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
    fn match_stats_observe_duration_increments_correct_bucket() {
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
    fn format_match_stats_includes_histogram_when_matches_present() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo);
        s.record_pair(Duration::from_secs(700), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("<30s:1"), "1 in <30s bucket: {line}");
        assert!(line.contains("10m+:1"), "1 in 10m+ bucket: {line}");
        assert!(line.contains("30s-1m:0"), "stable column with 0 count: {line}");
    }

    #[test]
    fn format_match_stats_omits_histogram_when_zero_matches() {
        let s = MatchStats::default();
        let line = format_match_stats(&s);
        assert!(!line.contains("|"), "no histogram section when 0 matches: {line}");
    }

    #[test]
    fn percentile_zero_when_no_matches() {
        let s = MatchStats::default();
        assert_eq!(s.percentile(0.5), Duration::ZERO);
    }

    #[test]
    fn percentile_lands_in_correct_bucket() {
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
    fn percentile_p100_returns_max_bucket_upper_bound() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(5), Format::Demo);
        assert_eq!(s.percentile(1.0), Duration::from_secs(30),
            "p100 with one sample = upper bound of its bucket");
    }

    #[test]
    fn format_match_stats_adds_percentile_when_at_least_two_matches() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo);
        s.record_pair(Duration::from_secs(15), Format::Demo);
        let line = format_match_stats(&s);
        assert!(line.contains("p50≤"), "p50 estimate present: {line}");
        assert!(line.contains("p95≤"), "p95 estimate present: {line}");
    }

    #[test]
    fn format_match_stats_omits_percentile_at_single_sample() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(15), Format::Demo);
        let line = format_match_stats(&s);
        assert!(!line.contains("p50"), "no p50 when only 1 sample: {line}");
    }

    #[test]
    fn match_stats_tracks_per_format_counts() {
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
    fn match_stats_format_breakdown_omitted_when_only_one_format_used() {
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
}
