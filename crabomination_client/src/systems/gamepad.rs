//! Gamepad play: each pad button presses the key it stands for, so the
//! pad drives everything the keyboard does — the selection cursor
//! (`kb_cursor`), prompts (`modal_keys`), every remappable shortcut — with no
//! second input path to keep in step.
//!
//! | Pad | Key | Does |
//! |---|---|---|
//! | D-pad | arrows | move the cursor / a prompt's focus |
//! | A (south) | Enter | play / select / press |
//! | B (east) | Esc | cancel / back |
//! | X (west) | Pass's key | pass priority |
//! | Y (north) | Hint's key | hint |
//! | RB | Attack's key | attack / confirm attackers |
//! | LB | End Turn's key | end turn |
//! | Select | Ability menu's key | open the ability menu |
//! | Start | F1 / Help's key | the shortcut overlay |
//!
//! The press is injected into `ButtonInput<KeyCode>` after Bevy's own input
//! pass, so a pad press is `just_pressed` for this frame exactly as a key's
//! is, and released with the pad button.

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::keybinds::{Action, Keybinds};

/// The key a pad button stands for, by the live bindings.
pub fn key_for(button: GamepadButton, binds: &Keybinds) -> Option<KeyCode> {
    Some(match button {
        GamepadButton::DPadUp => KeyCode::ArrowUp,
        GamepadButton::DPadDown => KeyCode::ArrowDown,
        GamepadButton::DPadLeft => KeyCode::ArrowLeft,
        GamepadButton::DPadRight => KeyCode::ArrowRight,
        GamepadButton::South => KeyCode::Enter,
        GamepadButton::East => KeyCode::Escape,
        GamepadButton::West => binds.key(Action::Pass),
        GamepadButton::North => binds.key(Action::Hint),
        GamepadButton::RightTrigger => binds.key(Action::Attack),
        GamepadButton::LeftTrigger => binds.key(Action::EndTurn),
        GamepadButton::Select => binds.key(Action::AbilityMenu),
        GamepadButton::Start => binds.key(Action::ToggleHelp),
        _ => return None,
    })
}

pub fn gamepad_as_keys(gamepads: Query<&Gamepad>, binds: Res<Keybinds>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    for pad in &gamepads {
        for &button in pad.get_just_pressed() {
            if let Some(key) = key_for(button, &binds) {
                keys.press(key);
            }
        }
        for &button in pad.get_just_released() {
            if let Some(key) = key_for(button, &binds) {
                keys.release(key);
            }
        }
    }
}

pub struct GamepadKeysPlugin;

impl Plugin for GamepadKeysPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, gamepad_as_keys.after(InputSystems));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_buttons_follow_the_live_bindings() {
        let mut binds = Keybinds::default();
        assert_eq!(key_for(GamepadButton::West, &binds), Some(KeyCode::Space));
        assert_eq!(key_for(GamepadButton::South, &binds), Some(KeyCode::Enter));
        binds.set(Action::Pass, KeyCode::KeyJ);
        assert_eq!(key_for(GamepadButton::West, &binds), Some(KeyCode::KeyJ), "a remap moves the pad too");
        assert_eq!(key_for(GamepadButton::Mode, &binds), None);
    }

    /// A pad press reaches the keyboard input as a fresh key press.
    #[test]
    fn a_pad_press_is_a_key_press() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(key_for(GamepadButton::South, &Keybinds::default()).unwrap());
        assert!(keys.just_pressed(KeyCode::Enter));
    }
}
