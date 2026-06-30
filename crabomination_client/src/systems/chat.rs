//! In-match chat. `T` opens an input bar above the hand; Enter sends the
//! line as [`ClientMsg::Chat`] (the server relays it to every seat and
//! spectator), Escape cancels. Incoming [`ServerMsg::Chat`] lines land in
//! the game log tinted with the info color.

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crabomination::net::ClientMsg;

use crate::game::GameLog;
use crate::net_plugin::{ChatInbox, NetOutbox};

const MAX_CHAT_LEN: usize = 200;

/// Whether the chat input bar is open, plus the in-progress line. While
/// `open`, gameplay keyboard shortcuts are suppressed (see
/// `handle_game_input`'s early-out).
#[derive(Resource, Default)]
pub struct ChatInputState {
    pub open: bool,
    pub buffer: String,
}

#[derive(Component)]
pub struct ChatInputBar;

#[derive(Component)]
pub struct ChatInputText;

/// Drain relayed chat lines into the game log.
pub fn drain_chat_inbox(mut inbox: ResMut<ChatInbox>, mut log: ResMut<GameLog>) {
    for (_seat, name, text) in inbox.0.drain(..) {
        log.push_colored(format!("💬 {name}: {text}"), crate::theme::TEXT_INFO);
    }
}

/// Open the chat bar with `T` (when no other text surface owns the
/// keyboard), type into it, Enter to send, Escape to cancel.
pub fn handle_chat_input(
    mut state: ResMut<ChatInputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: MessageReader<KeyboardInput>,
    outbox: Option<Res<NetOutbox>>,
    console: Res<crate::systems::debug_console::DebugConsoleState>,
    export_prompt: Res<crate::systems::export_prompt::ExportPromptState>,
) {
    if !state.open {
        if keyboard.just_pressed(KeyCode::KeyT)
            && !console.card_input_focused
            && !export_prompt.active
        {
            state.open = true;
            state.buffer.clear();
        }
        events.clear();
        return;
    }
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Escape => {
                state.open = false;
                state.buffer.clear();
                return;
            }
            Key::Enter => {
                let text = state.buffer.trim().to_string();
                if !text.is_empty()
                    && let Some(outbox) = &outbox
                {
                    outbox.submit_msg(ClientMsg::Chat { text });
                }
                state.open = false;
                state.buffer.clear();
                return;
            }
            Key::Backspace => {
                state.buffer.pop();
            }
            Key::Space if state.buffer.len() < MAX_CHAT_LEN => {
                state.buffer.push(' ');
            }
            Key::Character(s) => {
                for ch in s.chars() {
                    if state.buffer.len() >= MAX_CHAT_LEN {
                        break;
                    }
                    if !ch.is_control() {
                        state.buffer.push(ch);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Spawn / refresh / despawn the input bar to mirror `ChatInputState`.
pub fn sync_chat_ui(
    mut commands: Commands,
    state: Res<ChatInputState>,
    fonts: Option<Res<crate::theme::UiFonts>>,
    existing: Query<Entity, With<ChatInputBar>>,
    mut text_q: Query<&mut Text, With<ChatInputText>>,
) {
    let label = state
        .open
        .then(|| format!("💬 {}▏  (Enter to send · Esc to cancel)", state.buffer));
    match (label, existing.iter().next()) {
        (Some(label), None) => {
            let Some(fonts) = fonts else { return };
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(180.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ChatInputBar,
                    crate::systems::game_ui::InGameRoot,
                    Pickable::IGNORE,
                    GlobalZIndex(46),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(label),
                        ChatInputText,
                        fonts.tf(16.0),
                        TextColor(crate::theme::TEXT_INFO),
                        BackgroundColor(crate::theme::HUD_BG),
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
        (Some(label), Some(_)) => {
            for mut text in &mut text_q {
                if text.0 != label {
                    text.0 = label.clone();
                }
            }
        }
        (None, Some(e)) => {
            commands.entity(e).despawn();
        }
        (None, None) => {}
    }
}
