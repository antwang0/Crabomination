//! Keyboard play inside modals: while a decision prompt (or the alt-cost
//! picker, or the game-over screen) is up, Tab / the arrows move a focus
//! ring across its buttons and Enter or Space presses the focused one.
//!
//! The 3-D selection cursor (`kb_cursor`) stands down while a modal is up,
//! so the same keys never drive both. A press is the button's own
//! `Interaction::Pressed` for exactly one frame, set right after
//! `ui_button`'s pointer pass, so every existing click handler answers a
//! key exactly as it answers a click.

use bevy::prelude::*;

use crate::ui_button::{Button, Interaction};

/// The focused modal button, and the one a key pressed last frame (to
/// release).
#[derive(Resource, Default)]
pub struct ModalFocus {
    pub focused: Option<Entity>,
    pressed: Option<Entity>,
}

/// The modal roots keyboard focus walks.
pub type ModalRoots = Or<(
    With<crate::systems::decision_ui::DecisionModal>,
    With<crate::systems::game_ui::AltCastModal>,
    With<crate::systems::game_over::GameOverModalRoot>,
    With<crate::systems::intro::IntroPanel>,
)>;

/// Whether a keyboard-walkable modal is up — `kb_cursor` yields then.
pub fn modal_open(roots: &Query<(), ModalRoots>) -> bool {
    !roots.is_empty()
}

/// `root`'s buttons, depth first in spawn order (reading order).
fn buttons_under(root: Entity, children: &Query<&Children>, buttons: &Query<(), With<Button>>, out: &mut Vec<Entity>) {
    if buttons.contains(root) {
        out.push(root);
    }
    if let Ok(kids) = children.get(root) {
        for &kid in kids {
            buttons_under(kid, children, buttons, out);
        }
    }
}

/// Step `current` through `list` by `delta` (wrapping); the first one when
/// nothing is focused yet.
fn step(list: &[Entity], current: Option<Entity>, delta: isize) -> Option<Entity> {
    if list.is_empty() {
        return None;
    }
    let at = current.and_then(|c| list.iter().position(|e| *e == c));
    let n = list.len() as isize;
    let next = match at {
        Some(i) => (i as isize + delta).rem_euclid(n),
        None => 0,
    };
    Some(list[next as usize])
}

#[allow(clippy::too_many_arguments)]
pub fn modal_keyboard_nav(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    text_input: crate::systems::input_guard::TextInputGuard,
    roots: Query<Entity, ModalRoots>,
    children: Query<&Children>,
    buttons: Query<(), With<Button>>,
    mut interactions: Query<&mut Interaction>,
    mut focus: ResMut<ModalFocus>,
) {
    // Release last frame's key press, so a handler sees one press.
    if let Some(e) = focus.pressed.take()
        && let Ok(mut i) = interactions.get_mut(e)
        && *i == Interaction::Pressed
    {
        *i = Interaction::None;
    }
    let mut list = Vec::new();
    for root in &roots {
        buttons_under(root, &children, &buttons, &mut list);
    }
    if list.is_empty() {
        if let Some(old) = focus.focused.take() {
            commands.entity(old).try_remove::<Outline>();
        }
        return;
    }
    if text_input.typing() {
        return;
    }
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let delta = if keyboard.just_pressed(KeyCode::Tab) {
        Some(if shift { -1 } else { 1 })
    } else if keyboard.any_just_pressed([KeyCode::ArrowDown, KeyCode::ArrowRight]) {
        Some(1)
    } else if keyboard.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowLeft]) {
        Some(-1)
    } else {
        None
    };
    let stale = focus.focused.is_none_or(|f| !list.contains(&f));
    let next = match (delta, stale) {
        (Some(d), _) => step(&list, focus.focused.filter(|_| !stale), d),
        (None, true) => step(&list, None, 0),
        (None, false) => focus.focused,
    };
    if next != focus.focused {
        if let Some(old) = focus.focused {
            commands.entity(old).try_remove::<Outline>();
        }
        if let Some(new) = next {
            commands.entity(new).insert(Outline::new(Val::Px(2.0), Val::Px(1.0), crate::theme::ACCENT_YELLOW));
        }
        focus.focused = next;
    }
    if keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space])
        && let Some(f) = focus.focused
        && let Ok(mut i) = interactions.get_mut(f)
    {
        *i = Interaction::Pressed;
        focus.pressed = Some(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_steps_and_wraps() {
        let list: Vec<Entity> = (1..=3).map(Entity::from_raw_u32).map(Option::unwrap).collect();
        assert_eq!(step(&list, None, 1), Some(list[0]), "the first press lands on the first button");
        assert_eq!(step(&list, Some(list[0]), 1), Some(list[1]));
        assert_eq!(step(&list, Some(list[2]), 1), Some(list[0]), "wraps forward");
        assert_eq!(step(&list, Some(list[0]), -1), Some(list[2]), "wraps back");
        assert_eq!(step(&[], None, 1), None);
    }
}
