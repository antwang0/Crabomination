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
use std::sync::{Arc, Mutex};

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crabomination::cube::build_cube_state;
use crabomination::sos_mode::build_sos_state;
use crabomination::demo::{build_commander_state, build_demo_state};
use crabomination::game::GameState;
use crabomination::server::{
    ClientChannel, RandomBot, SeatOccupant, run_match, run_match_full, seat_pair,
    tcp_seat, SnapshotSink, SnapshotSinkState,
};

use crate::net_plugin::{NetInbox, NetOutbox};

// ── State + resources ────────────────────────────────────────────────────────

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    /// Audit-mode card picker. Selecting a card writes its name to
    /// `AuditTarget` and transitions into `InGame` with an
    /// audit-tailored state (see `crate::audit::build_audit_state`).
    Audit,
    /// 8-seat booster draft against 7 bots. Owned by
    /// `crate::systems::draft::DraftPlugin`; cards are drawn from
    /// either the cube pool or the Secrets of Strixhaven pool
    /// depending on `PendingDraftFormat`. On completion the resulting
    /// `DraftedDecks` resource is consumed by `start_net_session` so
    /// the post-draft match plays out via the normal InGame path.
    Drafting,
    /// Connected to a lobby server: browse, create (choosing a gamemode), or
    /// join a lobby. Owned by `crate::systems::lobby_ui`. Transitions to
    /// `InGame` once the server sends `MatchStarted` (the net session is
    /// already installed, so `start_net_session_from_menu` is a no-op then).
    Lobby,
    InGame,
}

/// The lobby server the user chose to connect to, plus the display name to
/// announce. Set by the menu's "Join LAN" action and consumed by
/// `lobby_ui::connect_to_lobby_server` on entry to [`AppState::Lobby`].
#[derive(Resource, Default)]
pub struct PendingLobbyServer(pub Option<LobbyConnect>);

/// A queued lobby connection request.
pub struct LobbyConnect {
    pub addr: String,
    pub name: String,
}

/// Set by the menu when the player picks "Draft"; read by the draft
/// plugin to choose which card pool the booster packs sample from.
/// Defaults to `Cube` so older save/restore paths still work.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingDraftFormat(pub MatchFormat);

impl Default for PendingDraftFormat {
    fn default() -> Self {
        Self(MatchFormat::Cube)
    }
}

/// Decks produced by `AppState::Drafting`'s opponent-select step. When
/// present at the `OnEnter(AppState::InGame)` boundary,
/// `start_net_session_from_menu` builds a 2-player match from these
/// decks via `crabomination::draft::build_draft_match_state` instead
/// of rolling a fresh random match. The resource is taken (drained) on
/// consumption so a follow-up rematch / new-game falls back to the
/// format's normal random path.
#[derive(Resource, Clone, Debug)]
pub struct DraftedDecks {
    pub player_deck: Vec<crabomination::cube::CardFactory>,
    pub opponent_deck: Vec<crabomination::cube::CardFactory>,
    pub opponent_label: String,
}

/// Filled in by the menu when the user picks an option; drained by
/// [`crate::net_plugin::start_net_session`] when entering `InGame`.
/// Carries the chosen format alongside the network mode so the in-game
/// match builder can pick the right deck pool.
#[derive(Resource, Default)]
pub struct PendingNetMode(pub Option<(NetMode, MatchFormat)>);

/// Inserted at startup with the value of `--load-state <path>`. When
/// `Some`, the menu auto-loads that file and skips straight into
/// inspection mode; otherwise the menu behaves normally.
#[derive(Resource, Default, Debug)]
pub struct CliBootHint(pub Option<std::path::PathBuf>);

/// Inserted at startup with the value of `--play <format>`. When `Some`,
/// the menu boots a local-bot match of that format directly (used to verify
/// format-specific layouts, e.g. the 4-player Commander table).
#[derive(Resource, Default, Debug)]
pub struct CliBootFormat(pub Option<MatchFormat>);

#[derive(Clone, Debug)]
pub enum NetMode {
    /// In-process server, RandomBot opponent.
    LocalBot,
    /// In-process server with two RandomBots; the local UI is a spectator.
    SpectateBots,
    /// Bind a TCP listener on `port`; pair the local in-process seat against
    /// the next remote client to connect.
    HostLan { port: u16 },
    /// Load a `<repo>/debug/state-*.json` snapshot and run the client in
    /// inspection mode (no live server; the HUD is read-only). Used for
    /// reproducing reported bugs from a saved state.
    LoadDebugState { path: std::path::PathBuf },
}

/// Which deck pool the match draws from. Modern uses the BRG / Goryo's
/// demo decks (`demo::build_demo_state`). Cube rolls a fresh random
/// 2-color deck per seat from the curated cube pools (`cube::build_cube_state`).
/// SoS rolls a random Strixhaven college per seat and builds a 60-card
/// deck from that college's ✅-only Secrets of Strixhaven cards
/// (`sos_mode::build_sos_state`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MatchFormat {
    #[default]
    Modern,
    Cube,
    Sos,
    /// 1v1 Commander: both seats run the Rofellos mono-green
    /// demo deck. 100-card singleton, 40 life, commander seated in
    /// the command zone before opening-hand draw. Built via
    /// `demo::build_commander_state`.
    Commander,
}

impl MatchFormat {
    /// Build a fresh `GameState` for this format. Public so the
    /// game-over "New Game" button can launch a follow-up match
    /// without going back through the menu UI.
    pub fn build_state_for_restart(self) -> GameState {
        self.build_state()
    }

    fn build_state(self) -> GameState {
        match self {
            MatchFormat::Modern => build_demo_state(),
            MatchFormat::Cube => build_cube_state(),
            MatchFormat::Sos => build_sos_state(),
            MatchFormat::Commander => build_commander_state(),
        }
    }

    /// Parse a `--play <format>` CLI argument. Case-insensitive; returns
    /// `None` for an unrecognised name so the launch falls through to the menu.
    pub fn from_cli(s: &str) -> Option<MatchFormat> {
        match s.to_ascii_lowercase().as_str() {
            "modern" => Some(MatchFormat::Modern),
            "cube" => Some(MatchFormat::Cube),
            "sos" => Some(MatchFormat::Sos),
            "commander" | "edh" => Some(MatchFormat::Commander),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MatchFormat::Modern => "Modern",
            MatchFormat::Cube => "Cube",
            MatchFormat::Sos => "SoS",
            MatchFormat::Commander => "Commander",
        }
    }

    /// Cycle to the next gamemode, for the lobby's format picker.
    pub fn next(self) -> MatchFormat {
        match self {
            MatchFormat::Modern => MatchFormat::Cube,
            MatchFormat::Cube => MatchFormat::Sos,
            MatchFormat::Sos => MatchFormat::Commander,
            MatchFormat::Commander => MatchFormat::Modern,
        }
    }

    /// Map to the wire gamemode for a `CreateLobby` request.
    pub fn to_lobby_format(self) -> crabomination::net::LobbyFormat {
        use crabomination::net::LobbyFormat as LF;
        match self {
            MatchFormat::Modern => LF::Modern,
            MatchFormat::Cube => LF::Cube,
            MatchFormat::Sos => LF::Sos,
            MatchFormat::Commander => LF::Commander,
        }
    }
}

/// Active text-edit field in the menu.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
enum FocusedField {
    #[default]
    None,
    PlayerName,
    HostPort,
    JoinAddr,
    DeckPath,
}

#[derive(Resource)]
pub(crate) struct MenuFields {
    pub(crate) player_name: String,
    host_port: String,
    join_addr: String,
    /// Path to a plain-text decklist (Arena / MTGO format) for the
    /// "Play Deck vs Bot" import flow.
    deck_path: String,
    focused: FocusedField,
    format: MatchFormat,
}

impl Default for MenuFields {
    fn default() -> Self {
        Self {
            // Display name shown to other players in lobbies. Seeded from the
            // OS username when available so it's meaningful out of the box.
            player_name: default_player_name(),
            // Default to the same port the standalone `crabomination_server`
            // binary uses, so a remote client can run the standalone server
            // on the same port and these defaults work out-of-the-box.
            host_port: "7777".to_string(),
            join_addr: std::env::var("CRAB_SERVER")
                .unwrap_or_else(|_| "127.0.0.1:7777".to_string()),
            deck_path: "deck.txt".to_string(),
            focused: FocusedField::None,
            format: MatchFormat::default(),
        }
    }
}

/// Maximum length of a player's display name (clamped on input and trimmed
/// before it's sent to the lobby server).
const MAX_PLAYER_NAME_LEN: usize = 20;

/// Seed the player name from the OS username, falling back to "Player".
fn default_player_name() -> String {
    let raw = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "Player".to_string()
    } else {
        trimmed.chars().take(MAX_PLAYER_NAME_LEN).collect()
    }
}

// ── Marker components ────────────────────────────────────────────────────────

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct PlayBotButton;

/// "Play Deck vs Bot" — reads the decklist file named in the DeckPath
/// field, validates it against the catalog, and starts a local-bot match
/// with the imported deck.
#[derive(Component)]
struct ImportDeckButton;

/// Feedback line under the import controls ("12 cards unknown: …",
/// "deck.txt not found", "Imported 60 cards").
#[derive(Component)]
struct MenuStatusText;

/// Current menu feedback message (import errors, validation results).
#[derive(Resource, Default)]
struct MenuStatus(String);

/// A successfully imported maindeck, consumed by `spawn_inprocess_bot`
/// (it outranks the format's stock decks, like `DraftedDecks`).
#[derive(Resource, Clone)]
pub struct ImportedDeck(pub Vec<crabomination::cube::CardFactory>);

#[derive(Component)]
struct SpectateBotsButton;

#[derive(Component)]
struct LoadDebugStateButton;

#[derive(Component)]
struct AuditCardsButton;

#[derive(Component)]
struct DraftButton;

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
            .init_resource::<PendingDraftFormat>()
            .init_resource::<PendingLobbyServer>()
            .init_resource::<MenuFields>()
            .init_resource::<MenuStatus>()
            .init_resource::<CliBootHint>()
            .init_resource::<CliBootFormat>()
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
                    refresh_menu_status,
                    apply_cli_boot_hint,
                )
                    .run_if(in_state(AppState::Menu)),
            );
    }
}

/// One-shot system: when the client was launched with `--load-state <path>`,
/// fire the load mode immediately and transition into InGame on the next
/// frame, bypassing the menu UI entirely.
fn apply_cli_boot_hint(
    mut hint: ResMut<CliBootHint>,
    mut boot_format: ResMut<CliBootFormat>,
    mut pending: ResMut<PendingNetMode>,
    mut next_state: ResMut<NextState<AppState>>,
    fields: Res<MenuFields>,
) {
    // `--play <format>` boots a local-bot match of that format directly.
    if let Some(format) = boot_format.0.take() {
        pending.0 = Some((NetMode::LocalBot, format));
        next_state.set(AppState::InGame);
        return;
    }
    let Some(path) = hint.0.take() else { return };
    pending.0 = Some((NetMode::LoadDebugState { path }, fields.format));
    next_state.set(AppState::InGame);
}

// ── UI setup ─────────────────────────────────────────────────────────────────

use crate::theme::{
    self, HoverTint, UiFonts, BUTTON_ACCENT_BG, BUTTON_DANGER_BG, BUTTON_INFO_BG,
    BUTTON_PRIMARY_BG, BUTTON_WARN_BG, FIELD_BG, FIELD_BG_FOCUSED, PANEL_BG,
    RADIUS_BUTTON, RADIUS_PANEL,
};

fn spawn_menu(mut commands: Commands, ui_fonts: Res<UiFonts>) {
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
                    border_radius: BorderRadius::all(RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("Crabomination"),
                    tf(28.0),
                    TextColor(theme::ACCENT_GOLD),
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
                        TextColor(theme::TEXT_BODY),
                    ));
                    fmt.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        format_toggle(row, &tf, MatchFormat::Modern);
                        format_toggle(row, &tf, MatchFormat::Cube);
                        format_toggle(row, &tf, MatchFormat::Sos);
                        format_toggle(row, &tf, MatchFormat::Commander);
                    });
                });

                // Play vs Bot
                button(p, &tf, "Play vs Bot", BUTTON_PRIMARY_BG, PlayBotButton);

                // Draft — opens the 8-seat booster draft for the
                // selected format (Cube or SoS). Modern / Commander
                // fall back to the Cube pool.
                button(p, &tf, "Draft (Cube / SoS)", BUTTON_INFO_BG, DraftButton);

                // Spectate Bot vs Bot
                button(
                    p,
                    &tf,
                    "Spectate Bot vs Bot",
                    BUTTON_ACCENT_BG,
                    SpectateBotsButton,
                );

                // Load Debug State (most recent file in <repo>/debug/)
                button(
                    p,
                    &tf,
                    "Load Latest Debug State",
                    BUTTON_DANGER_BG,
                    LoadDebugStateButton,
                );

                // Audit Cards — opens the card picker for verifying
                // individual card implementations one-by-one.
                button(
                    p,
                    &tf,
                    "Audit Cards",
                    BUTTON_ACCENT_BG,
                    AuditCardsButton,
                );

                // Import a decklist (Arena / MTGO text format) and play
                // it against the bot. The field holds a file path; status
                // feedback (unknown cards, size problems) renders below.
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|imp| {
                    button(imp, &tf, "Play Deck vs Bot", BUTTON_PRIMARY_BG, ImportDeckButton);
                    field(imp, &tf, "Deck file:", FocusedField::DeckPath);
                });

                // Display name (shown to other players in lobbies).
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|name| {
                    field(name, &tf, "Name:", FocusedField::PlayerName);
                });

                // Host LAN
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|host| {
                    button(host, &tf, "Host LAN Game", BUTTON_INFO_BG, HostButton);
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
                    button(join, &tf, "Join LAN Game", BUTTON_WARN_BG, JoinButton);
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
                    TextColor(theme::TEXT_PLACEHOLDER),
                ));

                p.spawn((
                    Text::new(""),
                    tf(12.0),
                    TextColor(theme::ACCENT_ORANGE),
                    MenuStatusText,
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
                border_radius: BorderRadius::all(RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(bg),
            HoverTint::new(bg),
            marker,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                tf(16.0),
                TextColor(theme::TEXT_PRIMARY),
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
                TextColor(theme::TEXT_BODY),
            ));
            row.spawn((
                Button,
                Node {
                    flex_grow: 1.0,
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(FIELD_BG),
                FieldButton(which),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(""),
                    tf(13.0),
                    TextColor(theme::TEXT_PRIMARY),
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
                border_radius: BorderRadius::all(RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(FIELD_BG),
            FormatToggleButton(format),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(format.label()),
                tf(13.0),
                TextColor(theme::TEXT_PRIMARY),
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
        *bg = BackgroundColor(if on { FIELD_BG_FOCUSED } else { FIELD_BG });
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
        FocusedField::PlayerName => fields.player_name.clone(),
        FocusedField::HostPort => fields.host_port.clone(),
        FocusedField::JoinAddr => fields.join_addr.clone(),
        FocusedField::DeckPath => fields.deck_path.clone(),
        FocusedField::None => return,
    };
    let max_len = match fields.focused {
        FocusedField::PlayerName => MAX_PLAYER_NAME_LEN,
        FocusedField::HostPort => 5,
        FocusedField::JoinAddr => 80,
        FocusedField::DeckPath => 160,
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
            FocusedField::PlayerName => fields.player_name = buf,
            FocusedField::HostPort => fields.host_port = buf,
            FocusedField::JoinAddr => fields.join_addr = buf,
            FocusedField::DeckPath => fields.deck_path = buf,
            FocusedField::None => {}
        }
    }
    fields.focused = next_focused;
}

fn accepts_char(field: FocusedField, ch: char) -> bool {
    match field {
        // Letters, digits, spaces, and a few name-safe punctuation marks.
        FocusedField::PlayerName => {
            ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '-' | '_' | '.')
        }
        FocusedField::HostPort => ch.is_ascii_digit(),
        FocusedField::JoinAddr => {
            ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '-' | '_')
        }
        // Filesystem paths: also slashes, ~, spaces.
        FocusedField::DeckPath => {
            ch.is_ascii_alphanumeric() || matches!(ch, '.' | '/' | '\\' | '~' | '-' | '_' | ' ')
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
            FocusedField::PlayerName => &fields.player_name,
            FocusedField::HostPort => &fields.host_port,
            FocusedField::JoinAddr => &fields.join_addr,
            FocusedField::DeckPath => &fields.deck_path,
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
            FIELD_BG_FOCUSED
        } else {
            FIELD_BG
        });
    }
}

/// Mirror `MenuStatus` into its text node.
fn refresh_menu_status(status: Res<MenuStatus>, mut q: Query<&mut Text, With<MenuStatusText>>) {
    if !status.is_changed() {
        return;
    }
    for mut t in &mut q {
        t.0 = status.0.clone();
    }
}

fn handle_action_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingNetMode>,
    mut pending_draft: ResMut<PendingDraftFormat>,
    mut lobby_server: ResMut<PendingLobbyServer>,
    mut status: ResMut<MenuStatus>,
    fields: Res<MenuFields>,
    import_q: Query<&Interaction, (Changed<Interaction>, With<ImportDeckButton>)>,
    play_q: Query<&Interaction, (Changed<Interaction>, With<PlayBotButton>)>,
    spectate_q: Query<&Interaction, (Changed<Interaction>, With<SpectateBotsButton>)>,
    load_q: Query<&Interaction, (Changed<Interaction>, With<LoadDebugStateButton>)>,
    audit_q: Query<&Interaction, (Changed<Interaction>, With<AuditCardsButton>)>,
    draft_q: Query<&Interaction, (Changed<Interaction>, With<DraftButton>)>,
    host_q: Query<&Interaction, (Changed<Interaction>, With<HostButton>)>,
    join_q: Query<&Interaction, (Changed<Interaction>, With<JoinButton>)>,
) {
    if audit_q.iter().any(|i| *i == Interaction::Pressed) {
        next_state.set(AppState::Audit);
        return;
    }
    let format = fields.format;
    if draft_q.iter().any(|i| *i == Interaction::Pressed) {
        // The draft plugin reads PendingDraftFormat at OnEnter; the
        // resulting DraftedDecks resource will be consumed by
        // start_net_session_from_menu when the user finishes drafting.
        // Stash the underlying mode so the post-draft match plays as a
        // local-bot game.
        pending_draft.0 = format;
        pending.0 = Some((NetMode::LocalBot, format));
        next_state.set(AppState::Drafting);
        return;
    }
    if play_q.iter().any(|i| *i == Interaction::Pressed) {
        pending.0 = Some((NetMode::LocalBot, format));
        next_state.set(AppState::InGame);
        return;
    }
    if import_q.iter().any(|i| *i == Interaction::Pressed) {
        let path = fields.deck_path.trim();
        match std::fs::read_to_string(path) {
            Err(e) => status.0 = format!("Can't read {path}: {e}"),
            Ok(text) => {
                let parsed = crabomination::decklist::parse_decklist(&text);
                if !parsed.unknown.is_empty() {
                    // Refuse rather than silently playing a partial deck.
                    let shown = parsed.unknown.iter().take(4).cloned()
                        .collect::<Vec<_>>().join(", ");
                    let more = parsed.unknown.len().saturating_sub(4);
                    status.0 = format!(
                        "{} card(s) not in the catalog: {shown}{}",
                        parsed.unknown.len(),
                        if more > 0 { format!(" (+{more} more)") } else { String::new() },
                    );
                } else if parsed.main.len() < 40 {
                    status.0 = format!(
                        "Deck has only {} cards (need at least 40)",
                        parsed.main.len()
                    );
                } else {
                    status.0.clear();
                    commands.insert_resource(ImportedDeck(parsed.main));
                    pending.0 = Some((NetMode::LocalBot, format));
                    next_state.set(AppState::InGame);
                }
            }
        }
        return;
    }
    if spectate_q.iter().any(|i| *i == Interaction::Pressed) {
        pending.0 = Some((NetMode::SpectateBots, format));
        next_state.set(AppState::InGame);
        return;
    }
    if load_q.iter().any(|i| *i == Interaction::Pressed) {
        match crate::debug_export::list_exports().into_iter().next() {
            Some(path) => {
                pending.0 = Some((NetMode::LoadDebugState { path }, format));
                next_state.set(AppState::InGame);
            }
            None => eprintln!("menu: no debug state files found in <repo>/debug/"),
        }
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
            // Connect, then browse lobbies on the server (which picks the
            // gamemode per lobby). The actual connect happens on entry to
            // `AppState::Lobby`.
            let name = sanitize_name(&fields.player_name);
            lobby_server.0 = Some(LobbyConnect { addr, name });
            next_state.set(AppState::Lobby);
        }
    }
}

/// Trim a display name and fall back to "Player" when it's blank, so the
/// server never receives an empty name.
fn sanitize_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "Player".to_string()
    } else {
        trimmed.to_string()
    }
}

// ── Network setup invoked by `OnEnter(InGame)` ───────────────────────────────

/// Read the queued `PendingNetMode` and install `NetOutbox`/`NetInbox`. Falls
/// back to a local Modern bot match if no choice was queued (e.g. tests
/// bypass the menu).
pub fn start_net_session_from_menu(world: &mut World) {
    // On the lobby path the net session is already installed (we connected to
    // the lobby server) and the match is running server-side; below we still
    // set up the in-game meta resources but skip re-spawning a session.
    let already_connected = world.contains_resource::<NetOutbox>();

    // Clear the play-by-play log so a new session (including an audit run)
    // doesn't show scrollback from the previous game.
    if let Some(mut log) = world.get_resource_mut::<crate::game::GameLog>() {
        log.entries.clear();
    }

    let (mode, format) = world
        .get_resource_mut::<PendingNetMode>()
        .and_then(|mut r| r.0.take())
        .unwrap_or((NetMode::LocalBot, MatchFormat::Modern));

    // Stash the chosen format + mode kind so the game-over modal can
    // (a) launch an equivalent rematch without revisiting the menu,
    // and (b) decide whether to show the auto-rematch counter
    // (Spectate Bot vs Bot only — Human vs Bot would be jarring).
    //
    // These must be inserted even on the lobby path (already_connected): the
    // in-game game-over systems read `ActiveMatchFormat` as a required
    // resource every frame, so skipping them used to crash the moment a lobby
    // match started.
    world.insert_resource(crate::systems::game_over::ActiveMatchFormat(format));
    let kind = match &mode {
        NetMode::SpectateBots => crate::systems::game_over::ActiveMatchKind::SpectateBotVsBot,
        _ => crate::systems::game_over::ActiveMatchKind::HumanVsBot,
    };
    world.insert_resource(kind);

    // Lobby flow: the net session was already installed when we connected to
    // the lobby server, and the match is already running server-side — just
    // consume it. Re-running the spawn below would clobber the live connection.
    if already_connected {
        return;
    }

    match mode {
        NetMode::LocalBot => spawn_inprocess_bot(world, format),
        NetMode::SpectateBots => spawn_spectate_bots(world, format),
        NetMode::LoadDebugState { path } => match spawn_loaded_debug_state(world, &path) {
            Ok(()) => eprintln!("net: loaded debug state from {}", path.display()),
            Err(e) => {
                eprintln!("net: load {} failed ({e}); falling back to local bot",
                    path.display());
                spawn_inprocess_bot(world, format);
            }
        },
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
    }
}

/// Stamp display names onto a freshly built local match state so the HUD
/// and log read "Alice" / "Bot" instead of the engine's "P0" / "P1"
/// placeholders. Seat 0 gets `seat0`; other seats get `other` (numbered
/// when there are several, e.g. Commander's three bots).
pub(crate) fn name_seats(state: &mut GameState, seat0: &str, other: &str) {
    let many_others = state.players.len() > 2;
    for (i, p) in state.players.iter_mut().enumerate() {
        p.name = if i == 0 {
            seat0.to_string()
        } else if many_others {
            format!("{other} {i}")
        } else {
            other.to_string()
        };
    }
}

/// The menu's display name (trimmed), falling back to "Player".
pub(crate) fn menu_player_name(world: &World) -> String {
    world
        .get_resource::<MenuFields>()
        .map(|f| f.player_name.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Player".to_string())
}

fn spawn_inprocess_bot(world: &mut World, format: MatchFormat) {
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
    let sink_for_match = Arc::clone(&sink);
    // Build the state first so we can size the occupant list to its
    // seat count. Two-player formats (Modern / Cube / SoS) end up with
    // [Human, Bot]; four-player formats (Commander) get
    // [Human, Bot, Bot, Bot]. Pre-fix this was hardcoded to two and
    // panicked when a Commander match brought 4 seats.
    //
    // Audit override: if the menu wrote a card name into `AuditTarget`
    // before entering InGame, build a tailored two-seat audit state
    // around that card instead of the chosen format.
    //
    // Draft override: if the user just finished an 8-seat draft, the
    // draft plugin will have written a `DraftedDecks` resource. Take
    // and consume it so a follow-up rematch falls back to the format's
    // random path. Drafted decks beat the audit + format paths.
    let audit_card: Option<String> = world
        .get_resource::<crate::audit::AuditTarget>()
        .and_then(|t| t.0.clone());
    let drafted: Option<DraftedDecks> = world
        .get_resource_mut::<DraftedDecks>()
        .map(|r| (*r).clone());
    if drafted.is_some() {
        world.remove_resource::<DraftedDecks>();
    }
    let human_name = menu_player_name(world);
    let imported: Option<ImportedDeck> = world.remove_resource::<ImportedDeck>();
    let state = if let Some(deck) = imported {
        // Imported decklist vs. the stock Modern bot deck.
        crabomination::draft::build_draft_match_state(
            deck.0,
            crabomination::demo::brg_combo_deck().to_vec(),
            human_name.clone(),
            "Bot".into(),
        )
    } else if let Some(decks) = drafted {
        crabomination::draft::build_draft_match_state(
            decks.player_deck,
            decks.opponent_deck,
            human_name,
            decks.opponent_label,
        )
    } else {
        let mut state = match audit_card.as_deref() {
            Some(name) => crate::audit::build_audit_state(name).unwrap_or_else(|| {
                eprintln!("audit: unknown card '{name}', falling back to {:?}", format);
                format.build_state()
            }),
            None => format.build_state(),
        };
        name_seats(&mut state, &human_name, "Bot");
        state
    };
    let n_seats = state.players.len();
    let mut occupants: Vec<SeatOccupant> = Vec::with_capacity(n_seats);
    occupants.push(SeatOccupant::Human(server_seat));
    for _ in 1..n_seats {
        occupants.push(SeatOccupant::Bot(Box::new(RandomBot::new())));
    }
    std::thread::spawn(move || {
        run_match_full(state, occupants, vec![], Some(sink_for_match));
    });
    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    world.insert_resource(LatestSnapshot(sink));
}

/// Shared handle to the authoritative engine state for the running
/// in-process match. Read-locked by the export prompt to embed both a
/// structured `GameSnapshot` and the full `GameState` JSON in saved
/// debug exports. `None`/empty when no in-process match is running, or
/// when the local client is connected to a remote (TCP) match — in
/// those cases the engine state lives on a different machine.
#[derive(Resource, Default, Clone)]
pub struct LatestSnapshot(pub SnapshotSink);

impl LatestSnapshot {
    pub fn read(&self) -> SnapshotSinkState {
        self.0.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

/// Spectate-only mode: both seats are RandomBots, the local UI hooks
/// up a spectator channel that mirrors seat 0's projection. Any actions
/// the local UI submits are silently dropped server-side.
fn spawn_spectate_bots(world: &mut World, format: MatchFormat) {
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
    let sink_for_match = Arc::clone(&sink);
    // Size the bot list to the format's seat count — Commander brings
    // 4 seats, the other formats bring 2.
    let mut state = format.build_state();
    for (i, p) in state.players.iter_mut().enumerate() {
        p.name = format!("Bot {}", i + 1);
    }
    let n_seats = state.players.len();
    let occupants: Vec<SeatOccupant> = (0..n_seats)
        .map(|_| SeatOccupant::Bot(Box::new(RandomBot::new())))
        .collect();
    std::thread::spawn(move || {
        run_match_full(state, occupants, vec![server_seat], Some(sink_for_match));
    });
    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    world.insert_resource(LatestSnapshot(sink));
}

fn spawn_host_lan(world: &mut World, port: u16, format: MatchFormat) -> std::io::Result<()> {
    let bind = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind)?;
    let (server_seat0, ClientChannel { tx, rx }) = seat_pair();
    let host_name = menu_player_name(world);

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
        let mut state = format.build_state();
        // Direct host mode has no lobby handshake to learn the joiner's
        // name, so seat 1 gets a generic label rather than "P1".
        name_seats(&mut state, &host_name, "Opponent");
        run_match(
            state,
            vec![
                SeatOccupant::Human(server_seat0),
                SeatOccupant::Human(server_seat1),
            ],
        );
        eprintln!("host: match ended");
    });

    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    Ok(())
}

/// Load a previously exported debug state file. If the export carries a
/// full `GameSnapshot` (the new format), the snapshot is restored into a
/// real `GameState` and run as an in-process match against a `RandomBot`
/// — meaning the user can keep playing from the saved board, which is
/// the whole point of this debug workflow.
///
/// Older view-only exports fall back to read-only inspection: the
/// `ClientView` is seeded into `CurrentView` directly, no `NetOutbox` is
/// installed (so the input handler bails out), and the player can poke
/// around the board but not advance it.
fn spawn_loaded_debug_state(world: &mut World, path: &std::path::Path) -> std::io::Result<()> {
    let export = crate::debug_export::load_debug_export(path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let log_prefix = format!(
        "Loaded debug state from {}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("?")
    );
    let bug_note = (!export.message.is_empty())
        .then(|| format!("  bug note: {}", export.message));

    // Prefer the bit-exact full GameState when present; fall back to
    // the schema-stable GameSnapshot (with the trigger-stack caveat);
    // fall back again to view-only inspection for legacy exports.
    let restored_with_dropped: Option<(GameState, usize, &'static str)> =
        if let Some(full) = export.full_state {
            Some((full, 0, "full GameState"))
        } else if let Some(snap) = export.snapshot.clone() {
            let dropped = snap.dropped_triggers;
            match snap.restore() {
                Ok(state) => Some((state, dropped, "snapshot")),
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("restore snapshot: {e}"),
                    ));
                }
            }
        } else {
            None
        };

    if let Some((restored, dropped_triggers, source_label)) = restored_with_dropped {
        // Spawn a live match seated as the original viewer; the second
        // seat is filled by a RandomBot so the user can play forward.
        let viewer_seat = export.view.your_seat;
        let bot_seat = if viewer_seat == 0 { 1 } else { 0 };
        let (server_seat, ClientChannel { tx, rx }) = seat_pair();
        let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
        let sink_for_match = Arc::clone(&sink);
        let occupants = if viewer_seat == 0 {
            vec![
                SeatOccupant::Human(server_seat),
                SeatOccupant::Bot(Box::new(RandomBot::new())),
            ]
        } else {
            vec![
                SeatOccupant::Bot(Box::new(RandomBot::new())),
                SeatOccupant::Human(server_seat),
            ]
        };
        let _ = bot_seat;
        std::thread::spawn(move || {
            run_match_full(restored, occupants, vec![], Some(sink_for_match));
        });
        world.insert_resource(NetOutbox::new(tx));
        world.insert_resource(NetInbox(Mutex::new(rx)));
        world.insert_resource(LatestSnapshot(sink));
        if let Some(mut log) = world.get_resource_mut::<crate::game::GameLog>() {
            log.push(log_prefix);
            if let Some(note) = bug_note {
                log.push(note);
            }
            log.push(format!(
                "Restored from {source_label} — playable from this state"
            ));
            if dropped_triggers > 0 {
                log.push(format!(
                    "  warning: {dropped_triggers} trigger(s) on the original stack were dropped"
                ));
            }
        }
        Ok(())
    } else {
        // Legacy view-only export: read-only inspection mode.
        if let Some(mut cv) = world.get_resource_mut::<crate::net_plugin::CurrentView>() {
            cv.0 = Some(export.view.clone());
        }
        if let Some(mut seat) = world.get_resource_mut::<crate::net_plugin::OurSeat>() {
            seat.0 = export.view.your_seat;
        }
        if let Some(mut log) = world.get_resource_mut::<crate::game::GameLog>() {
            log.push(log_prefix);
            if let Some(note) = bug_note {
                log.push(note);
            }
            log.push("View-only inspection (no full snapshot in this file)");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::game_over::ActiveMatchFormat;

    /// Regression: on the lobby path the net session is already installed
    /// (`NetOutbox` present), so `start_net_session_from_menu` skips re-spawning
    /// — but it must STILL insert `ActiveMatchFormat`, which the in-game
    /// game-over systems read as a required resource every frame. Skipping it
    /// used to crash the instant a lobby match started (e.g. on "Add Bot").
    #[test]
    fn lobby_path_still_inserts_active_match_format() {
        let mut world = World::new();
        // Simulate the lobby flow: a live connection is already installed.
        let (tx, _rx) = std::sync::mpsc::channel();
        world.insert_resource(NetOutbox::new(tx));

        start_net_session_from_menu(&mut world);

        assert!(
            world.contains_resource::<ActiveMatchFormat>(),
            "ActiveMatchFormat must be present on the lobby path or the \
             game-over systems panic on the first in-game frame",
        );
        assert!(
            world.contains_resource::<crate::systems::game_over::ActiveMatchKind>(),
        );
        // The live session must be left untouched (not clobbered by a respawn).
        assert!(world.contains_resource::<NetOutbox>());
    }
}

