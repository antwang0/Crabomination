//! "Describe the bug" prompt that appears after the user hits the
//! Export State HUD button (or `X`). While active it captures keyboard
//! input so the user can type a message; pressing Enter writes the
//! `<turn>-<step>-<unix>.json` file via [`crate::debug_export`], pressing
//! Escape cancels.

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::debug_export;
use crate::game::GameLog;
use crate::net_plugin::CurrentView;

/// Drives the export prompt's lifecycle. `active` toggles the modal;
/// `message` accumulates typed characters.
#[derive(Resource, Default)]
pub struct ExportPromptState {
    pub active: bool,
    pub message: String,
}

/// Root entity of the prompt's UI tree. Despawned in
/// [`teardown_export_prompt`] when the prompt closes.
#[derive(Component)]
pub struct ExportPromptRoot;

/// Marks the text node that displays the live message buffer plus a
/// trailing cursor.
#[derive(Component)]
pub struct ExportPromptText;

/// Open the prompt. Public so the HUD's input handler can call it on
/// button click / `X` keypress.
pub fn open_export_prompt(state: &mut ExportPromptState) {
    state.active = true;
    state.message.clear();
}

const MAX_MESSAGE_LEN: usize = 240;

/// Capture keyboard input and either save (Enter) or cancel (Escape) the
/// prompt. Runs every frame while `ExportPromptState.active` is true; the
/// caller's input handlers should bail out early in that same frame to
/// avoid double-binding (typing "x" mid-message must not re-open the
/// prompt, etc.).
pub fn handle_export_prompt_input(
    mut state: ResMut<ExportPromptState>,
    mut events: MessageReader<KeyboardInput>,
    view: Res<CurrentView>,
    snapshot: Option<Res<crate::menu::LatestSnapshot>>,
    mut log: ResMut<GameLog>,
) {
    if !state.active {
        // Drain otherwise-stale events so the next session starts clean.
        events.clear();
        return;
    }
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Escape => {
                state.active = false;
                state.message.clear();
                log.push("Export cancelled");
                return;
            }
            Key::Enter => {
                let Some(cv) = view.0.as_ref() else {
                    log.push("Export failed: no game state to capture");
                    state.active = false;
                    return;
                };
                let sink = snapshot.as_ref().map(|s| s.read()).unwrap_or_default();
                let had_snapshot = sink.snapshot.is_some();
                let had_full = sink.full_state_json.is_some();
                let full_state = sink
                    .full_state_json
                    .as_deref()
                    .and_then(|json| serde_json::from_str(json).ok());
                match debug_export::export_full(cv, &state.message, sink.snapshot, full_state) {
                    Ok(path) => {
                        let suffix = match (had_full, had_snapshot) {
                            (true, _) => " (full GameState + snapshot)",
                            (false, true) => " (snapshot only)",
                            _ => " (view only)",
                        };
                        log.push(format!("Exported state → {}{suffix}", path.display()));
                    }
                    Err(e) => log.push(format!("Export failed: {e}")),
                }
                state.active = false;
                state.message.clear();
                return;
            }
            Key::Backspace => {
                state.message.pop();
            }
            Key::Space if state.message.len() < MAX_MESSAGE_LEN => {
                state.message.push(' ');
            }
            Key::Character(s) => {
                for ch in s.chars() {
                    if state.message.len() >= MAX_MESSAGE_LEN {
                        break;
                    }
                    if !ch.is_control() {
                        state.message.push(ch);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Spawn the modal UI on activation; despawn on deactivation. Watches
/// `ExportPromptState` for changes so this only does work on transitions.
pub fn sync_export_prompt_ui(
    state: Res<ExportPromptState>,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<ExportPromptRoot>>,
    mut text_q: Query<&mut Text, With<ExportPromptText>>,
    mut commands: Commands,
) {
    if state.active && existing.is_empty() {
        let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ExportPromptRoot,
            ))
            .with_children(|root| {
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(28.0)),
                        row_gap: Val::Px(14.0),
                        align_items: AlignItems::Center,
                        min_width: Val::Px(420.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.12, 0.97)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Describe the bug"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.55)),
                    ));
                    p.spawn((
                        Text::new(format!("{}_", state.message)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            min_width: Val::Px(380.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 1.0)),
                        ExportPromptText,
                    ));
                    p.spawn((
                        Text::new("Enter to save · Esc to cancel"),
                        TextFont {
                            font,
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));
                });
            });
    } else if !state.active {
        for e in &existing {
            commands.entity(e).despawn();
        }
    }
    if state.is_changed()
        && let Ok(mut t) = text_q.single_mut()
    {
        t.0 = format!("{}_", state.message);
    }
}
