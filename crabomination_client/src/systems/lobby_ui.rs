//! Lobby browser screen (`AppState::Lobby`).
//!
//! Entered after the user picks "Join LAN": on enter we connect to the lobby
//! server and install the net session; the browser then renders
//! [`LobbyState`] (kept current by `poll_net`) and turns clicks into lobby
//! [`ClientMsg`]s. When the server starts the match (`LobbyState.match_started`)
//! we hand off to `AppState::InGame` — the connection is already live, so the
//! in-game setup is a no-op (see `menu::start_net_session_from_menu`).

use std::sync::Mutex;

use bevy::prelude::*;

use crabomination::net::ClientMsg;
use crabomination::server::{tcp_client, ClientChannel};

use crate::menu::{AppState, MatchFormat, PendingLobbyServer};
use crate::net_plugin::{LobbyState, NetConnection, NetInbox, NetOutbox};
use crate::theme::{self, HoverTint, UiFonts};

pub struct LobbyUiPlugin;

impl Plugin for LobbyUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LobbyCreateFormat>()
            .add_systems(OnEnter(AppState::Lobby), (connect_to_lobby_server, spawn_lobby_browser))
            .add_systems(OnExit(AppState::Lobby), despawn_lobby_browser)
            .add_systems(
                Update,
                (
                    rebuild_lobby_list,
                    update_lobby_status,
                    update_format_label,
                    update_bot_controls_visibility,
                    handle_lobby_buttons,
                    watch_match_start,
                )
                    .run_if(in_state(AppState::Lobby)),
            );
    }
}

// ── Components / resources ─────────────────────────────────────────────────────

#[derive(Component)]
struct LobbyRoot;
/// Container whose children are the per-lobby rows, rebuilt when the list
/// changes.
#[derive(Component)]
struct LobbyListPanel;
/// One rebuilt list row (tagged so the whole set can be cleared on refresh).
#[derive(Component)]
struct LobbyListRow;
/// Join button carrying the lobby id it joins.
#[derive(Component)]
struct LobbyRowJoinButton(u64);
#[derive(Component)]
struct LobbyCreateButton;
#[derive(Component)]
struct LobbyFormatCycleButton;
#[derive(Component)]
struct LobbyFormatLabel;
#[derive(Component)]
struct LobbyRefreshButton;
#[derive(Component)]
struct LobbyBackButton;
#[derive(Component)]
struct LobbyStatusText;
/// Row holding the host-only controls (add/remove bot, start); shown only to
/// the lobby host (seat 0).
#[derive(Component)]
struct LobbyBotControls;
#[derive(Component)]
struct LobbyAddBotButton;
#[derive(Component)]
struct LobbyRemoveBotButton;
#[derive(Component)]
struct LobbyStartButton;

/// The gamemode the Create button will use; cycled by the format button.
#[derive(Resource, Default)]
struct LobbyCreateFormat(MatchFormat);

// ── Connect on enter ───────────────────────────────────────────────────────────

/// Connect to the queued lobby server and install the net session, then greet
/// the server and ask for the lobby list. Bounces back to the menu on failure.
pub fn connect_to_lobby_server(world: &mut World) {
    let req = world
        .get_resource_mut::<PendingLobbyServer>()
        .and_then(|mut r| r.0.take());
    let Some(req) = req else {
        eprintln!("lobby: no server address queued");
        return;
    };

    match connect(&req.addr) {
        Ok((outbox, inbox, conn)) => {
            outbox.submit_msg(ClientMsg::JoinMatch { name: req.name.clone() });
            outbox.submit_msg(ClientMsg::ListLobbies);
            world.insert_resource(outbox);
            world.insert_resource(inbox);
            world.insert_resource(conn);
            // Remember the server so a dropped lobby match can reconnect.
            if let Some(mut r) = world.get_resource_mut::<crate::net_plugin::ResumeInfo>() {
                r.server_addr = Some(req.addr.clone());
            }
            eprintln!("lobby: connected to {} as \"{}\"", req.addr, req.name);
        }
        Err(e) => {
            eprintln!("lobby: connect {} failed: {e}", req.addr);
            if let Some(mut ns) = world.get_resource_mut::<NextState<AppState>>() {
                ns.set(AppState::Menu);
            }
        }
    }
}

fn connect(addr: &str) -> std::io::Result<(NetOutbox, NetInbox, NetConnection)> {
    let stream = std::net::TcpStream::connect(addr)?;
    // Keep a clone so "Back" can shut the socket promptly (otherwise the
    // server only notices via keepalive).
    let conn_handle = stream.try_clone().ok();
    let ClientChannel { tx, rx } = tcp_client(stream)?;
    Ok((NetOutbox::new(tx), NetInbox(Mutex::new(rx)), NetConnection(conn_handle)))
}

// ── UI spawn / despawn ─────────────────────────────────────────────────────────

fn spawn_lobby_browser(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    create_format: Res<LobbyCreateFormat>,
) {
    let tf = |size: f32| ui_fonts.tf(size);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            LobbyRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(14.0),
                    align_items: AlignItems::Stretch,
                    min_width: Val::Px(460.0),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("Lobbies"),
                    tf(26.0),
                    TextColor(theme::ACCENT_GOLD),
                ));

                p.spawn((
                    Text::new("Connecting…"),
                    tf(13.0),
                    TextColor(theme::TEXT_SECONDARY),
                    LobbyStatusText,
                ));

                // Scrollable-ish list container (children rebuilt on update).
                p.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        min_height: Val::Px(120.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::HUD_BG),
                    LobbyListPanel,
                ));

                // Create row: [Format: X] [Create Lobby].
                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    button(row, &tf, theme::FIELD_BG, LobbyFormatCycleButton)
                        .with_children(|b| {
                            b.spawn((
                                Text::new(format!("Format: {}", create_format.0.label())),
                                tf(13.0),
                                TextColor(theme::TEXT_PRIMARY),
                                LobbyFormatLabel,
                            ));
                        });
                    button(row, &tf, theme::BUTTON_PRIMARY_BG, LobbyCreateButton)
                        .with_children(|b| {
                            b.spawn((Text::new("Create Lobby"), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                });

                // Host controls — only shown to the lobby host (toggled by
                // `update_bot_controls_visibility`).
                p.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        align_items: AlignItems::Center,
                        display: Display::None,
                        ..default()
                    },
                    LobbyBotControls,
                ))
                .with_children(|row| {
                    button(row, &tf, theme::BUTTON_INFO_BG, LobbyAddBotButton)
                        .with_children(|b| {
                            b.spawn((Text::new("Add Bot"), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                    button(row, &tf, theme::BUTTON_NEUTRAL_BG, LobbyRemoveBotButton)
                        .with_children(|b| {
                            b.spawn((Text::new("Remove Bot"), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                    button(row, &tf, theme::BUTTON_PRIMARY_BG, LobbyStartButton)
                        .with_children(|b| {
                            b.spawn((Text::new("Start (fill w/ bots)"), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                });

                // Footer: [Refresh] [Back].
                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    button(row, &tf, theme::BUTTON_INFO_BG, LobbyRefreshButton)
                        .with_children(|b| {
                            b.spawn((Text::new("Refresh"), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                    button(row, &tf, theme::BUTTON_NEUTRAL_BG, LobbyBackButton)
                        .with_children(|b| {
                            b.spawn((Text::new("Back"), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                });
            });
        });
}

/// Spawn a themed button node with `marker`, returning the `EntityCommands`
/// so the caller can add a text child.
fn button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    _tf: &dyn Fn(f32) -> TextFont,
    bg: Color,
    marker: impl Component,
) -> bevy::ecs::system::EntityCommands<'a> {
    parent.spawn((
        Node {
            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
            border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
            ..default()
        },
        BackgroundColor(bg),
        HoverTint::new(bg),
        Button,
        marker,
    ))
}

fn despawn_lobby_browser(mut commands: Commands, q: Query<Entity, With<LobbyRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ── Live updates ───────────────────────────────────────────────────────────────

/// Rebuild the list rows whenever the advertised lobbies change.
fn rebuild_lobby_list(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    lobby: Res<LobbyState>,
    panel_q: Query<Entity, With<LobbyListPanel>>,
    rows_q: Query<Entity, With<LobbyListRow>>,
) {
    if !lobby.is_changed() {
        return;
    }
    let Ok(panel) = panel_q.single() else { return };
    for row in &rows_q {
        commands.entity(row).despawn();
    }
    let tf = |size: f32| ui_fonts.tf(size);
    // While waiting inside a lobby we don't offer Join buttons.
    let joined = lobby.joined.is_some();

    commands.entity(panel).with_children(|panel| {
        if lobby.lobbies.is_empty() {
            panel.spawn((
                Text::new("No open lobbies — create one below."),
                tf(13.0),
                TextColor(theme::TEXT_SECONDARY),
                LobbyListRow,
            ));
            return;
        }
        for info in &lobby.lobbies {
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    LobbyListRow,
                ))
                .with_children(|row| {
                    let occupied = info.players + info.bots;
                    let bot_note = if info.bots > 0 {
                        format!(" ({} bot{})", info.bots, if info.bots == 1 { "" } else { "s" })
                    } else {
                        String::new()
                    };
                    // Prefer the host's name; fall back to the lobby's own name.
                    let title = if info.host_name.is_empty() {
                        info.name.clone()
                    } else {
                        format!("{}'s lobby", info.host_name)
                    };
                    row.spawn((
                        Text::new(format!(
                            "{}  [{}]  {}/{}{}",
                            title,
                            info.format.label(),
                            occupied,
                            info.capacity,
                            bot_note,
                        )),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                    let full = occupied >= info.capacity;
                    if !joined && !full {
                        row.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                                ..default()
                            },
                            BackgroundColor(theme::BUTTON_PRIMARY_BG),
                            HoverTint::new(theme::BUTTON_PRIMARY_BG),
                            Button,
                            LobbyRowJoinButton(info.id),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Join"), tf(12.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                    }
                });
        }
    });
}

fn update_lobby_status(lobby: Res<LobbyState>, mut q: Query<&mut Text, With<LobbyStatusText>>) {
    if !lobby.is_changed() {
        return;
    }
    let Ok(mut text) = q.single_mut() else { return };
    let msg = if let Some(err) = &lobby.last_error {
        format!("⚠ {err}")
    } else if let Some((info, _slot)) = &lobby.joined {
        let occupied = info.players + info.bots;
        let bot_note = if info.bots > 0 {
            format!(" + {} bot", info.bots)
        } else {
            String::new()
        };
        let who = if info.member_names.is_empty() {
            String::new()
        } else {
            format!(" — {}", info.member_names.join(", "))
        };
        format!(
            "In lobby [{}] — waiting ({}/{}{}){}…",
            info.format.label(),
            occupied,
            info.capacity,
            bot_note,
            who,
        )
    } else {
        format!("{} open lobb{}", lobby.lobbies.len(), if lobby.lobbies.len() == 1 { "y" } else { "ies" })
    };
    text.0 = msg;
}

fn update_format_label(
    create_format: Res<LobbyCreateFormat>,
    mut q: Query<&mut Text, With<LobbyFormatLabel>>,
) {
    if !create_format.is_changed() {
        return;
    }
    if let Ok(mut text) = q.single_mut() {
        text.0 = format!("Format: {}", create_format.0.label());
    }
}

/// Show the host-control row only to the lobby host (seat 0) — only the host
/// may add/remove bots or start.
fn update_bot_controls_visibility(
    lobby: Res<LobbyState>,
    mut q: Query<&mut Node, With<LobbyBotControls>>,
) {
    if !lobby.is_changed() {
        return;
    }
    let is_host = lobby.joined.as_ref().map(|(_, slot)| *slot == 0).unwrap_or(false);
    if let Ok(mut node) = q.single_mut() {
        node.display = if is_host { Display::Flex } else { Display::None };
    }
}

// ── Input ───────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn handle_lobby_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    outbox: Option<Res<NetOutbox>>,
    mut create_format: ResMut<LobbyCreateFormat>,
    mut conn: ResMut<NetConnection>,
    mut lobby: ResMut<LobbyState>,
    fmt_q: Query<&Interaction, (Changed<Interaction>, With<LobbyFormatCycleButton>)>,
    create_q: Query<&Interaction, (Changed<Interaction>, With<LobbyCreateButton>)>,
    refresh_q: Query<&Interaction, (Changed<Interaction>, With<LobbyRefreshButton>)>,
    back_q: Query<&Interaction, (Changed<Interaction>, With<LobbyBackButton>)>,
    add_bot_q: Query<&Interaction, (Changed<Interaction>, With<LobbyAddBotButton>)>,
    remove_bot_q: Query<&Interaction, (Changed<Interaction>, With<LobbyRemoveBotButton>)>,
    start_q: Query<&Interaction, (Changed<Interaction>, With<LobbyStartButton>)>,
    join_q: Query<(&Interaction, &LobbyRowJoinButton), Changed<Interaction>>,
) {
    if fmt_q.iter().any(|i| *i == Interaction::Pressed) {
        create_format.0 = create_format.0.next();
    }
    if add_bot_q.iter().any(|i| *i == Interaction::Pressed)
        && let Some(o) = &outbox
    {
        o.submit_msg(ClientMsg::AddBotToLobby);
    }
    if remove_bot_q.iter().any(|i| *i == Interaction::Pressed)
        && let Some(o) = &outbox
    {
        o.submit_msg(ClientMsg::RemoveBotFromLobby);
    }
    if start_q.iter().any(|i| *i == Interaction::Pressed)
        && let Some(o) = &outbox
    {
        o.submit_msg(ClientMsg::StartLobby);
    }
    if create_q.iter().any(|i| *i == Interaction::Pressed)
        && let Some(o) = &outbox
    {
        o.submit_msg(ClientMsg::CreateLobby {
            name: format!("{} game", create_format.0.label()),
            format: create_format.0.to_lobby_format(),
        });
    }
    if refresh_q.iter().any(|i| *i == Interaction::Pressed)
        && let Some(o) = &outbox
    {
        o.submit_msg(ClientMsg::ListLobbies);
    }
    for (interaction, btn) in &join_q {
        if *interaction == Interaction::Pressed
            && let Some(o) = &outbox
        {
            o.submit_msg(ClientMsg::JoinLobby { lobby_id: btn.0 });
        }
    }
    if back_q.iter().any(|i| *i == Interaction::Pressed) {
        // Disconnect and return to the menu. (Going to InGame instead keeps the
        // session; this is the only path that tears it down from the lobby.)
        if let Some(stream) = conn.0.take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        commands.remove_resource::<NetOutbox>();
        commands.remove_resource::<NetInbox>();
        *lobby = LobbyState::default();
        next_state.set(AppState::Menu);
    }
}

/// The server filled our lobby and started the match — leave the browser. The
/// net session is already live, so `start_net_session_from_menu` no-ops.
fn watch_match_start(lobby: Res<LobbyState>, mut next_state: ResMut<NextState<AppState>>) {
    if lobby.match_started {
        next_state.set(AppState::InGame);
    }
}
