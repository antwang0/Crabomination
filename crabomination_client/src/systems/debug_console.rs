//! In-game debug console: add mana, conjure a card into hand, or bump
//! life. Toggle with `` ` `` (backquote). Sends [`DebugAction`]s through
//! the existing net channel — the server applies them to the sending
//! seat and re-broadcasts the view.

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crabomination::mana::Color as ManaColor;
use crabomination::net::DebugAction;

use crate::net_plugin::NetOutbox;

/// Resource tracking whether the panel is open and the card-name input
/// buffer. `card_input_focused` gates whether typed characters go into
/// the buffer; while focused, other input handlers (see
/// `handle_game_input`) early-out to avoid double-binding.
#[derive(Resource, Default)]
pub struct DebugConsoleState {
    pub open: bool,
    pub card_name: String,
    pub card_input_focused: bool,
}

/// Root of the panel UI. Despawned when `open` flips to false.
#[derive(Component)]
pub struct DebugConsoleRoot;

/// Text node mirroring `card_name`. Refreshed when the buffer or focus
/// state changes.
#[derive(Component)]
pub struct DebugConsoleCardText;

/// Buttons that emit a mana add.
#[derive(Component)]
pub struct DebugManaButton(pub Option<ManaColor>);

/// Buttons that bump life by `delta`.
#[derive(Component)]
pub struct DebugLifeButton(pub i32);

/// Button that submits the current `card_name` buffer.
#[derive(Component)]
pub struct DebugAddCardButton;

/// Clickable region around the card-name text — clicking it toggles
/// `card_input_focused`.
#[derive(Component)]
pub struct DebugCardInputBox;

/// Container holding the live autocomplete suggestion buttons under the
/// card-name input field. Rebuilt whenever the buffer changes.
#[derive(Component)]
pub struct DebugSuggestionsRoot;

/// One clickable suggestion row. Clicking it copies `name` into the
/// console's `card_name` buffer.
#[derive(Component)]
pub struct DebugSuggestion {
    pub name: String,
}

/// Maximum number of autocomplete rows to render at once. Keeps the
/// panel compact even when the prefix matches dozens of cards.
const MAX_SUGGESTIONS: usize = 8;

use crate::theme::{self, UiFonts};

/// Inactive text-input background inside the debug console (darker than
/// the general field bg to fit this panel's more compact, terminal-ish feel).
const INPUT_BG: Color = Color::srgb(0.10, 0.10, 0.16);
const INPUT_BG_FOCUSED: Color = Color::srgb(0.20, 0.20, 0.34);

/// Toggle the panel with `` ` ``. We watch the key directly via
/// `ButtonInput<KeyCode>` so the toggle works regardless of input focus.
pub fn toggle_debug_console(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugConsoleState>,
) {
    if keys.just_pressed(KeyCode::Backquote) {
        state.open = !state.open;
        if !state.open {
            state.card_input_focused = false;
        }
    }
}

/// Capture characters into `card_name` while the input is focused.
/// Enter submits, Escape unfocuses. Returns early when not focused so
/// the keypress events fall through to the gameplay handlers.
pub fn handle_debug_console_input(
    mut state: ResMut<DebugConsoleState>,
    mut events: MessageReader<KeyboardInput>,
    outbox: Option<Res<NetOutbox>>,
) {
    if !state.card_input_focused {
        events.clear();
        return;
    }
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Escape => {
                state.card_input_focused = false;
                return;
            }
            Key::Enter => {
                let name = state.card_name.trim().to_string();
                if !name.is_empty()
                    && let Some(outbox) = &outbox
                {
                    outbox.submit_debug(DebugAction::AddCardToHand { name });
                }
                state.card_name.clear();
                state.card_input_focused = false;
                return;
            }
            Key::Backspace => {
                state.card_name.pop();
            }
            Key::Space if state.card_name.len() < 64 => {
                state.card_name.push(' ');
            }
            Key::Character(s) => {
                for ch in s.chars() {
                    if state.card_name.len() >= 64 {
                        break;
                    }
                    // Skip the backquote that opens the panel so the toggle
                    // key doesn't leak into the buffer.
                    if ch == '`' || ch.is_control() {
                        continue;
                    }
                    state.card_name.push(ch);
                }
            }
            _ => {}
        }
    }
}

/// Spawn the panel on open, despawn on close, and refresh the
/// card-name text/background when the buffer or focus state changes.
pub fn sync_debug_console_ui(
    state: Res<DebugConsoleState>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<DebugConsoleRoot>>,
    mut text_q: Query<&mut Text, With<DebugConsoleCardText>>,
    mut box_q: Query<&mut BackgroundColor, With<DebugCardInputBox>>,
    suggestions_root_q: Query<Entity, With<DebugSuggestionsRoot>>,
    mut commands: Commands,
) {
    if state.open && existing.is_empty() {
        spawn_panel(&mut commands, &ui_fonts, &state);
    } else if !state.open {
        for e in &existing {
            commands.entity(e).despawn();
        }
    }
    if !state.is_changed() {
        return;
    }
    if let Ok(mut t) = text_q.single_mut() {
        t.0 = render_card_text(&state);
    }
    if let Ok(mut bg) = box_q.single_mut() {
        *bg = BackgroundColor(if state.card_input_focused {
            INPUT_BG_FOCUSED
        } else {
            INPUT_BG
        });
    }
    // Rebuild suggestions: despawn existing children and respawn from
    // the current buffer. Cheap — at most MAX_SUGGESTIONS rows.
    if let Ok(root) = suggestions_root_q.single() {
        commands.entity(root).despawn_related::<Children>();
        let suggestions = compute_suggestions(&state.card_name);
        if !suggestions.is_empty() {
            commands.entity(root).with_children(|p| {
                for name in suggestions {
                    p.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                        DebugSuggestion { name: name.clone() },
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(name),
                            ui_fonts.tf(11.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });
        }
    }
}

/// Best-effort fuzzy match: prefix matches first (sorted by length),
/// then any substring matches. Case-insensitive. Returns up to
/// `MAX_SUGGESTIONS` names. Empty buffer → no suggestions (we'd
/// otherwise spam the panel with every card).
fn compute_suggestions(buffer: &str) -> Vec<String> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let needle = trimmed.to_lowercase();
    let catalog = crate::audit::catalog();
    let mut prefix: Vec<&str> = Vec::new();
    let mut substr: Vec<&str> = Vec::new();
    for entry in &catalog {
        let lower = entry.name.to_lowercase();
        if lower.starts_with(&needle) {
            prefix.push(&entry.name);
        } else if lower.contains(&needle) {
            substr.push(&entry.name);
        }
    }
    prefix.sort_by_key(|s| s.len());
    substr.sort_by_key(|s| s.len());
    prefix.into_iter()
        .chain(substr)
        .take(MAX_SUGGESTIONS)
        .map(|s| s.to_string())
        .collect()
}

fn render_card_text(state: &DebugConsoleState) -> String {
    if state.card_input_focused {
        format!("{}_", state.card_name)
    } else if state.card_name.is_empty() {
        "<click to type card name>".to_string()
    } else {
        state.card_name.clone()
    }
}

fn spawn_panel(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    state: &DebugConsoleState,
) {
    let tf = |size: f32| ui_fonts.tf(size);

    commands
        .spawn((
            Node {
                // Anchored bottom-right so the panel doesn't overlap the
                // viewer's hand, deck pile, or hovered-card popup in the
                // top-left corner.
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(8.0),
                min_width: Val::Px(280.0),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
            DebugConsoleRoot,
            crate::systems::game_ui::InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Debug Console  (` to close)"),
                tf(14.0),
                TextColor(theme::ACCENT_GOLD),
            ));

            // ── Mana row ─────────────────────────────────────────────
            p.spawn((Text::new("Mana (+1 to your pool)"), tf(11.0), TextColor(theme::TEXT_SECONDARY)));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|row| {
                let entries: [(&str, Option<ManaColor>); 6] = [
                    ("W", Some(ManaColor::White)),
                    ("U", Some(ManaColor::Blue)),
                    ("B", Some(ManaColor::Black)),
                    ("R", Some(ManaColor::Red)),
                    ("G", Some(ManaColor::Green)),
                    ("C", None),
                ];
                for (label, color) in entries {
                    row.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                        Button,
                        DebugManaButton(color),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(label), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                    });
                }
            });

            // ── Life row ─────────────────────────────────────────────
            p.spawn((Text::new("Life"), tf(11.0), TextColor(theme::TEXT_SECONDARY)));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|row| {
                for (label, delta) in [("-5", -5), ("-1", -1), ("+1", 1), ("+5", 5)] {
                    row.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                        Button,
                        DebugLifeButton(delta),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(label), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                    });
                }
            });

            // ── Card-name input ──────────────────────────────────────
            p.spawn((Text::new("Add card to hand (Enter to submit)"), tf(11.0), TextColor(theme::TEXT_SECONDARY)));
            p.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    min_width: Val::Px(220.0),
                    ..default()
                },
                BackgroundColor(if state.card_input_focused { INPUT_BG_FOCUSED } else { INPUT_BG }),
                Button,
                DebugCardInputBox,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(render_card_text(state)),
                    tf(12.0),
                    TextColor(theme::TEXT_PRIMARY),
                    DebugConsoleCardText,
                    Pickable::IGNORE,
                ));
            });
            p.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                Button,
                DebugAddCardButton,
            ))
            .with_children(|b| {
                b.spawn((Text::new("Add Card"), tf(12.0), TextColor(theme::TEXT_PRIMARY)));
            });

            // Autocomplete suggestions populated by sync_debug_console_ui
            // whenever the buffer changes. Empty container at spawn time.
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                DebugSuggestionsRoot,
            ));
        });
}

/// Handle clicks on the mana, life, card-input, and add-card buttons.
/// Also drives hover-colour feedback on the click targets.
pub fn handle_debug_console_buttons(
    mut state: ResMut<DebugConsoleState>,
    mut mana_q: Query<(&Interaction, &DebugManaButton, &mut BackgroundColor), (Changed<Interaction>, Without<DebugLifeButton>, Without<DebugAddCardButton>, Without<DebugCardInputBox>)>,
    mut life_q: Query<(&Interaction, &DebugLifeButton, &mut BackgroundColor), (Changed<Interaction>, Without<DebugManaButton>, Without<DebugAddCardButton>, Without<DebugCardInputBox>)>,
    mut add_q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<DebugAddCardButton>, Without<DebugManaButton>, Without<DebugLifeButton>, Without<DebugCardInputBox>)>,
    mut box_q: Query<&Interaction, (Changed<Interaction>, With<DebugCardInputBox>, Without<DebugManaButton>, Without<DebugLifeButton>, Without<DebugAddCardButton>)>,
    outbox: Option<Res<NetOutbox>>,
) {
    let Some(outbox) = outbox else { return };

    for (interaction, btn, mut bg) in mana_q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                outbox.submit_debug(DebugAction::AddMana {
                    color: btn.0,
                    amount: 1,
                });
                *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT);
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_BG),
        }
    }

    for (interaction, btn, mut bg) in life_q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                outbox.submit_debug(DebugAction::AdjustLife { delta: btn.0 });
                *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT);
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_BG),
        }
    }

    for (interaction, mut bg) in add_q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                let name = state.card_name.trim().to_string();
                if !name.is_empty() {
                    outbox.submit_debug(DebugAction::AddCardToHand { name });
                }
                state.card_name.clear();
                state.card_input_focused = false;
                *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT);
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_BG),
        }
    }

    for interaction in box_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.card_input_focused = !state.card_input_focused;
        }
    }
}

/// Handle clicks on autocomplete suggestion rows: fill the buffer and
/// keep focus so the user can press Enter (or click "Add Card") to
/// submit. Kept in its own system because the per-row `Without<…>`
/// disjointness gymnastics on `handle_debug_console_buttons` are
/// already at their limit.
pub fn handle_debug_console_suggestions(
    mut state: ResMut<DebugConsoleState>,
    mut q: Query<
        (&Interaction, &DebugSuggestion, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, sug, mut bg) in q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                state.card_name = sug.name.clone();
                state.card_input_focused = true;
                *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT);
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_HOT),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_NEUTRAL_BG),
        }
    }
}
