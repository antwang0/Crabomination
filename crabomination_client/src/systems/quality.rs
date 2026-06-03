//! Settings menu — animation-speed slider + render-quality preset
//! toggles. Hidden by default; toggled by Esc via
//! [`handle_settings_toggle`]. The on-screen Pass/End/Next/Export
//! buttons live in `game_ui` and stay out of this menu.

use bevy::prelude::*;

use crate::render_quality::{ChangeQuality, RenderQuality};
use crate::systems::animate::{AnimationSpeed, ANIM_SPEED_MAX, ANIM_SPEED_MIN};
use crate::theme::{self, HoverTint, UiFonts};

/// Selected quality tier (green = "active").
const QUALITY_BTN_ACTIVE: Color = Color::srgb(0.15, 0.45, 0.15);

const SPEED_SLIDER_WIDTH: f32 = 220.0;
const SPEED_SLIDER_HEIGHT: f32 = 16.0;
const SPEED_TRACK_BG: Color = Color::srgb(0.10, 0.10, 0.14);
/// Blue fill bar inside the speed-slider track.
const SPEED_TRACK_FILL: Color = Color::srgb(0.30, 0.55, 0.85);

#[derive(Component)]
pub struct QualityButton(pub RenderQuality);

/// Marker on the clickable speed-slider track.
#[derive(Component)]
pub struct SpeedSliderTrack;

/// Marker on the fill bar inside the speed slider (its width tracks
/// the current animation speed on a log scale).
#[derive(Component)]
pub struct SpeedSliderFill;

/// Marker on the "Speed: Nx" text label above the slider.
#[derive(Component)]
pub struct SpeedSliderLabel;

/// Whether the settings modal is open. Toggled by Esc — see
/// `handle_settings_toggle` in this module.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct SettingsOpen(pub bool);

/// Set to `true` for one frame by `handle_settings_toggle` when it
/// consumed an Esc press, so the keyboard-cursor Esc handler can skip
/// itself and the user gets a single, predictable action per press.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct EscConsumed(pub bool);

/// Root entity of the settings modal, used by `sync_settings_visibility`
/// to flip the panel between `Display::Flex` and `Display::None`.
#[derive(Component)]
pub struct SettingsMenuRoot;

#[derive(Component)]
pub struct SettingsCloseButton;

/// "Leave Game" button in the settings menu — disconnects from the
/// server / ends the local match and returns to the main menu. See
/// [`handle_leave_game_button`].
#[derive(Component)]
pub struct LeaveGameButton;

pub fn setup_quality_panel(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    quality: Res<RenderQuality>,
    speed: Res<AnimationSpeed>,
) {
    let tf = |size: f32| ui_fonts.tf(size);

    // Full-screen scrim + centered modal. Display::None by default;
    // `sync_settings_visibility` flips it when `SettingsOpen` toggles.
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
                display: Display::None,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            SettingsMenuRoot,
            crate::systems::game_ui::InGameRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(12.0),
                    min_width: Val::Px(320.0),
                    align_items: AlignItems::Stretch,
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|p| {
            // Title.
            p.spawn((
                Text::new("Settings"),
                tf(20.0),
                TextColor(theme::ACCENT_GOLD),
            ));
            // ── Animation speed (top section) ────────────────────────
            p.spawn((
                Text::new(format_speed(speed.0)),
                tf(11.0),
                TextColor(theme::TEXT_SECONDARY),
                SpeedSliderLabel,
            ));
            // The track itself is a clickable Button so Bevy reports
            // `Interaction::Pressed` when the user mouse-downs on it.
            p.spawn((
                Node {
                    width: Val::Px(SPEED_SLIDER_WIDTH),
                    height: Val::Px(SPEED_SLIDER_HEIGHT),
                    align_items: AlignItems::Stretch,
                        ..default()
                },
                BackgroundColor(SPEED_TRACK_BG),
                Button,
                SpeedSliderTrack,
            ))
            .with_children(|track| {
                let frac = speed_to_fraction(speed.0);
                track.spawn((
                    Node {
                        width: Val::Percent(frac * 100.0),
                        height: Val::Percent(100.0),
                                ..default()
                    },
                    BackgroundColor(SPEED_TRACK_FILL),
                    SpeedSliderFill,
                    Pickable::IGNORE,
                ));
            });

            // ── Render quality (bottom section) ──────────────────────
            p.spawn((
                Text::new("Quality"),
                tf(11.0),
                TextColor(theme::TEXT_SECONDARY),
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|p| {
                for q in RenderQuality::ALL {
                    let bg = if q == *quality { QUALITY_BTN_ACTIVE } else { theme::BUTTON_NEUTRAL_BG };
                    p.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                            border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                            ..default()
                        },
                        BackgroundColor(bg),
                        Button,
                        QualityButton(q),
                    ))
                    .with_children(|p| {
                        p.spawn((Text::new(q.label()), tf(12.0), TextColor(theme::TEXT_PRIMARY)));
                    });
                }
            });

            // Bottom row: "Leave Game" on the left (disconnect + return
            // to the main menu) and "Close" on the right. Esc also
            // closes; the explicit button is useful when browsing with
            // the mouse only.
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_DANGER_BG),
                    HoverTint::new(theme::BUTTON_DANGER_BG),
                    LeaveGameButton,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Leave Game"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                        Pickable::IGNORE,
                    ));
                });
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    SettingsCloseButton,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Close (Esc)"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                        Pickable::IGNORE,
                    ));
                });
            });
        });
    });
}

// ─── Settings open/close plumbing ────────────────────────────────────

/// Reset `EscConsumed` to false at the very start of each frame so
/// the Esc-precedence chain has a clean slate to write to.
pub fn reset_esc_consumed(mut consumed: ResMut<EscConsumed>) {
    consumed.0 = false;
}

/// Toggle the settings modal on Esc. Precedence:
/// * Esc while the modal is open → close (consumes Esc).
/// * Esc while targeting / blocking / a cursor selection is active → no-op
///   (those handlers own the press).
/// * Otherwise → open the modal (consumes Esc).
///
/// Runs before the keyboard-cursor input system so that opening the
/// modal in the same frame doesn't also clear the cursor selection.
#[allow(clippy::too_many_arguments)]
pub fn handle_settings_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    cursor: Res<crate::systems::kb_cursor::KeyboardCursor>,
    targeting: Res<crate::game::TargetingState>,
    blocking: Res<crate::game::BlockingState>,
    export_prompt: Res<crate::systems::export_prompt::ExportPromptState>,
    debug_console: Res<crate::systems::debug_console::DebugConsoleState>,
    alt_cast: Res<crate::game::AltCastState>,
    close_btn: Query<&Interaction, (Changed<Interaction>, With<SettingsCloseButton>)>,
    mut settings: ResMut<SettingsOpen>,
    mut consumed: ResMut<EscConsumed>,
) {
    // Close button always closes.
    if close_btn.iter().any(|i| *i == Interaction::Pressed) {
        settings.0 = false;
        return;
    }
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    if export_prompt.active || debug_console.card_input_focused {
        return;
    }
    if settings.0 {
        settings.0 = false;
        consumed.0 = true;
        return;
    }
    // Don't open settings on top of an active modal/selection — those
    // handlers want this Esc press.
    if targeting.active
        || blocking.selected_blocker.is_some()
        || cursor.selection.is_some()
        || alt_cast.pending.is_some()
    {
        return;
    }
    settings.0 = true;
    consumed.0 = true;
}

/// Show / hide the settings modal whenever `SettingsOpen` flips.
pub fn sync_settings_visibility(
    settings: Res<SettingsOpen>,
    mut q: Query<&mut Node, With<SettingsMenuRoot>>,
) {
    if !settings.is_changed() {
        return;
    }
    let Ok(mut node) = q.single_mut() else { return };
    node.display = if settings.0 { Display::Flex } else { Display::None };
}

/// Map a speed value in `[ANIM_SPEED_MIN, ANIM_SPEED_MAX]` to a 0..1
/// fraction on a log scale. 1× sits at the midpoint of the range.
fn speed_to_fraction(speed: f32) -> f32 {
    let s = speed.clamp(ANIM_SPEED_MIN, ANIM_SPEED_MAX);
    let log_min = ANIM_SPEED_MIN.log10();
    let log_max = ANIM_SPEED_MAX.log10();
    ((s.log10() - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
}

fn fraction_to_speed(frac: f32) -> f32 {
    let log_min = ANIM_SPEED_MIN.log10();
    let log_max = ANIM_SPEED_MAX.log10();
    10f32.powf(log_min + frac.clamp(0.0, 1.0) * (log_max - log_min))
}

fn format_speed(speed: f32) -> String {
    if (0.95..=1.05).contains(&speed) {
        "Speed: 1.0×".into()
    } else if speed < 1.0 {
        format!("Speed: {:.2}×", speed)
    } else {
        format!("Speed: {:.1}×", speed)
    }
}

/// Drive the speed slider via raw cursor + mouse-button state instead of
/// Bevy's `Interaction::Pressed`, so dragging *off* the track keeps
/// updating the slider (the natural feel for a draggable). Picking up
/// `Interaction` only registered the initial frame the cursor was over
/// the track and stalled the moment you dragged past either edge.
pub fn handle_speed_slider(
    track_q: Query<(&ComputedNode, &GlobalTransform), With<SpeedSliderTrack>>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut dragging: Local<bool>,
    mut speed: ResMut<AnimationSpeed>,
) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((node, xform)) = track_q.single() else { return };

    let center = xform.translation();
    let size = node.size();
    let half = size * 0.5;
    let left = center.x - half.x;
    let right = center.x + half.x;
    let top = center.y - half.y;
    let bottom = center.y + half.y;
    let over = cursor.x >= left && cursor.x <= right && cursor.y >= top && cursor.y <= bottom;

    if !mouse.pressed(MouseButton::Left) {
        *dragging = false;
        return;
    }
    if mouse.just_pressed(MouseButton::Left) && over {
        *dragging = true;
    }
    if !*dragging {
        return;
    }

    let frac = ((cursor.x - left) / (right - left).max(1.0)).clamp(0.0, 1.0);
    let new_speed = fraction_to_speed(frac);
    // Snap to 1.0× within ~3% of centre — small slop avoids fiddly precision.
    let center_frac = speed_to_fraction(1.0);
    speed.0 = if (frac - center_frac).abs() < 0.03 { 1.0 } else { new_speed };
}

/// Refresh the slider's fill width and the "Speed: Nx" label whenever
/// the `AnimationSpeed` resource changes (slider drag, keyboard hotkey,
/// or any other writer).
pub fn update_speed_slider_visuals(
    speed: Res<AnimationSpeed>,
    mut fill_q: Query<&mut Node, With<SpeedSliderFill>>,
    mut label_q: Query<&mut Text, With<SpeedSliderLabel>>,
) {
    if !speed.is_changed() {
        return;
    }
    let frac = speed_to_fraction(speed.0);
    for mut node in &mut fill_q {
        node.width = Val::Percent(frac * 100.0);
    }
    for mut text in &mut label_q {
        text.0 = format_speed(speed.0);
    }
}

/// "Leave Game" button — close the settings menu and return to the
/// main menu. The live network session (TCP socket / channel
/// resources) is torn down by
/// [`crate::net_plugin::teardown_net_session`] on `OnExit(InGame)`.
/// Clearing [`PendingNetMode`] mirrors the game-over "New Game" path so
/// the next menu visit starts without a stale queued mode.
pub fn handle_leave_game_button(
    leave_q: Query<&Interaction, (Changed<Interaction>, With<LeaveGameButton>)>,
    mut settings: ResMut<SettingsOpen>,
    mut pending: ResMut<crate::menu::PendingNetMode>,
    mut next_state: ResMut<NextState<crate::menu::AppState>>,
) {
    if !leave_q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    settings.0 = false;
    pending.0 = None;
    next_state.set(crate::menu::AppState::Menu);
}

pub fn handle_quality_buttons(
    mut buttons: Query<(&Interaction, &QualityButton, &mut BackgroundColor)>,
    quality: Res<RenderQuality>,
    mut messages: MessageWriter<ChangeQuality>,
) {
    let pressed = buttons
        .iter()
        .find(|(i, _, _)| **i == Interaction::Pressed)
        .map(|(_, btn, _)| btn.0);

    if let Some(q) = pressed {
        messages.write(ChangeQuality(q));
    }

    if pressed.is_some() || quality.is_changed() {
        let active = pressed.unwrap_or(*quality);
        for (_, btn, mut bg) in buttons.iter_mut() {
            *bg = if btn.0 == active {
                BackgroundColor(QUALITY_BTN_ACTIVE)
            } else {
                BackgroundColor(theme::BUTTON_NEUTRAL_BG)
            };
        }
    }
}
