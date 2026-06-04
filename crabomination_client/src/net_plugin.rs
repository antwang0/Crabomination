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
    net::{ClientMsg, ClientView, DebugAction, GameEventWire, ServerMsg},
};

/// Send game actions to the match server. Also remembers the most
/// recent *cast* action so the manual-mana-tap flow can re-arm and
/// re-submit it once the player taps enough mana (see `poll_net` /
/// `drive_pending_mana_cast`).
#[derive(Resource)]
#[allow(dead_code)]
pub struct NetOutbox(pub mpsc::Sender<ClientMsg>, Mutex<Option<GameAction>>);

/// True for the player-initiated cast actions that go through the
/// engine's forced-only mana payment (and can therefore come back as
/// `ManualTapRequired`).
fn is_cast_action(a: &GameAction) -> bool {
    matches!(
        a,
        GameAction::CastSpell { .. }
            | GameAction::CastSpellBack { .. }
            | GameAction::CastSpellDelve { .. }
            | GameAction::CastSpellAlternative { .. }
            | GameAction::CastFromCommandZone { .. }
    )
}

impl NetOutbox {
    pub fn new(tx: mpsc::Sender<ClientMsg>) -> Self {
        Self(tx, Mutex::new(None))
    }

    pub fn submit(&self, action: GameAction) {
        if is_cast_action(&action)
            && let Ok(mut last) = self.1.lock()
        {
            *last = Some(action.clone());
        }
        let _ = self.0.send(ClientMsg::SubmitAction(action));
    }

    /// The most recent cast action submitted — used to re-arm a cast the
    /// engine rejected pending manual mana tapping.
    pub fn last_cast(&self) -> Option<GameAction> {
        self.1.lock().ok().and_then(|g| g.clone())
    }

    /// Send a debug-console cheat. The server applies it to whichever
    /// seat owns this channel.
    pub fn submit_debug(&self, action: DebugAction) {
        let _ = self.0.send(ClientMsg::Debug(action));
    }
}

/// A cast the engine rejected with `ManualTapRequired`: the player has a
/// choice of which mana sources to tap, so we hold the (fully-formed,
/// already-targeted) cast action and re-submit it each time the player
/// taps another source — the engine accepts as soon as the pool covers
/// the cost. Cancelled with Escape.
pub struct PendingCast {
    pub action: GameAction,
    /// The player's mana-pool total when we last (re-)submitted; a change
    /// means they tapped/added a source, so we try the cast again.
    pub last_pool_total: u32,
    /// Human-readable hint (the engine's message, carrying the cost) shown
    /// in the on-screen banner.
    pub hint: String,
}

#[derive(Resource, Default)]
pub struct PendingManaCast(pub Option<PendingCast>);

/// Marker substring of `GameError::ManualTapRequired`'s message. Kept in
/// sync with `crabomination::game::GameError::ManualTapRequired`.
const MANUAL_TAP_MARKER: &str = "Tap mana to pay";

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

/// Client-side mirror of the server's lobby protocol, kept current by
/// [`poll_net`]. The lobby-browser UI renders from this; cleared between
/// sessions by [`teardown_net_session`].
#[derive(Resource, Default)]
pub struct LobbyState {
    /// Latest advertised open lobbies (from `ServerMsg::LobbyList`).
    pub lobbies: Vec<crabomination::net::LobbyInfo>,
    /// The lobby we've created/joined and are waiting in, with our slot.
    pub joined: Option<(crabomination::net::LobbyInfo, usize)>,
    /// Most recent lobby error, for display in the browser.
    pub last_error: Option<String>,
    /// Set once `MatchStarted` arrives so the browser can hand off to InGame.
    pub match_started: bool,
}

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

/// Live TCP socket handle for a networked match, kept so leaving a game
/// can `shutdown` it immediately rather than waiting for the ~2-minute
/// keepalive timeout to reap a half-open connection. `None` for
/// in-process matches (vs-bot, host, spectate), where dropping
/// [`NetOutbox`] already tears the channel down.
#[derive(Resource, Default)]
pub struct NetConnection(pub Option<std::net::TcpStream>);

/// Registers network resources and the polling + startup systems.
pub struct SinglePlayerPlugin;

impl Plugin for SinglePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentView>()
            .init_resource::<OurSeat>()
            .init_resource::<LatestServerEvents>()
            .init_resource::<MatchEnded>()
            .init_resource::<NetConnection>()
            .init_resource::<PendingManaCast>()
            .init_resource::<LobbyState>()
            .add_systems(PreUpdate, poll_net)
            .add_systems(Update, (drive_pending_mana_cast, update_pending_cast_banner));
        // Network installation happens via `crate::menu::start_net_session_from_menu`
        // on entry to `AppState::InGame` — see `main.rs` wiring.
    }
}

/// Drain the inbox each pre-update tick. Applies `YourSeat`, `View`, and
/// `Events` messages to their respective resources; logs `ActionError`s.
#[allow(clippy::too_many_arguments)]
pub fn poll_net(
    inbox: Option<Res<NetInbox>>,
    outbox: Option<Res<NetOutbox>>,
    mut view: ResMut<CurrentView>,
    mut seat: ResMut<OurSeat>,
    mut events: ResMut<LatestServerEvents>,
    mut ended: ResMut<MatchEnded>,
    mut pending_cast: ResMut<PendingManaCast>,
    mut lobby: ResMut<LobbyState>,
) {
    let Some(inbox) = inbox else { return };
    events.0.clear();
    for msg in inbox.drain() {
        match msg {
            ServerMsg::YourSeat(s) => seat.0 = s,
            // The match is starting — the lobby browser uses this to leave the
            // browser and enter the game.
            ServerMsg::MatchStarted => lobby.match_started = true,
            ServerMsg::View(v) => view.0 = Some(*v),
            ServerMsg::Events(evs) => events.0 = evs,
            // Combined per-action frame: apply the events (for animation)
            // and the post-action view together.
            ServerMsg::Update { events: evs, view: v } => {
                events.0 = evs;
                view.0 = Some(*v);
            }
            ServerMsg::ActionError(e) => {
                // `ManualTapRequired`: the player has a choice of which mana
                // to tap. Arm a pending cast that re-fires once they tap
                // enough — rather than just dropping the action on the floor.
                if e.contains(MANUAL_TAP_MARKER) {
                    if let Some(outbox) = &outbox
                        && let Some(action) = outbox.last_cast()
                    {
                        let total = view
                            .0
                            .as_ref()
                            .and_then(|cv| cv.players.iter().find(|p| p.seat == cv.your_seat))
                            .map(|p| p.mana_pool.total())
                            .unwrap_or(0);
                        pending_cast.0 = Some(PendingCast { action, last_pool_total: total, hint: e });
                    }
                } else {
                    eprintln!("net: server rejected action: {e}");
                }
            }
            ServerMsg::MatchOver { winner } => ended.0 = Some(winner),
            // ── Lobby protocol → client-side mirror (rendered by the lobby
            //    browser UI) ────────────────────────────────────────────────
            ServerMsg::LobbyList { lobbies } => {
                lobby.lobbies = lobbies;
                lobby.last_error = None;
            }
            ServerMsg::LobbyJoined { lobby: info, your_slot } => {
                lobby.joined = Some((info, your_slot));
                lobby.last_error = None;
            }
            ServerMsg::LobbyUpdated { lobby: info } => {
                let slot = lobby.joined.as_ref().map(|(_, s)| *s).unwrap_or(0);
                lobby.joined = Some((info, slot));
            }
            ServerMsg::LobbyError { message } => {
                eprintln!("lobby: {message}");
                lobby.last_error = Some(message);
            }
        }
    }
}

/// `OnExit(AppState::InGame)` — tear down the live network session so
/// leaving a match (via the settings menu, the game-over screen, or a
/// rematch into a different mode) actually disconnects: shut the TCP
/// socket down if one is open, drop the channel + snapshot resources,
/// and clear the cached view so the next match starts from a clean
/// slate. In-process matches have no socket — dropping [`NetOutbox`]
/// disconnects the seat channel, which lets the server-side match
/// thread observe the drop and exit.
pub fn teardown_net_session(
    mut commands: Commands,
    mut conn: ResMut<NetConnection>,
    mut view: ResMut<CurrentView>,
    mut ended: ResMut<MatchEnded>,
    mut pending_cast: ResMut<PendingManaCast>,
    mut lobby: ResMut<LobbyState>,
) {
    if let Some(stream) = conn.0.take() {
        let _ = stream.shutdown(std::net::Shutdown::Both);
    }
    commands.remove_resource::<NetOutbox>();
    commands.remove_resource::<NetInbox>();
    commands.remove_resource::<crate::menu::LatestSnapshot>();
    view.0 = None;
    ended.0 = None;
    pending_cast.0 = None;
    *lobby = LobbyState::default();
}

/// Drive a `PendingCast`: re-submit the held cast each time the player's
/// mana pool changes (they tapped/added a source), so the engine accepts
/// it as soon as the pool covers the cost. Clears when the card leaves
/// the castable zone (it resolved or moved) or the player presses Escape.
pub fn drive_pending_mana_cast(
    mut pending: ResMut<PendingManaCast>,
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    keys: Res<bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>>,
) {
    if pending.0.is_none() {
        return;
    }
    if keys.just_pressed(bevy::input::keyboard::KeyCode::Escape) {
        pending.0 = None;
        return;
    }
    let Some(outbox) = outbox else {
        pending.0 = None;
        return;
    };
    // No live view (between matches) → drop any stale pending cast.
    let Some(cv) = &view.0 else {
        pending.0 = None;
        return;
    };
    let Some(pc) = pending.0.as_mut() else { return };

    let card_id = cast_action_card_id(&pc.action);
    let Some(me) = cv.players.iter().find(|p| p.seat == cv.your_seat) else { return };
    // Still castable? (in hand or the command zone). If not, it resolved or
    // moved — drop the pending cast.
    let present = me.hand.iter().any(|h| h.id() == card_id)
        || me.command.iter().any(|h| h.id() == card_id);
    if !present {
        pending.0 = None;
        return;
    }
    // Re-attempt only when the pool changed (the player tapped a source) —
    // otherwise we'd spam the server every frame.
    let total = me.mana_pool.total();
    if total != pc.last_pool_total {
        pc.last_pool_total = total;
        outbox.submit(pc.action.clone());
    }
}

/// Marker for the on-screen "tap mana to pay …" banner.
#[derive(Component)]
struct PendingCastBanner;

/// Show a top-of-screen banner while a cast is waiting on manual mana
/// tapping, so the player knows to tap their sources (or press Escape).
fn update_pending_cast_banner(
    mut commands: Commands,
    pending: Res<PendingManaCast>,
    fonts: Option<Res<crate::theme::UiFonts>>,
    existing: Query<Entity, With<PendingCastBanner>>,
) {
    match (&pending.0, existing.iter().next()) {
        (Some(pc), None) => {
            let Some(fonts) = fonts else { return };
            let label = format!("{} — tap mana sources, or Esc to cancel", pc.hint);
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(64.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    PendingCastBanner,
                    crate::systems::game_ui::InGameRoot,
                    Pickable::IGNORE,
                    GlobalZIndex(40),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(label),
                        fonts.tf(16.0),
                        TextColor(crate::theme::ACCENT_GOLD),
                        BackgroundColor(Color::srgba(0.04, 0.06, 0.12, 0.92)),
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
        (None, Some(e)) => {
            commands.entity(e).despawn();
        }
        _ => {}
    }
}

/// The card id a cast action targets, for tracking whether a pending cast
/// is still castable.
fn cast_action_card_id(action: &GameAction) -> crabomination::card::CardId {
    match action {
        GameAction::CastSpell { card_id, .. }
        | GameAction::CastSpellBack { card_id, .. }
        | GameAction::CastSpellDelve { card_id, .. }
        | GameAction::CastSpellAlternative { card_id, .. }
        | GameAction::CastFromCommandZone { card_id, .. } => *card_id,
        // Non-cast actions never arm a pending cast; return a sentinel that
        // won't match any real card so the pending cast clears.
        _ => crabomination::card::CardId(u32::MAX),
    }
}
