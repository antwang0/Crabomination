//! Authoritative match server.
//!
//! A match owns a [`GameState`] and talks to N seats. Each seat is either a
//! network-style [`SeatChannel`] (human client on the other end, possibly
//! over TCP) or an in-process [`Bot`] that reads state directly. The actor
//! validates every action against the seat that sent it, forwards it to the
//! engine, then broadcasts a per-seat [`ClientView`] plus the generated
//! events to every human seat.
//!
//! The actor is synchronous — it blocks on a merged receiver and runs on a
//! single thread. Transports (in-process channel bridge for singleplayer,
//! TCP for networked play) wrap this with their own I/O.

use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::game::{GameAction, GameState, TurnStep};
use crate::net::{ClientMsg, GameEventWire, ServerMsg};
use crate::snapshot::GameSnapshot;

/// Shared snapshot sink. The match actor writes the latest authoritative
/// engine state here after every accepted action, so an in-process
/// client can grab it for debug exports without fighting the borrow
/// checker over `&mut GameState`. Carries both:
///
/// - `snapshot` — structured, schema-stable `GameSnapshot` (lossy on
///   triggers) suitable for human inspection of saved files.
/// - `full_state_json` — full `GameState` serialized via `serde_json`
///   (bit-exact replay). Stored as a String to keep the sink `Clone +
///   Send + Sync` without cloning the engine state on every publish.
#[derive(Debug, Default, Clone)]
pub struct SnapshotSinkState {
    pub snapshot: Option<GameSnapshot>,
    pub full_state_json: Option<String>,
}

pub type SnapshotSink = Arc<Mutex<SnapshotSinkState>>;

pub mod bot;
pub mod tcp;
pub mod view;

pub use bot::{Bot, RandomBot};
pub use tcp::{tcp_client, tcp_seat};
pub use view::project;

/// A safety limit on how many actions a bot (or chain of bots) can take
/// between human inputs. The loop polls bots to a fixed point; if that fixed
/// point never arrives, something is wrong and we'd rather panic than spin.
const BOT_TICK_BUDGET: usize = 10_000;

/// Wall-clock deadline for "no accepted action since the last progress tick."
/// In Spectate Bot vs Bot mode the only sender on `merged_rx` is the local
/// UI's spectator channel, which doesn't volunteer actions; if every bot
/// rejects on the next tick (sorcery-speed cast with non-empty stack, etc.)
/// the actor would otherwise block forever on `recv()`. We poll with this
/// timeout instead so the watchdog can fire with a state dump + panic.
const BOT_DEADLOCK_TIMEOUT: Duration = Duration::from_secs(15);

/// Where the deadlock watchdog dumps the live `GameSnapshot` JSON. Resolved
/// relative to CWD; the local client launches matches from the repo root,
/// matching the `<repo>/debug/` convention used by the in-game export.
const DEADLOCK_DUMP_DIR: &str = "debug";

/// The server-side end of a seat connection. The `tx` is where the server
/// sends [`ServerMsg`]s to this seat's client; `rx` is where the server
/// receives [`ClientMsg`]s from it.
pub struct SeatChannel {
    pub tx: mpsc::Sender<ServerMsg>,
    pub rx: mpsc::Receiver<ClientMsg>,
}

/// The client-side end of a seat connection. Mirror of [`SeatChannel`].
pub struct ClientChannel {
    pub tx: mpsc::Sender<ClientMsg>,
    pub rx: mpsc::Receiver<ServerMsg>,
}

/// Create a linked pair of channels — one end for the server (a seat), one
/// for the client that occupies that seat.
pub fn seat_pair() -> (SeatChannel, ClientChannel) {
    let (c_tx, s_rx) = mpsc::channel(); // client → server
    let (s_tx, c_rx) = mpsc::channel(); // server → client
    (
        SeatChannel { tx: s_tx, rx: s_rx },
        ClientChannel { tx: c_tx, rx: c_rx },
    )
}

/// How a seat is being driven for this match.
pub enum SeatOccupant {
    /// A remote (or in-process-wrapped) human client reached over a channel.
    Human(SeatChannel),
    /// An in-process bot that reads authoritative state directly.
    Bot(Box<dyn Bot>),
}

/// Run a match to completion on the current thread. Returns when the game
/// ends, all human seat channels have been dropped, or — if every seat is a
/// bot — when the game ends.
pub fn run_match(state: GameState, occupants: Vec<SeatOccupant>) {
    run_match_full(state, occupants, vec![], None);
}

/// Variant of [`run_match`] that also broadcasts every event/view to a list
/// of read-only spectator channels. Spectators see the seat-0 view (same as
/// any other observer) and may submit `ClientMsg::SubmitAction`s, but the
/// server silently drops those — they are not seated. Used by the
/// "Spectate Bot Match" mode in the menu.
pub fn run_match_spectated(
    state: GameState,
    occupants: Vec<SeatOccupant>,
    spectators: Vec<SeatChannel>,
) {
    run_match_full(state, occupants, spectators, None);
}

/// Run a match with an optional snapshot sink. After every accepted
/// action, the actor stores a fresh `GameSnapshot` of the authoritative
/// state in the sink, so in-process clients can read it (under
/// `Mutex::lock`) for debug-export purposes.
pub fn run_match_full(
    mut state: GameState,
    occupants: Vec<SeatOccupant>,
    spectators: Vec<SeatChannel>,
    snapshot_sink: Option<SnapshotSink>,
) {
    let n = occupants.len();
    assert_eq!(n, state.players.len(), "occupant count must match player count");

    // Only run the pre-game mulligan when the state is genuinely at game
    // start (Untap of turn 1). Tests/fixtures that hand-craft a state mid-game
    // bypass this; otherwise PlayLand etc. would be rejected with
    // `DecisionPending` because the mulligan installs a pending decision.
    if state.step == TurnStep::Untap && state.turn_number == 1 {
        state.start_mulligan_phase();
    }

    let (merged_tx, merged_rx) = mpsc::channel::<(usize, ClientMsg)>();
    let mut seat_tx: Vec<Option<mpsc::Sender<ServerMsg>>> = Vec::with_capacity(n);
    let mut bots: Vec<Option<Box<dyn Bot>>> = Vec::with_capacity(n);
    let mut human_seat_count = 0;

    // Spectator channels: passive observers that get the same broadcast
    // stream as a Human seat (seat 0's view) but whose incoming messages
    // are discarded by the actor. They keep the match alive so a pure
    // bot-vs-bot run streamed to a UI spectator doesn't return early.
    let mut spectator_tx: Vec<mpsc::Sender<ServerMsg>> = Vec::with_capacity(spectators.len());
    let spectator_count = spectators.len();
    for spec in spectators {
        let _ = spec.tx.send(ServerMsg::YourSeat(0));
        let _ = spec.tx.send(ServerMsg::MatchStarted);
        let _ = spec.tx.send(ServerMsg::View(view::project(&state, 0)));
        // Drain the spectator's incoming stream into a sentinel forwarder
        // that uses seat = usize::MAX; the actor checks for this and
        // ignores any actions tagged with it. Without the drain, a UI
        // that never reads its rx would still run fine, but a UI that
        // submits would block its own send queue.
        let forward_tx = merged_tx.clone();
        let rx = spec.rx;
        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                if forward_tx.send((usize::MAX, msg)).is_err() {
                    break;
                }
            }
        });
        spectator_tx.push(spec.tx);
    }

    for (i, occ) in occupants.into_iter().enumerate() {
        match occ {
            SeatOccupant::Human(seat) => {
                let _ = seat.tx.send(ServerMsg::YourSeat(i));
                let _ = seat.tx.send(ServerMsg::MatchStarted);
                let _ = seat.tx.send(ServerMsg::View(view::project(&state, i)));
                let forward_tx = merged_tx.clone();
                let rx = seat.rx;
                thread::spawn(move || {
                    while let Ok(msg) = rx.recv() {
                        if forward_tx.send((i, msg)).is_err() {
                            break;
                        }
                    }
                });
                seat_tx.push(Some(seat.tx));
                bots.push(None);
                human_seat_count += 1;
            }
            SeatOccupant::Bot(b) => {
                seat_tx.push(None);
                bots.push(Some(b));
            }
        }
    }
    drop(merged_tx);

    // Initial snapshot so clients can read the authoritative state
    // before any action has been processed.
    publish_snapshot(&state, &snapshot_sink);

    // Watchdog: if `BOT_DEADLOCK_TIMEOUT` passes between accepted
    // actions in a no-human (Spectate Bot vs Bot) match, dump the
    // GameSnapshot and panic with a clear message instead of hanging
    // silently on `merged_rx.recv()`. Pure-human matches don't trip
    // this — humans can take their time deciding.
    let mut last_progress_at = Instant::now();

    loop {
        if drive_bots(
            &mut state,
            &mut bots,
            &seat_tx,
            &spectator_tx,
            &snapshot_sink,
            &mut last_progress_at,
        ) {
            broadcast_match_over(&state, &seat_tx, &spectator_tx);
            return;
        }

        if human_seat_count == 0 && spectator_count == 0 {
            return;
        }

        // In a humanless (spectator-only) match, poll with a timeout
        // so the deadlock watchdog can fire. Otherwise wait
        // indefinitely — humans deciding shouldn't trigger the
        // watchdog.
        let next = if human_seat_count == 0 {
            match merged_rx.recv_timeout(BOT_DEADLOCK_TIMEOUT) {
                Ok(msg) => Some(msg),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if last_progress_at.elapsed() >= BOT_DEADLOCK_TIMEOUT {
                        report_deadlock(&state, last_progress_at.elapsed());
                    }
                    None
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        } else {
            match merged_rx.recv() {
                Ok(msg) => Some(msg),
                Err(_) => return,
            }
        };
        let Some((seat, msg)) = next else { continue };

        // Spectator-tagged messages (seat == usize::MAX) are silently
        // ignored — spectators don't act, they only watch.
        if seat == usize::MAX { continue; }

        match msg {
            ClientMsg::JoinMatch { .. } => {}
            ClientMsg::SubmitAction(action) => {
                let accepted = handle_action(&mut state, seat, action, &seat_tx, &spectator_tx);
                if accepted {
                    last_progress_at = Instant::now();
                    publish_snapshot(&state, &snapshot_sink);
                }
                if state.is_game_over() {
                    broadcast_match_over(&state, &seat_tx, &spectator_tx);
                    return;
                }
            }
        }
    }
}

fn publish_snapshot(state: &GameState, sink: &Option<SnapshotSink>) {
    let Some(sink) = sink else { return };
    let snapshot = GameSnapshot::capture(state);
    let full_state_json = serde_json::to_string(state).ok();
    if let Ok(mut guard) = sink.lock() {
        *guard = SnapshotSinkState {
            snapshot: Some(snapshot),
            full_state_json,
        };
    }
}

/// Poll every bot seat to a fixed point: each pass asks every bot whether it
/// wants to act, and we repeat until a full pass produces no actions.
/// Returns `true` if the game ended during bot play. Updates
/// `last_progress_at` whenever a bot's action is accepted, so the
/// outer-loop watchdog can distinguish "bots are thinking forever" from
/// "bots successfully ended the game."
fn drive_bots(
    state: &mut GameState,
    bots: &mut [Option<Box<dyn Bot>>],
    seat_tx: &[Option<mpsc::Sender<ServerMsg>>],
    spectator_tx: &[mpsc::Sender<ServerMsg>],
    snapshot_sink: &Option<SnapshotSink>,
    last_progress_at: &mut Instant,
) -> bool {
    let mut budget: usize = BOT_TICK_BUDGET;
    loop {
        let mut any_acted = false;
        for (seat, slot) in bots.iter_mut().enumerate() {
            let Some(bot) = slot.as_mut() else { continue };
            let Some(action) = bot.next_action(state, seat) else {
                continue;
            };
            // Only count the action if it actually changed state. A rejected
            // action (wrong priority, illegal move, etc.) must not count as
            // progress — otherwise the loop burns the entire budget retrying
            // the same failing action.
            if handle_action(state, seat, action, seat_tx, spectator_tx) {
                publish_snapshot(state, snapshot_sink);
                *last_progress_at = Instant::now();
                if state.is_game_over() {
                    return true;
                }
                any_acted = true;
            }
        }
        if !any_acted {
            return false;
        }
        // Bot loop is making progress on every tick but never ending
        // the game (priority ping-pong, infinite trigger chain, etc.).
        // Dump the live state for debugging before panicking — silent
        // panics with no state are a nightmare to triage from a bug
        // report.
        if budget == 0 {
            report_deadlock(state, Duration::from_secs(0));
        }
        budget -= 1;
    }
}

/// Dump the live `GameSnapshot` (plus diagnostic metadata) to
/// `<cwd>/{DEADLOCK_DUMP_DIR}/deadlock-t<turn>-<unix>.json`. Returns
/// the file path on success, `None` if dir-create / serialize / write
/// failed. Pure I/O — no panic, no logging — so tests can invoke it
/// directly to verify the output schema.
pub fn dump_deadlock_state(state: &GameState, elapsed: Duration) -> Option<std::path::PathBuf> {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    let dir = PathBuf::from(DEADLOCK_DUMP_DIR);
    fs::create_dir_all(&dir).ok()?;
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let path = dir.join(format!(
        "deadlock-t{}-{unix}-{nanos:09}.json",
        state.turn_number
    ));

    let snapshot = GameSnapshot::capture(state);
    // Best-effort full-state serialization: a bit-exact `GameState`
    // dump is the most useful artifact for triaging a deadlock since
    // it includes pending_decision, suspend_signal, delayed_triggers,
    // and continuous_effects which the schema-stable snapshot omits.
    // If `serde_json::to_value` somehow fails (it shouldn't — every
    // engine type is `Serialize` now), we fall through with a `null`
    // and ship just the snapshot.
    let full_state = serde_json::to_value(state).unwrap_or(serde_json::Value::Null);
    let json = serde_json::to_string_pretty(&serde_json::json!({
        "kind": "bot_deadlock",
        "elapsed_secs": elapsed.as_secs(),
        "turn": state.turn_number,
        "step": format!("{:?}", state.step),
        "active_player": state.active_player_idx,
        "priority": state.player_with_priority(),
        "stack_len": state.stack.len(),
        "pending_decision": state.pending_decision.is_some(),
        "snapshot": snapshot,
        "full_state": full_state,
    }))
    .ok()?;
    fs::write(&path, json).ok()?;
    Some(path)
}

/// Watchdog tripwire: dump the state and panic. Called by both the
/// wall-clock deadlock detector (`merged_rx.recv_timeout` exhaustion in
/// `run_match_full`) and the per-tick budget detector
/// (`BOT_TICK_BUDGET` exhaustion in `drive_bots`).
fn report_deadlock(state: &GameState, elapsed: Duration) -> ! {
    let written = dump_deadlock_state(state, elapsed);
    eprintln!(
        "BOT DEADLOCK detected after {elapsed:?} idle on turn {} {:?} \
         (active={}, priority={}, stack={}).",
        state.turn_number,
        state.step,
        state.active_player_idx,
        state.player_with_priority(),
        state.stack.len(),
    );
    if let Some(p) = written.as_ref() {
        eprintln!("  state dumped → {}", p.display());
    } else {
        eprintln!("  (state dump failed; check {DEADLOCK_DUMP_DIR}/ permissions)");
    }
    panic!(
        "bot deadlock — no accepted action for {:?}; see {DEADLOCK_DUMP_DIR}/deadlock-*.json",
        elapsed,
    );
}

fn broadcast_match_over(
    state: &GameState,
    seat_tx: &[Option<mpsc::Sender<ServerMsg>>],
    spectator_tx: &[mpsc::Sender<ServerMsg>],
) {
    let winner = state.game_over.unwrap_or(None);
    for tx in seat_tx.iter().flatten() {
        let _ = tx.send(ServerMsg::MatchOver { winner });
    }
    for tx in spectator_tx {
        let _ = tx.send(ServerMsg::MatchOver { winner });
    }
}

/// Apply one action and broadcast results. Returns `true` if the action was
/// accepted (state changed), `false` if it was rejected.
fn handle_action(
    state: &mut GameState,
    seat: usize,
    action: GameAction,
    seat_tx: &[Option<mpsc::Sender<ServerMsg>>],
    spectator_tx: &[mpsc::Sender<ServerMsg>],
) -> bool {
    let expected = expected_actor(state, &action);
    if seat != expected {
        let err = format!("seat {seat} may not act now (expected seat {expected})");
        report_error(seat, &err, seat_tx);
        return false;
    }
    match state.perform_action(action) {
        Ok(events) => {
            let wire_events: Vec<GameEventWire> = events.iter().map(Into::into).collect();
            for (i, maybe_tx) in seat_tx.iter().enumerate() {
                if let Some(tx) = maybe_tx {
                    let _ = tx.send(ServerMsg::Events(wire_events.clone()));
                    let _ = tx.send(ServerMsg::View(view::project(state, i)));
                }
            }
            // Spectators always see seat-0's projection so they get a
            // stable POV across the match.
            for tx in spectator_tx {
                let _ = tx.send(ServerMsg::Events(wire_events.clone()));
                let _ = tx.send(ServerMsg::View(view::project(state, 0)));
            }
            true
        }
        Err(e) => {
            report_error(seat, &e.to_string(), seat_tx);
            false
        }
    }
}

fn report_error(seat: usize, err: &str, seat_tx: &[Option<mpsc::Sender<ServerMsg>>]) {
    if let Some(tx) = seat_tx[seat].as_ref() {
        let _ = tx.send(ServerMsg::ActionError(err.to_string()));
    } else {
        // Bot seat — no channel. Surface for debugging; bots shouldn't be
        // submitting illegal actions in normal play.
        eprintln!("server: bot seat {seat} action rejected: {err}");
    }
}

/// Which seat the engine will attribute the next action to. Used by the
/// server to reject actions submitted by the wrong client.
fn expected_actor(state: &GameState, _action: &GameAction) -> usize {
    if let Some(pd) = &state.pending_decision {
        return pd.acting_player();
    }
    // The actor is whoever currently holds priority (including each
    // defender's separate window during DeclareBlockers in multiplayer).
    state.player_with_priority()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardId;
    use crate::catalog;
    use crate::game::{GameAction, TurnStep};
    use crate::net::{ClientMsg, ServerMsg};
    use crate::player::Player;
    use crate::server::bot::RandomBot;

    fn two_player_game() -> GameState {
        let mut state = GameState::new(vec![Player::new(0, "P0"), Player::new(1, "P1")]);
        // Start in main phase so PlayLand is legal.
        state.step = TurnStep::PreCombatMain;
        state
    }

    fn drain_initial(seat: &ClientChannel) {
        // Discard YourSeat, MatchStarted, and initial View.
        for _ in 0..3 {
            let _ = seat.rx.recv();
        }
    }

    #[test]
    fn wrong_seat_action_rejected() {
        let mut state = two_player_game();
        let card_id = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Seat 1 tries to play seat 0's land — must be rejected.
        c1.tx.send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        // Give the actor a moment, then expect an ActionError back on c1.
        let reply = c1.rx.recv().unwrap();
        match reply {
            ServerMsg::ActionError(_) => {}
            other => panic!("expected ActionError, got {:?}", other),
        }

        // Drop channels to end the match.
        drop(c0);
        drop(c1);
        handle.join().unwrap();
    }

    #[test]
    fn correct_seat_action_broadcasts_view() {
        let mut state = two_player_game();
        let card_id = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        c0.tx.send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        // Both seats should receive Events + View.
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::Events(_)));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::View(_)));
        assert!(matches!(c1.rx.recv().unwrap(), ServerMsg::Events(_)));
        assert!(matches!(c1.rx.recv().unwrap(), ServerMsg::View(_)));

        drop(c0);
        drop(c1);
        handle.join().unwrap();
    }

    #[test]
    fn unknown_card_error_surfaces_as_action_error() {
        let state = two_player_game();
        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Reference a card that doesn't exist.
        c0.tx.send(ClientMsg::SubmitAction(GameAction::PlayLand(CardId(999))))
            .unwrap();

        match c0.rx.recv().unwrap() {
            ServerMsg::ActionError(_) => {}
            other => panic!("expected ActionError, got {:?}", other),
        }

        drop(c0);
        drop(c1);
        handle.join().unwrap();
    }

    #[test]
    fn human_vs_bot_initial_handshake() {
        let state = two_player_game();
        let (s0, c0) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Human(s0),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            )
        });

        // Human (seat 0) receives the standard three opening messages.
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::YourSeat(0)));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::MatchStarted));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::View(_)));

        // Dropping the human channel terminates the match cleanly.
        drop(c0);
        handle.join().unwrap();
    }

    #[test]
    fn human_vs_bot_action_broadcasts_to_human() {
        let mut state = two_player_game();
        let card_id = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Human(s0),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            )
        });

        drain_initial(&c0);

        // Human plays a land — the human seat should still receive Events + View.
        c0.tx
            .send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::Events(_)));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::View(_)));

        drop(c0);
        handle.join().unwrap();
    }

    // ── Concurrency ──────────────────────────────────────────────────────────

    use std::time::{Duration, Instant};

    /// Drain up to `cap` messages from `rx` within `timeout`. Returns whatever
    /// arrived; useful when the order between two concurrent senders is
    /// non-deterministic but we want to assert on the multiset of outcomes.
    fn drain_within(
        rx: &mpsc::Receiver<ServerMsg>,
        cap: usize,
        timeout: Duration,
    ) -> Vec<ServerMsg> {
        let deadline = Instant::now() + timeout;
        let mut out = Vec::new();
        while out.len() < cap {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            match rx.recv_timeout(deadline - now) {
                Ok(m) => out.push(m),
                Err(_) => break,
            }
        }
        out
    }

    /// Two human seats fire `PlayLand` from independent sender threads. The
    /// actor processes them serially — seat 0 holds priority so its play is
    /// accepted (broadcast lands on both seats), and seat 1's attempt is
    /// rejected with an `ActionError`. The point of this test is to confirm
    /// concurrent submissions don't deadlock or interleave the broadcast with
    /// itself: each accepted action emits a single `Events + View` pair to
    /// every seat, in order.
    #[test]
    fn concurrent_submissions_processed_serially() {
        let mut state = two_player_game();
        let s0_card = state.add_card_to_hand(0, catalog::plains());
        let s1_card = state.add_card_to_hand(1, catalog::plains());

        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Two independent threads submit at roughly the same moment.
        let t0 = c0.tx.clone();
        let t1 = c1.tx.clone();
        let h0 = thread::spawn(move || {
            t0.send(ClientMsg::SubmitAction(GameAction::PlayLand(s0_card)))
                .unwrap();
        });
        let h1 = thread::spawn(move || {
            t1.send(ClientMsg::SubmitAction(GameAction::PlayLand(s1_card)))
                .unwrap();
        });
        h0.join().unwrap();
        h1.join().unwrap();

        let m0 = drain_within(&c0.rx, 6, Duration::from_secs(2));
        let m1 = drain_within(&c1.rx, 6, Duration::from_secs(2));

        let events0 = m0.iter().filter(|m| matches!(m, ServerMsg::Events(_))).count();
        let views0 = m0.iter().filter(|m| matches!(m, ServerMsg::View(_))).count();
        let errs1 = m1.iter().filter(|m| matches!(m, ServerMsg::ActionError(_))).count();
        let events1 = m1.iter().filter(|m| matches!(m, ServerMsg::Events(_))).count();

        assert_eq!(events0, 1, "c0 should see exactly one Events broadcast: {m0:?}");
        assert_eq!(views0, 1, "c0 should see exactly one View broadcast: {m0:?}");
        assert_eq!(errs1, 1, "c1 should see exactly one ActionError: {m1:?}");
        assert_eq!(events1, 1, "c1 should see exactly one Events broadcast: {m1:?}");

        drop(c0);
        drop(c1);
        handle.join().unwrap();
    }

    /// One human dropping their channel mid-match must not crash the actor or
    /// stop it from servicing the remaining human. The forwarder thread for
    /// the dropped seat exits when its `recv` returns Err; the seat's `tx`
    /// is still in `seat_tx` but `let _ = tx.send(...)` swallows the error
    /// so broadcasts don't propagate the panic.
    #[test]
    fn human_disconnect_mid_match_does_not_crash_actor() {
        let mut state = two_player_game();
        let card_id = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Seat 1 disconnects before doing anything. Give the forwarder
        // thread a moment to notice and exit.
        drop(c1);
        thread::sleep(Duration::from_millis(20));

        // Seat 0 keeps playing — this must succeed and produce a normal
        // Events + View pair on c0, no panic on the broadcast to the
        // already-dead seat 1.
        c0.tx
            .send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        let m0 = drain_within(&c0.rx, 2, Duration::from_secs(2));
        assert!(
            m0.iter().any(|m| matches!(m, ServerMsg::Events(_))),
            "c0 missing Events after peer disconnect: {m0:?}"
        );

        drop(c0);
        handle.join().unwrap();
    }

    /// When every human seat drops, the match thread must terminate cleanly
    /// — the merged receive on the actor side errors out (all forwarder
    /// threads have exited and dropped their `merged_tx` clones), and
    /// `run_match` returns rather than spinning.
    #[test]
    fn all_humans_disconnect_terminates_match() {
        let state = two_player_game();
        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drop(c0);
        drop(c1);

        // The match must end on its own within a reasonable window; we
        // wrap join in a timeout via a signaling channel.
        let (done_tx, done_rx) = mpsc::channel();
        let watcher = thread::spawn(move || {
            handle.join().unwrap();
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("run_match should terminate after all humans disconnect");
        watcher.join().unwrap();
    }

    /// A bot-only match has no merged_rx blocking step: the actor pumps
    /// `drive_bots` until the game ends. With both players at 1 life and a
    /// hasted attacker, combat damage in turn 1 wins the game — exercising
    /// the no-humans branch end-to-end.
    #[test]
    fn bot_vs_bot_runs_to_completion() {
        let mut state = two_player_game();
        state.players[0].life = 1;
        state.players[1].life = 1;
        let bear = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        state.clear_sickness(bear);

        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });

        done_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("bot-vs-bot match should reach game over within 10s");
        handle.join().unwrap();
    }

    /// Bot-vs-bot smoke test on the random Cube format. Each seat gets a
    /// fresh random two-color deck; the match must reach game over within
    /// the timeout. Repeated five times to shake out target-selection
    /// freezes (Vandalblast → lone artifact, Counterspell → lone creature,
    /// etc.) that only show up under specific runtime card combinations.
    #[test]
    fn bot_vs_bot_random_cube_decks_terminate() {
        use crate::cube::build_cube_state;
        for trial in 0..5 {
            let state = build_cube_state();
            let (done_tx, done_rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                run_match(
                    state,
                    vec![
                        SeatOccupant::Bot(Box::new(RandomBot::new())),
                        SeatOccupant::Bot(Box::new(RandomBot::new())),
                    ],
                );
                let _ = done_tx.send(());
            });
            done_rx
                .recv_timeout(Duration::from_secs(30))
                .unwrap_or_else(|_| panic!("cube bot-vs-bot trial {trial} did not terminate"));
            handle.join().unwrap();
        }
    }

    /// A spectator channel attached to a 2-bot match must receive
    /// MatchStarted/View on hookup and MatchOver when the game ends. Any
    /// action the spectator submits is silently dropped (no ActionError
    /// surfaces, the match keeps running).
    #[test]
    fn spectator_receives_broadcasts_and_match_over() {
        let mut state = two_player_game();
        state.players[0].life = 1;
        state.players[1].life = 1;
        let bear = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        state.clear_sickness(bear);

        let (spec_seat, spec_client) = seat_pair();
        let handle = thread::spawn(move || {
            run_match_spectated(
                state,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
                vec![spec_seat],
            );
        });

        // Initial handshake.
        assert!(matches!(spec_client.rx.recv().unwrap(), ServerMsg::YourSeat(0)));
        assert!(matches!(spec_client.rx.recv().unwrap(), ServerMsg::MatchStarted));
        assert!(matches!(spec_client.rx.recv().unwrap(), ServerMsg::View(_)));

        // The spectator should eventually see MatchOver. Drain everything
        // and assert at least one MatchOver arrived.
        let drained = drain_within(&spec_client.rx, 200, Duration::from_secs(10));
        assert!(
            drained.iter().any(|m| matches!(m, ServerMsg::MatchOver { .. })),
            "spectator must receive MatchOver: {:?}",
            drained.iter().map(std::mem::discriminant).collect::<Vec<_>>(),
        );

        drop(spec_client);
        handle.join().unwrap();
    }

    /// Bot-vs-bot on the Modern demo decks (BRG / Goryo's). Same shape as
    /// the cube test but uses the curated demo decks; both regress the
    /// match-driver against bot loop deadlocks.
    #[test]
    fn bot_vs_bot_modern_demo_decks_terminate() {
        use crate::demo::build_demo_state;
        let state = build_demo_state();
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(30))
            .expect("modern demo bot-vs-bot match must terminate");
        handle.join().unwrap();
    }

    /// `dump_deadlock_state` writes a self-describing JSON file under
    /// `debug/` with `kind: "bot_deadlock"` plus the live snapshot.
    /// The watchdog (`report_deadlock`) wraps this with a panic; the
    /// test exercises the I/O path without tripping the panic so we
    /// can assert on the output schema.
    #[test]
    fn dump_deadlock_state_writes_self_describing_json() {
        use serde_json::Value;
        use std::fs;
        let mut state = two_player_game();
        state.players[0].life = 13;
        state.add_card_to_battlefield(0, catalog::tireless_tracker());

        let path = dump_deadlock_state(&state, Duration::from_secs(7))
            .expect("dump_deadlock_state should succeed");
        let body = fs::read_to_string(&path).expect("read back");
        let parsed: Value = serde_json::from_str(&body).expect("valid JSON");
        assert_eq!(parsed["kind"], "bot_deadlock");
        assert_eq!(parsed["elapsed_secs"], 7);
        assert_eq!(parsed["turn"], state.turn_number);
        assert_eq!(parsed["snapshot"]["players"][0]["life"], 13);
        assert!(
            parsed["snapshot"]["battlefield"]
                .as_array()
                .map(|a| a.iter().any(|c| c["name"] == "Tireless Tracker"))
                .unwrap_or(false),
            "deadlock dump should include battlefield contents: {body}",
        );
        // Full-state dump enables bit-exact replay (preserves
        // pending_decision, delayed triggers, etc. that the
        // snapshot omits). Smoke-check that it's present and
        // round-trips player life through the engine's Serialize
        // impl.
        assert_eq!(
            parsed["full_state"]["players"][0]["life"], 13,
            "deadlock dump should embed the full GameState",
        );
        let _ = fs::remove_file(&path);
    }

    /// `LatestSnapshot` (the spectator's read-port for the in-game
    /// debug exporter) must be populated with the *initial* state
    /// before any action is processed. Otherwise a match that
    /// deadlocks immediately would leave the export prompt with
    /// nothing to capture beyond the seat-projected `ClientView`.
    #[test]
    fn snapshot_sink_is_populated_before_first_action() {
        let mut state = two_player_game();
        // Seed something distinctive so the test can verify the
        // snapshot really came from this state and not the default.
        state.players[0].life = 7;
        state.add_card_to_battlefield(0, catalog::grizzly_bears());

        let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
        let sink_for_match = Arc::clone(&sink);

        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match_full(
                state,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
                vec![],
                Some(sink_for_match),
            );
            let _ = done_tx.send(());
        });

        // Poll the sink until populated or until the match ends —
        // even with no input the actor publishes the initial state
        // before entering its loop.
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(guard) = sink.lock()
                && let Some(snap) = guard.snapshot.as_ref()
            {
                assert_eq!(
                    snap.players[0].life, 7,
                    "initial snapshot must reflect the seeded state",
                );
                assert!(
                    snap.battlefield.iter().any(|c| c.name == "Grizzly Bears"),
                    "initial snapshot must include the seeded battlefield",
                );
                break;
            }
            if Instant::now() >= deadline {
                panic!("snapshot sink was never populated within 5s");
            }
            thread::sleep(Duration::from_millis(20));
        }

        // Don't care whether the match ends — just don't leak the
        // thread on a slow CI runner.
        let _ = done_rx.recv_timeout(Duration::from_secs(30));
        let _ = handle.join();
    }
}
