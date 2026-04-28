//! Pre-game main menu.
//!
//! Lets the user pick how to start the match:
//!
//! - **Play vs Bot** — in-process server, RandomBot opponent (no network).
//! - **Host LAN Game** — spawn a TCP listener; local player joins via an
//!   in-process channel; second seat is filled by the next remote client to
//!   connect. (Use this + a second client running "Join" on another machine.)
//! - **Join LAN Game** — connect to a remote `addr:port`.
//!
//! The menu writes [`PendingNetMode`] and transitions to [`AppState::InGame`];
//! [`crate::net_plugin::start_net_session`] reads it on entry.
//!
//! A simple keyboard-driven text input lets the user edit the join address
//! and the host port; clicks on the field activate it.

use std::net::TcpListener;
use std::sync::Mutex;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crabomination::cube::build_cube_state;
use crabomination::demo::build_demo_state;
use crabomination::game::GameState;
use crabomination::server::{
    ClientChannel, RandomBot, SeatOccupant, run_match, seat_pair, tcp_client, tcp_seat,
};
use crabomination::net::ClientMsg;

use crate::net_plugin::{NetInbox, NetOutbox};

// ── State + resources ────────────────────────────────────────────────────────

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

/// Filled in by the menu when the user picks an option; drained by
/// [`crate::net_plugin::start_net_session`] when entering `InGame`.
/// Carries the chosen format alongside the network mode so the in-game
/// match builder can pick the right deck pool.
#[derive(Resource, Default)]
pub struct PendingNetMode(pub Option<(NetMode, MatchFormat)>);

#[derive(Clone, Debug)]
pub enum NetMode {
    /// In-process server, RandomBot opponent.
    LocalBot,
    /// Bind a TCP listener on `port`; pair the local in-process seat against
    /// the next remote client to connect.
    HostLan { port: u16 },
    /// Connect a TCP client to `addr` (host:port).
    JoinLan { addr: String },
}

/// Which deck pool the match draws from. Modern uses the BRG / Goryo's
/// demo decks (`demo::build_demo_state`). Cube rolls a fresh random
/// 2-color deck per seat from the curated cube pools (`cube::build_cube_state`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MatchFormat {
    #[default]
    Modern,
    Cube,
}

impl MatchFormat {
    fn build_state(self) -> GameState {
        match self {
            MatchFormat::Modern => build_demo_state(),
            MatchFormat::Cube => build_cube_state(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            MatchFormat::Modern => "Modern",
            MatchFormat::Cube => "Cube",
        }
    }
}

/// Active text-edit field in the menu.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
enum FocusedField {
    #[default]
    None,
    HostPort,
    JoinAddr,
}

#[derive(Resource)]
struct MenuFields {
    host_port: String,
    join_addr: String,
    focused: FocusedField,
    format: MatchFormat,
}

impl Default for MenuFields {
    fn default() -> Self {
        Self {
            // Default to the same port the standalone `crabomination_server`
            // binary uses, so a remote client can run the standalone server
            // on the same port and these defaults work out-of-the-box.
            host_port: "7777".to_string(),
            join_addr: std::env::var("CRAB_SERVER")
                .unwrap_or_else(|_| "127.0.0.1:7777".to_string()),
            focused: FocusedField::None,
            format: MatchFormat::default(),
        }
    }
}

// ── Marker components ────────────────────────────────────────────────────────

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct PlayBotButton;

#[derive(Component)]
struct HostButton;

#[derive(Component)]
struct JoinButton;

#[derive(Component)]
struct FieldButton(FocusedField);

#[derive(Component)]
struct FieldText(FocusedField);

#[derive(Component)]
struct FormatToggleButton(MatchFormat);

// ── Plugin ───────────────────────────────────────────────────────────────────

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<PendingNetMode>()
            .init_resource::<MenuFields>()
            .add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(OnExit(AppState::Menu), despawn_menu)
            .add_systems(
                Update,
                (
                    handle_field_focus,
                    handle_text_input,
                    refresh_field_text,
                    handle_format_toggle,
                    refresh_format_toggle_visuals,
                    handle_action_buttons,
                )
                    .run_if(in_state(AppState::Menu)),
            );
    }
}

// ── UI setup ─────────────────────────────────────────────────────────────────

const PANEL_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.96);
const FIELD_BG_OFF: Color = Color::srgba(0.16, 0.16, 0.22, 1.0);
const FIELD_BG_ON: Color = Color::srgba(0.28, 0.28, 0.50, 1.0);
const PLAY_BG: Color = Color::srgba(0.18, 0.45, 0.20, 1.0);
const HOST_BG: Color = Color::srgba(0.20, 0.30, 0.55, 1.0);
const JOIN_BG: Color = Color::srgba(0.45, 0.30, 0.15, 1.0);

fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let tf = |size: f32| TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    };

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            MenuRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(28.0)),
                    row_gap: Val::Px(18.0),
                    align_items: AlignItems::Center,
                    min_width: Val::Px(380.0),
                    ..default()
                },
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("Crabomination"),
                    tf(28.0),
                    TextColor(Color::srgb(1.0, 0.85, 0.55)),
                ));

                // Format selector — Modern (BRG / Goryo's demo decks) vs
                // Cube (random 2-color deck per seat).
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|fmt| {
                    fmt.spawn((
                        Text::new("Format"),
                        tf(13.0),
                        TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                    ));
                    fmt.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        format_toggle(row, &tf, MatchFormat::Modern);
                        format_toggle(row, &tf, MatchFormat::Cube);
                    });
                });

                // Play vs Bot
                button(p, &tf, "Play vs Bot", PLAY_BG, PlayBotButton);

                // Host LAN
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|host| {
                    button(host, &tf, "Host LAN Game", HOST_BG, HostButton);
                    field(
                        host,
                        &tf,
                        "Port:",
                        FocusedField::HostPort,
                    );
                });

                // Join LAN
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|join| {
                    button(join, &tf, "Join LAN Game", JOIN_BG, JoinButton);
                    field(
                        join,
                        &tf,
                        "Server:",
                        FocusedField::JoinAddr,
                    );
                });

                p.spawn((
                    Text::new("Click a text field to edit. Backspace deletes."),
                    tf(11.0),
                    TextColor(Color::srgba(0.65, 0.65, 0.65, 1.0)),
                ));
            });
        });
}

fn button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    label: &str,
    bg: Color,
    marker: M,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            marker,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                tf(16.0),
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        });
}

fn field(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    label: &str,
    which: FocusedField,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                tf(13.0),
                TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
            ));
            row.spawn((
                Button,
                Node {
                    flex_grow: 1.0,
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(FIELD_BG_OFF),
                FieldButton(which),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(""),
                    tf(13.0),
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                    FieldText(which),
                ));
            });
        });
}

fn format_toggle(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    format: MatchFormat,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(FIELD_BG_OFF),
            FormatToggleButton(format),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(format.label()),
                tf(13.0),
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        });
}

fn despawn_menu(mut commands: Commands, q: Query<Entity, With<MenuRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ── Input handling ───────────────────────────────────────────────────────────

fn handle_field_focus(
    mut fields: ResMut<MenuFields>,
    mut buttons: Query<(&Interaction, &FieldButton, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, fb, mut bg) in &mut buttons {
        if *interaction == Interaction::Pressed {
            fields.focused = fb.0;
        }
        let on = fields.focused == fb.0 && fb.0 != FocusedField::None;
        *bg = BackgroundColor(if on { FIELD_BG_ON } else { FIELD_BG_OFF });
    }
}

fn handle_text_input(
    mut fields: ResMut<MenuFields>,
    mut events: MessageReader<KeyboardInput>,
) {
    if fields.focused == FocusedField::None {
        return;
    }
    let mut next_focused = fields.focused;
    let mut buf = match fields.focused {
        FocusedField::HostPort => fields.host_port.clone(),
        FocusedField::JoinAddr => fields.join_addr.clone(),
        FocusedField::None => return,
    };
    let max_len = match fields.focused {
        FocusedField::HostPort => 5,
        FocusedField::JoinAddr => 80,
        FocusedField::None => return,
    };
    let mut changed = false;
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Backspace => {
                buf.pop();
                changed = true;
            }
            Key::Enter | Key::Escape => {
                next_focused = FocusedField::None;
            }
            Key::Character(s) => {
                for ch in s.chars() {
                    if buf.len() >= max_len {
                        break;
                    }
                    if accepts_char(fields.focused, ch) {
                        buf.push(ch);
                        changed = true;
                    }
                }
            }
            _ => {}
        }
    }
    if changed {
        match fields.focused {
            FocusedField::HostPort => fields.host_port = buf,
            FocusedField::JoinAddr => fields.join_addr = buf,
            FocusedField::None => {}
        }
    }
    fields.focused = next_focused;
}

fn accepts_char(field: FocusedField, ch: char) -> bool {
    match field {
        FocusedField::HostPort => ch.is_ascii_digit(),
        FocusedField::JoinAddr => {
            ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '-' | '_')
        }
        FocusedField::None => false,
    }
}

fn refresh_field_text(
    fields: Res<MenuFields>,
    mut q: Query<(&FieldText, &mut Text)>,
) {
    if !fields.is_changed() {
        return;
    }
    for (which, mut t) in &mut q {
        let value = match which.0 {
            FocusedField::HostPort => &fields.host_port,
            FocusedField::JoinAddr => &fields.join_addr,
            FocusedField::None => continue,
        };
        let cursor = if fields.focused == which.0 { "_" } else { "" };
        t.0 = format!("{value}{cursor}");
    }
}

fn handle_format_toggle(
    mut fields: ResMut<MenuFields>,
    buttons: Query<(&Interaction, &FormatToggleButton), Changed<Interaction>>,
) {
    for (interaction, toggle) in &buttons {
        if *interaction == Interaction::Pressed {
            fields.format = toggle.0;
        }
    }
}

fn refresh_format_toggle_visuals(
    fields: Res<MenuFields>,
    mut buttons: Query<(&FormatToggleButton, &mut BackgroundColor)>,
) {
    if !fields.is_changed() {
        return;
    }
    for (toggle, mut bg) in &mut buttons {
        *bg = BackgroundColor(if toggle.0 == fields.format {
            FIELD_BG_ON
        } else {
            FIELD_BG_OFF
        });
    }
}

fn handle_action_buttons(
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingNetMode>,
    fields: Res<MenuFields>,
    play_q: Query<&Interaction, (Changed<Interaction>, With<PlayBotButton>)>,
    host_q: Query<&Interaction, (Changed<Interaction>, With<HostButton>)>,
    join_q: Query<&Interaction, (Changed<Interaction>, With<JoinButton>)>,
) {
    let format = fields.format;
    if play_q.iter().any(|i| *i == Interaction::Pressed) {
        pending.0 = Some((NetMode::LocalBot, format));
        next_state.set(AppState::InGame);
        return;
    }
    if host_q.iter().any(|i| *i == Interaction::Pressed) {
        if let Ok(port) = fields.host_port.parse::<u16>() {
            pending.0 = Some((NetMode::HostLan { port }, format));
            next_state.set(AppState::InGame);
        } else {
            eprintln!("menu: invalid host port `{}`", fields.host_port);
        }
        return;
    }
    if join_q.iter().any(|i| *i == Interaction::Pressed) {
        let addr = fields.join_addr.trim().to_string();
        if !addr.is_empty() {
            pending.0 = Some((NetMode::JoinLan { addr }, format));
            next_state.set(AppState::InGame);
        }
    }
}

// ── Network setup invoked by `OnEnter(InGame)` ───────────────────────────────

/// Read the queued `PendingNetMode` and install `NetOutbox`/`NetInbox`. Falls
/// back to a local Modern bot match if no choice was queued (e.g. tests
/// bypass the menu).
pub fn start_net_session_from_menu(world: &mut World) {
    let (mode, format) = world
        .get_resource_mut::<PendingNetMode>()
        .and_then(|mut r| r.0.take())
        .unwrap_or((NetMode::LocalBot, MatchFormat::Modern));

    match mode {
        NetMode::LocalBot => spawn_inprocess_bot(world, format),
        NetMode::HostLan { port } => match spawn_host_lan(world, port, format) {
            Ok(()) => eprintln!(
                "net: hosting {fmt:?} on 0.0.0.0:{port} — waiting for opponent",
                fmt = format
            ),
            Err(e) => {
                eprintln!("net: host failed ({e}); falling back to local bot");
                spawn_inprocess_bot(world, format);
            }
        },
        NetMode::JoinLan { addr } => match spawn_join_lan(world, &addr) {
            Ok(()) => eprintln!("net: connected to {addr}"),
            Err(e) => {
                eprintln!("net: join {addr} failed ({e}); falling back to local bot");
                spawn_inprocess_bot(world, format);
            }
        },
    }
}

fn spawn_inprocess_bot(world: &mut World, format: MatchFormat) {
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    std::thread::spawn(move || {
        run_match(
            format.build_state(),
            vec![
                SeatOccupant::Human(server_seat),
                SeatOccupant::Bot(Box::new(RandomBot::new())),
            ],
        );
    });
    world.insert_resource(NetOutbox(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
}

fn spawn_host_lan(world: &mut World, port: u16, format: MatchFormat) -> std::io::Result<()> {
    let bind = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind)?;
    let (server_seat0, ClientChannel { tx, rx }) = seat_pair();

    std::thread::spawn(move || {
        let (stream, peer) = match listener.accept() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("host: accept failed: {e}");
                return;
            }
        };
        eprintln!("host: opponent connected from {peer}");
        let server_seat1 = match tcp_seat(stream) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("host: tcp_seat failed: {e}");
                return;
            }
        };
        run_match(
            format.build_state(),
            vec![
                SeatOccupant::Human(server_seat0),
                SeatOccupant::Human(server_seat1),
            ],
        );
        eprintln!("host: match ended");
    });

    world.insert_resource(NetOutbox(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    Ok(())
}

fn spawn_join_lan(world: &mut World, addr: &str) -> std::io::Result<()> {
    let stream = std::net::TcpStream::connect(addr)?;
    let ClientChannel { tx, rx } = tcp_client(stream)?;
    let _ = tx.send(ClientMsg::JoinMatch { name: "client".into() });
    world.insert_resource(NetOutbox(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    Ok(())
}

