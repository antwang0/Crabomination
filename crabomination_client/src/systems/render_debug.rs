//! **TEMPORARY** live tonemapper + colour-grading scrubber.
//!
//! Opens with the `\` (Backslash) key. Lets you cycle the main camera's
//! `Tonemapping` and drag sliders for the `ColorGrading` post-saturation,
//! section contrast, and exposure — applied to the 3-D camera in real time —
//! so the "cards look washed out" grading can be dialled in by eye instead of
//! guessed at. The `RenderDebug` resource defaults match the committed
//! `scene_color_grading()` baseline, so with the panel untouched the look is
//! unchanged.
//!
//! Remove this module (and its registration in `main.rs`) once the values are
//! settled and baked into `scene_color_grading()` / the camera tonemapping.

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection};

use crate::MainCamera;
use crate::theme::{self, UiFonts};

/// The tonemappers offered in the selector, in cycle order.
const TONEMAPPERS: &[(Tonemapping, &str)] = &[
    (Tonemapping::None, "None"),
    (Tonemapping::Reinhard, "Reinhard"),
    (Tonemapping::ReinhardLuminance, "ReinhardLuminance"),
    (Tonemapping::AcesFitted, "AcesFitted"),
    (Tonemapping::AgX, "AgX"),
    (Tonemapping::SomewhatBoringDisplayTransform, "SomewhatBoring"),
    (Tonemapping::TonyMcMapface, "TonyMcMapface"),
    (Tonemapping::BlenderFilmic, "BlenderFilmic"),
    (Tonemapping::KhronosPbrNeutral, "KhronosPbrNeutral"),
];

const TRACK_W: f32 = 180.0;
const HANDLE_W: f32 = 12.0;

/// Live render-debug state. Defaults mirror `scene_color_grading()`.
#[derive(Resource)]
pub struct RenderDebug {
    open: bool,
    tonemap: usize,
    saturation: f32,
    contrast: f32,
    lift: f32,
    exposure: f32,
}

impl Default for RenderDebug {
    fn default() -> Self {
        Self {
            open: false,
            // Index of TonyMcMapface (the engine default the baseline uses).
            tonemap: 6,
            // Mirrors `scene_color_grading()` — 0.9 read better than the
            // washed-out look (your tuning).
            saturation: 0.9,
            contrast: 1.0,
            lift: 0.0,
            exposure: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Param {
    Saturation,
    Contrast,
    /// Black level — additive `ColorGradingSection::lift`. The real de-wash
    /// knob: a *negative* lift pulls blacks down and restores punch. Far more
    /// visible than section `contrast`, which Bevy applies pre-tonemap in log
    /// space (a weak lever for SDR card art).
    Lift,
    Exposure,
}

impl Param {
    fn range(self) -> (f32, f32) {
        match self {
            Param::Saturation => (0.0, 2.0),
            Param::Contrast => (0.5, 1.5),
            Param::Lift => (-0.3, 0.3),
            Param::Exposure => (-2.0, 2.0),
        }
    }
    fn get(self, rd: &RenderDebug) -> f32 {
        match self {
            Param::Saturation => rd.saturation,
            Param::Contrast => rd.contrast,
            Param::Lift => rd.lift,
            Param::Exposure => rd.exposure,
        }
    }
    fn set(self, rd: &mut RenderDebug, v: f32) {
        let (lo, hi) = self.range();
        let v = v.clamp(lo, hi);
        match self {
            Param::Saturation => rd.saturation = v,
            Param::Contrast => rd.contrast = v,
            Param::Lift => rd.lift = v,
            Param::Exposure => rd.exposure = v,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Param::Saturation => "Saturation",
            Param::Contrast => "Contrast",
            Param::Lift => "Black level",
            Param::Exposure => "Exposure",
        }
    }
    /// Fraction in 0..=1 for the current value.
    fn frac(self, rd: &RenderDebug) -> f32 {
        let (lo, hi) = self.range();
        ((self.get(rd) - lo) / (hi - lo)).clamp(0.0, 1.0)
    }
}

// ── Markers ──────────────────────────────────────────────────────────────────

#[derive(Component)]
pub(crate) struct RenderDebugPanel;
#[derive(Component)]
pub(crate) struct SliderTrack(Param);
#[derive(Component)]
pub(crate) struct SliderHandle(Param);
#[derive(Component)]
pub(crate) struct ValueText(Param);
#[derive(Component)]
pub(crate) struct TonemapText;
#[derive(Component)]
pub(crate) enum DebugButton {
    TonemapPrev,
    TonemapNext,
    Reset,
}

// ── Toggle + (de)spawn ───────────────────────────────────────────────────────

pub fn toggle_render_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut rd: ResMut<RenderDebug>,
    panel: Query<Entity, With<RenderDebugPanel>>,
    fonts: Res<UiFonts>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::Backslash) {
        return;
    }
    rd.open = !rd.open;
    if rd.open {
        spawn_panel(&mut commands, &fonts, &rd);
    } else {
        for e in &panel {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_panel(commands: &mut Commands, fonts: &UiFonts, rd: &RenderDebug) {
    let tf = |s: f32| fonts.tf(s);
    commands
        .spawn((
            RenderDebugPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
            GlobalZIndex(1000),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("RENDER DEBUG  (\\ to close)"),
                tf(13.0),
                TextColor(theme::ACCENT_GOLD),
            ));

            // Tonemapper selector row: [◀]  Name  [▶]
            p.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },))
                .with_children(|row| {
                    nav_button(row, fonts, "◀", DebugButton::TonemapPrev);
                    row.spawn((
                        Text::new(TONEMAPPERS[rd.tonemap].1),
                        tf(13.0),
                        TextColor(theme::TEXT_BODY),
                        TonemapText,
                        Node {
                            width: Val::Px(180.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ));
                    nav_button(row, fonts, "▶", DebugButton::TonemapNext);
                });

            slider_row(p, fonts, Param::Saturation, rd);
            slider_row(p, fonts, Param::Contrast, rd);
            slider_row(p, fonts, Param::Lift, rd);
            slider_row(p, fonts, Param::Exposure, rd);

            // Reset button.
            p.spawn((
                Button,
                DebugButton::Reset,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    align_self: AlignSelf::FlexStart,
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                theme::HoverTint::new(theme::BUTTON_NEUTRAL_BG),
            ))
            .with_children(|b| {
                b.spawn((Text::new("Reset"), tf(12.0), TextColor(theme::TEXT_BODY)));
            });
        });
}

fn nav_button(parent: &mut ChildSpawnerCommands, fonts: &UiFonts, glyph: &str, which: DebugButton) {
    parent
        .spawn((
            Button,
            which,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(theme::BUTTON_TERTIARY_BG),
            theme::HoverTint::new(theme::BUTTON_TERTIARY_BG),
        ))
        .with_children(|b| {
            b.spawn((Text::new(glyph), fonts.tf(13.0), TextColor(theme::TEXT_BODY)));
        });
}

fn slider_row(parent: &mut ChildSpawnerCommands, fonts: &UiFonts, param: Param, rd: &RenderDebug) {
    let tf = |s: f32| fonts.tf(s);
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new(param.label()),
                tf(12.0),
                TextColor(theme::TEXT_SECONDARY),
                Node { width: Val::Px(80.0), ..default() },
            ));
            // Track (draggable) with a handle child.
            row.spawn((
                SliderTrack(param),
                Node {
                    width: Val::Px(TRACK_W),
                    height: Val::Px(14.0),
                    border_radius: BorderRadius::all(Val::Px(7.0)),
                    ..default()
                },
                BackgroundColor(theme::FIELD_BG),
            ))
            .with_children(|track| {
                track.spawn((
                    SliderHandle(param),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(param.frac(rd) * (TRACK_W - HANDLE_W)),
                        width: Val::Px(HANDLE_W),
                        height: Val::Px(14.0),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::ACCENT_BLUE),
                    // The handle ignores picking so drags hit the track.
                    Pickable::IGNORE,
                ));
            })
            .observe(on_slider_drag);
            row.spawn((
                Text::new(format!("{:.2}", param.get(rd))),
                tf(12.0),
                TextColor(theme::TEXT_BODY),
                ValueText(param),
                Node { width: Val::Px(44.0), ..default() },
            ));
        });
}

// ── Interaction ──────────────────────────────────────────────────────────────

fn on_slider_drag(
    ev: On<Pointer<Drag>>,
    tracks: Query<&SliderTrack>,
    mut rd: ResMut<RenderDebug>,
) {
    let Ok(track) = tracks.get(ev.entity) else { return };
    let param = track.0;
    let (lo, hi) = param.range();
    // Drag delta is in screen pixels; map across the usable track width.
    let step = ev.event.delta.x / (TRACK_W - HANDLE_W) * (hi - lo);
    let v = param.get(&rd) + step;
    param.set(&mut rd, v);
}

pub fn handle_debug_buttons(
    buttons: Query<(&Interaction, &DebugButton), Changed<Interaction>>,
    mut rd: ResMut<RenderDebug>,
) {
    for (interaction, which) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match which {
            DebugButton::TonemapPrev => {
                rd.tonemap = (rd.tonemap + TONEMAPPERS.len() - 1) % TONEMAPPERS.len();
            }
            DebugButton::TonemapNext => {
                rd.tonemap = (rd.tonemap + 1) % TONEMAPPERS.len();
            }
            DebugButton::Reset => {
                let def = RenderDebug::default();
                rd.tonemap = def.tonemap;
                rd.saturation = def.saturation;
                rd.contrast = def.contrast;
                rd.exposure = def.exposure;
            }
        }
    }
}

/// Mirror `RenderDebug` into the panel widgets (handle positions, value
/// readouts, tonemapper name) whenever it changes while open.
pub fn sync_render_debug_panel(
    rd: Res<RenderDebug>,
    mut handles: Query<(&SliderHandle, &mut Node)>,
    mut values: Query<(&ValueText, &mut Text), Without<TonemapText>>,
    mut tonemap_text: Query<&mut Text, With<TonemapText>>,
) {
    if !rd.is_changed() {
        return;
    }
    for (h, mut node) in &mut handles {
        node.left = Val::Px(h.0.frac(&rd) * (TRACK_W - HANDLE_W));
    }
    for (v, mut text) in &mut values {
        text.0 = format!("{:.2}", v.0.get(&rd));
    }
    if let Ok(mut t) = tonemap_text.single_mut() {
        t.0 = TONEMAPPERS[rd.tonemap].1.to_string();
    }
}

/// Apply `RenderDebug` to the main camera's `Tonemapping` + `ColorGrading`
/// whenever it changes (including the initial insert, so the baseline is set).
pub fn apply_render_debug(
    rd: Res<RenderDebug>,
    cam: Query<Entity, With<MainCamera>>,
    mut commands: Commands,
) {
    if !rd.is_changed() {
        return;
    }
    let Ok(cam) = cam.single() else { return };
    let grading = ColorGrading::with_identical_sections(
        ColorGradingGlobal {
            exposure: rd.exposure,
            post_saturation: rd.saturation,
            ..default()
        },
        ColorGradingSection {
            contrast: rd.contrast,
            lift: rd.lift,
            ..default()
        },
    );
    commands
        .entity(cam)
        .insert((TONEMAPPERS[rd.tonemap].0, grading));
}
