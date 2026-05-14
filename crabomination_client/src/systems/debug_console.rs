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

const PANEL_BG: Color = Color::srgba(0.05, 0.05, 0.10, 0.92);
const BTN_BG: Color = Color::srgb(0.18, 0.18, 0.26);
const BTN_BG_HOT: Color = Color::srgb(0.30, 0.30, 0.42);
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
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<DebugConsoleRoot>>,
    mut text_q: Query<&mut Text, With<DebugConsoleCardText>>,
    mut box_q: Query<&mut BackgroundColor, With<DebugCardInputBox>>,
    mut commands: Commands,
) {
    if state.open && existing.is_empty() {
        spawn_panel(&mut commands, &asset_server, &state);
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
    asset_server: &AssetServer,
    state: &DebugConsoleState,
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
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(8.0),
                min_width: Val::Px(280.0),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            DebugConsoleRoot,
            crate::systems::game_ui::InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Debug Console  (` to close)"),
                tf(14.0),
                TextColor(Color::srgb(1.0, 0.85, 0.55)),
            ));

            // ── Mana row ─────────────────────────────────────────────
            p.spawn((Text::new("Mana (+1 to your pool)"), tf(11.0), TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0))));
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
                        BackgroundColor(BTN_BG),
                        Button,
                        DebugManaButton(color),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(label), tf(13.0), TextColor(Color::WHITE)));
                    });
                }
            });

            // ── Life row ─────────────────────────────────────────────
            p.spawn((Text::new("Life"), tf(11.0), TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0))));
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
                        BackgroundColor(BTN_BG),
                        Button,
                        DebugLifeButton(delta),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(label), tf(13.0), TextColor(Color::WHITE)));
                    });
                }
            });

            // ── Card-name input ──────────────────────────────────────
            p.spawn((Text::new("Add card to hand (Enter to submit)"), tf(11.0), TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0))));
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
                    TextColor(Color::WHITE),
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
                BackgroundColor(BTN_BG),
                Button,
                DebugAddCardButton,
            ))
            .with_children(|b| {
                b.spawn((Text::new("Add Card"), tf(12.0), TextColor(Color::WHITE)));
            });
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
                *bg = BackgroundColor(BTN_BG_HOT);
            }
            Interaction::Hovered => *bg = BackgroundColor(BTN_BG_HOT),
            Interaction::None => *bg = BackgroundColor(BTN_BG),
        }
    }

    for (interaction, btn, mut bg) in life_q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                outbox.submit_debug(DebugAction::AdjustLife { delta: btn.0 });
                *bg = BackgroundColor(BTN_BG_HOT);
            }
            Interaction::Hovered => *bg = BackgroundColor(BTN_BG_HOT),
            Interaction::None => *bg = BackgroundColor(BTN_BG),
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
                *bg = BackgroundColor(BTN_BG_HOT);
            }
            Interaction::Hovered => *bg = BackgroundColor(BTN_BG_HOT),
            Interaction::None => *bg = BackgroundColor(BTN_BG),
        }
    }

    for interaction in box_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.card_input_focused = !state.card_input_focused;
        }
    }
}
