//! Centered "Game Over" modal with three follow-up actions:
//!
//! 1. **Rematch** — restart the same match mode + format in-place,
//!    re-using the running Bevy app (no menu transition). For Cube
//!    that re-rolls the random decks; for Modern that re-shuffles the
//!    fixed decklists.
//! 2. **New Game** — return to the main menu so the user can pick a
//!    different mode / format.
//! 3. **Auto-rematch** (Spectate Bot vs Bot only) — a numeric input
//!    that, when set to N > 0, automatically rematches N times after
//!    each game ends. Useful for soak-testing the bot loop.
//!
//! Modal opens whenever `CurrentView.0.game_over.is_some()`.

use std::sync::{Arc, Mutex};

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crabomination::server::{
    run_match_full, RandomBot, SeatOccupant, SnapshotSink, SnapshotSinkState,
};

use crate::card::Card;
use crate::game::GameLog;
use crate::menu::{AppState, LatestSnapshot, MatchFormat, PendingNetMode};
use crate::net_plugin::{CurrentView, MatchEnded, NetInbox, NetOutbox};

#[derive(Component)]
pub struct GameOverModalRoot;

#[derive(Component)]
pub struct RematchButton;

#[derive(Component)]
pub struct NewGameButton;

#[derive(Component)]
pub struct AutoRematchSetButton;

#[derive(Component)]
pub struct AutoRematchInput;

#[derive(Component)]
pub struct AutoRematchInputText;

/// Resource: which format the active match was launched with. Used by
/// the rematch flow to start an equivalent fresh match without
/// asking the user to revisit the menu.
#[derive(Resource, Default, Clone, Copy)]
pub struct ActiveMatchFormat(pub MatchFormat);

/// What kind of match we're running. Drives whether the game-over
/// modal shows the auto-rematch counter (Spectate Bot vs Bot only —
/// auto-replaying a Human-vs-Bot match would yank the player back
/// into a fresh game without their consent).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ActiveMatchKind {
    #[default]
    HumanVsBot,
    SpectateBotVsBot,
}

/// Pending auto-rematch counter. Each time a game ends, if this is >0,
/// the rematch flow triggers automatically and the counter decrements.
#[derive(Resource, Default)]
pub struct AutoRematchState {
    /// Free-text edit buffer bound to the modal's number input.
    pub draft: String,
    /// Active countdown — "play this many more games before stopping."
    /// Set when the user clicks the "Set" button on the input; consumed
    /// by `apply_auto_rematch_on_game_over`.
    pub remaining: u32,
    /// True while focus is on the number input (typed characters
    /// route to `draft` instead of being consumed by gameplay).
    pub focused: bool,
}

const ACCENT: Color = Color::srgb(1.0, 0.85, 0.55);
const REMATCH_BG: Color = Color::srgba(0.18, 0.45, 0.20, 1.0);
const NEW_GAME_BG: Color = Color::srgba(0.20, 0.30, 0.55, 1.0);
const SET_BG: Color = Color::srgba(0.32, 0.20, 0.45, 1.0);
const FIELD_BG: Color = Color::srgba(0.16, 0.16, 0.22, 1.0);
const FIELD_BG_FOCUSED: Color = Color::srgba(0.28, 0.28, 0.50, 1.0);

#[allow(clippy::too_many_arguments)]
pub fn sync_game_over_modal(
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<GameOverModalRoot>>,
    auto: Res<AutoRematchState>,
    kind: Res<ActiveMatchKind>,
    mut commands: Commands,
) {
    let game_over = view.0.as_ref().and_then(|cv| cv.game_over);
    let Some(winner) = game_over else {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    };
    if !existing.is_empty() {
        return;
    }
    let Some(cv) = view.0.as_ref() else { return };
    let title = match winner {
        Some(seat) => {
            let name = cv
                .players
                .iter()
                .find(|p| p.seat == seat)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("Seat {seat}"));
            if seat == cv.your_seat {
                format!("Victory! {name} wins.")
            } else {
                format!("Defeat. {name} wins.")
            }
        }
        None => "Draw.".to_string(),
    };
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let show_auto_rematch = matches!(*kind, ActiveMatchKind::SpectateBotVsBot);

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            GameOverModalRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(40.0)),
                    row_gap: Val::Px(20.0),
                    align_items: AlignItems::Center,
                    min_width: Val::Px(440.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.06, 0.12, 0.97)),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("Game Over"),
                    TextFont { font: font.clone(), font_size: 36.0, ..default() },
                    TextColor(ACCENT),
                ));
                p.spawn((
                    Text::new(title),
                    TextFont { font: font.clone(), font_size: 22.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                // Action buttons row: Rematch + New Game.
                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button,
                        Node { padding: UiRect::axes(Val::Px(24.0), Val::Px(11.0)), ..default() },
                        BackgroundColor(REMATCH_BG),
                        RematchButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Rematch"),
                            TextFont { font: font.clone(), font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });
                    row.spawn((
                        Button,
                        Node { padding: UiRect::axes(Val::Px(24.0), Val::Px(11.0)), ..default() },
                        BackgroundColor(NEW_GAME_BG),
                        NewGameButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("New Game"),
                            TextFont { font: font.clone(), font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });
                });

                // Spectate-only: auto-rematch counter.
                if show_auto_rematch {
                    p.spawn((
                        Text::new("Auto-rematch (bot vs bot only)"),
                        TextFont { font: font.clone(), font_size: 13.0, ..default() },
                        TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                    ));
                    p.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        let bg = if auto.focused { FIELD_BG_FOCUSED } else { FIELD_BG };
                        row.spawn((
                            Button,
                            Node {
                                min_width: Val::Px(80.0),
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            AutoRematchInput,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(format_auto_input(&auto)),
                                TextFont { font: font.clone(), font_size: 16.0, ..default() },
                                TextColor(Color::WHITE),
                                Pickable::IGNORE,
                                AutoRematchInputText,
                            ));
                        });
                        row.spawn((
                            Button,
                            Node { padding: UiRect::axes(Val::Px(16.0), Val::Px(7.0)), ..default() },
                            BackgroundColor(SET_BG),
                            AutoRematchSetButton,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Set"),
                                TextFont { font, font_size: 14.0, ..default() },
                                TextColor(Color::WHITE),
                                Pickable::IGNORE,
                            ));
                        });
                    });
                }
            });
        });
}

fn format_auto_input(auto: &AutoRematchState) -> String {
    let cursor = if auto.focused { "_" } else { "" };
    if auto.remaining > 0 {
        format!("{}  ({} left){cursor}", auto.draft, auto.remaining)
    } else {
        format!("{}{cursor}", auto.draft)
    }
}

/// Refresh the input's text whenever the buffer / focus / countdown
/// changes.
pub fn refresh_auto_rematch_text(
    auto: Res<AutoRematchState>,
    mut q: Query<&mut Text, With<AutoRematchInputText>>,
) {
    if !auto.is_changed() {
        return;
    }
    for mut t in &mut q {
        t.0 = format_auto_input(&auto);
    }
}

/// Toggle keyboard focus on the auto-rematch input box.
pub fn handle_auto_rematch_focus(
    mut auto: ResMut<AutoRematchState>,
    input_q: Query<&Interaction, (Changed<Interaction>, With<AutoRematchInput>)>,
    set_q: Query<&Interaction, (Changed<Interaction>, With<AutoRematchSetButton>)>,
    rematch_q: Query<&Interaction, (Changed<Interaction>, With<RematchButton>)>,
    new_q: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
) {
    if input_q.iter().any(|i| *i == Interaction::Pressed) {
        auto.focused = true;
    }
    // Clicking any other modal control implicitly drops focus.
    let any_other = set_q.iter().chain(rematch_q.iter()).chain(new_q.iter())
        .any(|i| *i == Interaction::Pressed);
    if any_other {
        auto.focused = false;
    }
}

/// Capture digit/backspace input while the auto-rematch input is focused.
pub fn handle_auto_rematch_keys(
    mut auto: ResMut<AutoRematchState>,
    mut events: MessageReader<KeyboardInput>,
) {
    if !auto.focused {
        events.clear();
        return;
    }
    for ev in events.read() {
        if !ev.state.is_pressed() { continue; }
        match &ev.logical_key {
            Key::Backspace => { auto.draft.pop(); }
            Key::Enter => {
                commit_draft(&mut auto);
                auto.focused = false;
            }
            Key::Escape => {
                auto.draft.clear();
                auto.focused = false;
            }
            Key::Character(s) => {
                for ch in s.chars() {
                    if ch.is_ascii_digit() && auto.draft.len() < 4 {
                        auto.draft.push(ch);
                    }
                }
            }
            _ => {}
        }
    }
}

fn commit_draft(auto: &mut AutoRematchState) {
    if let Ok(n) = auto.draft.parse::<u32>() {
        auto.remaining = n;
    }
}

/// "Set" button: parse the current draft into `remaining`. Drops focus.
pub fn handle_auto_rematch_set(
    mut auto: ResMut<AutoRematchState>,
    set_q: Query<&Interaction, (Changed<Interaction>, With<AutoRematchSetButton>)>,
) {
    if set_q.iter().any(|i| *i == Interaction::Pressed) {
        commit_draft(&mut auto);
        auto.focused = false;
    }
}

/// "New Game" button — leaves the in-game scene back to the main
/// menu so the user can pick a different mode/format. Cleanup of
/// HUD entities and Card visuals happens in `OnExit(InGame)`.
pub fn handle_new_game_button(
    button_q: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingNetMode>,
    mut auto: ResMut<AutoRematchState>,
) {
    if !button_q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    // Clear any queued auto-rematch — the user wants out.
    auto.remaining = 0;
    auto.draft.clear();
    auto.focused = false;
    // Drop any pending mode from the prior menu visit; the new menu
    // session will populate it fresh on the user's next selection.
    pending.0 = None;
    next_state.set(AppState::Menu);
}

/// "Rematch" button — restart the same mode + format in place. Same
/// rebuild logic also fires when the auto-rematch counter is non-zero.
#[allow(clippy::too_many_arguments)]
pub fn handle_rematch_button(
    button_q: Query<&Interaction, (Changed<Interaction>, With<RematchButton>)>,
    modal_q: Query<Entity, With<GameOverModalRoot>>,
    cards_q: Query<Entity, With<Card>>,
    view: ResMut<CurrentView>,
    ended: ResMut<MatchEnded>,
    log: ResMut<GameLog>,
    format: Res<ActiveMatchFormat>,
    kind: Res<ActiveMatchKind>,
    commands: Commands,
) {
    if !button_q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    rematch_in_place(commands, modal_q, cards_q, view, ended, log, *format, *kind);
}

/// When a game ends and `auto_rematch.remaining > 0`, fire a rematch
/// automatically and decrement the counter.
#[allow(clippy::too_many_arguments)]
pub fn apply_auto_rematch_on_game_over(
    view: ResMut<CurrentView>,
    modal_q: Query<Entity, With<GameOverModalRoot>>,
    cards_q: Query<Entity, With<Card>>,
    ended: ResMut<MatchEnded>,
    log: ResMut<GameLog>,
    format: Res<ActiveMatchFormat>,
    kind: Res<ActiveMatchKind>,
    mut auto: ResMut<AutoRematchState>,
    commands: Commands,
) {
    // Read game_over off the same `ResMut<CurrentView>` we'll write
    // through; taking a separate `Res<CurrentView>` here panics Bevy
    // with B0002 (`Res + ResMut` of the same resource in one system).
    let game_over = view.0.as_ref().and_then(|cv| cv.game_over).is_some();
    if !game_over { return; }
    if !matches!(*kind, ActiveMatchKind::SpectateBotVsBot) { return; }
    if auto.remaining == 0 { return; }
    auto.remaining -= 1;
    rematch_in_place(commands, modal_q, cards_q, view, ended, log, *format, *kind);
}

#[allow(clippy::too_many_arguments)]
fn rematch_in_place(
    mut commands: Commands,
    modal_q: Query<Entity, With<GameOverModalRoot>>,
    cards_q: Query<Entity, With<Card>>,
    mut view: ResMut<CurrentView>,
    mut ended: ResMut<MatchEnded>,
    mut log: ResMut<GameLog>,
    format: ActiveMatchFormat,
    kind: ActiveMatchKind,
) {
    for e in &modal_q {
        commands.entity(e).despawn();
    }
    for e in &cards_q {
        commands.entity(e).despawn();
    }

    view.0 = None;
    ended.0 = None;
    log.entries.clear();
    log.push("Rematch — fresh deal");

    use crabomination::server::{seat_pair, ClientChannel};
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
    let sink_for_match = Arc::clone(&sink);
    let chosen = format.0;

    match kind {
        ActiveMatchKind::HumanVsBot => {
            std::thread::spawn(move || {
                run_match_full(
                    chosen.build_state_for_restart(),
                    vec![
                        SeatOccupant::Human(server_seat),
                        SeatOccupant::Bot(Box::new(RandomBot::new())),
                    ],
                    vec![],
                    Some(sink_for_match),
                );
            });
        }
        ActiveMatchKind::SpectateBotVsBot => {
            std::thread::spawn(move || {
                run_match_full(
                    chosen.build_state_for_restart(),
                    vec![
                        SeatOccupant::Bot(Box::new(RandomBot::new())),
                        SeatOccupant::Bot(Box::new(RandomBot::new())),
                    ],
                    vec![server_seat],
                    Some(sink_for_match),
                );
            });
        }
    }
    commands.insert_resource(NetOutbox(tx));
    commands.insert_resource(NetInbox(Mutex::new(rx)));
    commands.insert_resource(LatestSnapshot(sink));
}

/// `OnExit(AppState::InGame)` cleanup so a return-to-menu doesn't
/// leave the HUD floating on top of the menu UI. Despawns every UI
/// node tagged `InGameRoot` (covers the corner panels, the action
/// button row, the quality panel, etc.) plus every `Card` entity.
/// The game-over modal is included via its own marker.
pub fn cleanup_in_game_entities(
    mut commands: Commands,
    in_game_roots: Query<Entity, With<crate::systems::game_ui::InGameRoot>>,
    cards: Query<Entity, With<Card>>,
    modals: Query<Entity, With<GameOverModalRoot>>,
    mut auto: ResMut<AutoRematchState>,
) {
    for e in &in_game_roots {
        commands.entity(e).despawn();
    }
    for e in &cards {
        commands.entity(e).despawn();
    }
    for e in &modals {
        commands.entity(e).despawn();
    }
    // Reset auto-rematch state; a fresh InGame session shouldn't
    // inherit a queue from a previous one.
    *auto = AutoRematchState::default();
}
