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

use std::net::TcpStream;
use std::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};

use bevy::prelude::*;

use crabomination::{
    game::GameAction,
    net::{ClientMsg, ClientView, DebugAction, GameEventWire, ServerMsg},
    server::{tcp_client, ClientChannel},
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
            | GameAction::CastPrepareSpell { .. }
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

    /// Send a raw `ClientMsg` — used for lobby commands (list / create / join
    /// / leave), which aren't game actions.
    pub fn submit_msg(&self, msg: ClientMsg) {
        let _ = self.0.send(msg);
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
    /// Drain all pending messages, reporting whether the channel has
    /// disconnected (the reader thread exited because the socket closed). The
    /// flag drives mid-match reconnection.
    pub fn drain(&self) -> (Vec<ServerMsg>, bool) {
        let rx = self.0.lock().unwrap();
        let mut msgs = Vec::new();
        let disconnected = loop {
            match rx.try_recv() {
                Ok(m) => msgs.push(m),
                Err(mpsc::TryRecvError::Empty) => break false,
                Err(mpsc::TryRecvError::Disconnected) => break true,
            }
        };
        (msgs, disconnected)
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
    /// Latest advertised in-progress matches available to spectate (from
    /// `ServerMsg::SpectatableList`).
    pub spectatable: Vec<crabomination::net::SpectatableInfo>,
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

/// State for reconnecting to a dropped lobby match. The server issues a
/// `ResumeToken` at match start; if the connection then drops mid-match,
/// `maybe_reconnect` opens a fresh connection to `server_addr` and re-claims
/// the seat with `Resume { token }`. `None` token ⇒ not a reconnectable match
/// (in-process / spectate), so a drop is treated as a normal end.
#[derive(Resource, Default)]
pub struct ResumeInfo {
    pub token: Option<String>,
    pub server_addr: Option<String>,
    /// Set by `poll_net` when the connection drops; cleared on a successful
    /// reconnect attempt.
    pub lost: bool,
    /// Consecutive failed reconnect attempts; reset once messages flow again.
    pub attempts: u32,
    pub last_attempt: Option<std::time::Instant>,
}

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
            .init_resource::<ResumeInfo>()
            .init_resource::<RopeClock>()
            .add_systems(PreUpdate, poll_net)
            .add_systems(
                Update,
                (
                    drive_pending_mana_cast,
                    update_pending_cast_banner,
                    update_spectator_banner,
                    update_reconnect_banner,
                    update_rope_banner,
                ),
            )
            // Reconnect runs only when a reconnectable match's link has dropped.
            .add_systems(Update, maybe_reconnect.run_if(|r: Res<ResumeInfo>| r.lost));
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
    mut resume: ResMut<ResumeInfo>,
    mut rope: ResMut<RopeClock>,
    time: Res<Time>,
) {
    let Some(inbox) = inbox else { return };
    events.0.clear();
    let (msgs, disconnected) = inbox.drain();
    // Set when a game-state message arrives, so a (re)established link resets
    // the reconnect backoff — but a bare `LobbyError` (a rejected resume) does
    // not, letting `maybe_reconnect` exhaust its attempts and bail to the menu.
    let mut got_game_msg = false;
    for msg in msgs {
        match msg {
            ServerMsg::YourSeat(s) => {
                seat.0 = s;
                got_game_msg = true;
            }
            // The match is starting — the lobby browser uses this to leave the
            // browser and enter the game.
            ServerMsg::MatchStarted => {
                lobby.match_started = true;
                got_game_msg = true;
            }
            ServerMsg::View(v) => {
                view.0 = Some(*v);
                got_game_msg = true;
                // Any accepted action resets the server's rope; a fresh
                // `Rope` follows if a clock is still running for us.
                rope.deadline = None;
            }
            ServerMsg::Events(evs) => events.0 = evs,
            // Combined per-action frame: apply the events (for animation)
            // and the post-action view together.
            ServerMsg::Update { events: evs, view: v } => {
                events.0 = evs;
                view.0 = Some(*v);
                got_game_msg = true;
                rope.deadline = None;
            }
            // The per-action rope armed for this seat — start the local
            // countdown (rendered by `update_rope_banner`).
            ServerMsg::Rope { seconds } => {
                rope.deadline = Some(time.elapsed_secs_f64() + seconds as f64);
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
                } else if e.contains("action timeout") {
                    // The rope fired and the server acted for us — show a
                    // transient notice instead of a silent log line.
                    rope.fired_until = Some(time.elapsed_secs_f64() + 4.0);
                } else {
                    eprintln!("net: server rejected action: {e}");
                }
            }
            ServerMsg::MatchOver { winner } => {
                ended.0 = Some(winner);
                // Game's over — don't try to reconnect when the socket closes.
                resume.token = None;
            }
            // Reconnect: stash the token so a mid-match drop can re-claim the seat.
            ServerMsg::ResumeToken { token } => resume.token = Some(token),
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
            ServerMsg::SpectatableList { matches } => {
                lobby.spectatable = matches;
            }
        }
    }
    if got_game_msg {
        resume.attempts = 0;
        resume.lost = false;
    }
    // The reader thread exited (socket closed). If this is a reconnectable
    // match (we hold a resume token), flag it for `maybe_reconnect`.
    if disconnected && resume.token.is_some() {
        resume.lost = true;
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
    mut resume: ResMut<ResumeInfo>,
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
    *resume = ResumeInfo::default();
}

/// How long to wait between reconnect attempts, and how many to make before
/// giving up and returning to the menu.
const RECONNECT_RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_RECONNECT_ATTEMPTS: u32 = 8;

/// Mid-match reconnect: when a reconnectable match's link drops (`ResumeInfo.
/// lost`), open a fresh connection to the server and re-claim the seat with
/// the stored resume token. Backs off between tries and, after
/// `MAX_RECONNECT_ATTEMPTS`, gives up and returns to the menu. Runs only while
/// `lost` is set (see the run condition on registration).
pub fn maybe_reconnect(world: &mut World) {
    let (token, addr, attempts, last) = {
        let r = world.resource::<ResumeInfo>();
        (r.token.clone(), r.server_addr.clone(), r.attempts, r.last_attempt)
    };
    let (Some(token), Some(addr)) = (token, addr) else {
        world.resource_mut::<ResumeInfo>().lost = false;
        return;
    };

    let now = Instant::now();
    if let Some(last) = last
        && now.duration_since(last) < RECONNECT_RETRY_DELAY
    {
        return; // still backing off
    }

    if attempts >= MAX_RECONNECT_ATTEMPTS {
        eprintln!("reconnect: gave up after {attempts} attempts — returning to menu");
        {
            let mut r = world.resource_mut::<ResumeInfo>();
            r.lost = false;
            r.token = None;
        }
        if let Some(mut ns) = world.get_resource_mut::<NextState<crate::menu::AppState>>() {
            ns.set(crate::menu::AppState::Menu);
        }
        return;
    }

    eprintln!("reconnect: attempt {} to {addr}…", attempts + 1);
    {
        let mut r = world.resource_mut::<ResumeInfo>();
        r.attempts += 1;
        r.last_attempt = Some(now);
        // Clear the flag for this attempt; `poll_net` re-sets it if the new
        // link also drops (or never delivers a game message).
        r.lost = false;
    }

    match reconnect_with_token(&addr, &token) {
        Ok((outbox, inbox, conn)) => {
            world.insert_resource(outbox);
            world.insert_resource(inbox);
            world.insert_resource(conn);
        }
        Err(e) => {
            eprintln!("reconnect: connect failed: {e}");
            // Retry after the backoff delay.
            world.resource_mut::<ResumeInfo>().lost = true;
        }
    }
}

/// Open a fresh connection and immediately send `Resume { token }`.
fn reconnect_with_token(
    addr: &str,
    token: &str,
) -> std::io::Result<(NetOutbox, NetInbox, NetConnection)> {
    let stream = TcpStream::connect(addr)?;
    let conn_handle = stream.try_clone().ok();
    let ClientChannel { tx, rx } = tcp_client(stream)?;
    let _ = tx.send(ClientMsg::Resume { token: token.to_string() });
    Ok((NetOutbox::new(tx), NetInbox(Mutex::new(rx)), NetConnection(conn_handle)))
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
/// "{1}{U} to go" — the part of `cost` the viewer's current pool doesn't
/// cover yet, for the manual-tap banner. Display-only approximation
/// (hybrid/Phyrexian/snow pips count as their colored side; X as 0): the
/// engine remains authoritative about acceptance — this just tells the
/// player roughly what's still missing as they tap.
fn remaining_cost_label(
    cost: &crabomination::mana::ManaCost,
    pool: &crabomination::mana::ManaPool,
) -> Option<String> {
    use crabomination::mana::{Color as MC, ManaSymbol};
    const COLORS: [(MC, char); 5] = [
        (MC::White, 'W'),
        (MC::Blue, 'U'),
        (MC::Black, 'B'),
        (MC::Red, 'R'),
        (MC::Green, 'G'),
    ];
    let mut pool_left: Vec<(MC, u32)> = COLORS.iter().map(|(c, _)| (*c, pool.amount(*c))).collect();
    let colored_total: u32 = pool_left.iter().map(|(_, n)| n).sum();
    let other_pool = pool.total().saturating_sub(colored_total); // colorless & friends
    let mut need: Vec<(char, u32)> = Vec::new(); // colored pips still missing
    let mut generic = 0u32;
    for sym in &cost.symbols {
        let colored = match sym {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => Some(*c),
            ManaSymbol::Hybrid(a, _) => Some(*a),
            ManaSymbol::MonoHybrid(_, c) => Some(*c),
            ManaSymbol::Generic(n) | ManaSymbol::Colorless(n) => {
                generic += n;
                None
            }
            ManaSymbol::Snow => {
                generic += 1;
                None
            }
            ManaSymbol::X => None,
        };
        if let Some(c) = colored {
            let slot = pool_left.iter_mut().find(|(pc, _)| *pc == c).expect("five colors");
            if slot.1 > 0 {
                slot.1 -= 1;
            } else {
                let letter = COLORS.iter().find(|(pc, _)| *pc == c).expect("five colors").1;
                match need.iter_mut().find(|(l, _)| *l == letter) {
                    Some(e) => e.1 += 1,
                    None => need.push((letter, 1)),
                }
            }
        }
    }
    // Whatever pool remains (colored leftovers + colorless) covers generic.
    let leftover: u32 = pool_left.iter().map(|(_, n)| n).sum::<u32>() + other_pool;
    let generic_missing = generic.saturating_sub(leftover);
    if need.is_empty() && generic_missing == 0 {
        return None;
    }
    let mut out = String::new();
    if generic_missing > 0 {
        out.push_str(&format!("{{{generic_missing}}}"));
    }
    for (letter, n) in need {
        for _ in 0..n {
            out.push_str(&format!("{{{letter}}}"));
        }
    }
    Some(out)
}

/// Marker for the banner's text node so the remaining-cost readout can be
/// updated in place as the player taps sources.
#[derive(Component)]
struct PendingCastBannerText;

fn pending_cast_banner_label(
    pc: &PendingCast,
    view: &CurrentView,
    card_names: &crate::game::CardNames,
) -> String {
    let remaining = view.0.as_ref().and_then(|cv| {
        let me = cv.players.iter().find(|p| p.seat == cv.your_seat)?;
        let name = card_names.get(cast_action_card_id(&pc.action));
        let def = crabomination::catalog::lookup_by_name(&name)?;
        remaining_cost_label(&def.cost, &me.mana_pool)
    });
    match remaining {
        Some(missing) => {
            format!("{} — {missing} to go · tap mana sources, or Esc to cancel", pc.hint)
        }
        None => format!("{} — tap mana sources, or Esc to cancel", pc.hint),
    }
}

fn update_pending_cast_banner(
    mut commands: Commands,
    pending: Res<PendingManaCast>,
    fonts: Option<Res<crate::theme::UiFonts>>,
    view: Res<CurrentView>,
    card_names: Res<crate::game::CardNames>,
    existing: Query<Entity, With<PendingCastBanner>>,
    mut text_q: Query<&mut Text, With<PendingCastBannerText>>,
) {
    match (&pending.0, existing.iter().next()) {
        (Some(pc), Some(_)) => {
            // Live update: the remaining-cost readout shrinks as sources tap.
            let label = pending_cast_banner_label(pc, &view, &card_names);
            for mut text in &mut text_q {
                if text.0 != label {
                    text.0 = label.clone();
                }
            }
        }
        (Some(pc), None) => {
            let Some(fonts) = fonts else { return };
            let label = pending_cast_banner_label(pc, &view, &card_names);
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
                        PendingCastBannerText,
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

/// Marker for the persistent "👁 Spectating" banner shown to read-only
/// spectators.
#[derive(Component)]
struct SpectatorBanner;

/// Show a banner while this client is spectating a match (its seat is the
/// [`crabomination::net::SPECTATOR_SEAT`] sentinel) and a live view is
/// present. Mirrors `update_pending_cast_banner`: spawn when the condition
/// holds, despawn when it clears (match left / ended → `CurrentView` cleared).
fn update_spectator_banner(
    mut commands: Commands,
    seat: Res<OurSeat>,
    view: Res<CurrentView>,
    fonts: Option<Res<crate::theme::UiFonts>>,
    existing: Query<Entity, With<SpectatorBanner>>,
) {
    let spectating = seat.0 == crabomination::net::SPECTATOR_SEAT && view.0.is_some();
    match (spectating, existing.iter().next()) {
        (true, None) => {
            let Some(fonts) = fonts else { return };
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(8.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    SpectatorBanner,
                    crate::systems::game_ui::InGameRoot,
                    Pickable::IGNORE,
                    GlobalZIndex(40),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new("👁 Spectating — read only"),
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
        (false, Some(e)) => {
            commands.entity(e).despawn();
        }
        _ => {}
    }
}

/// Marker for the "connection lost — reconnecting" banner shown while
/// `maybe_reconnect` is retrying a dropped match link.
#[derive(Component)]
struct ReconnectBanner;

/// Marker for the banner's text node so the attempt counter can be updated
/// in place between retries.
#[derive(Component)]
struct ReconnectBannerText;

/// Surface the mid-match reconnect loop (`maybe_reconnect`) to the player.
/// Without this the board simply freezes while the background retries run.
/// Shown while a reconnectable match (`ResumeInfo.token` held) is either
/// flagged lost or mid-retry (`attempts` only resets once messages flow
/// again — see `poll_net`); despawned on recovery or once the loop gives
/// up and bails to the menu.
fn update_reconnect_banner(
    mut commands: Commands,
    resume: Res<ResumeInfo>,
    fonts: Option<Res<crate::theme::UiFonts>>,
    existing: Query<Entity, With<ReconnectBanner>>,
    mut text_q: Query<&mut Text, With<ReconnectBannerText>>,
) {
    let reconnecting = resume.token.is_some() && (resume.lost || resume.attempts > 0);
    let label = format!(
        "⟳ Connection lost — reconnecting (attempt {} of {MAX_RECONNECT_ATTEMPTS})…",
        resume.attempts.max(1)
    );
    match (reconnecting, existing.iter().next()) {
        (true, None) => {
            let Some(fonts) = fonts else { return };
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(48.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ReconnectBanner,
                    crate::systems::game_ui::InGameRoot,
                    Pickable::IGNORE,
                    GlobalZIndex(45),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(label),
                        ReconnectBannerText,
                        fonts.tf(16.0),
                        TextColor(crate::theme::ACCENT_ORANGE),
                        BackgroundColor(crate::theme::HUD_BG_DANGER),
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
        (true, Some(_)) => {
            for mut text in &mut text_q {
                if text.0 != label {
                    text.0 = label.clone();
                }
            }
        }
        (false, Some(e)) => {
            commands.entity(e).despawn();
        }
        _ => {}
    }
}

/// Local mirror of the server's per-action rope (`ServerMsg::Rope`):
/// `deadline` is in `Time::elapsed_secs_f64` terms; `fired_until` shows the
/// "server acted for you" notice briefly after the rope expires.
#[derive(Resource, Default)]
pub struct RopeClock {
    pub deadline: Option<f64>,
    pub fired_until: Option<f64>,
}

/// Marker for the rope-countdown banner.
#[derive(Component)]
struct RopeBanner;

/// Marker for the banner's text node (updated in place each tick).
#[derive(Component)]
struct RopeBannerText;

/// Countdown toast for the per-action rope: appears when ≤ 15s remain
/// ("act or the server acts for you"), and shows a short "time's up"
/// notice after the rope fires.
fn update_rope_banner(
    mut commands: Commands,
    rope: Res<RopeClock>,
    time: Res<Time>,
    fonts: Option<Res<crate::theme::UiFonts>>,
    existing: Query<Entity, With<RopeBanner>>,
    mut text_q: Query<&mut Text, With<RopeBannerText>>,
) {
    let now = time.elapsed_secs_f64();
    let label = if rope.fired_until.is_some_and(|u| now < u) {
        Some("⏱ Time's up — the server acted for you".to_string())
    } else {
        rope.deadline
            .map(|d| d - now)
            .filter(|&rem| rem > 0.0 && rem <= 15.0)
            .map(|rem| format!("⏱ Auto-act in {}s", rem.ceil() as u32))
    };
    match (label, existing.iter().next()) {
        (Some(label), None) => {
            let Some(fonts) = fonts else { return };
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(86.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    RopeBanner,
                    crate::systems::game_ui::InGameRoot,
                    Pickable::IGNORE,
                    GlobalZIndex(45),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(label),
                        RopeBannerText,
                        fonts.tf(16.0),
                        TextColor(crate::theme::ACCENT_ORANGE),
                        BackgroundColor(crate::theme::HUD_BG_DANGER),
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
        (Some(label), Some(_)) => {
            for mut text in &mut text_q {
                if text.0 != label {
                    text.0 = label.clone();
                }
            }
        }
        (None, Some(e)) => {
            commands.entity(e).despawn();
        }
        _ => {}
    }
}

/// The card id a cast action targets, for tracking whether a pending cast
/// is still castable.
pub fn cast_action_card_id(action: &GameAction) -> crabomination::card::CardId {
    match action {
        GameAction::CastSpell { card_id, .. }
        | GameAction::CastSpellBack { card_id, .. }
        | GameAction::CastSpellDelve { card_id, .. }
        | GameAction::CastSpellAlternative { card_id, .. }
        | GameAction::CastFromCommandZone { card_id, .. } => *card_id,
        // The pending-cast tracker keys off the prepared creature — it's
        // the persistent object the re-armed cast references.
        GameAction::CastPrepareSpell { creature_id, .. } => *creature_id,
        // Non-cast actions never arm a pending cast; return a sentinel that
        // won't match any real card so the pending cast clears.
        _ => crabomination::card::CardId(u32::MAX),
    }
}
