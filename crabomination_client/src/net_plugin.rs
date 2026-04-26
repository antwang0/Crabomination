//! Singleplayer/multiplayer network bridge.
//!
//! [`SinglePlayerPlugin`] registers network resources and a `PreUpdate`
//! polling system. The actual network session is opened by the menu module
//! on `OnEnter(AppState::InGame)`; until the user picks a mode no
//! `NetOutbox`/`NetInbox` is installed.
//!
//! # Resources provided
//!
//! | Resource | Description |
//! |---|---|
//! | [`NetOutbox`] | Send [`GameAction`]s to the server |
//! | [`NetInbox`] | Raw server messages (drained each frame by [`poll_net`]) |
//! | [`CurrentView`] | Latest per-seat [`ClientView`] from the server |
//! | [`OurSeat`] | Which seat index this client controls |
//! | [`LatestServerEvents`] | Events from the most recent server action batch |

use std::sync::{Mutex, mpsc};

use bevy::prelude::*;

use crabomination::{
    game::GameAction,
    net::{ClientMsg, ClientView, GameEventWire, ServerMsg},
};

/// Send game actions to the match server.
#[derive(Resource)]
#[allow(dead_code)]
pub struct NetOutbox(pub mpsc::Sender<ClientMsg>);

impl NetOutbox {
    pub fn submit(&self, action: GameAction) {
        let _ = self.0.send(ClientMsg::SubmitAction(action));
    }
}

/// Receive raw server messages. [`Mutex`]-wrapped because [`mpsc::Receiver`]
/// is `!Sync` and Bevy [`Resource`]s must be `Sync`.
#[derive(Resource)]
pub struct NetInbox(pub Mutex<mpsc::Receiver<ServerMsg>>);

impl NetInbox {
    pub fn drain(&self) -> Vec<ServerMsg> {
        let rx = self.0.lock().unwrap();
        std::iter::from_fn(|| rx.try_recv().ok()).collect()
    }
}

/// The latest authoritative view projected for this seat by the server.
#[derive(Resource, Default)]
pub struct CurrentView(pub Option<ClientView>);

/// Seat index assigned by the server during handshake.
#[derive(Resource, Default)]
pub struct OurSeat(pub usize);

/// Events produced by the most recent server action, cleared each frame before
/// new messages arrive. Systems that drive animations should read this once
/// per action batch (the same frame events arrive) before it is overwritten.
#[derive(Resource, Default)]
pub struct LatestServerEvents(pub Vec<GameEventWire>);

/// Whether the match server has signalled game-over.
#[derive(Resource, Default)]
pub struct MatchEnded(pub Option<Option<usize>>);

/// Registers network resources and the polling + startup systems.
pub struct SinglePlayerPlugin;

impl Plugin for SinglePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentView>()
            .init_resource::<OurSeat>()
            .init_resource::<LatestServerEvents>()
            .init_resource::<MatchEnded>()
            .add_systems(PreUpdate, poll_net);
        // Network installation happens via `crate::menu::start_net_session_from_menu`
        // on entry to `AppState::InGame` — see `main.rs` wiring.
    }
}

/// Drain the inbox each pre-update tick. Applies `YourSeat`, `View`, and
/// `Events` messages to their respective resources; logs `ActionError`s.
pub fn poll_net(
    inbox: Option<Res<NetInbox>>,
    mut view: ResMut<CurrentView>,
    mut seat: ResMut<OurSeat>,
    mut events: ResMut<LatestServerEvents>,
    mut ended: ResMut<MatchEnded>,
) {
    let Some(inbox) = inbox else { return };
    events.0.clear();
    for msg in inbox.drain() {
        match msg {
            ServerMsg::YourSeat(s) => seat.0 = s,
            ServerMsg::MatchStarted => {}
            ServerMsg::View(v) => view.0 = Some(v),
            ServerMsg::Events(evs) => events.0 = evs,
            ServerMsg::ActionError(e) => eprintln!("net: server rejected action: {e}"),
            ServerMsg::MatchOver { winner } => ended.0 = Some(winner),
        }
    }
}
