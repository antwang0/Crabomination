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

use config::{deck_overrides, pairing_timeout_from_env, usize_from_env, Format};
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
    let slots = SlotManager::new(
        usize_from_env("CRAB_MAX_CONNS", DEFAULT_MAX_CONNS),
        usize_from_env("CRAB_MAX_CONNS_PER_IP", DEFAULT_MAX_CONNS_PER_IP),
    );

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
                s.record_pair(duration, bin_format);
                s.observe_turns(outcome.final_turn);
                s.observe_winner(outcome.winner);
                if let Some(Some(w)) = outcome.winner {
                    s.observe_win_life_delta(w, &outcome.final_life_totals);
                    s.observe_win_kind(w, &outcome.final_life_totals, &outcome.loss_reasons);
                }
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
                eprintln!("refusing {peer}: {reason:?}");
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
            s.observe_win_kind(w, &outcome.final_life_totals, &outcome.loss_reasons);
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
            s.observe_win_kind(w, &outcome.final_life_totals, &outcome.loss_reasons);
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

