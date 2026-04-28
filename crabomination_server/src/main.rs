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

use std::env;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

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
            _ => Self::Demo,
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

fn main() {
    let bind = env::var("CRAB_BIND").unwrap_or_else(|_| "0.0.0.0:7777".to_string());
    let bot_mode = env::var("CRAB_BOT").ok().as_deref() == Some("1");
    let format = Format::from_env();
    let format = Arc::new(format);

    let listener = TcpListener::bind(&bind).expect("failed to bind");
    eprintln!(
        "crabomination_server listening on {bind} (bot_mode={bot_mode}, format={})",
        format.label(),
    );

    if bot_mode {
        loop {
            let (stream, peer) = match listener.accept() {
                Ok(p) => p,
                Err(e) => { eprintln!("accept failed: {e}"); continue; }
            };
            eprintln!("client {peer} → bot match");
            let fmt = Arc::clone(&format);
            thread::spawn(move || run_bot_match(stream, *fmt));
        }
    } else {
        loop {
            let (a_stream, a_peer) = match listener.accept() {
                Ok(p) => p,
                Err(e) => { eprintln!("accept failed: {e}"); continue; }
            };
            eprintln!("seat 0: {a_peer} (waiting for opponent)");
            let (b_stream, b_peer) = match listener.accept() {
                Ok(p) => p,
                Err(e) => {
                    // Tell seat 0 we couldn't pair them so they get EOF
                    // instead of a hung socket.
                    eprintln!("accept failed for seat 1 ({e}); dropping seat 0 ({a_peer})");
                    let _ = a_stream.shutdown(std::net::Shutdown::Both);
                    drop(a_stream);
                    continue;
                }
            };
            eprintln!("seat 1: {b_peer} → starting match");
            let fmt = Arc::clone(&format);
            thread::spawn(move || run_pair_match(a_stream, b_stream, *fmt));
        }
    }
}

fn run_bot_match(stream: TcpStream, format: Format) {
    let seat = match tcp_seat(stream) {
        Ok(s) => s,
        Err(e) => { eprintln!("tcp_seat failed: {e}"); return; }
    };
    run_match(
        format.build(),
        vec![
            SeatOccupant::Human(seat),
            SeatOccupant::Bot(Box::new(RandomBot::new())),
        ],
    );
    eprintln!("bot match ended");
}

fn run_pair_match(a: TcpStream, b: TcpStream, format: Format) {
    let a_seat = match tcp_seat(a) { Ok(s) => s, Err(e) => { eprintln!("seat 0 wrap failed: {e}"); return; } };
    let b_seat = match tcp_seat(b) { Ok(s) => s, Err(e) => { eprintln!("seat 1 wrap failed: {e}"); return; } };
    run_match(
        format.build(),
        vec![SeatOccupant::Human(a_seat), SeatOccupant::Human(b_seat)],
    );
    eprintln!("pair match ended");
}
