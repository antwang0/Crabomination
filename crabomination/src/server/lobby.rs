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
use crate::net::{ClientMsg, LobbyFormat, LobbyInfo, ServerMsg};

use super::{run_match, MatchOutcome, RandomBot, SeatChannel, SeatOccupant};

/// Callback invoked when a lobby-started match finishes, with its gamemode,
/// wall-clock duration, and outcome. Lets the server binary fold lobby matches
/// into its rolling match stats / logs — lobby mode is the default, so without
/// this the server would have no per-match observability at all.
pub type MatchEndHook = Arc<dyn Fn(LobbyFormat, Duration, MatchOutcome) + Send + Sync>;

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
            // Browsing connections aren't in a match; ignore game traffic.
            ClientMsg::SubmitAction(_) | ClientMsg::Debug(_) => LobbyOutcome::default(),
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
        // A 1-seat gamemode would start instantly; none exist today, but the
        // check guards against an exotic format stranding the creator.
        self.maybe_start(id, &mut out);
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
        self.maybe_start(lobby_id, &mut out);
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
        self.maybe_start(lobby_id, &mut out);
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
        out.start = Some(StartMatch { format: lobby.format, state: lobby.state, seats });
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
pub fn serve_lobbies<G: Send + 'static>(
    new_conns: mpsc::Receiver<(ConnId, SeatChannel, G)>,
    on_match_end: MatchEndHook,
) {
    let mut mgr = LobbyManager::new();
    let mut channels: HashMap<ConnId, (SeatChannel, G)> = HashMap::new();
    let mut accepting = true;

    loop {
        // Intake newly-accepted connections.
        loop {
            match new_conns.try_recv() {
                Ok((id, ch, guard)) => {
                    eprintln!("lobby: conn {} connected ({} online)", id.0, channels.len() + 1);
                    let outcome = mgr.register(id);
                    channels.insert(id, (ch, guard));
                    apply(&mut channels, outcome, &on_match_end);
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
                    Some(Ok(msg)) => {
                        let outcome = mgr.handle(id, msg);
                        apply(&mut channels, outcome, &on_match_end);
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
                        apply(&mut channels, outcome, &on_match_end);
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

/// Deliver an outcome's messages and, if a lobby filled, move its members'
/// channels (and their guards) out of the pool and spawn the match.
fn apply<G: Send + 'static>(
    channels: &mut HashMap<ConnId, (SeatChannel, G)>,
    outcome: LobbyOutcome,
    on_match_end: &MatchEndHook,
) {
    for (id, msg) in outcome.sends {
        if let Some((ch, _)) = channels.get(&id) {
            let _ = ch.tx.send(msg);
        }
    }
    if let Some(start) = outcome.start {
        let mut occupants: Vec<SeatOccupant> = Vec::with_capacity(start.seats.len());
        let mut guards: Vec<G> = Vec::new();
        let mut all_present = true;
        let (mut humans, mut bots) = (0usize, 0usize);
        for seat in &start.seats {
            match seat {
                SeatSpec::Human(id) => match channels.remove(id) {
                    Some((ch, guard)) => {
                        occupants.push(SeatOccupant::Human(ch));
                        guards.push(guard);
                        humans += 1;
                    }
                    None => all_present = false,
                },
                SeatSpec::Bot => {
                    occupants.push(SeatOccupant::Bot(Box::new(RandomBot::new())));
                    bots += 1;
                }
            }
        }
        // Only start if every human seat's channel was still present; a member
        // that vanished between fill and spawn aborts the start (their seat
        // would just disconnect immediately otherwise).
        if all_present && occupants.len() == start.seats.len() {
            let format = start.format;
            let hook = Arc::clone(on_match_end);
            eprintln!("lobby: starting {} match ({humans} human, {bots} bot)", format.label());
            thread::spawn(move || {
                // Hold the per-connection guards for the match's lifetime so a
                // connection cap acquired at accept time isn't released early.
                let _guards = guards;
                let started = Instant::now();
                let outcome = run_match(start.state, occupants);
                hook(format, started.elapsed(), outcome);
            });
        }
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
    fn create_then_join_fills_and_starts_a_two_seat_match() {
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

        // Seat 2 joins → lobby is full → a match starts with both members.
        let out = m.handle(ConnId(2), ClientMsg::JoinLobby { lobby_id });
        let start = out.start.expect("filling the lobby starts the match");
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
    fn adding_a_bot_fills_a_two_seat_lobby_and_starts_human_vs_bot() {
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
        assert_eq!(info.players, 1);
        assert_eq!(info.bots, 0);

        // Adding one bot fills the 2-seat Modern lobby → match starts.
        let out = m.handle(ConnId(1), ClientMsg::AddBotToLobby);
        let start = out.start.expect("a bot filling the lobby starts the match");
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

        // Client 2 joins; the lobby fills and the match starts.
        c2.tx.send(ClientMsg::JoinLobby { lobby_id }).unwrap();

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
}
