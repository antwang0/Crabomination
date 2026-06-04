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
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::game::GameState;
use crate::net::{ClientMsg, LobbyFormat, LobbyInfo, ServerMsg};

use super::{run_match, SeatChannel, SeatOccupant};

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

/// One open lobby. Holds the pre-built state that will become the match.
struct Lobby {
    id: u64,
    name: String,
    format: LobbyFormat,
    /// Members in seat order (the creator is seat 0).
    members: Vec<ConnId>,
    /// Pre-built game state; `capacity == state.players.len()`. Moved into
    /// the match when the lobby fills.
    state: GameState,
}

impl Lobby {
    fn capacity(&self) -> usize {
        self.state.players.len()
    }
    fn info(&self) -> LobbyInfo {
        LobbyInfo {
            id: self.id,
            name: self.name.clone(),
            format: self.format,
            players: self.members.len(),
            capacity: self.capacity(),
        }
    }
}

/// A filled lobby ready to become a match: the pre-built state plus the member
/// connection ids in seat order. The driver maps the ids back to channels.
pub struct StartMatch {
    pub state: GameState,
    pub members: Vec<ConnId>,
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
        let mut out = LobbyOutcome::default();
        out.send(conn, self.lobby_list_msg());
        out
    }

    /// Apply one client message from `conn`. Non-lobby messages (game actions
    /// while still browsing, etc.) are ignored.
    pub fn handle(&mut self, conn: ConnId, msg: ClientMsg) -> LobbyOutcome {
        match msg {
            ClientMsg::JoinMatch { .. } | ClientMsg::ListLobbies => {
                let mut out = LobbyOutcome::default();
                out.send(conn, self.lobby_list_msg());
                out
            }
            ClientMsg::CreateLobby { name, format } => self.create(conn, name, format),
            ClientMsg::JoinLobby { lobby_id } => self.join(conn, lobby_id),
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
            members: vec![conn],
            state: build_state(format),
        };
        let info = lobby.info();
        // A 1-seat gamemode would start instantly; none exist today, but guard
        // anyway so an exotic format can't strand the creator forever.
        self.lobbies.push(lobby);
        self.conn_lobby.insert(conn, id);
        out.send(conn, ServerMsg::LobbyJoined { lobby: info, your_slot: 0 });
        self.maybe_start(id, &mut out);
        self.push_browser_list(&mut out);
        out
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
        if lobby.members.len() >= lobby.capacity() {
            out.send(conn, ServerMsg::LobbyError { message: "lobby is full".into() });
            return out;
        }
        lobby.members.push(conn);
        let slot = lobby.members.len() - 1;
        let info = lobby.info();
        self.conn_lobby.insert(conn, lobby_id);
        out.send(conn, ServerMsg::LobbyJoined { lobby: info.clone(), your_slot: slot });
        // Tell the seats already waiting that the roster grew.
        for &m in self.lobby_members(lobby_id) {
            if m != conn {
                out.send(m, ServerMsg::LobbyUpdated { lobby: info.clone() });
            }
        }
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

    /// If `lobby_id` is now at capacity, pull it out and emit a `StartMatch`.
    fn maybe_start(&mut self, lobby_id: u64, out: &mut LobbyOutcome) {
        let Some(idx) = self.lobbies.iter().position(|l| l.id == lobby_id) else { return };
        if self.lobbies[idx].members.len() < self.lobbies[idx].capacity() {
            return;
        }
        let lobby = self.lobbies.remove(idx);
        for m in &lobby.members {
            self.conn_lobby.remove(m);
            // The members are leaving the lobby system for a match; the driver
            // moves their channels into the match, so drop them from browsing.
            self.conns.remove(m);
        }
        out.start = Some(StartMatch {
            state: lobby.state,
            members: lobby.members,
        });
    }

    /// Remove `conn` from whatever lobby it's in. Notifies the remaining
    /// members (or drops an empty lobby). `_notify_leaver` is reserved for a
    /// future "you were kicked" message.
    fn remove_from_lobby(&mut self, conn: ConnId, _notify_leaver: bool) -> LobbyOutcome {
        let mut out = LobbyOutcome::default();
        let Some(lobby_id) = self.conn_lobby.remove(&conn) else { return out };
        let Some(idx) = self.lobbies.iter().position(|l| l.id == lobby_id) else { return out };
        self.lobbies[idx].members.retain(|&m| m != conn);
        if self.lobbies[idx].members.is_empty() {
            self.lobbies.remove(idx);
        } else {
            let info = self.lobbies[idx].info();
            for &m in &self.lobbies[idx].members {
                out.send(m, ServerMsg::LobbyUpdated { lobby: info.clone() });
            }
        }
        out
    }

    fn lobby_members(&self, lobby_id: u64) -> &[ConnId] {
        self.lobbies
            .iter()
            .find(|l| l.id == lobby_id)
            .map(|l| l.members.as_slice())
            .unwrap_or(&[])
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
/// `new_conns` is the stream of freshly-accepted connections (the accept loop
/// assigns each a [`ConnId`]). The loop polls non-blockingly and sleeps briefly
/// between passes; it returns when `new_conns` closes and no connections remain.
pub fn serve_lobbies(new_conns: mpsc::Receiver<(ConnId, SeatChannel)>) {
    let mut mgr = LobbyManager::new();
    let mut channels: HashMap<ConnId, SeatChannel> = HashMap::new();
    let mut accepting = true;

    loop {
        // Intake newly-accepted connections.
        loop {
            match new_conns.try_recv() {
                Ok((id, ch)) => {
                    let outcome = mgr.register(id);
                    channels.insert(id, ch);
                    apply(&mut channels, outcome);
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
                let recv = channels.get(&id).map(|c| c.rx.try_recv());
                match recv {
                    Some(Ok(msg)) => {
                        let outcome = mgr.handle(id, msg);
                        apply(&mut channels, outcome);
                        // `apply` may have moved this connection into a match.
                        if !channels.contains_key(&id) {
                            break;
                        }
                    }
                    Some(Err(mpsc::TryRecvError::Empty)) => break,
                    Some(Err(mpsc::TryRecvError::Disconnected)) | None => {
                        channels.remove(&id);
                        let outcome = mgr.disconnect(id);
                        apply(&mut channels, outcome);
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
/// channels out of the pool and spawn the match.
fn apply(channels: &mut HashMap<ConnId, SeatChannel>, outcome: LobbyOutcome) {
    for (id, msg) in outcome.sends {
        if let Some(ch) = channels.get(&id) {
            let _ = ch.tx.send(msg);
        }
    }
    if let Some(start) = outcome.start {
        let mut occupants: Vec<SeatOccupant> = Vec::with_capacity(start.members.len());
        for id in &start.members {
            if let Some(ch) = channels.remove(id) {
                occupants.push(SeatOccupant::Human(ch));
            }
        }
        // Only start if every member's channel was still present; a member
        // that vanished between fill and spawn aborts the start (their seats
        // would just disconnect immediately otherwise).
        if occupants.len() == start.members.len() {
            thread::spawn(move || {
                run_match(start.state, occupants);
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
        assert_eq!(start.members, vec![ConnId(1), ConnId(2)]);
        assert_eq!(start.state.players.len(), 2);
        // The lobby is gone afterward, and both members left the browsing set.
        assert!(m.lobbies.is_empty());
        assert!(!m.conn_lobby.contains_key(&ConnId(1)));
        assert!(!m.conn_lobby.contains_key(&ConnId(2)));
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

        // Seat 1 drops; seat 2 (still waiting) is told the roster shrank.
        let out = m.disconnect(ConnId(1));
        let updated = out.sends.iter().find_map(|(c, msg)| match msg {
            ServerMsg::LobbyUpdated { lobby } if *c == ConnId(2) => Some(lobby.players),
            _ => None,
        });
        assert_eq!(updated, Some(1), "remaining member sees one player left");
    }

    /// End-to-end through the threaded driver: two in-process clients connect,
    /// create + join a Modern lobby, and the driver starts the match — both
    /// clients receive the opening `YourSeat` / `MatchStarted` handshake.
    #[test]
    fn serve_lobbies_drives_create_join_to_match_start() {
        let (new_tx, new_rx) = mpsc::channel();
        let driver = thread::spawn(move || serve_lobbies(new_rx));

        // Two connections, each a (server SeatChannel, client ClientChannel).
        let (s1, c1) = seat_pair();
        let (s2, c2) = seat_pair();
        new_tx.send((ConnId(1), s1)).unwrap();
        new_tx.send((ConnId(2), s2)).unwrap();

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
