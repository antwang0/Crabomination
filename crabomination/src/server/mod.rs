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
use crate::net::{ClientMsg, DebugAction, GameEventWire, ServerMsg};
use crate::snapshot::GameSnapshot;

/// Shared snapshot sink. The match actor stores the latest authoritative
/// engine state here after every accepted action, so an in-process client
/// can grab it for debug exports without fighting the borrow checker over
/// `&mut GameState`.
///
/// The state is held behind an `Arc` so a publish is a single
/// `GameState::clone` (refcounted, cheap to hand to readers) rather than a
/// per-action `serde_json::to_string` — the export consumer is rare, so the
/// two derived fidelity levels are produced lazily, on demand:
///
/// - [`snapshot`](Self::snapshot) — structured, schema-stable `GameSnapshot`
///   (lossy on triggers) suitable for human inspection of saved files.
/// - [`full_state`](Self::full_state) — a full `GameState` clone for
///   bit-exact replay (preserves triggers, delayed triggers, continuous
///   effects, pending decisions).
#[derive(Default, Clone)]
pub struct SnapshotSinkState {
    /// Latest published authoritative state, or `None` before the first
    /// publish. Shared via `Arc`, so cloning the sink state is cheap.
    pub state: Option<Arc<GameState>>,
}

// `GameState` isn't `Debug`, so report only whether a state is present.
impl std::fmt::Debug for SnapshotSinkState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotSinkState")
            .field("state", &self.state.as_ref().map(|_| "<GameState>"))
            .finish()
    }
}

impl SnapshotSinkState {
    /// Schema-stable snapshot of the latest published state, captured on
    /// demand. `None` before the first publish.
    pub fn snapshot(&self) -> Option<GameSnapshot> {
        self.state.as_ref().map(|s| GameSnapshot::capture(s))
    }

    /// An owned clone of the latest published `GameState` for bit-exact
    /// replay, on demand. `None` before the first publish.
    pub fn full_state(&self) -> Option<GameState> {
        self.state.as_ref().map(|s| s.as_ref().clone())
    }
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

/// How long a reconnectable match stays alive with *every* human seat
/// disconnected, waiting for at least one to reattach, before giving up and
/// ending the match. Only applies when reconnection is enabled (the standalone
/// server passes a reattach channel); in-process and non-reconnectable matches
/// end the instant their last human drops, exactly as before.
const RECONNECT_GRACE: Duration = Duration::from_secs(60);

/// Messages the match actor multiplexes onto its single inbound channel.
/// Seat forwarders, the spectator drain, and (for reconnectable matches) the
/// reattach drain all feed this enum so the actor can `recv` one stream.
enum Inbox {
    /// A client message from `seat` (or `usize::MAX` for a spectator, whose
    /// messages the actor ignores).
    FromSeat(usize, ClientMsg),
    /// `seat`'s forwarder observed its client disconnect. `epoch` identifies
    /// which connection generation reported it, so a stale disconnect from a
    /// socket that was already replaced by a reconnect is ignored.
    Disconnected(usize, u64),
    /// A (re)connecting client has taken over `seat` with a fresh channel.
    /// Only produced for reconnectable matches.
    Attach(usize, SeatChannel),
}

/// Spawn the per-seat forwarder: pump the seat's inbound `ClientMsg`s onto the
/// actor's merged channel tagged with `seat`, then — when the client's socket
/// closes — emit a single [`Inbox::Disconnected`] stamped with `epoch` so the
/// actor can distinguish a live disconnect from a stale one after a reconnect.
fn spawn_seat_forwarder(
    seat: usize,
    epoch: u64,
    rx: mpsc::Receiver<ClientMsg>,
    merged_tx: mpsc::Sender<Inbox>,
) {
    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            if merged_tx.send(Inbox::FromSeat(seat, msg)).is_err() {
                return;
            }
        }
        // Client gone — best-effort notify the actor (it may already have
        // returned, in which case the send simply fails and we exit).
        let _ = merged_tx.send(Inbox::Disconnected(seat, epoch));
    });
}

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
/// Summary of a completed match returned by `run_match_full` (and its
/// siblings). Captures end-of-game metrics that callers — typically
/// the binary's match-stats recorder — fold into rolling aggregates.
/// Each field is zero / `None` if the match aborted before reaching the
/// relevant data point.
#[derive(Debug, Clone, Default)]
pub struct MatchOutcome {
    /// Final `turn_number` value observed at match end. Useful for
    /// "avg turns per match" operator metrics.
    pub final_turn: u32,
    /// `None` if the match aborted (channel disconnect, watchdog) before
    /// the game's `game_over` field was populated. `Some(None)` for a
    /// draw, `Some(Some(seat))` for the winning seat. Useful for
    /// win-rate metrics across bot-vs-bot ladders.
    pub winner: Option<Option<usize>>,
    /// Per-seat life total observed at match end. Same length as
    /// `state.players`. Useful for "how close was the loss" tracking
    /// and for differentiating decking-out (life 1+) from face-damage
    /// (life ≤ 0) victories. Empty if the match aborted before the
    /// game state was inspected.
    pub final_life_totals: Vec<i32>,
    /// Per-seat manner-of-loss classification, parallel to
    /// `final_life_totals`. `None` for seats that weren't eliminated
    /// (the winner, or everyone in a draw/abort); `Some(reason)` for
    /// each eliminated seat. Lets operator metrics split a ladder's
    /// losses into life-damage vs poison vs deck-out — the deck-out
    /// bucket is the one the new dredge/mill shells push on. Empty if
    /// the match aborted before the game state was inspected.
    pub loss_reasons: Vec<Option<LossReason>>,
    /// Per-seat library size observed at match end, parallel to
    /// `final_life_totals`. Lets operator metrics see "how close to
    /// decking out" each seat was — the natural companion to the
    /// `LossReason::Decked` bucket and the dredge / mill ladders, where
    /// the winner often ends a hair above an empty library. Empty if the
    /// match aborted before the game state was inspected.
    pub final_library_sizes: Vec<usize>,
    pub final_graveyard_sizes: Vec<usize>,
    /// Per-seat count of permanents the seat controls on the battlefield
    /// at match end, parallel to `final_life_totals`. A "board development"
    /// proxy: pairs with `final_turn` to tell a fast face-damage win
    /// (small boards, low turn) apart from a grindy attrition game (wide
    /// boards, high turn). Empty if the match aborted before inspection.
    pub final_board_sizes: Vec<usize>,
}

/// How an eliminated player lost, inferred from their final state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LossReason {
    /// Life total fell to 0 or below (CR 104.3a / 704.5a).
    LifeDepleted,
    /// Ten or more poison counters (CR 104.3c / 704.5c).
    Poison,
    /// Tried to draw from an empty library (CR 104.3a / 704.5c).
    Decked,
    /// Eliminated for some other reason (concession, "you lose the
    /// game" effect, etc.) not distinguishable from final state alone.
    Other,
}

/// Classify why `p` lost, or `None` if `p` is still in the game. The
/// checks are ordered most-specific-first: a player at ≤0 life is
/// reported as `LifeDepleted` even if their library also happens to be
/// empty, matching the order in which state-based actions would have
/// fired.
fn classify_loss(p: &crate::player::Player) -> Option<LossReason> {
    if !p.eliminated {
        return None;
    }
    if p.life <= 0 {
        Some(LossReason::LifeDepleted)
    } else if p.poison_counters >= 10 {
        Some(LossReason::Poison)
    } else if p.library.is_empty() {
        Some(LossReason::Decked)
    } else {
        Some(LossReason::Other)
    }
}

/// Capture a MatchOutcome from the current GameState. Used at every
/// exit path of `run_match_full` so winner / life metrics are always
/// populated (even on watchdog / disconnect exits).
fn capture_outcome(state: &GameState) -> MatchOutcome {
    MatchOutcome {
        final_turn: state.turn_number,
        winner: state.game_over,
        final_life_totals: state.players.iter().map(|p| p.life).collect(),
        loss_reasons: state.players.iter().map(classify_loss).collect(),
        final_library_sizes: state.players.iter().map(|p| p.library.len()).collect(),
        final_graveyard_sizes: state.players.iter().map(|p| p.graveyard.len()).collect(),
        final_board_sizes: (0..state.players.len())
            .map(|seat| state.battlefield.iter().filter(|c| c.controller == seat).count())
            .collect(),
    }
}

pub fn run_match(state: GameState, occupants: Vec<SeatOccupant>) -> MatchOutcome {
    run_match_full(state, occupants, vec![], None)
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
) -> MatchOutcome {
    run_match_full(state, occupants, spectators, None)
}

/// Run a match with an optional snapshot sink. After every accepted
/// action, the actor stores a fresh `GameSnapshot` of the authoritative
/// state in the sink, so in-process clients can read it (under
/// `Mutex::lock`) for debug-export purposes.
pub fn run_match_full(
    state: GameState,
    occupants: Vec<SeatOccupant>,
    spectators: Vec<SeatChannel>,
    snapshot_sink: Option<SnapshotSink>,
) -> MatchOutcome {
    run_match_reconnectable(state, occupants, spectators, snapshot_sink, None)
}

/// Variant of [`run_match_full`] that supports mid-match reconnection.
///
/// When a human seat's client disconnects, the match is *not* torn down
/// immediately: the actor drains `reattach_rx` for an [`Inbox::Attach`]
/// carrying a fresh `SeatChannel` for that seat, replaying
/// `YourSeat`/`MatchStarted`/`View` to the new connection. Only once *every*
/// human seat has been gone for longer than [`RECONNECT_GRACE`] with no
/// reattach does the match end.
///
/// Passing `reattach_rx == None` (as [`run_match_full`] does) disables this
/// entirely — the match ends the instant its last human drops, identical to
/// the pre-reconnect behavior — so non-reconnectable callers are unaffected.
pub fn run_match_reconnectable(
    state: GameState,
    occupants: Vec<SeatOccupant>,
    spectators: Vec<SeatChannel>,
    snapshot_sink: Option<SnapshotSink>,
    reattach_rx: Option<mpsc::Receiver<(usize, SeatChannel)>>,
) -> MatchOutcome {
    run_match_inner(state, occupants, spectators, snapshot_sink, reattach_rx, RECONNECT_GRACE)
}

/// Core match actor. Separated from [`run_match_reconnectable`] only so the
/// reconnect grace window is injectable — tests drive it with a short grace
/// instead of the production [`RECONNECT_GRACE`].
fn run_match_inner(
    mut state: GameState,
    occupants: Vec<SeatOccupant>,
    spectators: Vec<SeatChannel>,
    snapshot_sink: Option<SnapshotSink>,
    reattach_rx: Option<mpsc::Receiver<(usize, SeatChannel)>>,
    reconnect_grace: Duration,
) -> MatchOutcome {
    let n = occupants.len();
    assert_eq!(n, state.players.len(), "occupant count must match player count");

    // Only run the pre-game mulligan when the state is genuinely at game
    // start (Untap of turn 1). Tests/fixtures that hand-craft a state mid-game
    // bypass this; otherwise PlayLand etc. would be rejected with
    // `DecisionPending` because the mulligan installs a pending decision.
    if state.step == TurnStep::Untap && state.turn_number == 1 {
        state.start_mulligan_phase();
    }

    let reconnect_enabled = reattach_rx.is_some();
    let (merged_tx, merged_rx) = mpsc::channel::<Inbox>();
    let mut seat_tx: Vec<Option<mpsc::Sender<ServerMsg>>> = Vec::with_capacity(n);
    let mut bots: Vec<Option<Box<dyn Bot>>> = Vec::with_capacity(n);
    // Per-seat connection epoch (bumped on each reattach) and current
    // connected flag. Bot seats stay at epoch 0 / `connected=false` and are
    // never tracked for liveness.
    let mut seat_epoch: Vec<u64> = vec![0; n];
    let mut connected: Vec<bool> = vec![false; n];
    let mut human_seats = 0usize;

    // Spectator channels: passive observers that get the same broadcast
    // stream as a Human seat (seat 0's view) but whose incoming messages
    // are discarded by the actor. They keep the match alive so a pure
    // bot-vs-bot run streamed to a UI spectator doesn't return early.
    let mut spectator_tx: Vec<mpsc::Sender<ServerMsg>> = Vec::with_capacity(spectators.len());
    let spectator_count = spectators.len();
    for spec in spectators {
        let _ = spec.tx.send(ServerMsg::YourSeat(0));
        let _ = spec.tx.send(ServerMsg::MatchStarted);
        let _ = spec.tx.send(ServerMsg::View(Box::new(view::project(&state, 0))));
        // Drain the spectator's incoming stream into a sentinel forwarder
        // that uses seat = usize::MAX; the actor checks for this and
        // ignores any actions tagged with it. Without the drain, a UI
        // that never reads its rx would still run fine, but a UI that
        // submits would block its own send queue.
        let forward_tx = merged_tx.clone();
        let rx = spec.rx;
        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                if forward_tx.send(Inbox::FromSeat(usize::MAX, msg)).is_err() {
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
                let _ = seat.tx.send(ServerMsg::View(Box::new(view::project(&state, i))));
                spawn_seat_forwarder(i, seat_epoch[i], seat.rx, merged_tx.clone());
                seat_tx.push(Some(seat.tx));
                bots.push(None);
                connected[i] = true;
                human_seats += 1;
            }
            SeatOccupant::Bot(b) => {
                seat_tx.push(None);
                bots.push(Some(b));
            }
        }
    }
    let mut connected_humans = human_seats;

    // Reattach drain: forwards reconnecting `SeatChannel`s into the actor as
    // `Inbox::Attach`. Holding a `merged_tx` clone keeps the merged channel
    // alive across full human disconnects (so the actor can wait out the
    // grace window). Only spawned for reconnectable matches.
    if let Some(reattach_rx) = reattach_rx {
        let forward_tx = merged_tx.clone();
        thread::spawn(move || {
            while let Ok((seat, ch)) = reattach_rx.recv() {
                if forward_tx.send(Inbox::Attach(seat, ch)).is_err() {
                    break;
                }
            }
        });
    }
    // A spare sender used to spawn forwarders for reattaching seats. Kept
    // only when reconnection is enabled; doubles as a keep-alive for the
    // merged channel. Non-reconnectable matches drop every `merged_tx` here
    // so the channel disconnects (and the actor returns) once all forwarders
    // exit, exactly as before.
    let attach_forward_tx = reconnect_enabled.then(|| merged_tx.clone());
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
    // For reconnectable matches: once *every* human has dropped, the instant
    // after which we stop waiting for a reattach and end the match. `None`
    // while at least one human is connected (or always, when reconnection is
    // disabled — those matches end immediately on full disconnect instead).
    let mut disconnect_deadline: Option<Instant> = None;

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
            return capture_outcome(&state);
        }

        if human_seats == 0 && spectator_count == 0 {
            return capture_outcome(&state);
        }

        // Pick how long to block waiting for the next message:
        // - spectator-only (no human seats): poll on the deadlock watchdog.
        // - all humans gone within the reconnect grace window: poll until
        //   the deadline, then end the match if no one reattached.
        // - otherwise: wait indefinitely (humans deciding mustn't time out).
        let next: Option<Inbox> = if human_seats == 0 {
            match merged_rx.recv_timeout(BOT_DEADLOCK_TIMEOUT) {
                Ok(msg) => Some(msg),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if last_progress_at.elapsed() >= BOT_DEADLOCK_TIMEOUT {
                        report_deadlock(&state, last_progress_at.elapsed());
                    }
                    None
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return capture_outcome(&state),
            }
        } else if let Some(deadline) = disconnect_deadline {
            let now = Instant::now();
            if now >= deadline {
                // Grace expired with no reattach — give up on the match.
                return capture_outcome(&state);
            }
            match merged_rx.recv_timeout(deadline - now) {
                Ok(msg) => Some(msg),
                Err(mpsc::RecvTimeoutError::Timeout) => None, // loop re-checks the deadline
                Err(mpsc::RecvTimeoutError::Disconnected) => return capture_outcome(&state),
            }
        } else {
            match merged_rx.recv() {
                Ok(msg) => Some(msg),
                Err(_) => return capture_outcome(&state),
            }
        };
        let Some(inbox) = next else { continue };

        let (seat, msg) = match inbox {
            Inbox::FromSeat(seat, msg) => (seat, msg),
            Inbox::Disconnected(seat, epoch) => {
                // Ignore a stale disconnect from a connection that was
                // already superseded by a reattach (epoch mismatch).
                if seat < n && connected[seat] && epoch == seat_epoch[seat] {
                    connected[seat] = false;
                    connected_humans = connected_humans.saturating_sub(1);
                    if connected_humans == 0 && spectator_count == 0 {
                        if reconnect_enabled {
                            disconnect_deadline = Some(Instant::now() + reconnect_grace);
                        } else {
                            return capture_outcome(&state);
                        }
                    }
                }
                continue;
            }
            Inbox::Attach(seat, ch) => {
                if seat < n {
                    seat_epoch[seat] += 1;
                    let _ = ch.tx.send(ServerMsg::YourSeat(seat));
                    let _ = ch.tx.send(ServerMsg::MatchStarted);
                    let _ = ch.tx.send(ServerMsg::View(Box::new(view::project(&state, seat))));
                    if let Some(fwd) = attach_forward_tx.as_ref() {
                        spawn_seat_forwarder(seat, seat_epoch[seat], ch.rx, fwd.clone());
                    }
                    seat_tx[seat] = Some(ch.tx);
                    if !connected[seat] {
                        connected[seat] = true;
                        connected_humans += 1;
                    }
                    // Someone's back — cancel any pending grace expiry.
                    disconnect_deadline = None;
                }
                continue;
            }
        };

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
                    return capture_outcome(&state);
                }
            }
            ClientMsg::Debug(debug) => {
                if apply_debug(&mut state, seat, debug, &seat_tx, &spectator_tx) {
                    last_progress_at = Instant::now();
                    publish_snapshot(&state, &snapshot_sink);
                }
            }
        }
    }
}

fn publish_snapshot(state: &GameState, sink: &Option<SnapshotSink>) {
    let Some(sink) = sink else { return };
    // One refcounted clone of the live state. The `GameSnapshot` capture and
    // the JSON serialization that the export consumer needs are deferred to
    // `SnapshotSinkState::snapshot` / `full_state`, since the vast majority
    // of publishes are overwritten by the next action before anyone reads
    // them.
    let shared = Arc::new(state.clone());
    if let Ok(mut guard) = sink.lock() {
        guard.state = Some(shared);
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
            // One combined frame per seat: events + the post-action view,
            // halving the per-action writes/flushes versus the old
            // Events-then-View pair.
            for (i, maybe_tx) in seat_tx.iter().enumerate() {
                if let Some(tx) = maybe_tx {
                    let _ = tx.send(ServerMsg::Update {
                        events: wire_events.clone(),
                        view: Box::new(view::project(state, i)),
                    });
                }
            }
            // Spectators always see seat-0's projection so they get a
            // stable POV across the match.
            for tx in spectator_tx {
                let _ = tx.send(ServerMsg::Update {
                    events: wire_events.clone(),
                    view: Box::new(view::project(state, 0)),
                });
            }
            true
        }
        Err(e) => {
            // `ManualTapRequired` tapped the cost's forced colored sources
            // before stopping for the player's generic choice — that's a
            // real state change, so push an updated view (the normal error
            // path sends none) so the client sees the tapped sources.
            if matches!(e, crate::game::GameError::ManualTapRequired { .. }) {
                for (i, maybe_tx) in seat_tx.iter().enumerate() {
                    if let Some(tx) = maybe_tx {
                        let _ = tx.send(ServerMsg::View(Box::new(view::project(state, i))));
                    }
                }
                for tx in spectator_tx {
                    let _ = tx.send(ServerMsg::View(Box::new(view::project(state, 0))));
                }
            }
            report_error(seat, &e.to_string(), seat_tx);
            false
        }
    }
}

/// Look up a card by name for the debug console, tolerant of casing.
/// Tries an exact match first (cheap), then falls back to a linear
/// case-insensitive scan over every known factory so users can type
/// `ghor-clan rampager` instead of `Ghor-Clan Rampager`.
fn lookup_debug_card(name: &str) -> Option<crate::card::CardDefinition> {
    if let Some(def) = crate::catalog::lookup_by_name(name) {
        return Some(def);
    }
    let target = name.trim().to_ascii_lowercase();
    for factory in crate::catalog::all_known_factories() {
        let def = factory();
        if def.name.eq_ignore_ascii_case(&target) {
            return Some(def);
        }
        if let Some(back) = def.back_face.as_ref()
            && back.name.eq_ignore_ascii_case(&target)
        {
            return Some(def);
        }
    }
    None
}

/// Apply a debug-console cheat directly to the authoritative state and
/// broadcast a fresh view to every seat. Returns `true` on success.
/// No event stream is emitted — these mutations bypass the engine's
/// rules and don't have corresponding `GameEvent`s — but every observer
/// receives the updated `View` so their UI rerenders. Out-of-range
/// seats and unknown card names are silently dropped (we keep this
/// best-effort because it's a developer tool, not a player-facing API).
fn apply_debug(
    state: &mut GameState,
    seat: usize,
    debug: DebugAction,
    seat_tx: &[Option<mpsc::Sender<ServerMsg>>],
    spectator_tx: &[mpsc::Sender<ServerMsg>],
) -> bool {
    if seat >= state.players.len() {
        return false;
    }
    let changed = match debug {
        DebugAction::AddMana { color, amount } => {
            if amount == 0 {
                return false;
            }
            match color {
                Some(c) => state.players[seat].mana_pool.add(c, amount),
                None => state.players[seat].mana_pool.add_colorless(amount),
            }
            true
        }
        DebugAction::AddCardToHand { name } => {
            match lookup_debug_card(&name) {
                Some(def) => {
                    state.add_card_to_hand(seat, def);
                    true
                }
                None => {
                    report_error(seat, &format!("debug: unknown card '{name}'"), seat_tx);
                    false
                }
            }
        }
        DebugAction::AdjustLife { delta } => {
            if delta == 0 {
                return false;
            }
            // Routes through the team-aware helper so that in 2HG the
            // shared pool is the one nudged, not the individual seat's
            // dormant `life` field.
            state.adjust_life(seat, delta);
            true
        }
    };
    if !changed {
        return false;
    }
    for (i, maybe_tx) in seat_tx.iter().enumerate() {
        if let Some(tx) = maybe_tx {
            let _ = tx.send(ServerMsg::View(Box::new(view::project(state, i))));
        }
    }
    for tx in spectator_tx {
        let _ = tx.send(ServerMsg::View(Box::new(view::project(state, 0))));
    }
    true
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
    use crate::net::{ClientMsg, GameEventWire, ServerMsg};
    use crate::player::Player;
    use crate::server::bot::RandomBot;

    fn two_player_game() -> GameState {
        let mut state = GameState::new(vec![Player::new(0, "P0"), Player::new(1, "P1")]);
        // Start in main phase so PlayLand is legal.
        state.step = TurnStep::PreCombatMain;
        state
    }

    #[test]
    fn classify_loss_distinguishes_manner_of_elimination() {
        // Not eliminated → None regardless of life.
        let mut p = Player::new(0, "P0");
        p.life = 5;
        assert_eq!(classify_loss(&p), None);

        // Life depleted takes priority.
        p.eliminated = true;
        p.life = 0;
        assert_eq!(classify_loss(&p), Some(LossReason::LifeDepleted));

        // Poison while still at positive life.
        p.life = 7;
        p.poison_counters = 10;
        assert_eq!(classify_loss(&p), Some(LossReason::Poison));

        // Deck-out: alive, un-poisoned, empty library.
        p.poison_counters = 0;
        assert!(p.library.is_empty());
        assert_eq!(classify_loss(&p), Some(LossReason::Decked));

        // Otherwise: a non-empty library with no other lethal condition.
        p.library.push(crate::card::CardInstance::new(
            CardId(999),
            catalog::grizzly_bears(),
            0,
        ));
        assert_eq!(classify_loss(&p), Some(LossReason::Other));
    }

    #[test]
    fn capture_outcome_reports_loss_reasons_parallel_to_seats() {
        let mut state = two_player_game();
        state.players[1].eliminated = true;
        state.players[1].life = -3;
        let outcome = capture_outcome(&state);
        assert_eq!(outcome.loss_reasons.len(), state.players.len());
        assert_eq!(outcome.loss_reasons[0], None, "winner has no loss reason");
        assert_eq!(outcome.loss_reasons[1], Some(LossReason::LifeDepleted));
        // Library sizes are captured parallel to the seats.
        assert_eq!(outcome.final_library_sizes.len(), state.players.len());
        assert_eq!(
            outcome.final_library_sizes[0],
            state.players[0].library.len()
        );
        // Graveyard sizes are captured parallel to the seats too.
        assert_eq!(outcome.final_graveyard_sizes.len(), state.players.len());
        assert_eq!(
            outcome.final_graveyard_sizes[0],
            state.players[0].graveyard.len()
        );
        // Board sizes count the permanents each seat controls.
        let bear = state.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let _ = bear;
        let outcome2 = capture_outcome(&state);
        assert_eq!(outcome2.final_board_sizes.len(), state.players.len());
        assert_eq!(outcome2.final_board_sizes[0], 1, "seat 0 controls one permanent");
        assert_eq!(outcome2.final_board_sizes[1], 0);
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

        // Each seat receives a single combined Update frame (not a separate
        // Events then View), carrying both the action's events and the
        // post-action view.
        match c0.rx.recv().unwrap() {
            ServerMsg::Update { events, view } => {
                assert!(
                    events.iter().any(|e| matches!(e, GameEventWire::LandPlayed { .. })),
                    "Update must carry the action's events: {events:?}",
                );
                assert!(
                    view.battlefield.iter().any(|p| p.id == card_id),
                    "Update's view must reflect the played land",
                );
            }
            other => panic!("expected combined Update, got {other:?}"),
        }
        assert!(matches!(c1.rx.recv().unwrap(), ServerMsg::Update { .. }));

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

        // Human plays a land — the human seat should still receive a
        // combined Events+View Update.
        c0.tx
            .send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::Update { .. }));

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
    /// itself: each accepted action emits a single combined `Update` (events
    /// + view) to every seat, in order.
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

        let updates0 = m0.iter().filter(|m| matches!(m, ServerMsg::Update { .. })).count();
        let errs1 = m1.iter().filter(|m| matches!(m, ServerMsg::ActionError(_))).count();
        let updates1 = m1.iter().filter(|m| matches!(m, ServerMsg::Update { .. })).count();

        assert_eq!(updates0, 1, "c0 should see exactly one Update broadcast: {m0:?}");
        assert_eq!(errs1, 1, "c1 should see exactly one ActionError: {m1:?}");
        assert_eq!(updates1, 1, "c1 should see exactly one Update broadcast: {m1:?}");

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
        // combined Update on c0, no panic on the broadcast to the
        // already-dead seat 1.
        c0.tx
            .send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        let m0 = drain_within(&c0.rx, 2, Duration::from_secs(2));
        assert!(
            m0.iter().any(|m| matches!(m, ServerMsg::Update { .. })),
            "c0 missing Update after peer disconnect: {m0:?}"
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

    // ── Reconnection (run_match_reconnectable) ───────────────────────────────

    /// With reconnection enabled, one seat dropping doesn't end the match
    /// (the other human plays on), and reattaching that seat with a fresh
    /// channel replays YourSeat/MatchStarted/View reflecting the *current*
    /// mid-match state, after which the seat is live again.
    #[test]
    fn reconnect_reattach_replays_state_and_resumes() {
        let mut state = two_player_game();
        let card0 = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let (reattach_tx, reattach_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match_inner(
                state,
                vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)],
                vec![],
                None,
                Some(reattach_rx),
                // Short grace: seat 0 stays connected through the body, so the
                // window only matters at teardown (keeps the test fast).
                Duration::from_millis(200),
            )
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Seat 1 disconnects; seat 0 is still connected so the match lives.
        drop(c1);
        thread::sleep(Duration::from_millis(30));

        // Seat 0 plays its land — proves the match is still running.
        c0.tx
            .send(ClientMsg::SubmitAction(GameAction::PlayLand(card0)))
            .unwrap();
        assert!(
            matches!(c0.rx.recv().unwrap(), ServerMsg::Update { .. }),
            "match still live for seat 0 after seat 1 dropped",
        );

        // Reconnect seat 1 with a brand-new channel.
        let (s1b, c1b) = seat_pair();
        reattach_tx.send((1, s1b)).unwrap();

        // The reconnected client gets the standard handshake plus a view that
        // reflects seat 0's already-played land.
        assert!(matches!(c1b.rx.recv().unwrap(), ServerMsg::YourSeat(1)));
        assert!(matches!(c1b.rx.recv().unwrap(), ServerMsg::MatchStarted));
        match c1b.rx.recv().unwrap() {
            ServerMsg::View(v) => assert!(
                v.battlefield.iter().any(|p| p.id == card0),
                "reattach view must reflect the mid-match state",
            ),
            other => panic!("expected replayed View, got {other:?}"),
        }

        drop(c0);
        drop(c1b);
        drop(reattach_tx);
        let _ = handle.join();
    }

    /// With reconnection enabled, all humans dropping starts a grace window;
    /// if no one reattaches before it elapses, the match ends.
    #[test]
    fn reconnect_grace_expiry_ends_match() {
        let state = two_player_game();
        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let (reattach_tx, reattach_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let outcome = run_match_inner(
                state,
                vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)],
                vec![],
                None,
                Some(reattach_rx),
                Duration::from_millis(200),
            );
            let _ = done_tx.send(());
            outcome
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Both gone, nobody reattaches → match ends ~200ms later.
        drop(c0);
        drop(c1);
        done_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("match must end after the reconnect grace expires");
        drop(reattach_tx);
        let _ = handle.join();
    }

    /// A reattach inside the grace window cancels the pending expiry and keeps
    /// the match alive.
    #[test]
    fn reconnect_within_grace_keeps_match_alive() {
        let state = two_player_game();
        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let (reattach_tx, reattach_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let outcome = run_match_inner(
                state,
                vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)],
                vec![],
                None,
                Some(reattach_rx),
                Duration::from_millis(400),
            );
            let _ = done_tx.send(());
            outcome
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Both drop, then seat 0 reconnects well within the 400ms grace.
        drop(c0);
        drop(c1);
        thread::sleep(Duration::from_millis(50));
        let (s0b, c0b) = seat_pair();
        reattach_tx.send((0, s0b)).unwrap();
        assert!(matches!(c0b.rx.recv().unwrap(), ServerMsg::YourSeat(0)));
        assert!(matches!(c0b.rx.recv().unwrap(), ServerMsg::MatchStarted));
        assert!(matches!(c0b.rx.recv().unwrap(), ServerMsg::View(_)));

        // With seat 0 reconnected the deadline is cleared, so the match must
        // NOT end even after the original grace would have elapsed.
        assert!(
            done_rx.recv_timeout(Duration::from_millis(600)).is_err(),
            "a within-grace reattach must keep the match running",
        );

        // Tear down: drop the reconnected seat; with all humans gone again and
        // no further reattach, the match ends after the grace window.
        drop(c0b);
        drop(reattach_tx);
        done_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("match ends after the final grace window");
        let _ = handle.join();
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

    /// MatchOutcome should capture winner + life totals at game end —
    /// not just the turn count. Pins the new fields (`winner`,
    /// `final_life_totals`) against a deterministic 1-life-vs-1-life
    /// bot-vs-bot match.
    #[test]
    fn match_outcome_captures_winner_and_life_totals() {
        let mut state = two_player_game();
        state.players[0].life = 1;
        state.players[1].life = 1;
        // Seat 0 has a 2/2 attacker, seat 1 has nothing. Seat 0 should
        // win once it untaps (haste not required since end-of-step) /
        // attacks.
        let bear = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        state.clear_sickness(bear);
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            )
        });
        let outcome = handle.join().expect("match thread finishes");
        assert_eq!(outcome.final_life_totals.len(), 2, "two seats");
        // Winner is populated (either side could win depending on bot
        // dice, but the field must not be None given the game ended).
        assert!(outcome.winner.is_some(), "winner field populated post-game-over");
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

    /// 4-player bot FFA on the Commander demo deck (Rofellos mono-green
    /// mirror). Verifies the Phase I/J/L/M pipeline end-to-end:
    /// command zone populated at setup for all 4 seats, replacement
    /// effect bounces commanders back if killed, cast-from-CZ + tax
    /// accounting runs through `RandomBot` action picks across all
    /// 4 players, 21-commander-damage SBA still terminates the game.
    /// Phase A/B/C/D/E machinery is also exercised: 4-seat turn
    /// rotation, APNAP trigger ordering, team-aware attack/block
    /// validation collapsing to FFA semantics.
    #[test]
    fn bot_vs_bot_commander_demo_terminates() {
        use crate::demo::build_commander_state;
        let state = build_commander_state();
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(120))
            .expect("commander 4-player FFA bot match must terminate");
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
                .is_some_and(|a| a.iter().any(|c| c["name"] == "Tireless Tracker")),
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
                && let Some(snap) = guard.snapshot()
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
