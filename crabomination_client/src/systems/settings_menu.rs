//! Main-menu Settings panel — window mode / resolution, render quality,
//! animation speed, and hand sorting, all persisted through
//! `config::ConfigStore` so they survive restarts.
//!
//! Lives in `AppState::Menu` (the in-game Esc panel keeps its quality +
//! speed sliders for mid-match tweaks; both write the same resources, and
//! the persistence systems mirror those into `config.toml`).

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use crate::config::{ConfigStore, WindowModeCfg};
use crate::render_quality::{ChangeQuality, RenderQuality};
use crate::systems::animate::AnimationSpeed;
use crate::theme::{self, HoverTint, UiFonts};

/// Whether the menu settings panel is open.
#[derive(Resource, Default)]
pub struct MenuSettingsOpen(pub bool);

/// The "Settings" button on the main menu.
#[derive(Component)]
pub struct OpenSettingsButton;

/// Root of the settings panel overlay.
#[derive(Component)]
pub struct SettingsPanelRoot;

/// One cycling setting row. Clicking advances the setting; the label
/// re-renders from live state every frame.
#[derive(Component, Clone, Copy, PartialEq)]
pub enum SettingRow {
    WindowMode,
    Resolution,
    Maximize,
    Quality,
    AnimSpeed,
    SortHand,
}

#[derive(Component)]
pub struct SettingsCloseButton;

/// Windowed-mode resolution presets the Resolution row cycles through.
const RESOLUTIONS: [(u32, u32); 5] =
    [(1280, 800), (1600, 1000), (1920, 1080), (2560, 1440), (3840, 2160)];

const SPEEDS: [f32; 4] = [0.5, 1.0, 1.5, 2.0];

/// Toggle the panel from the menu button (Esc also closes).
pub fn handle_settings_open(
    mut open: ResMut<MenuSettingsOpen>,
    keyboard: Res<ButtonInput<KeyCode>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<OpenSettingsButton>)>,
    closes: Query<&Interaction, (Changed<Interaction>, With<SettingsCloseButton>)>,
) {
    if buttons.iter().any(|i| *i == Interaction::Pressed) {
        open.0 = !open.0;
    }
    if open.0
        && (keyboard.just_pressed(KeyCode::Escape)
            || closes.iter().any(|i| *i == Interaction::Pressed))
    {
        open.0 = false;
    }
}

/// Spawn / despawn the panel to match `MenuSettingsOpen`.
pub fn sync_settings_panel(
    mut commands: Commands,
    open: Res<MenuSettingsOpen>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<SettingsPanelRoot>>,
) {
    match (open.0, existing.iter().next()) {
        (true, None) => spawn_panel(&mut commands, &ui_fonts),
        (false, Some(e)) => commands.entity(e).despawn(),
        _ => {}
    }
}

fn spawn_panel(commands: &mut Commands, ui_fonts: &UiFonts) {
    let tf = |size: f32| ui_fonts.tf(size);
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            SettingsPanelRoot,
            GlobalZIndex(60),
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(10.0),
                min_width: Val::Px(420.0),
                align_items: AlignItems::Stretch,
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new("Settings"),
            tf(22.0),
            TextColor(theme::ACCENT_GOLD),
            Node { align_self: AlignSelf::Center, ..default() },
        ));
        for row in [
            SettingRow::WindowMode,
            SettingRow::Resolution,
            SettingRow::Maximize,
            SettingRow::Quality,
            SettingRow::AnimSpeed,
            SettingRow::SortHand,
        ] {
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                row,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(""),
                    tf(14.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        }
        p.spawn((
            Text::new("Changes apply immediately and persist to config.toml"),
            tf(11.0),
            TextColor(theme::TEXT_MUTED),
            Node { align_self: AlignSelf::Center, ..default() },
        ));
        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                align_self: AlignSelf::Center,
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(theme::BUTTON_PRIMARY_BG),
            HoverTint::new(theme::BUTTON_PRIMARY_BG),
            SettingsCloseButton,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Done (Esc)"),
                tf(14.0),
                TextColor(theme::TEXT_PRIMARY),
                Pickable::IGNORE,
            ));
        });
    });
}

/// Re-render every row label from live state.
pub fn update_setting_labels(
    store: Res<ConfigStore>,
    quality: Res<RenderQuality>,
    speed: Res<AnimationSpeed>,
    rows: Query<(&SettingRow, &Children)>,
    mut texts: Query<&mut Text>,
) {
    let g = &store.0.graphics;
    for (row, children) in &rows {
        let label = match row {
            SettingRow::WindowMode => format!(
                "Window:  {}",
                match g.window_mode {
                    WindowModeCfg::Windowed => "Windowed",
                    WindowModeCfg::Borderless => "Borderless fullscreen",
                }
            ),
            SettingRow::Resolution => {
                format!("Resolution:  {} × {}", g.window_width, g.window_height)
            }
            SettingRow::Maximize => format!(
                "Maximize on launch:  {}",
                if g.maximize_on_launch { "On" } else { "Off" }
            ),
            SettingRow::Quality => format!("Render quality:  {}", quality.label()),
            SettingRow::AnimSpeed => format!("Animation speed:  {:.1}×", speed.0),
            SettingRow::SortHand => format!(
                "Sort hand:  {}",
                if store.0.gameplay.sort_hand { "On" } else { "Off" }
            ),
        };
        for child in children.iter() {
            if let Ok(mut t) = texts.get_mut(child)
                && t.0 != label
            {
                t.0 = label.clone();
            }
        }
    }
}

/// Advance the clicked setting, apply it live, and persist.
#[allow(clippy::too_many_arguments)]
pub fn handle_setting_rows(
    mut store: ResMut<ConfigStore>,
    mut quality_msgs: MessageWriter<ChangeQuality>,
    quality: Res<RenderQuality>,
    mut speed: ResMut<AnimationSpeed>,
    mut gameplay: ResMut<crate::config::GameplayConfig>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    clicked: Query<(&Interaction, &SettingRow), Changed<Interaction>>,
) {
    let mut dirty = false;
    for (interaction, row) in &clicked {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match row {
            SettingRow::WindowMode => {
                let g = &mut store.0.graphics;
                g.window_mode = match g.window_mode {
                    WindowModeCfg::Windowed => WindowModeCfg::Borderless,
                    WindowModeCfg::Borderless => WindowModeCfg::Windowed,
                };
                if let Ok(mut window) = windows.single_mut() {
                    window.mode = match g.window_mode {
                        WindowModeCfg::Windowed => WindowMode::Windowed,
                        WindowModeCfg::Borderless => WindowMode::BorderlessFullscreen(
                            MonitorSelection::Current,
                        ),
                    };
                }
                dirty = true;
            }
            SettingRow::Resolution => {
                let g = &mut store.0.graphics;
                let current = (g.window_width, g.window_height);
                let idx = RESOLUTIONS.iter().position(|&r| r == current).unwrap_or(0);
                let (w, h) = RESOLUTIONS[(idx + 1) % RESOLUTIONS.len()];
                g.window_width = w;
                g.window_height = h;
                // Picking an explicit size implies "don't maximize over it".
                g.maximize_on_launch = false;
                if let Ok(mut window) = windows.single_mut() {
                    window.set_maximized(false);
                    window.resolution = bevy::window::WindowResolution::new(w, h);
                }
                dirty = true;
            }
            SettingRow::Maximize => {
                let g = &mut store.0.graphics;
                g.maximize_on_launch = !g.maximize_on_launch;
                if let Ok(mut window) = windows.single_mut()
                    && g.maximize_on_launch
                {
                    window.set_maximized(true);
                }
                dirty = true;
            }
            SettingRow::Quality => {
                let all = RenderQuality::ALL;
                let idx = all.iter().position(|q| *q == *quality).unwrap_or(0);
                let next = all[(idx + 1) % all.len()];
                quality_msgs.write(ChangeQuality(next));
                store.0.graphics.render_quality = next;
                dirty = true;
            }
            SettingRow::AnimSpeed => {
                let idx = SPEEDS
                    .iter()
                    .position(|s| (*s - speed.0).abs() < 0.01)
                    .unwrap_or(SPEEDS.len() - 1);
                speed.0 = SPEEDS[(idx + 1) % SPEEDS.len()];
                // `persist_animation_speed` mirrors this into the config.
            }
            SettingRow::SortHand => {
                gameplay.sort_hand = !gameplay.sort_hand;
                store.0.gameplay.sort_hand = gameplay.sort_hand;
                dirty = true;
            }
        }
    }
    if dirty {
        crate::config::save(&store.0);
    }
}
