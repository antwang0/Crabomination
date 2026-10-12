//! Mouse-wheel scrolling for `Overflow::scroll_y()` nodes.
//!
//! Tag a scrollable panel with Bevy's `ui_widgets::ScrollArea` (part of
//! `DefaultPlugins` since 0.20). A wheel or trackpad event over the panel,
//! or over anything inside it, bubbles up through picking to the nearest
//! `ScrollArea` and moves its `ScrollPosition`, clamped to the content. So
//! the panel must be pickable: not `Pickable::IGNORE` itself, though its
//! rows may be.
//!
//! Until 0.20 the client carried its own handler: `bevy_ui` 0.19 read no
//! `MouseWheel` at all, and a node with `Overflow::scroll_y()` and nobody
//! writing its `ScrollPosition` *clips* its overflow and is silently
//! unreachable — four panels were in exactly that state (the game log, the
//! graveyard and exile browsers, and the library-search grid).
//!
//! ⚠ A scrollable that is a **flex item in a column container** also needs
//! `min_height: Val::Px(0.0)`. Flexbox's default `min-height: auto`
//! resolves to the content's height and outranks `max_height`, so the node
//! grows to fit its children, never overflows, and never scrolls.

use bevy::input::mouse::MouseScrollPixelsPerLine;
use bevy::prelude::*;

/// Wheel speed: one `MouseScrollUnit::Line` detent ≈ this many logical
/// pixels, chosen so a detent advances roughly one card row. Bevy's step is
/// one global setting; the client's own handler let the denser lists (deck
/// picker, saved decks, deck import) step 22-28 px instead.
pub const SCROLL_LINE_PX: f32 = 60.0;

pub struct ScrollPlugin;

impl Plugin for ScrollPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, |mut step: ResMut<MouseScrollPixelsPerLine>| **step = SCROLL_LINE_PX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::NormalizedRenderTarget;
    use bevy::input::mouse::MouseScrollUnit;
    use bevy::input::touch::TouchPhase;
    use bevy::picking::backend::HitData;
    use bevy::picking::events::{Pointer, PointerScroll};
    use bevy::picking::pointer::{Location, PointerId};
    use bevy::ui::ComputedNode;
    use bevy::ui_widgets::{ScrollArea, ScrollAreaPlugin};

    /// A wheel detent over a row scrolls the panel the row is in by
    /// [`SCROLL_LINE_PX`]. The client's first handler hit-tested the node's
    /// `GlobalTransform`, which a `bevy_ui` node doesn't have, so it matched
    /// nothing: the log, the zone browsers and the library search never
    /// scrolled.
    #[test]
    fn a_wheel_detent_over_a_row_scrolls_its_panel() {
        let mut app = App::new();
        app.add_plugins(ScrollAreaPlugin)
            .init_resource::<MouseScrollPixelsPerLine>()
            .add_plugins(ScrollPlugin);
        // Picking's traversal reads `Option<&Window>`, which fails on a world
        // that has never registered `Window` — and then nothing bubbles.
        app.world_mut().register_component::<Window>();
        app.update();
        let panel = app
            .world_mut()
            .spawn((
                Node { overflow: Overflow::scroll_y(), ..default() },
                ScrollArea,
                ComputedNode {
                    size: Vec2::new(200.0, 100.0),
                    content_size: Vec2::new(200.0, 1000.0),
                    inverse_scale_factor: 1.0,
                    ..default()
                },
            ))
            .id();
        let row = app.world_mut().spawn((Node::default(), ChildOf(panel))).id();
        let location = Location {
            target: NormalizedRenderTarget::None { width: 800, height: 600 },
            position: Vec2::ZERO,
        };
        app.world_mut().trigger(PointerScroll {
            entity: row,
            pointer: Pointer::new(PointerId::Mouse, location),
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            phase: TouchPhase::Moved,
        });
        app.world_mut().flush();
        assert_eq!(app.world().get::<ScrollPosition>(panel).unwrap().y, SCROLL_LINE_PX);
    }
}
