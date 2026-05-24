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
}

impl Format {
    fn from_env() -> Self {
        match env::var("CRAB_FORMAT").ok().as_deref() {
            Some("cube") => Self::Cube,
            Some("demo") | None => Self::Demo,
            // Anything else: warn so a typo (`CRAB_FORMAT=Cube`) doesn't
            // silently fall back to demo without a hint.
            Some(other) => {
                eprintln!(
                    "warning: CRAB_FORMAT={other:?} not recognized — \
                     falling back to demo. Valid: \"demo\" | \"cube\"."
                );
                Self::Demo
            }
        }
    }
    fn build(&self) -> GameState {
        match self {
            Self::Demo => build_demo_state(),
            Self::Cube => build_cube_state(),
        }
    }
    fn label(&self) -> &'static str {
        match self {
            Self::Demo => "demo",
            Self::Cube => "cube",
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
}

impl MatchStats {
    fn record_bot(&mut self, d: Duration) {
        self.bot_matches += 1;
        self.total_duration += d;
    }
    fn record_pair(&mut self, d: Duration) {
        self.pair_matches += 1;
        self.total_duration += d;
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
    format!(
        "served {} match{}: {} bot, {} pair; avg duration {}",
        n,
        if n == 1 { "" } else { "es" },
        s.bot_matches,
        s.pair_matches,
        format_duration(s.avg_duration()),
    )
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
    run_match(
        format.build(),
        vec![
            SeatOccupant::Human(seat),
            SeatOccupant::Bot(Box::new(RandomBot::new())),
        ],
    );
    let duration = started.elapsed();
    let stats_snapshot = {
        let mut s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
        s.record_bot(duration);
        *s
    };
    eprintln!(
        "bot match ended ({}, format={}, duration={}) — {}",
        peer,
        format.label(),
        format_duration(duration),
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
    run_match(
        format.build(),
        vec![SeatOccupant::Human(a_seat), SeatOccupant::Human(b_seat)],
    );
    let duration = started.elapsed();
    let stats_snapshot = {
        let mut s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
        s.record_pair(duration);
        *s
    };
    eprintln!(
        "pair match ended ({} ↔ {}, format={}, duration={}) — {}",
        a_peer,
        b_peer,
        format.label(),
        format_duration(duration),
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
        s.record_bot(Duration::from_secs(60));
        s.record_pair(Duration::from_secs(120));
        s.record_bot(Duration::from_secs(30));
        assert_eq!(s.total_matches(), 3);
        assert_eq!(s.bot_matches, 2);
        assert_eq!(s.pair_matches, 1);
        assert_eq!(s.total_duration, Duration::from_secs(210));
        assert_eq!(s.avg_duration(), Duration::from_secs(70));
    }

    #[test]
    fn format_match_stats_renders_summary() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60));
        let line = format_match_stats(&s);
        assert!(line.contains("served 1 match"));
        assert!(line.contains("1 bot"));
        assert!(line.contains("0 pair"));
    }

    #[test]
    fn format_match_stats_pluralizes_at_two() {
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(60));
        s.record_pair(Duration::from_secs(120));
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
}
