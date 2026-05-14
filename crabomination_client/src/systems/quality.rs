//! Quality preset selector panel + animation-speed slider (bottom-right HUD).

use bevy::prelude::*;

use crate::render_quality::{ChangeQuality, RenderQuality};
use crate::systems::animate::{AnimationSpeed, ANIM_SPEED_MAX, ANIM_SPEED_MIN};

const QUALITY_BTN_ACTIVE: Color = Color::srgb(0.15, 0.45, 0.15);
const QUALITY_BTN_INACTIVE: Color = Color::srgb(0.12, 0.12, 0.18);

const SPEED_SLIDER_WIDTH: f32 = 180.0;
const SPEED_SLIDER_HEIGHT: f32 = 16.0;
const SPEED_TRACK_BG: Color = Color::srgb(0.10, 0.10, 0.14);
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

pub fn setup_quality_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    quality: Res<RenderQuality>,
    speed: Res<AnimationSpeed>,
) {
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
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(6.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            crate::systems::game_ui::InGameRoot,
        ))
        .with_children(|p| {
            // ── Animation speed (top section) ────────────────────────
            p.spawn((
                Text::new(format_speed(speed.0)),
                tf(11.0),
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
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
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|p| {
                for q in RenderQuality::ALL {
                    let bg = if q == *quality { QUALITY_BTN_ACTIVE } else { QUALITY_BTN_INACTIVE };
                    p.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        Button,
                        QualityButton(q),
                    ))
                    .with_children(|p| {
                        p.spawn((Text::new(q.label()), tf(12.0), TextColor(Color::WHITE)));
                    });
                }
            });
        });
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
                BackgroundColor(QUALITY_BTN_INACTIVE)
            };
        }
    }
}
