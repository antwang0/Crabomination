//! Standalone TCP match server.
//!
//! Listens on `CRAB_BIND` (default `0.0.0.0:7777`). Each pair of incoming TCP
//! connections is paired into a 2-seat match. If `CRAB_BOT=1`, the server
//! instead seats one connecting client against a `RandomBot` and starts the
//! match immediately.

use std::env;
use std::net::{TcpListener, TcpStream};
use std::thread;

use crabomination::demo::build_demo_state;
use crabomination::server::{run_match, tcp_seat, RandomBot, SeatOccupant};

fn main() {
    let bind = env::var("CRAB_BIND").unwrap_or_else(|_| "0.0.0.0:7777".to_string());
    let bot_mode = env::var("CRAB_BOT").ok().as_deref() == Some("1");

    let listener = TcpListener::bind(&bind).expect("failed to bind");
    eprintln!("crabomination_server listening on {bind} (bot_mode={bot_mode})");

    if bot_mode {
        loop {
            let (stream, peer) = match listener.accept() {
                Ok(p) => p,
                Err(e) => { eprintln!("accept failed: {e}"); continue; }
            };
            eprintln!("client {peer} → bot match");
            thread::spawn(move || run_bot_match(stream));
        }
    } else {
        loop {
            let (a_stream, a_peer) = match listener.accept() {
                Ok(p) => p,
                Err(e) => { eprintln!("accept failed: {e}"); continue; }
            };
            eprintln!("seat 0: {a_peer}");
            let (b_stream, b_peer) = match listener.accept() {
                Ok(p) => p,
                Err(e) => { eprintln!("accept failed: {e}"); drop(a_stream); continue; }
            };
            eprintln!("seat 1: {b_peer} → starting match");
            thread::spawn(move || run_pair_match(a_stream, b_stream));
        }
    }
}

fn run_bot_match(stream: TcpStream) {
    let seat = match tcp_seat(stream) {
        Ok(s) => s,
        Err(e) => { eprintln!("tcp_seat failed: {e}"); return; }
    };
    run_match(
        build_demo_state(),
        vec![
            SeatOccupant::Human(seat),
            SeatOccupant::Bot(Box::new(RandomBot::new())),
        ],
    );
    eprintln!("bot match ended");
}

fn run_pair_match(a: TcpStream, b: TcpStream) {
    let a_seat = match tcp_seat(a) { Ok(s) => s, Err(e) => { eprintln!("seat 0 wrap failed: {e}"); return; } };
    let b_seat = match tcp_seat(b) { Ok(s) => s, Err(e) => { eprintln!("seat 1 wrap failed: {e}"); return; } };
    run_match(
        build_demo_state(),
        vec![SeatOccupant::Human(a_seat), SeatOccupant::Human(b_seat)],
    );
    eprintln!("pair match ended");
}
