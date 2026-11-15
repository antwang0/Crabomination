//! Settings → Keyboard shortcuts: every remappable binding
//! (`crate::keybinds::BINDINGS`), its key, and a click-then-press remap.
//!
//! Click a row and press a key to move it there (Esc cancels the capture);
//! a binding that now fires together with another says so in red. Changes
//! apply at once and persist to `config.toml`'s `[keybinds]` (only the
//! bindings moved off their default). Opened from the main menu's Settings
//! panel; the in-game HUD reads the keys when a game starts.

use bevy::prelude::*;

use crate::config::ConfigStore;
use crate::keybinds::{binding, key_label, Action, Keybinds, Section, BINDINGS, REMAPPABLE_KEYS};
use crate::theme::{self, HoverTint, UiFonts};
use crate::ui_button::{Button, Interaction};

/// Whether the panel is up, and the binding waiting for its new key.
#[derive(Resource, Default)]
pub struct KeymapPanel {
    pub open: bool,
    pub capturing: Option<Action>,
}

#[derive(Component)]
pub struct KeymapPanelRoot;

/// A binding's row: click to remap it.
#[derive(Component)]
pub struct KeymapRow(pub Action);

#[derive(Component)]
pub struct KeymapResetButton;

#[derive(Component)]
pub struct KeymapDoneButton;

/// Row clicks, Reset, Done, and the key a capture is waiting for. Runs after
/// the Settings panel's own Esc handling, which leaves the panel open while
/// this one is up.
#[allow(clippy::too_many_arguments)]
pub fn handle_keymap_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel: ResMut<KeymapPanel>,
    mut binds: ResMut<Keybinds>,
    mut store: ResMut<ConfigStore>,
    rows: Query<(&Interaction, &KeymapRow), Changed<Interaction>>,
    reset: Query<&Interaction, (Changed<Interaction>, With<KeymapResetButton>)>,
    done: Query<&Interaction, (Changed<Interaction>, With<KeymapDoneButton>)>,
) {
    if !panel.open {
        return;
    }
    let done_pressed = done.iter().any(|i| *i == Interaction::Pressed);
    let reset_pressed = reset.iter().any(|i| *i == Interaction::Pressed);
    let mut changed = false;
    if let Some(action) = panel.capturing {
        if keyboard.just_pressed(KeyCode::Escape) {
            panel.capturing = None;
            return;
        }
        if let Some(key) = keyboard.get_just_pressed().copied().find(|k| REMAPPABLE_KEYS.contains(k)) {
            binds.set(action, key);
            panel.capturing = None;
            changed = true;
        }
    } else if keyboard.just_pressed(KeyCode::Escape) || done_pressed {
        panel.open = false;
        return;
    }
    for (interaction, row) in &rows {
        if *interaction == Interaction::Pressed {
            panel.capturing = Some(row.0);
        }
    }
    if reset_pressed {
        binds.reset();
        panel.capturing = None;
        changed = true;
    }
    if changed {
        store.0.keybinds = binds.overrides();
        crate::config::save(&store.0);
    }
}

/// `action`'s row: its key (or the capture prompt), and what it now fires
/// together with.
fn row_text(binds: &Keybinds, action: Action, capturing: bool) -> (String, Option<String>) {
    let key = if capturing { "press a key…".to_string() } else { key_label(binds.key(action)) };
    let clashes = binds.clashes(action);
    let clash = (!clashes.is_empty()).then(|| {
        let names: Vec<&str> = clashes.iter().map(|a| binding(*a).label).collect();
        format!("also fires: {}", names.join(", "))
    });
    (key, clash)
}

/// Spawn the panel when it opens, rebuild it when a binding or the capture
/// changes, despawn it when it closes.
pub fn sync_keymap_panel(
    mut commands: Commands,
    panel: Res<KeymapPanel>,
    binds: Res<Keybinds>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<KeymapPanelRoot>>,
) {
    if !panel.is_changed() && !binds.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    if !panel.open {
        return;
    }
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
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            KeymapPanelRoot,
            // Over the Settings panel it opens from.
            GlobalZIndex(theme::layer::TOP_PROMPT + 1),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(8.0),
                    min_width: Val::Px(560.0),
                    max_height: Val::Percent(90.0),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|outer| {
                outer.spawn((
                    Text::new("Keyboard shortcuts"),
                    tf(20.0),
                    TextColor(theme::ACCENT_GOLD),
                    Node { align_self: AlignSelf::Center, ..default() },
                ));
                // The list scrolls; the title and the buttons stay put. A
                // scrollable flex item needs `min_height: 0` (`scroll.rs`).
                outer.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        overflow: Overflow::scroll_y(),
                        min_height: Val::Px(0.0),
                        flex_shrink: 1.0,
                        ..default()
                    },
                    bevy::ui_widgets::ScrollArea,
                ))
                .with_children(|p| {
                for section in Section::ALL.into_iter().filter(|s| BINDINGS.iter().any(|b| b.section == *s)) {
                    p.spawn((
                        Text::new(section.title()),
                        tf(14.0),
                        TextColor(theme::ACCENT_BLUE),
                        Node { margin: UiRect::top(Val::Px(8.0)), ..default() },
                    ));
                    for b in BINDINGS.iter().filter(|b| b.section == section) {
                        let capturing = panel.capturing == Some(b.action);
                        let (key, clash) = row_text(&binds, b.action, capturing);
                        let bg = if capturing { theme::BUTTON_PRIMARY_BG } else { theme::BUTTON_NEUTRAL_BG };
                        p.spawn((
                            Button,
                            Node {
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                                ..default()
                            },
                            BackgroundColor(bg),
                            HoverTint::new(bg),
                            KeymapRow(b.action),
                        ))
                        .with_children(|row| {
                            row.spawn((Node { flex_direction: FlexDirection::Row, ..default() }, Pickable::IGNORE))
                                .with_children(|line| {
                                    line.spawn((Node { width: Val::Px(110.0), ..default() }, Pickable::IGNORE))
                                        .with_children(|k| {
                                            k.spawn((Text::new(key), tf(13.0), TextColor(theme::ACCENT_YELLOW), Pickable::IGNORE));
                                        });
                                    line.spawn((Text::new(b.label), tf(13.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                                });
                            if let Some(clash) = clash {
                                row.spawn((Text::new(clash), tf(11.0), TextColor(theme::TEXT_DANGER), Pickable::IGNORE));
                            }
                        });
                    }
                }
                });
                outer.spawn((
                    Text::new("Click a shortcut, then press its new key (Esc cancels). Saved to config.toml."),
                    tf(11.0),
                    TextColor(theme::TEXT_MUTED),
                    Node { align_self: AlignSelf::Center, margin: UiRect::top(Val::Px(8.0)), ..default() },
                ));
                outer.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    align_self: AlignSelf::Center,
                    ..default()
                })
                .with_children(|buttons| {
                    for (label, bg, done) in
                        [("Reset all", theme::BUTTON_DANGER_BG, false), ("Done (Esc)", theme::BUTTON_PRIMARY_BG, true)]
                    {
                        let mut b = buttons.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                                ..default()
                            },
                            BackgroundColor(bg),
                            HoverTint::new(bg),
                        ));
                        if done {
                            b.insert(KeymapDoneButton);
                        } else {
                            b.insert(KeymapResetButton);
                        }
                        b.with_children(|t| {
                            t.spawn((Text::new(label), tf(14.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                        });
                    }
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_names_its_key_and_its_clash() {
        let mut binds = Keybinds::default();
        let (key, clash) = row_text(&binds, Action::Chat, false);
        assert_eq!(key, "T");
        assert_eq!(clash, None);
        binds.set(Action::Chat, KeyCode::KeyG);
        let (_, clash) = row_text(&binds, Action::Chat, false);
        assert!(clash.unwrap().contains("graveyard"));
        let (key, _) = row_text(&binds, Action::Chat, true);
        assert!(key.starts_with("press a key"));
    }
}
