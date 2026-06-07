//! Pre-match lobby system.
//!
//! Without lobbies, a connecting client is paired into a match immediately and
//! the gamemode is whatever the server was configured with. The lobby layer
//! sits between "connected" and "in a match": a connection browses the open
//! lobbies, then either creates one (choosing a [`LobbyFormat`]) or joins an
//! existing one. When a lobby reaches the seat count its gamemode requires, the
//! server starts the match for everyone in it.
//!
//! Two pieces:
//! - [`LobbyManager`] — a pure state machine over connection ids. It owns the
//!   open lobbies (each with a pre-built [`GameState`], so its capacity is
//!   exactly `state.players.len()`) and turns one client command into a set of
//!   messages to send plus an optional "start this match now" directive. No
//!   threads, no I/O — directly unit-testable.
//! - [`serve_lobbies`] — the threaded driver. It owns the socket-backed
//!   [`SeatChannel`]s, polls them for lobby commands, applies each to the
//!   manager, ships the resulting messages, and hands a filled lobby's channels
//!   to [`run_match`].

use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use crate::game::GameState;
use crate::net::{ClientMsg, LobbyFormat, LobbyInfo, ServerMsg, SpectatableInfo};

use super::{
    run_match_reconnectable_spectatable, MatchOutcome, RandomBot, SeatChannel, SeatOccupant,
};

/// Callback invoked when a lobby-started match finishes, with its gamemode,
/// wall-clock duration, and outcome. Lets the server binary fold lobby matches
/// into its rolling match stats / logs — lobby mode is the default, so without
/// this the server would have no per-match observability at all.
pub type MatchEndHook = Arc<dyn Fn(LobbyFormat, Duration, MatchOutcome) + Send + Sync>;

/// Where a valid [`ClientMsg::Resume`] token routes a reconnecting connection:
/// the running match's reattach channel and the seat to re-claim. One entry
/// per human seat; the entries for a match share its `reattach_tx`.
struct ResumeTarget {
    reattach_tx: mpsc::Sender<(usize, SeatChannel)>,
    seat: usize,
}

/// A live match registered for spectating. `spectate_tx` injects a fresh
/// `SeatChannel` into the running match as a read-only spectator;
/// `spectator_guards` holds each spectator's connection-slot guard for the
/// match's lifetime (so the connection cap stays held while watching), dropped
/// when the match ends and its registry entry is removed. `info` is the
/// listing advertised to browsers.
struct RunningMatch<G> {
    spectate_tx: mpsc::Sender<SeatChannel>,
    spectator_guards: Vec<G>,
    info: crate::net::SpectatableInfo,
}

/// Sent by a finished match thread back to the driver so it can prune the
/// match's resume tokens *and* drop its spectator registry entry (releasing any
/// held spectator guards).
struct MatchDone {
    match_id: u64,
    tokens: Vec<String>,
}

/// Generate an unguessable resume token. Two freshly-seeded `RandomState`s
/// (seeded from the OS RNG for HashMap DoS-resistance) yield 128 bits of
/// effectively-random hex — enough that another player can't guess a token
/// and hijack a seat on a LAN game.
fn new_token() -> String {
    use std::hash::BuildHasher;
    let a = std::collections::hash_map::RandomState::new().hash_one(0u8);
    let b = std::collections::hash_map::RandomState::new().hash_one(0u8);
    format!("{a:016x}{b:016x}")
}

/// Identifier the driver assigns to each connection so the [`LobbyManager`]
/// can reason about membership without touching the channels themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnId(pub u64);

/// Build the opening [`GameState`] for a gamemode. The lobby's capacity is
/// read from this state's player count, so the two can never disagree.
pub fn build_state(format: LobbyFormat) -> GameState {
    match format {
        LobbyFormat::Modern => crate::demo::build_demo_state(),
        LobbyFormat::Cube => crate::cube::build_cube_state(),
        LobbyFormat::Sos => crate::sos_mode::build_sos_state(),
        LobbyFormat::Commander => crate::demo::build_commander_state(),
    }
}

/// One seat in a lobby: a human connection (with its display name, captured
/// when seated) or a bot.
#[derive(Clone)]
enum Slot {
    Human { conn: ConnId, name: String },
    Bot,
}

/// One open lobby. Holds the pre-built state that will become the match.
struct Lobby {
    id: u64,
    name: String,
    format: LobbyFormat,
    /// Seats in order (the creator is seat 0). A mix of human connections and
    /// bots; the match starts when `seats.len() == capacity`.
    seats: Vec<Slot>,
    /// Pre-built game state; `capacity == state.players.len()`. Moved into
    /// the match when the lobby fills.
    state: GameState,
}

impl Lobby {
    fn capacity(&self) -> usize {
        self.state.players.len()
    }
    fn human_count(&self) -> usize {
        self.seats.iter().filter(|s| matches!(s, Slot::Human { .. })).count()
    }
    fn bot_count(&self) -> usize {
        self.seats.iter().filter(|s| matches!(s, Slot::Bot)).count()
    }
    /// The human connections seated here, in seat order.
    fn members(&self) -> Vec<ConnId> {
        self.seats
            .iter()
            .filter_map(|s| match s {
                Slot::Human { conn, .. } => Some(*conn),
                Slot::Bot => None,
            })
            .collect()
    }
    /// The human members' display names, in seat order.
    fn member_names(&self) -> Vec<String> {
        self.seats
            .iter()
            .filter_map(|s| match s {
                Slot::Human { name, .. } => Some(name.clone()),
                Slot::Bot => None,
            })
            .collect()
    }
    fn info(&self) -> LobbyInfo {
        let member_names = self.member_names();
        let host_name = member_names.first().cloned().unwrap_or_default();
        LobbyInfo {
            id: self.id,
            name: self.name.clone(),
            format: self.format,
            host_name,
            member_names,
            players: self.human_count(),
            bots: self.bot_count(),
            capacity: self.capacity(),
        }
    }
}

/// A seat in a filled lobby, ready to become a match occupant.
pub enum SeatSpec {
    Human(ConnId),
    Bot,
}

/// A filled lobby ready to become a match: the gamemode, pre-built state, and
/// seats in order. The driver maps each `Human` seat back to its channel and
/// each `Bot` seat to a fresh `RandomBot`.
pub struct StartMatch {
    pub format: LobbyFormat,
    pub state: GameState,
    pub seats: Vec<SeatSpec>,
    /// One label per seat, in seat order: a human's display name or "Bot".
    /// Surfaced to spectators in [`crate::net::SpectatableInfo`].
    pub seat_labels: Vec<String>,
}

/// Result of applying one command: messages to deliver, plus an optional match
/// to start. Kept data-only so [`LobbyManager`] stays I/O-free and testable.
#[derive(Default)]
pub struct LobbyOutcome {
    pub sends: Vec<(ConnId, ServerMsg)>,
    pub start: Option<StartMatch>,
}

impl LobbyOutcome {
    fn send(&mut self, conn: ConnId, msg: ServerMsg) {
        self.sends.push((conn, msg));
    }
}

/// Pure lobby state machine. See the module docs.
#[derive(Default)]
pub struct LobbyManager {
    lobbies: Vec<Lobby>,
    /// Every live connection (browsing or in a lobby).
    conns: HashSet<ConnId>,
    /// Connection → the lobby id it currently sits in. Absent ⇒ browsing.
    conn_lobby: HashMap<ConnId, u64>,
    /// Connection → display name (from `JoinMatch`), captured into a seat's
    /// `Slot::Human` when the connection creates or joins a lobby.
    conn_name: HashMap<ConnId, String>,
    next_id: u64,
}

impl LobbyManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// A new connection arrived and is browsing. Greets it with the current
    /// lobby list.
    pub fn register(&mut self, conn: ConnId) -> LobbyOutcome {
        self.conns.insert(conn);
        self.conn_name.entry(conn).or_insert_with(|| "Player".to_string());
        let mut out = LobbyOutcome::default();
        out.send(conn, self.lobby_list_msg());
        out
    }

    /// Display name for `conn`, defaulting to "Player".
    fn name_of(&self, conn: ConnId) -> String {
        self.conn_name.get(&conn).cloned().unwrap_or_else(|| "Player".to_string())
    }

    /// Apply one client message from `conn`. Non-lobby messages (game actions
    /// while still browsing, etc.) are ignored.
    pub fn handle(&mut self, conn: ConnId, msg: ClientMsg) -> LobbyOutcome {
        match msg {
            ClientMsg::JoinMatch { name } => {
                // Remember the announced name (used when this connection later
                // takes a seat); a connection already seated keeps the name it
                // was seated with.
                let clean = name.trim();
                if !clean.is_empty() {
                    self.conn_name.insert(conn, clean.chars().take(24).collect());
                }
                let mut out = LobbyOutcome::default();
                out.send(conn, self.lobby_list_msg());
                out
            }
            ClientMsg::ListLobbies => {
                let mut out = LobbyOutcome::default();
                out.send(conn, self.lobby_list_msg());
                out
            }
            ClientMsg::CreateLobby { name, format } => self.create(conn, name, format),
            ClientMsg::JoinLobby { lobby_id } => self.join(conn, lobby_id),
            ClientMsg::AddBotToLobby => self.add_bot(conn),
            ClientMsg::RemoveBotFromLobby => self.remove_bot(conn),
            ClientMsg::StartLobby => self.start_lobby(conn),
            ClientMsg::LeaveLobby => self.leave(conn),
            // `Resume` / spectate commands are handled by the driver (they need
            // the channel + running-match registry, not lobby state); they
            // never reach the manager. Game traffic from a browsing connection
            // is ignored.
            ClientMsg::Resume { .. }
            | ClientMsg::ListSpectatable
            | ClientMsg::SpectateMatch { .. }
            | ClientMsg::SubmitAction(_)
            | ClientMsg::Debug(_) => LobbyOutcome::default(),
        }
    }

    /// A connection dropped. Remove it from any lobby (tidying or notifying as
    /// needed) and forget it.
    pub fn disconnect(&mut self, conn: ConnId) -> LobbyOutcome {
        let mut out = self.remove_from_lobby(conn, false);
        self.conns.remove(&conn);
        self.conn_name.remove(&conn);
        self.push_browser_list(&mut out);
        out
    }

    fn create(&mut self, conn: ConnId, name: String, format: LobbyFormat) -> LobbyOutcome {
        let mut out = LobbyOutcome::default();
        if self.conn_lobby.contains_key(&conn) {
            out.send(conn, ServerMsg::LobbyError {
                message: "already in a lobby".into(),
            });
            return out;
        }
        let id = self.next_id;
        self.next_id += 1;
        let lobby = Lobby {
            id,
            name,
            format,
            seats: vec![Slot::Human { conn, name: self.name_of(conn) }],
            state: build_state(format),
        };
        self.lobbies.push(lobby);
        self.conn_lobby.insert(conn, id);
        self.notify_members(id, &mut out);
        // No auto-start: a lobby only begins when the host explicitly starts it
        // (`StartLobby`), even once it's full.
        self.push_browser_list(&mut out);
        out
    }

    /// Is `conn` the host (seat 0 / first human) of `lobby_id`?
    fn is_host(&self, conn: ConnId, lobby_id: u64) -> bool {
        self.lobbies
            .iter()
            .find(|l| l.id == lobby_id)
            .and_then(|l| l.members().first().copied())
            == Some(conn)
    }

    /// Resolve `conn`'s lobby and confirm it's the host, or push the
    /// appropriate `LobbyError` and return `None`.
    fn host_lobby(&mut self, conn: ConnId, out: &mut LobbyOutcome) -> Option<u64> {
        let Some(&lobby_id) = self.conn_lobby.get(&conn) else {
            out.send(conn, ServerMsg::LobbyError { message: "not in a lobby".into() });
            return None;
        };
        if !self.is_host(conn, lobby_id) {
            out.send(conn, ServerMsg::LobbyError { message: "only the host can do that".into() });
            return None;
        }
        Some(lobby_id)
    }

    /// Add a bot seat to the lobby (host-only). Fills a seat (and may start the
    /// match); errors if already full.
    fn add_bot(&mut self, conn: ConnId) -> LobbyOutcome {
        let mut out = LobbyOutcome::default();
        let Some(lobby_id) = self.host_lobby(conn, &mut out) else { return out };
        let lobby = self.lobbies.iter_mut().find(|l| l.id == lobby_id).unwrap();
        if lobby.seats.len() >= lobby.capacity() {
            out.send(conn, ServerMsg::LobbyError { message: "lobby is full".into() });
            return out;
        }
        lobby.seats.push(Slot::Bot);
        self.notify_members(lobby_id, &mut out);
        self.push_browser_list(&mut out);
        out
    }

    /// Remove the most-recently-added bot seat (host-only).
    fn remove_bot(&mut self, conn: ConnId) -> LobbyOutcome {
        let mut out = LobbyOutcome::default();
        let Some(lobby_id) = self.host_lobby(conn, &mut out) else { return out };
        let lobby = self.lobbies.iter_mut().find(|l| l.id == lobby_id).unwrap();
        if let Some(pos) = lobby.seats.iter().rposition(|s| matches!(s, Slot::Bot)) {
            lobby.seats.remove(pos);
            self.notify_members(lobby_id, &mut out);
            self.push_browser_list(&mut out);
        }
        out
    }

    /// Host requests an immediate start, filling every empty seat with a bot.
    fn start_lobby(&mut self, conn: ConnId) -> LobbyOutcome {
        let mut out = LobbyOutcome::default();
        let Some(lobby_id) = self.host_lobby(conn, &mut out) else { return out };
        let lobby = self.lobbies.iter_mut().find(|l| l.id == lobby_id).unwrap();
        while lobby.seats.len() < lobby.capacity() {
            lobby.seats.push(Slot::Bot);
        }
        self.maybe_start(lobby_id, &mut out);
        self.push_browser_list(&mut out);
        out
    }

    /// Send every human member of `lobby_id` a `LobbyJoined` carrying their
    /// *current* seat index, so each client's view of who's in the lobby — and
    /// whether it is the host (slot 0) — stays accurate across joins, leaves,
    /// and bot add/remove (any of which can shift seat indices).
    fn notify_members(&self, lobby_id: u64, out: &mut LobbyOutcome) {
        let Some(lobby) = self.lobbies.iter().find(|l| l.id == lobby_id) else { return };
        let info = lobby.info();
        for (i, slot) in lobby.seats.iter().enumerate() {
            if let Slot::Human { conn, .. } = slot {
                out.send(*conn, ServerMsg::LobbyJoined { lobby: info.clone(), your_slot: i });
            }
        }
    }

    fn join(&mut self, conn: ConnId, lobby_id: u64) -> LobbyOutcome {
        let mut out = LobbyOutcome::default();
        if self.conn_lobby.contains_key(&conn) {
            out.send(conn, ServerMsg::LobbyError {
                message: "already in a lobby".into(),
            });
            return out;
        }
        let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == lobby_id) else {
            out.send(conn, ServerMsg::LobbyError {
                message: "lobby no longer exists".into(),
            });
            return out;
        };
        if lobby.seats.len() >= lobby.capacity() {
            out.send(conn, ServerMsg::LobbyError { message: "lobby is full".into() });
            return out;
        }
        let name = self.name_of(conn);
        let lobby = self.lobbies.iter_mut().find(|l| l.id == lobby_id).unwrap();
        lobby.seats.push(Slot::Human { conn, name });
        self.conn_lobby.insert(conn, lobby_id);
        // Tell the new member and everyone already waiting their current slot.
        self.notify_members(lobby_id, &mut out);
        self.push_browser_list(&mut out);
        out
    }

    fn leave(&mut self, conn: ConnId) -> LobbyOutcome {
        let mut out = self.remove_from_lobby(conn, false);
        // The leaver is browsing again — hand them a fresh list.
        if self.conns.contains(&conn) {
            out.send(conn, self.lobby_list_msg());
        }
        self.push_browser_list(&mut out);
        out
    }

    /// If `lobby_id` is now at capacity (humans + bots), pull it out and emit
    /// a `StartMatch` carrying the seats in order.
    fn maybe_start(&mut self, lobby_id: u64, out: &mut LobbyOutcome) {
        let Some(idx) = self.lobbies.iter().position(|l| l.id == lobby_id) else { return };
        if self.lobbies[idx].seats.len() < self.lobbies[idx].capacity() {
            return;
        }
        let lobby = self.lobbies.remove(idx);
        for m in lobby.members() {
            self.conn_lobby.remove(&m);
            // The members are leaving the lobby system for a match; the driver
            // moves their channels into the match, so drop them from browsing.
            self.conns.remove(&m);
        }
        let seats = lobby
            .seats
            .iter()
            .map(|s| match s {
                Slot::Human { conn, .. } => SeatSpec::Human(*conn),
                Slot::Bot => SeatSpec::Bot,
            })
            .collect();
        let seat_labels = lobby
            .seats
            .iter()
            .map(|s| match s {
                Slot::Human { name, .. } => name.clone(),
                Slot::Bot => "Bot".to_string(),
            })
            .collect();
        out.start =
            Some(StartMatch { format: lobby.format, state: lobby.state, seats, seat_labels });
    }

    /// Remove `conn` from whatever lobby it's in. Notifies the remaining human
    /// members, or drops the lobby entirely once no humans are left (a
    /// bot-only lobby can't start). `_notify_leaver` is reserved for a future
    /// "you were kicked" message.
    fn remove_from_lobby(&mut self, conn: ConnId, _notify_leaver: bool) -> LobbyOutcome {
        let mut out = LobbyOutcome::default();
        let Some(lobby_id) = self.conn_lobby.remove(&conn) else { return out };
        let Some(idx) = self.lobbies.iter().position(|l| l.id == lobby_id) else { return out };
        self.lobbies[idx]
            .seats
            .retain(|s| !matches!(s, Slot::Human { conn: c, .. } if *c == conn));
        if self.lobbies[idx].human_count() == 0 {
            self.lobbies.remove(idx);
        } else {
            // Seats shifted — re-send each remaining member their new slot
            // (this also transfers the host to the new seat 0).
            self.notify_members(lobby_id, &mut out);
        }
        out
    }

    fn lobby_list_msg(&self) -> ServerMsg {
        ServerMsg::LobbyList {
            lobbies: self.lobbies.iter().map(|l| l.info()).collect(),
        }
    }

    /// Push a refreshed lobby list to every connection that is browsing (not
    /// currently inside a lobby), so open browsers update live.
    fn push_browser_list(&self, out: &mut LobbyOutcome) {
        let msg = self.lobby_list_msg();
        for &conn in &self.conns {
            if !self.conn_lobby.contains_key(&conn) {
                out.sends.push((conn, msg.clone()));
            }
        }
    }
}

/// Threaded lobby driver. Owns the connections' [`SeatChannel`]s, feeds their
/// inbound lobby commands to a [`LobbyManager`], delivers the outgoing
/// messages, and spawns a match thread when a lobby fills.
///
/// Each connection carries an opaque guard `G` (e.g. a connection-slot RAII
/// token from the server binary). The guard lives exactly as long as the
/// connection: it is dropped when the connection disconnects, and it travels
/// *with* the channel into the match thread when a lobby fills, so a cap held
/// at accept time stays held for the match's duration. Tests pass `()`.
///
/// `new_conns` is the stream of freshly-accepted connections (the accept loop
/// assigns each a [`ConnId`]). The loop polls non-blockingly and sleeps briefly
/// between passes; it returns when `new_conns` closes and no connections remain.
/// `on_match_end` is invoked when a lobby-started match finishes (for stats /
/// logging) — pass `Arc::new(|_, _, _| {})` to ignore it.
///
/// Once a lobby fills, the driver also registers the match for spectating: a
/// browsing connection can `ListSpectatable` to see the running matches and
/// `SpectateMatch { match_id }` to attach to one as a read-only spectator (it
/// is moved into the match's spectator slot, holding its connection guard for
/// the watch). The registry entry is dropped when the match ends.
pub fn serve_lobbies<G: Send + 'static>(
    new_conns: mpsc::Receiver<(ConnId, SeatChannel, G)>,
    on_match_end: MatchEndHook,
) {
    let mut mgr = LobbyManager::new();
    let mut channels: HashMap<ConnId, (SeatChannel, G)> = HashMap::new();
    // Live resume tokens → the running match's reattach channel + seat.
    let mut registry: HashMap<String, ResumeTarget> = HashMap::new();
    // Live matches available to spectate, keyed by match id.
    let mut running: HashMap<u64, RunningMatch<G>> = HashMap::new();
    let mut next_match_id: u64 = 0;
    // A finished match sends a `MatchDone` here so the driver can prune its
    // resume tokens (otherwise a stale token would only be cleared on a failed
    // resume) and drop its spectator registry entry.
    let (done_tx, done_rx) = mpsc::channel::<MatchDone>();
    let mut accepting = true;

    loop {
        // Prune tokens + spectator registry of matches that have ended.
        while let Ok(done) = done_rx.try_recv() {
            for t in done.tokens {
                registry.remove(&t);
            }
            running.remove(&done.match_id);
        }

        // Intake newly-accepted connections.
        loop {
            match new_conns.try_recv() {
                Ok((id, ch, guard)) => {
                    eprintln!("lobby: conn {} connected ({} online)", id.0, channels.len() + 1);
                    let outcome = mgr.register(id);
                    channels.insert(id, (ch, guard));
                    apply(
                        &mut channels,
                        outcome,
                        &on_match_end,
                        &mut registry,
                        &mut running,
                        &mut next_match_id,
                        &done_tx,
                    );
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    accepting = false;
                    break;
                }
            }
        }

        // Drain each connection's pending lobby commands.
        let ids: Vec<ConnId> = channels.keys().copied().collect();
        for id in ids {
            loop {
                let recv = channels.get(&id).map(|(c, _)| c.rx.try_recv());
                match recv {
                    Some(Ok(ClientMsg::Resume { token })) => {
                        // Driver-level: route this connection back into a match.
                        resume_into_match(&mut channels, &mut registry, id, &token);
                        if !channels.contains_key(&id) {
                            break; // moved into the match
                        }
                    }
                    Some(Ok(ClientMsg::ListSpectatable)) => {
                        // Driver-level: reply from the running-match registry.
                        if let Some((ch, _)) = channels.get(&id) {
                            let matches = running.values().map(|r| r.info.clone()).collect();
                            let _ = ch.tx.send(ServerMsg::SpectatableList { matches });
                        }
                    }
                    Some(Ok(ClientMsg::SpectateMatch { match_id })) => {
                        // Driver-level: move this connection into the match as a
                        // read-only spectator.
                        spectate_into_match(&mut channels, &mut running, id, match_id);
                        if !channels.contains_key(&id) {
                            break; // moved into the match
                        }
                    }
                    Some(Ok(msg)) => {
                        let outcome = mgr.handle(id, msg);
                        apply(
                            &mut channels,
                            outcome,
                            &on_match_end,
                            &mut registry,
                            &mut running,
                            &mut next_match_id,
                            &done_tx,
                        );
                        // `apply` may have moved this connection into a match.
                        if !channels.contains_key(&id) {
                            break;
                        }
                    }
                    Some(Err(mpsc::TryRecvError::Empty)) => break,
                    Some(Err(mpsc::TryRecvError::Disconnected)) | None => {
                        channels.remove(&id); // drops the guard → frees the slot
                        eprintln!("lobby: conn {} disconnected ({} online)", id.0, channels.len());
                        let outcome = mgr.disconnect(id);
                        apply(
                            &mut channels,
                            outcome,
                            &on_match_end,
                            &mut registry,
                            &mut running,
                            &mut next_match_id,
                            &done_tx,
                        );
                        break;
                    }
                }
            }
        }

        if !accepting && channels.is_empty() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// Route a reconnecting connection back into its match using a resume token.
/// On success the connection's channel is handed to the match's reattach
/// channel (and removed from the browsing pool); its transient slot guard is
/// dropped, since the match already holds the original seat's guard for the
/// connection cap.
fn resume_into_match<G>(
    channels: &mut HashMap<ConnId, (SeatChannel, G)>,
    registry: &mut HashMap<String, ResumeTarget>,
    id: ConnId,
    token: &str,
) {
    let Some((seat, reattach_tx)) = registry
        .get(token)
        .map(|t| (t.seat, t.reattach_tx.clone()))
    else {
        if let Some((ch, _)) = channels.get(&id) {
            let _ = ch.tx.send(ServerMsg::LobbyError {
                message: "invalid or expired resume token".into(),
            });
        }
        return;
    };
    let Some((ch, _guard)) = channels.remove(&id) else { return };
    match reattach_tx.send((seat, ch)) {
        Ok(()) => eprintln!("lobby: conn {} resumed into seat {seat}", id.0),
        Err(mpsc::SendError((_, ch))) => {
            // The match ended between issuing the token and this resume.
            registry.remove(token);
            let _ = ch.tx.send(ServerMsg::LobbyError {
                message: "match already ended".into(),
            });
        }
    }
}

/// Route a connection into a running match as a read-only spectator. On
/// success the connection's channel is handed to the match's `spectate_tx`
/// (and removed from the browsing pool) and its slot guard is parked in the
/// match's registry entry, so the connection cap stays held for the watch
/// session and is released when the match ends.
fn spectate_into_match<G>(
    channels: &mut HashMap<ConnId, (SeatChannel, G)>,
    running: &mut HashMap<u64, RunningMatch<G>>,
    id: ConnId,
    match_id: u64,
) {
    let Some(entry) = running.get(&match_id) else {
        if let Some((ch, _)) = channels.get(&id) {
            let _ = ch.tx.send(ServerMsg::LobbyError {
                message: "match no longer exists".into(),
            });
        }
        return;
    };
    let spectate_tx = entry.spectate_tx.clone();
    let Some((ch, guard)) = channels.remove(&id) else { return };
    match spectate_tx.send(ch) {
        Ok(()) => {
            eprintln!("lobby: conn {} spectating match {match_id}", id.0);
            // Hold the spectator's slot guard for the match's lifetime.
            if let Some(entry) = running.get_mut(&match_id) {
                entry.spectator_guards.push(guard);
            }
        }
        Err(mpsc::SendError(ch)) => {
            // The match ended between listing and this spectate request.
            running.remove(&match_id);
            let _ = ch.tx.send(ServerMsg::LobbyError {
                message: "match already ended".into(),
            });
        }
    }
}

/// Deliver an outcome's messages and, if a lobby filled, move its members'
/// channels (and their guards) out of the pool and spawn a *reconnectable*
/// match — issuing each human seat a resume token first.
#[allow(clippy::too_many_arguments)]
fn apply<G: Send + 'static>(
    channels: &mut HashMap<ConnId, (SeatChannel, G)>,
    outcome: LobbyOutcome,
    on_match_end: &MatchEndHook,
    registry: &mut HashMap<String, ResumeTarget>,
    running: &mut HashMap<u64, RunningMatch<G>>,
    next_match_id: &mut u64,
    done_tx: &mpsc::Sender<MatchDone>,
) {
    for (id, msg) in outcome.sends {
        if let Some((ch, _)) = channels.get(&id) {
            let _ = ch.tx.send(msg);
        }
    }
    if let Some(start) = outcome.start {
        // Abort the start if any human seat's channel vanished between fill and
        // spawn (their seat would just disconnect immediately otherwise).
        let all_present = start.seats.iter().all(|s| match s {
            SeatSpec::Human(id) => channels.contains_key(id),
            SeatSpec::Bot => true,
        });
        if !all_present {
            return;
        }

        let (reattach_tx, reattach_rx) = mpsc::channel::<(usize, SeatChannel)>();
        let mut occupants: Vec<SeatOccupant> = Vec::with_capacity(start.seats.len());
        let mut guards: Vec<G> = Vec::new();
        let mut tokens: Vec<String> = Vec::new();
        let (mut humans, mut bots) = (0usize, 0usize);
        for (seat_idx, seat) in start.seats.iter().enumerate() {
            match seat {
                SeatSpec::Human(id) => {
                    let (ch, guard) = channels.remove(id).expect("checked all_present");
                    // Issue a per-seat resume token before the match begins; it
                    // arrives ahead of the actor's YourSeat/MatchStarted.
                    let token = new_token();
                    let _ = ch.tx.send(ServerMsg::ResumeToken { token: token.clone() });
                    registry.insert(token.clone(), ResumeTarget {
                        reattach_tx: reattach_tx.clone(),
                        seat: seat_idx,
                    });
                    tokens.push(token);
                    occupants.push(SeatOccupant::Human(ch));
                    guards.push(guard);
                    humans += 1;
                }
                SeatSpec::Bot => {
                    occupants.push(SeatOccupant::Bot(Box::new(RandomBot::new())));
                    bots += 1;
                }
            }
        }

        let format = start.format;
        // Register the match for spectating: a `spectate_tx` to inject
        // read-only spectators mid-match, plus a listing for browsers. The id
        // is handed back to the driver on completion (via `MatchDone`) so the
        // entry — and any spectator guards it holds — is dropped.
        let match_id = *next_match_id;
        *next_match_id += 1;
        let (spectate_tx, spectate_rx) = mpsc::channel::<SeatChannel>();
        running.insert(match_id, RunningMatch {
            spectate_tx,
            spectator_guards: Vec::new(),
            info: SpectatableInfo {
                match_id,
                format,
                seat_labels: start.seat_labels.clone(),
                turn: start.state.turn_number,
            },
        });

        let hook = Arc::clone(on_match_end);
        let done = done_tx.clone();
        eprintln!("lobby: starting {} match ({humans} human, {bots} bot)", format.label());
        thread::spawn(move || {
            // Hold the per-connection guards for the match's lifetime so a
            // connection cap acquired at accept time isn't released early.
            let _guards = guards;
            let started = Instant::now();
            let outcome = run_match_reconnectable_spectatable(
                start.state,
                occupants,
                vec![],
                None,
                Some(reattach_rx),
                Some(spectate_rx),
            );
            // Match over — let the driver prune our resume tokens + registry.
            let _ = done.send(MatchDone { match_id, tokens });
            hook(format, started.elapsed(), outcome);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::seat_pair;
    use crate::server::ClientChannel;
    use std::time::Instant;

    fn drain<T>(rx: &mpsc::Receiver<T>) -> Vec<T> {
        std::iter::from_fn(|| rx.try_recv().ok()).collect()
    }

    #[test]
    fn register_greets_with_empty_lobby_list() {
        let mut m = LobbyManager::new();
        let out = m.register(ConnId(1));
        assert_eq!(out.sends.len(), 1);
        assert!(matches!(
            &out.sends[0].1,
            ServerMsg::LobbyList { lobbies } if lobbies.is_empty()
        ));
    }

    #[test]
    fn create_join_then_host_starts_a_two_seat_match() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        m.register(ConnId(2));

        // Seat 1 creates a Modern (2-seat) lobby.
        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "alice's game".into(),
            format: LobbyFormat::Modern,
        });
        // Creator is told they joined as slot 0; not full yet, no match.
        assert!(out.start.is_none());
        let joined = out.sends.iter().find_map(|(c, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, your_slot } if *c == ConnId(1) => {
                Some((lobby.clone(), *your_slot))
            }
            _ => None,
        });
        let (info, slot) = joined.expect("creator gets LobbyJoined");
        assert_eq!(slot, 0);
        assert_eq!(info.capacity, 2);
        assert_eq!(info.players, 1);
        let lobby_id = info.id;

        // The other browser sees the new lobby in a pushed list.
        assert!(out.sends.iter().any(|(c, msg)| *c == ConnId(2)
            && matches!(msg, ServerMsg::LobbyList { lobbies } if lobbies.len() == 1)));

        // Seat 2 joins → lobby is full but does NOT auto-start.
        let out = m.handle(ConnId(2), ClientMsg::JoinLobby { lobby_id });
        assert!(out.start.is_none(), "a full lobby waits for the host to start");
        // The host explicitly starts → the match begins with both members.
        let out = m.handle(ConnId(1), ClientMsg::StartLobby);
        let start = out.start.expect("the host starting begins the match");
        assert_eq!(start.seats.len(), 2);
        assert!(matches!(start.seats[0], SeatSpec::Human(ConnId(1))));
        assert!(matches!(start.seats[1], SeatSpec::Human(ConnId(2))));
        assert_eq!(start.state.players.len(), 2);
        // The lobby is gone afterward, and both members left the browsing set.
        assert!(m.lobbies.is_empty());
        assert!(!m.conn_lobby.contains_key(&ConnId(1)));
        assert!(!m.conn_lobby.contains_key(&ConnId(2)));
    }

    #[test]
    fn full_lobby_does_not_auto_start_until_the_host_starts() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "vs bot".into(),
            format: LobbyFormat::Modern,
        });
        let info = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.clone()),
            _ => None,
        }).unwrap();
        assert_eq!((info.players, info.bots), (1, 0));

        // Adding a bot fills the 2-seat lobby but does NOT start it.
        let out = m.handle(ConnId(1), ClientMsg::AddBotToLobby);
        assert!(out.start.is_none(), "filling the lobby must not auto-start it");
        assert_eq!(m.lobbies.len(), 1, "the lobby is still open, full, waiting");

        // The host starts → the (human + bot) match begins.
        let out = m.handle(ConnId(1), ClientMsg::StartLobby);
        let start = out.start.expect("the host starting begins the match");
        assert_eq!(start.seats.len(), 2);
        assert!(matches!(start.seats[0], SeatSpec::Human(ConnId(1))));
        assert!(matches!(start.seats[1], SeatSpec::Bot));
        assert!(m.lobbies.is_empty());
    }

    #[test]
    fn add_then_remove_bot_in_a_four_seat_lobby_tracks_counts() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "edh".into(),
            format: LobbyFormat::Commander, // 4 seats
        });
        let id = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.id),
            _ => None,
        }).unwrap();

        // Add two bots (1 human + 2 bots = 3/4, not yet full).
        let out = m.handle(ConnId(1), ClientMsg::AddBotToLobby);
        assert!(out.start.is_none());
        let out = m.handle(ConnId(1), ClientMsg::AddBotToLobby);
        assert!(out.start.is_none());
        let info = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.clone()),
            _ => None,
        }).unwrap();
        assert_eq!((info.players, info.bots), (1, 2));

        // Remove one bot → back to 1 human + 1 bot.
        let out = m.handle(ConnId(1), ClientMsg::RemoveBotFromLobby);
        let info = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.clone()),
            _ => None,
        }).unwrap();
        assert_eq!((info.players, info.bots), (1, 1));

        // Still has a human, so it survives; lobby remains open.
        let _ = id;
        assert_eq!(m.lobbies.len(), 1);
    }

    #[test]
    fn add_bot_while_browsing_errors() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        let out = m.handle(ConnId(1), ClientMsg::AddBotToLobby);
        assert!(out.sends.iter().any(|(_, msg)| matches!(msg, ServerMsg::LobbyError { .. })));
        assert!(out.start.is_none());
    }

    #[test]
    fn lobby_info_carries_host_and_member_names() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        m.register(ConnId(2));
        // Names announced via JoinMatch are captured when seated.
        m.handle(ConnId(1), ClientMsg::JoinMatch { name: "Alice".into() });
        m.handle(ConnId(2), ClientMsg::JoinMatch { name: "Bob".into() });

        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "g".into(),
            format: LobbyFormat::Commander, // 4 seats: won't fill on join
        });
        let info = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.clone()),
            _ => None,
        }).unwrap();
        assert_eq!(info.host_name, "Alice");
        assert_eq!(info.member_names, vec!["Alice".to_string()]);

        let lobby_id = info.id;
        let out = m.handle(ConnId(2), ClientMsg::JoinLobby { lobby_id });
        let info = out.sends.iter().find_map(|(c, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } if *c == ConnId(2) => Some(lobby.clone()),
            _ => None,
        }).unwrap();
        assert_eq!(info.host_name, "Alice", "host stays the creator");
        assert_eq!(info.member_names, vec!["Alice".to_string(), "Bob".to_string()]);
    }

    #[test]
    fn join_missing_or_full_lobby_errors() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        let out = m.handle(ConnId(1), ClientMsg::JoinLobby { lobby_id: 999 });
        assert!(out.sends.iter().any(|(_, msg)| matches!(msg, ServerMsg::LobbyError { .. })));
        assert!(out.start.is_none());
    }

    #[test]
    fn leaving_an_empty_lobby_removes_it() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "solo".into(),
            format: LobbyFormat::Modern,
        });
        let id = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.id),
            _ => None,
        }).unwrap();
        assert_eq!(m.lobbies.len(), 1);
        m.handle(ConnId(1), ClientMsg::LeaveLobby);
        assert!(m.lobbies.is_empty(), "empty lobby is cleaned up on leave");
        // Joining the now-gone lobby errors.
        let out = m.handle(ConnId(1), ClientMsg::JoinLobby { lobby_id: id });
        assert!(out.sends.iter().any(|(_, msg)| matches!(msg, ServerMsg::LobbyError { .. })));
    }

    #[test]
    fn disconnect_in_a_waiting_lobby_notifies_the_remaining_member() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        m.register(ConnId(2));
        // 4-seat Commander lobby so two members can wait without it filling.
        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "edh".into(),
            format: LobbyFormat::Commander,
        });
        let id = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.id),
            _ => None,
        }).unwrap();
        m.handle(ConnId(2), ClientMsg::JoinLobby { lobby_id: id });

        // Seat 1 (the host) drops; seat 2 is told the roster shrank AND that
        // it is now seat 0 (the new host).
        let out = m.disconnect(ConnId(1));
        let updated = out.sends.iter().find_map(|(c, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, your_slot } if *c == ConnId(2) => {
                Some((lobby.players, *your_slot))
            }
            _ => None,
        });
        assert_eq!(updated, Some((1, 0)), "remaining member becomes the host at slot 0");
    }

    #[test]
    fn host_start_fills_empty_seats_with_bots() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "edh".into(),
            format: LobbyFormat::Commander, // 4 seats
        });
        let id = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.id),
            _ => None,
        }).unwrap();

        // Host starts now → the other 3 seats fill with bots and the match
        // begins (1 human + 3 bots).
        let out = m.handle(ConnId(1), ClientMsg::StartLobby);
        let start = out.start.expect("host start fills + starts");
        assert_eq!(start.seats.len(), 4);
        assert!(matches!(start.seats[0], SeatSpec::Human(ConnId(1))));
        assert!(start.seats[1..].iter().all(|s| matches!(s, SeatSpec::Bot)));
        let _ = id;
    }

    #[test]
    fn only_host_may_add_bots_or_start() {
        let mut m = LobbyManager::new();
        m.register(ConnId(1));
        m.register(ConnId(2));
        let out = m.handle(ConnId(1), ClientMsg::CreateLobby {
            name: "edh".into(),
            format: LobbyFormat::Commander,
        });
        let id = out.sends.iter().find_map(|(_, msg)| match msg {
            ServerMsg::LobbyJoined { lobby, .. } => Some(lobby.id),
            _ => None,
        }).unwrap();
        m.handle(ConnId(2), ClientMsg::JoinLobby { lobby_id: id });

        // Non-host seat 2 can't add bots or start.
        for cmd in [ClientMsg::AddBotToLobby, ClientMsg::RemoveBotFromLobby, ClientMsg::StartLobby] {
            let out = m.handle(ConnId(2), cmd);
            assert!(out.start.is_none());
            assert!(
                out.sends.iter().any(|(c, msg)|
                    *c == ConnId(2) && matches!(msg, ServerMsg::LobbyError { .. })),
                "non-host action must be refused",
            );
        }
        // Host (seat 1) can add a bot.
        let out = m.handle(ConnId(1), ClientMsg::AddBotToLobby);
        assert!(out.sends.iter().any(|(_, msg)|
            matches!(msg, ServerMsg::LobbyJoined { lobby, .. } if lobby.bots == 1)));
    }

    /// End-to-end through the threaded driver: two in-process clients connect,
    /// create + join a Modern lobby, and the driver starts the match — both
    /// clients receive the opening `YourSeat` / `MatchStarted` handshake.
    #[test]
    fn serve_lobbies_drives_create_join_to_match_start() {
        let (new_tx, new_rx) = mpsc::channel();
        let driver = thread::spawn(move || serve_lobbies(new_rx, Arc::new(|_, _, _| {})));

        // Two connections, each a (server SeatChannel, client ClientChannel).
        let (s1, c1) = seat_pair();
        let (s2, c2) = seat_pair();
        new_tx.send((ConnId(1), s1, ())).unwrap();
        new_tx.send((ConnId(2), s2, ())).unwrap();

        // Each gets an initial LobbyList from `register`.
        let id = await_lobby_then_create(&c1);
        // Client 1 created a Modern lobby — wait for its id via LobbyJoined.
        let lobby_id = id;

        // Client 2 joins; the lobby is now full but waits for the host.
        c2.tx.send(ClientMsg::JoinLobby { lobby_id }).unwrap();
        // Once the host sees the joiner, it starts the match.
        await_player_count(&c1, 2);
        c1.tx.send(ClientMsg::StartLobby).unwrap();

        assert!(
            recv_match_started(&c1),
            "creator should enter the match",
        );
        assert!(
            recv_match_started(&c2),
            "joiner should enter the match",
        );

        // Dropping the clients ends the match + lets the driver wind down.
        drop(c1);
        drop(c2);
        drop(new_tx);
        let _ = driver.join();
    }

    /// Drain `c` until it observes a `LobbyJoined` showing at least `players`
    /// human seats.
    fn await_player_count(c: &ClientChannel, players: usize) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            for m in drain(&c.rx) {
                if let ServerMsg::LobbyJoined { lobby, .. } = m
                    && lobby.players >= players
                {
                    return;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("never observed {players} players in the lobby");
    }

    /// Send `CreateLobby` once the first `LobbyList` arrives, then return the
    /// created lobby's id from the `LobbyJoined` reply.
    fn await_lobby_then_create(c: &ClientChannel) -> u64 {
        let deadline = Instant::now() + Duration::from_secs(2);
        // Wait for the initial list.
        while Instant::now() < deadline {
            if drain(&c.rx).iter().any(|m| matches!(m, ServerMsg::LobbyList { .. })) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        c.tx.send(ClientMsg::CreateLobby {
            name: "auto".into(),
            format: LobbyFormat::Modern,
        }).unwrap();
        while Instant::now() < deadline {
            for m in drain(&c.rx) {
                if let ServerMsg::LobbyJoined { lobby, .. } = m {
                    return lobby.id;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("never received LobbyJoined for the created lobby");
    }

    fn recv_match_started(c: &ClientChannel) -> bool {
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            for m in drain(&c.rx) {
                if matches!(m, ServerMsg::MatchStarted) {
                    return true;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }

    /// End-to-end reconnect through the driver: a host creates a Modern lobby,
    /// fills it with a bot to start a (human + bot) match, drops its
    /// connection, then a fresh connection presents the resume token and is
    /// routed back into the match (receiving the replayed handshake).
    #[test]
    fn reconnect_via_resume_token_rejoins_a_lobby_match() {
        let (new_tx, new_rx) = mpsc::channel();
        let driver = thread::spawn(move || serve_lobbies(new_rx, Arc::new(|_, _, _| {})));

        let (s1, c1) = seat_pair();
        new_tx.send((ConnId(1), s1, ())).unwrap();

        // Create a Modern (2-seat) lobby, add a bot, then start the match.
        let _ = await_lobby_then_create(&c1);
        c1.tx.send(ClientMsg::AddBotToLobby).unwrap();
        c1.tx.send(ClientMsg::StartLobby).unwrap();

        // Collect the resume token and confirm we entered the match (without
        // discarding either — they arrive in the same burst).
        let mut token = None;
        let mut started = false;
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && !(token.is_some() && started) {
            for m in drain(&c1.rx) {
                match m {
                    ServerMsg::ResumeToken { token: t } => token = Some(t),
                    ServerMsg::MatchStarted => started = true,
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        let token = token.expect("a resume token is issued at match start");
        assert!(started, "creator enters the match");

        // The human drops out of the live match (the seat enters the grace
        // window — the match keeps running).
        drop(c1);
        thread::sleep(Duration::from_millis(100));

        // A fresh connection resumes with the token and is routed back in.
        let (s2, c2) = seat_pair();
        new_tx.send((ConnId(2), s2, ())).unwrap();
        thread::sleep(Duration::from_millis(100)); // let the driver register it
        c2.tx.send(ClientMsg::Resume { token }).unwrap();

        assert!(
            recv_match_started(&c2),
            "the reconnecting client re-enters the match",
        );

        drop(c2);
        drop(new_tx);
        let _ = driver.join();
    }

    /// An unknown / expired resume token is rejected with a LobbyError, and the
    /// connection stays browsing.
    #[test]
    fn resume_with_unknown_token_errors() {
        let (new_tx, new_rx) = mpsc::channel();
        let driver = thread::spawn(move || serve_lobbies(new_rx, Arc::new(|_, _, _| {})));

        let (s1, c1) = seat_pair();
        new_tx.send((ConnId(1), s1, ())).unwrap();
        c1.tx.send(ClientMsg::Resume { token: "bogus".into() }).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut errored = false;
        while Instant::now() < deadline && !errored {
            for m in drain(&c1.rx) {
                if matches!(m, ServerMsg::LobbyError { .. }) {
                    errored = true;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(errored, "an unknown resume token is rejected");

        drop(c1);
        drop(new_tx);
        let _ = driver.join();
    }

    // ── Spectating (ListSpectatable / SpectateMatch) ─────────────────────────

    /// Poll `ListSpectatable` until at least one running match is advertised,
    /// returning the first match's id (and asserting the listing metadata).
    fn await_spectatable(c: &ClientChannel) -> u64 {
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            c.tx.send(ClientMsg::ListSpectatable).unwrap();
            thread::sleep(Duration::from_millis(20));
            for m in drain(&c.rx) {
                if let ServerMsg::SpectatableList { matches } = m
                    && let Some(first) = matches.first()
                {
                    assert!(
                        !first.seat_labels.is_empty(),
                        "a listed match advertises its seat labels",
                    );
                    return first.match_id;
                }
            }
        }
        panic!("never observed a spectatable match");
    }

    /// End-to-end: a host starts a Modern match vs a bot; a second connection
    /// lists the running matches and attaches as a read-only spectator,
    /// receiving the spectator handshake (sentinel seat + a spectator view).
    #[test]
    fn spectate_a_running_lobby_match() {
        let (new_tx, new_rx) = mpsc::channel();
        let driver = thread::spawn(move || serve_lobbies(new_rx, Arc::new(|_, _, _| {})));

        // conn 1 hosts a (human + bot) Modern match.
        let (s1, c1) = seat_pair();
        new_tx.send((ConnId(1), s1, ())).unwrap();
        let _ = await_lobby_then_create(&c1);
        c1.tx.send(ClientMsg::AddBotToLobby).unwrap();
        c1.tx.send(ClientMsg::StartLobby).unwrap();
        assert!(recv_match_started(&c1), "host enters the match");

        // conn 2 connects, discovers the running match, and spectates it.
        let (s2, c2) = seat_pair();
        new_tx.send((ConnId(2), s2, ())).unwrap();
        let match_id = await_spectatable(&c2);
        c2.tx.send(ClientMsg::SpectateMatch { match_id }).unwrap();

        // The spectator gets the sentinel seat and a spectator-safe view.
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut saw_view = false;
        while Instant::now() < deadline && !saw_view {
            for m in drain(&c2.rx) {
                if let ServerMsg::View(v) = m
                    && v.your_seat == crate::net::SPECTATOR_SEAT
                {
                    saw_view = true;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(saw_view, "spectator receives a spectator-seat view");

        drop(c1);
        drop(c2);
        drop(new_tx);
        let _ = driver.join();
    }

    /// Spectating an unknown match id is rejected with a LobbyError, and the
    /// connection stays browsing.
    #[test]
    fn spectate_unknown_match_errors() {
        let (new_tx, new_rx) = mpsc::channel();
        let driver = thread::spawn(move || serve_lobbies(new_rx, Arc::new(|_, _, _| {})));

        let (s1, c1) = seat_pair();
        new_tx.send((ConnId(1), s1, ())).unwrap();
        c1.tx.send(ClientMsg::SpectateMatch { match_id: 999 }).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut errored = false;
        while Instant::now() < deadline && !errored {
            for m in drain(&c1.rx) {
                if matches!(m, ServerMsg::LobbyError { .. }) {
                    errored = true;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(errored, "spectating an unknown match id is rejected");

        drop(c1);
        drop(new_tx);
        let _ = driver.join();
    }
}
