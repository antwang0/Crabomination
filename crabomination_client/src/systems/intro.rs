//! First-game welcome: the six things a new player needs, once.
//!
//! The first time a game starts (not a spectated one), a panel lists how
//! to play a card, read one, pass, attack, get unstuck and find the rest —
//! naming the keys as they are bound now. "Got it" (or Enter, or a pad's A)
//! dismisses it for good (`GameplayConfig::seen_intro`).

use bevy::prelude::*;

use crate::config::ConfigStore;
use crate::keybinds::{Action, Keybinds};
use crate::net_plugin::CurrentView;
use crate::theme::{self, HoverTint, UiFonts};
use crate::ui_button::{Button, Interaction};

#[derive(Component)]
pub struct IntroPanel;

#[derive(Component)]
pub struct IntroCloseButton;

/// The tips, keys filled in from the live bindings.
pub fn tips(binds: &Keybinds) -> Vec<(String, String)> {
    let k = |a: Action| binds.label(a);
    vec![
        (
            "Play a card".into(),
            "Click it, or drag it onto the table. Right-click for its other ways to cast — the hint on the card says what."
                .into(),
        ),
        (
            "Read a card".into(),
            "Hover it; hold Alt for a big view. Your playable cards glow green.".into(),
        ),
        (
            "Pass".into(),
            format!("{} passes priority. Click a step in the phase chart (left) to choose where the game stops for you.", k(Action::Pass)),
        ),
        (
            "Attack".into(),
            format!(
                "In combat, click your creatures, then the player or planeswalker they attack; {} confirms (with none chosen, everything attacks).",
                k(Action::Attack)
            ),
        ),
        (
            "Stuck?".into(),
            format!("{} asks what the bot would do for you. {} takes back your last action.", k(Action::Hint), k(Action::TakeBack)),
        ),
        (
            "Everything else".into(),
            format!("{} lists every shortcut. Esc opens the menu, with Settings.", k(Action::ToggleHelp)),
        ),
    ]
}

/// Show the panel on the first view of a first played game.
pub fn show_intro_once(
    mut commands: Commands,
    view: Res<CurrentView>,
    kind: Res<crate::systems::game_over::ActiveMatchKind>,
    store: Res<ConfigStore>,
    binds: Res<Keybinds>,
    ui_fonts: Res<UiFonts>,
    existing: Query<(), With<IntroPanel>>,
    harness: Res<crate::layout_harness::HarnessArgs>,
    mut shown: Local<bool>,
) {
    // A harness screenshot is of the board, not of the welcome.
    let forced = harness.intro;
    if *shown
        || (store.0.gameplay.seen_intro && !forced)
        || view.0.is_none()
        || !existing.is_empty()
        || (harness.screenshot.is_some() && !forced)
    {
        return;
    }
    if *kind == crate::systems::game_over::ActiveMatchKind::SpectateBotVsBot {
        return;
    }
    *shown = true;
    let tf = |s: f32| ui_fonts.tf(s);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            IntroPanel,
            crate::systems::game_ui::InGameRoot,
            GlobalZIndex(theme::layer::MODAL),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(10.0),
                    width: Val::Px(620.0),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|p| {
                p.spawn((Text::new("Welcome to Crabomination"), tf(22.0), TextColor(theme::ACCENT_GOLD), Node { align_self: AlignSelf::Center, ..default() }));
                for (title, body) in tips(&binds) {
                    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), ..default() }).with_children(|t| {
                        t.spawn((Text::new(title), tf(15.0), TextColor(theme::ACCENT_BLUE)));
                        t.spawn((Text::new(body), tf(13.0), TextColor(theme::TEXT_BODY)));
                    });
                }
                p.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(22.0), Val::Px(9.0)),
                        align_self: AlignSelf::Center,
                        margin: UiRect::top(Val::Px(6.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_PRIMARY_BG),
                    HoverTint::new(theme::BUTTON_PRIMARY_BG),
                    IntroCloseButton,
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Got it"), tf(15.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                });
            });
        });
}

/// "Got it": close it, and don't show it again.
pub fn close_intro(
    mut commands: Commands,
    button: Query<&Interaction, (Changed<Interaction>, With<IntroCloseButton>)>,
    panel: Query<Entity, With<IntroPanel>>,
    mut store: ResMut<ConfigStore>,
) {
    if !button.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    for e in &panel {
        commands.entity(e).despawn();
    }
    store.0.gameplay.seen_intro = true;
    crate::config::save(&store.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tips_name_the_live_keys() {
        let mut binds = Keybinds::default();
        let pass = |b: &Keybinds| tips(b).into_iter().find(|(t, _)| t == "Pass").unwrap().1;
        assert!(pass(&binds).starts_with("Space passes"));
        binds.set(Action::Pass, KeyCode::KeyJ);
        assert!(pass(&binds).starts_with("J passes"));
    }
}
