//! The client's clickable-UI-node marker and its press/hover state.
//!
//! Bevy 0.20 deprecated `bevy::ui::Button` and `bevy::ui::Interaction` (to be
//! removed in 0.21) in favour of `ui_widgets::Button`, which reports through
//! `Pressed` / `Hovered` components and an `Activate` event on *release*. The
//! client's ~200 button handlers poll `Interaction::Pressed` and act on the
//! *press*, and `ui_widgets::Button` presses on any mouse button (the right-
//! click path on buttons would fire the left-click action) and stops picking
//! events from bubbling. So the old pair lives on here, driven by the picking
//! `HoverMap` the UI already curates with `Pickable::IGNORE`:
//!
//! - a left click or touch on a hovered button sets `Pressed`;
//! - `Pressed` holds, even off the button, until that press is released;
//! - otherwise a button is `Hovered` while the pointer is over it or a child.
//!
//! Hits are credited to the nearest `Interaction` ancestor, as Bevy's
//! `FocusPolicy::Block` on its `Button` made the topmost button capture the
//! pointer: a text label gives its hover to the button around it, and a
//! button nested in another gives it to neither parent.

use bevy::picking::PickingSystems;
use bevy::picking::hover::HoverMap;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

/// Marks a UI node as clickable; it gets an [`Interaction`].
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[require(Node, Interaction)]
pub struct Button;

/// A clickable node's pointer state, updated in `PreUpdate`.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interaction {
    /// The node is being pressed.
    Pressed,
    /// The pointer is over the node.
    Hovered,
    /// Nothing is happening.
    #[default]
    None,
}

pub struct UiButtonPlugin;

impl Plugin for UiButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_interaction.after(PickingSystems::Hover));
    }
}

fn update_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    hover_map: Option<Res<HoverMap>>,
    parents: Query<&ChildOf>,
    mut nodes: Query<(Entity, &mut Interaction, Option<&InheritedVisibility>)>,
    mut hovered: Local<HashSet<Entity>>,
    mut to_reset: Local<Vec<Entity>>,
) {
    // A tap pressed and released in one frame shows `Pressed` for exactly
    // that frame.
    for entity in to_reset.drain(..) {
        if let Ok((_, mut interaction, _)) = nodes.get_mut(entity) {
            interaction.set_if_neq(Interaction::None);
        }
    }
    hovered.clear();
    for hit in hover_map.iter().flat_map(|m| m.values()).flat_map(|hits| hits.keys()) {
        let mut entity = *hit;
        loop {
            if nodes.contains(entity) {
                hovered.insert(entity);
                break;
            }
            let Ok(parent) = parents.get(entity) else { break };
            entity = parent.parent();
        }
    }

    let released = mouse.just_released(MouseButton::Left) || touches.any_just_released();
    let clicked = mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed();
    for (entity, mut interaction, visibility) in &mut nodes {
        let visible = visibility.is_none_or(|v| v.get());
        let mut next = *interaction;
        if released && next == Interaction::Pressed {
            next = Interaction::None;
        }
        if visible && hovered.contains(&entity) {
            if clicked {
                next = Interaction::Pressed;
                if released {
                    to_reset.push(entity);
                }
            } else if next == Interaction::None {
                next = Interaction::Hovered;
            }
        } else if next == Interaction::Hovered || !visible {
            next = Interaction::None;
        }
        // Change detection is the handlers' edge: `Changed<Interaction>`.
        interaction.set_if_neq(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::picking::backend::HitData;
    use bevy::picking::pointer::PointerId;

    fn hover(app: &mut App, over: &[Entity]) {
        let camera = Entity::PLACEHOLDER;
        let hits = over.iter().map(|e| (*e, HitData::new(camera, 0.0, None, None))).collect();
        app.world_mut().resource_mut::<HoverMap>().0 = [(PointerId::Mouse, hits)].into_iter().collect();
    }

    fn mouse(app: &mut App, f: impl FnOnce(&mut ButtonInput<MouseButton>)) {
        let mut input = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        input.clear();
        f(&mut input);
    }

    fn state(app: &App, e: Entity) -> Interaction {
        *app.world().get::<Interaction>(e).unwrap()
    }

    /// The transitions of Bevy 0.19's `ui_focus_system`, which every
    /// `Changed<Interaction>` handler in the client was written against.
    #[test]
    fn a_left_press_holds_until_release_and_the_topmost_button_takes_it() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<Touches>()
            .init_resource::<HoverMap>()
            .add_systems(Update, update_interaction);
        let outer = app.world_mut().spawn(Interaction::None).id();
        let label = app.world_mut().spawn(ChildOf(outer)).id();
        let inner = app.world_mut().spawn((Interaction::None, ChildOf(outer))).id();

        // A label's hover is its button's.
        hover(&mut app, &[label]);
        app.update();
        assert_eq!((state(&app, outer), state(&app, inner)), (Interaction::Hovered, Interaction::None));

        // Right-click does not press.
        mouse(&mut app, |m| m.press(MouseButton::Right));
        app.update();
        assert_eq!(state(&app, outer), Interaction::Hovered);

        // Left press holds off the button until it is released.
        mouse(&mut app, |m| m.press(MouseButton::Left));
        app.update();
        assert_eq!(state(&app, outer), Interaction::Pressed);
        hover(&mut app, &[]);
        mouse(&mut app, |_| {});
        app.update();
        assert_eq!(state(&app, outer), Interaction::Pressed);
        mouse(&mut app, |m| m.release(MouseButton::Left));
        app.update();
        assert_eq!(state(&app, outer), Interaction::None);

        // A nested button captures the pointer from the one around it.
        hover(&mut app, &[inner]);
        mouse(&mut app, |_| {});
        app.update();
        assert_eq!((state(&app, outer), state(&app, inner)), (Interaction::None, Interaction::Hovered));

        // A press and release in one frame is `Pressed` for that frame only.
        mouse(&mut app, |m| {
            m.press(MouseButton::Left);
            m.release(MouseButton::Left);
        });
        app.update();
        assert_eq!(state(&app, inner), Interaction::Pressed);
        mouse(&mut app, |_| {});
        app.update();
        assert_eq!(state(&app, inner), Interaction::Hovered);
    }
}
