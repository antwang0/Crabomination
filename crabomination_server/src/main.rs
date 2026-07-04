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
//! - `CRAB_ACTION_TIMEOUT_SECS` — per-action timeout (the "rope"). When set
//!   (> 0) and a human seat holds the next action, the match actor waits at
//!   most this long, then acts for them (AutoDecider answer for a pending
//!   decision, else a priority pass). Unset / 0 = humans may think forever.
//! - `CRAB_DECK` / `CRAB_BOT_DECK` — paths to Arena/MTGO-format decklists
//!   for seat 0 / seat 1 of demo-format matches (Modern construction rules
//!   enforced at boot). Unset seats keep the stock BRG / Goryo's decks.

use std::env;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use crabomination::net::LobbyFormat;
use crabomination::server::{
    run_match, serve_lobbies, tcp_seat, ConnId, MatchOutcome, RandomBot, SeatOccupant,
};

mod config;
mod slots;
mod stats;

use config::{deck_overrides, pairing_timeout_from_env, usize_from_env_min, Format};
use config::{DEFAULT_MAX_CONNS, DEFAULT_MAX_CONNS_PER_IP};
use slots::{SlotGuard, SlotManager};
use stats::{format_duration, format_match_stats, match_stats};

fn main() {
    let bind = env::var("CRAB_BIND").unwrap_or_else(|_| "0.0.0.0:7777".to_string());
    let bot_mode = env::var("CRAB_BOT").ok().as_deref() == Some("1");
    // Lobby mode is the default for the non-bot path: each client browses,
    // then creates or joins a lobby and picks the gamemode there. Set
    // `CRAB_LOBBY=0` for the legacy auto-pairing behavior (two clients paired
    // into the server-fixed `CRAB_FORMAT`).
    let lobby_mode = !bot_mode && env::var("CRAB_LOBBY").ok().as_deref() != Some("0");
    // Format is `Copy`, so each match thread captures a fresh copy via the
    // `move` closure — no Arc needed.
    let format = Format::from_env();
    // Validate CRAB_DECK / CRAB_BOT_DECK at boot (exits on a bad list).
    let _ = deck_overrides();
    let pairing_timeout = pairing_timeout_from_env();
    // Connection caps must be at least 1 — a `0` (typo or empty-string-ish
    // misconfig) would otherwise silently refuse every client, so treat
    // sub-1 values as a misconfig and fall back to the default.
    let global_conns = usize_from_env_min("CRAB_MAX_CONNS", DEFAULT_MAX_CONNS, 1);
    let per_ip_conns = usize_from_env_min("CRAB_MAX_CONNS_PER_IP", DEFAULT_MAX_CONNS_PER_IP, 1);
    if per_ip_conns > global_conns {
        eprintln!(
            "warning: CRAB_MAX_CONNS_PER_IP={per_ip_conns} exceeds CRAB_MAX_CONNS={global_conns}; \
             clamping the per-IP cap to {global_conns} (a per-IP cap above the global cap is a no-op)."
        );
    }
    let slots = SlotManager::new(global_conns, per_ip_conns);

    let listener = match TcpListener::bind(&bind) {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "crabomination_server: failed to bind {bind}: {e}\n\
                 (set CRAB_BIND to choose a different address/port)"
            );
            std::process::exit(1);
        }
    };
    eprintln!(
        "crabomination_server listening on {bind} (bot_mode={bot_mode}, lobby_mode={lobby_mode}, \
         format={}, pairing_timeout={}s, max_conns={}, max_conns_per_ip={})",
        format.label(),
        pairing_timeout.as_secs(),
        slots.global_cap,
        slots.per_ip_cap,
    );

    if lobby_mode {
        run_lobby_server(&listener, &slots);
    } else if bot_mode {
        loop {
            let (stream, peer) = match accept_with_backoff(&listener) {
                Some(p) => p,
                None => continue,
            };
            let guard = match slots.try_acquire(peer.ip()) {
                Ok(g) => g,
                Err(reason) => {
                    let s = slots.snapshot();
                    eprintln!(
                        "refusing {peer}: {reason:?} (occupancy {}/peak {}, refused {}g/{}ip @ {rate}% of attempts, {ips} distinct IPs, max {max}/IP)",
                        s.current, s.peak, s.refused_global, s.refused_per_ip,
                        rate = s.refusal_rate_pct(), ips = s.distinct_ips, max = s.max_per_ip,
                    );
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

/// Lobby-mode accept loop. Each accepted connection acquires a slot, is
/// wrapped into a `SeatChannel`, and is handed (with its slot guard) to the
/// [`serve_lobbies`] driver, which runs the browse/create/join protocol and
/// starts a match when a lobby fills. The slot guard rides along with the
/// connection, so the connection cap stays held for the lobby + match.
fn run_lobby_server(listener: &TcpListener, slots: &SlotManager) -> ! {
    let (conn_tx, conn_rx) = mpsc::channel::<(ConnId, _, SlotGuard)>();
    // Fold each lobby-started match into the rolling stats + log a summary,
    // so lobby mode (the default) is as observable as the legacy pair mode.
    let on_match_end: crabomination::server::lobby::MatchEndHook =
        Arc::new(|format: LobbyFormat, duration, outcome: MatchOutcome| {
            let bin_format = Format::from_lobby(format);
            let snapshot = {
                let mut s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
                // Route through the single `record_outcome` accumulator (same as
                // the bot/pair paths) so lobby mode — the default — also folds in
                // decisive/draw turn sums. Replicating the observe_* calls by hand
                // here previously dropped those, leaving `turns(win/draw)` at 0/0.
                s.record_outcome(&outcome, bin_format, duration, true);
                *s
            };
            eprintln!(
                "lobby match ended (format={}, duration={}, turns={}) — {}",
                bin_format.label(),
                format_duration(duration),
                outcome.final_turn,
                format_match_stats(&snapshot),
            );
        });
    thread::spawn(move || serve_lobbies(conn_rx, on_match_end));

    let mut next_id: u64 = 0;
    loop {
        let (stream, peer) = match accept_with_backoff(listener) {
            Some(p) => p,
            None => continue,
        };
        let guard = match slots.try_acquire(peer.ip()) {
            Ok(g) => g,
            Err(reason) => {
                let s = slots.snapshot();
                eprintln!(
                    "refusing {peer}: {reason:?} (occupancy {}/peak {}, refused {}g/{}ip @ {rate}% of attempts, {ips} distinct IPs, max {max}/peak {pmax}/IP)",
                    s.current, s.peak, s.refused_global, s.refused_per_ip,
                    rate = s.refusal_rate_pct(), ips = s.distinct_ips, max = s.max_per_ip, pmax = s.peak_per_ip,
                );
                let _ = stream.shutdown(std::net::Shutdown::Both);
                continue;
            }
        };
        let seat = match tcp_seat(stream) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("tcp_seat failed for {peer}: {e}");
                continue; // dropping `guard` frees the slot
            }
        };
        let id = ConnId(next_id);
        next_id += 1;
        eprintln!("client {peer} → lobby (conn {})", id.0);
        if conn_tx.send((id, seat, guard)).is_err() {
            eprintln!("lobby driver exited; stopping accept loop");
            std::process::exit(1);
        }
    }
}

/// Run a match body, catching any panic so a single buggy game logs with
/// context and lets the thread unwind cleanly (dropping its seats/streams and
/// releasing its slot) instead of dying with an opaque default backtrace.
/// Returns `None` if the match panicked.
fn run_match_caught(
    state: crabomination::game::GameState,
    seats: Vec<SeatOccupant>,
    ctx: &str,
) -> Option<MatchOutcome> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_match(state, seats))) {
        Ok(outcome) => Some(outcome),
        Err(payload) => {
            eprintln!(
                "match aborted: engine panic ({ctx}): {} — connections dropped, slot released",
                panic_message(payload.as_ref()),
            );
            None
        }
    }
}

/// Recover the human-readable message from a caught panic payload. `panic!`
/// stores a `&'static str` or a `String`; anything else logs as a generic
/// note so the abort line always carries *some* context.
fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn run_bot_match(stream: TcpStream, peer: std::net::SocketAddr, format: Format) {
    let seat = match tcp_seat(stream) {
        Ok(s) => s,
        Err(e) => { eprintln!("tcp_seat failed for {peer}: {e}"); return; }
    };
    let started = Instant::now();
    let Some(outcome) = run_match_caught(
        format.build(),
        vec![
            SeatOccupant::Human(seat),
            SeatOccupant::Bot(Box::new(RandomBot::new())),
        ],
        &format!("bot match {peer}, format={}", format.label()),
    ) else {
        return;
    };
    let duration = started.elapsed();
    let stats_snapshot = {
        let mut s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
        s.record_outcome(&outcome, format, duration, false);
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
    let Some(outcome) = run_match_caught(
        format.build(),
        vec![SeatOccupant::Human(a_seat), SeatOccupant::Human(b_seat)],
        &format!("pair match {a_peer} ↔ {b_peer}, format={}", format.label()),
    ) else {
        return;
    };
    let duration = started.elapsed();
    let stats_snapshot = {
        let mut s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
        s.record_outcome(&outcome, format, duration, true);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_PAIRING_TIMEOUT;
    use crate::slots::SlotRefusal;
    use crate::stats::{format_index, format_label_for_bucket, MatchStats, SEAT_BUCKET_COUNT};
    use crabomination::server::LossReason;
    use std::env;
    use std::net::IpAddr;

    #[test]
    fn win_kind_buckets_reconcile_with_wins() {
        let mut s = MatchStats::default();
        // Seat 0 wins by lethal damage (seat 1 at -1).
        s.observe_win_kind(0, &[20, -1], &[None, Some(LossReason::LifeDepleted)]);
        // Seat 0 wins by decking (seat 1 alive at 5 life).
        s.observe_win_kind(0, &[20, 5], &[None, Some(LossReason::Decked)]);
        assert_eq!(s.damage_wins, 1, "one lethal-damage win");
        assert_eq!(s.deckout_wins, 1, "one alternate win");
        assert_eq!(s.deck_wins, 1, "the alternate win was a deck-out");
        // The two buckets partition every classified win.
        assert_eq!(s.damage_wins + s.deckout_wins, 2);
    }

    #[test]
    fn panic_message_recovers_str_and_string_payloads() {
        let s = std::panic::catch_unwind(|| panic!("boom")).unwrap_err();
        assert_eq!(panic_message(s.as_ref()), "boom");
        let owned = std::panic::catch_unwind(|| panic!("{}", format!("n={}", 3))).unwrap_err();
        assert_eq!(panic_message(owned.as_ref()), "n=3");
        let other = std::panic::catch_unwind(|| std::panic::panic_any(7u32)).unwrap_err();
        assert_eq!(panic_message(other.as_ref()), "non-string panic payload");
    }

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
        let f = with_env("CRAB_FORMAT", Some("pauper"), Format::from_env);
        assert_eq!(f.label(), "demo", "unrecognized value falls back to demo");
    }

    #[test]
    pub(crate) fn format_from_env_is_case_and_whitespace_insensitive() {
        assert_eq!(with_env("CRAB_FORMAT", Some("Cube"), Format::from_env).label(), "cube");
        assert_eq!(with_env("CRAB_FORMAT", Some("  COMMANDER "), Format::from_env).label(), "commander");
    }

    #[test]
    pub(crate) fn deck_overrides_ignored_outside_demo_format() {
        let set = ["CRAB_DECK", "CRAB_BOT_DECK"];
        // Demo honors both → nothing flagged.
        assert!(Format::Demo.ignored_override_keys(&set).is_empty());
        // Cube / SOS / Commander flag every present override key.
        assert_eq!(Format::Cube.ignored_override_keys(&set), set);
        assert_eq!(Format::Sos.ignored_override_keys(&["CRAB_DECK"]), ["CRAB_DECK"]);
        assert!(Format::Commander.ignored_override_keys(&[]).is_empty());
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

    #[test]
    pub(crate) fn pairing_timeout_clamps_absurd_values() {
        use crate::config::MAX_PAIRING_TIMEOUT;
        // One year of seconds is far above the 24h cap → clamped.
        let t = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("31536000"), pairing_timeout_from_env);
        assert_eq!(t, MAX_PAIRING_TIMEOUT);
        // A value at the cap is preserved exactly.
        let exact = with_env("CRAB_PAIRING_TIMEOUT_SECS", Some("86400"), pairing_timeout_from_env);
        assert_eq!(exact, MAX_PAIRING_TIMEOUT);
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
    pub(crate) fn slot_manager_snapshot_tracks_peak_and_refusals() {
        let s = SlotManager::new(2, 1);
        let a = s.try_acquire(ip("10.0.0.1")).expect("first");
        let b = s.try_acquire(ip("10.0.0.2")).expect("second");
        // Global cap is 2 → a third distinct IP is refused globally.
        assert!(matches!(s.try_acquire(ip("10.0.0.3")), Err(SlotRefusal::GlobalCapReached)));
        // Free a global slot, then the first IP (at its per-IP cap of 1) is
        // refused on the per-IP path.
        drop(b);
        assert!(matches!(s.try_acquire(ip("10.0.0.1")), Err(SlotRefusal::PerIpCapReached)));
        let snap = s.snapshot();
        assert_eq!(snap.current, 1, "only seat a still held");
        assert_eq!(snap.peak, 2, "peaked at two concurrent");
        assert_eq!(snap.refused_global, 1);
        assert_eq!(snap.refused_per_ip, 1);
        assert_eq!(snap.distinct_ips, 1, "one IP still holds a slot");
        drop(a);
        let final_snap = s.snapshot();
        assert_eq!(final_snap.current, 0, "all released");
        assert_eq!(final_snap.distinct_ips, 0, "no IPs hold slots once released");
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
        let v = with_env("CRAB_MAX_CONNS", None, || usize_from_env_min("CRAB_MAX_CONNS", 42, 0));
        assert_eq!(v, 42);
    }

    #[test]
    pub(crate) fn usize_from_env_min_zero_allows_zero() {
        // With min 0 the parser preserves a literal 0 (the "unlimited" reading).
        let v = with_env("CRAB_MAX_CONNS", Some("0"), || usize_from_env_min("CRAB_MAX_CONNS", 42, 0));
        assert_eq!(v, 0);
    }

    #[test]
    pub(crate) fn usize_from_env_min_one_rejects_zero() {
        // Connection caps use min 1: a 0 is a misconfig → default.
        let v = with_env("CRAB_MAX_CONNS", Some("0"), || usize_from_env_min("CRAB_MAX_CONNS", 42, 1));
        assert_eq!(v, 42);
    }

    #[test]
    pub(crate) fn usize_from_env_rejects_garbage() {
        let v = with_env("CRAB_MAX_CONNS", Some("nope"), || {
            usize_from_env_min("CRAB_MAX_CONNS", 42, 0)
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
    pub(crate) fn draw_pct_is_share_of_all_matches() {
        // 2 draws out of 4 total → 50% (vs decisive_pct, which is over resolved).
        let mut s = MatchStats { bot_matches: 4, ..Default::default() };
        s.observe_winner(Some(Some(0)));
        s.observe_winner(Some(None));
        s.observe_winner(Some(None));
        s.observe_winner(None); // inconclusive
        assert_eq!(s.draws, 2);
        assert_eq!(s.draw_pct(), 50);
        // decisive_pct counts only resolved (1 win + 2 draws) → 1/3 = 33%.
        assert_eq!(s.decisive_pct(), 33);
        // Empty stats → 0, no divide-by-zero.
        assert_eq!(MatchStats::default().draw_pct(), 0);
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
    pub(crate) fn record_outcome_folds_all_counters_at_once() {
        use crabomination::server::MatchOutcome;
        let mut s = MatchStats::default();
        let outcome = MatchOutcome {
            final_turn: 9,
            winner: Some(Some(0)),
            final_life_totals: vec![14, 0],
            loss_reasons: vec![None, Some(LossReason::LifeDepleted)],
            ..Default::default()
        };
        s.record_outcome(&outcome, Format::Demo, std::time::Duration::from_secs(30), false);
        assert_eq!(s.bot_matches, 1, "bot-kind tally bumped");
        assert_eq!(s.pair_matches, 0);
        assert_eq!(s.wins, 1, "winner observed");
        assert_eq!(s.total_turns, 9, "turns folded in");
        assert_eq!(s.deckout_wins, 0, "life-depletion win is not an alternate win");
        // The pair path bumps the other tally and accumulates.
        s.record_outcome(&outcome, Format::Demo, std::time::Duration::from_secs(30), true);
        assert_eq!(s.pair_matches, 1);
        assert_eq!(s.total_turns, 18);
    }

    #[test]
    pub(crate) fn split_turn_averages_separate_wins_from_draws() {
        use crabomination::server::MatchOutcome;
        let mut s = MatchStats::default();
        let d = std::time::Duration::from_secs(30);
        // Two decisive wins at 6 and 10 turns → avg 8.
        for t in [6u32, 10] {
            let o = MatchOutcome {
                final_turn: t,
                winner: Some(Some(0)),
                final_life_totals: vec![14, 0],
                loss_reasons: vec![None, Some(LossReason::LifeDepleted)],
                ..Default::default()
            };
            s.record_outcome(&o, Format::Demo, d, false);
        }
        // One draw at a 40-turn grind.
        let draw = MatchOutcome {
            final_turn: 40,
            winner: Some(None),
            final_life_totals: vec![3, 3],
            ..Default::default()
        };
        s.record_outcome(&draw, Format::Demo, d, false);
        assert_eq!(s.avg_decisive_turns(), 8, "decisive average excludes the draw");
        assert_eq!(s.avg_draw_turns(), 40, "draw average tracks the grind");
        assert!(
            format_match_stats(&s).contains("turns(win/draw)=8/40"),
            "split rendered once a draw exists"
        );
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
        // Concession / "you lose the game" → alternate + the `other` bucket.
        s.observe_win_kind(1, &[4, 7], &[Some(LossReason::Other), None]);
        assert_eq!(s.deckout_wins, 3);
        assert_eq!(s.other_wins, 1);
        // other_pct is a share of recorded wins.
        let shared = MatchStats { wins: 4, other_wins: 1, ..Default::default() };
        assert_eq!(shared.other_pct(), 25);
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
    pub(crate) fn format_match_stats_shows_per_format_avg_turns() {
        let mut s = MatchStats::default();
        // Two demo matches (8 and 12 turns → avg 10), one cube (20 turns).
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_format_turns(Format::Demo, 8);
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_format_turns(Format::Demo, 12);
        s.record_bot(Duration::from_secs(60), Format::Cube);
        s.observe_format_turns(Format::Cube, 20);
        assert_eq!(s.format_avg_turns(format_index(Format::Demo)), Some(10));
        assert_eq!(s.format_avg_turns(format_index(Format::Cube)), Some(20));
        let line = format_match_stats(&s);
        assert!(line.contains("demo:2(10t)"), "got: {line}");
        assert!(line.contains("cube:1(20t)"), "got: {line}");
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
        // p90 crosses into the open blowout bucket (delta 40 → lower edge 16).
        assert_eq!(s.win_life_delta_percentile(0.9), 16, "p90 catches the blowout");
        let rendered = format_match_stats_with_win(&s);
        assert!(rendered.contains("p50=3") && rendered.contains("p90=16"));
    }

    #[test]
    pub(crate) fn win_life_delta_percentile_clamps_and_handles_empty() {
        let s = MatchStats::default();
        assert_eq!(s.win_life_delta_percentile(0.5), 0, "no samples → 0");
        let mut s = MatchStats::default();
        s.observe_win_life_delta(0, &[9, 0]); // delta 9 → bucket 3 (upper edge 10)
        assert_eq!(s.win_life_delta_percentile(-1.0), 10, "p<0 clamps to p0");
        assert_eq!(s.win_life_delta_percentile(2.0), 10, "p>1 clamps to p100");
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
    pub(crate) fn format_match_stats_reports_stuck_matches() {
        let mut s = MatchStats::default();
        // One resolved win, one bare match with no recorded outcome (stuck).
        s.record_bot(Duration::from_secs(60), Format::Demo);
        s.observe_winner(Some(Some(0)));
        s.record_bot(Duration::from_secs(60), Format::Demo);
        assert_eq!(s.unresolved(), 1);
        let line = format_match_stats(&s);
        assert!(line.contains("stuck=1"), "summary surfaces stuck count: {line}");
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
    pub(crate) fn format_match_stats_renders_turn_distribution_past_threshold() {
        let mut s = MatchStats::default();
        // Four matches: no turn distribution yet (threshold is 5).
        for _ in 0..4 {
            s.record_bot(Duration::from_secs(1), Format::Demo);
            s.observe_turns(6);
        }
        assert!(!format_match_stats(&s).contains("turns_p50"), "hidden below 5 matches");
        // Fifth match crosses the threshold.
        s.record_bot(Duration::from_secs(1), Format::Demo);
        s.observe_turns(6);
        let line = format_match_stats(&s);
        assert!(line.contains("turns_p50="), "turn distribution now shown");
        assert!(line.contains("p95="));
        assert!(line.contains("σ="));
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
    pub(crate) fn turn_percentile_ranks_against_its_own_histogram() {
        // Turn samples outnumber recorded matches (the two counters drifted).
        // The percentile must rank against the turn histogram's own sum, so
        // p50 still lands in the populated short bucket rather than skewing to
        // the tail because the denominator was too small.
        let mut s = MatchStats::default();
        s.record_bot(Duration::from_secs(1), Format::Demo); // total_matches == 1
        for _ in 0..9 {
            s.observe_turns(2);
        }
        s.observe_turns(30);
        assert_eq!(s.turn_percentile(0.5), 2, "p50 uses the 10-sample turn histogram");
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

