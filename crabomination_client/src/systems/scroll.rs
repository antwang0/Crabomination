//! Mouse-wheel scrolling for `Overflow::scroll_y()` nodes.
//!
//! Bevy does not ship a wheel-to-scroll system: `bevy_ui` 0.19 contains no
//! `MouseWheel` reader at all, and `ScrollPosition`'s own docs note that
//! setting it "does nothing on a `Node` without setting at least one
//! `OverflowAxis` to `OverflowAxis::Scroll`" — but not the converse, which
//! is the trap. A node with `Overflow::scroll_y()` and nobody writing its
//! `ScrollPosition` *clips* its overflow and is silently unreachable.
//!
//! Four panels were in exactly that state (the game log, the graveyard and
//! exile browsers, and the library-search decision grid, whose comment
//! claimed "the mouse wheel pages through long candidate lists"). The two
//! that did work — the draft packs and the audit picker — each carried a
//! private copy of this handler, identical apart from the marker type and
//! the line-pixel constant. Tag a scrollable with [`Scrollable`] instead;
//! `handle_scroll` is registered once, unconditionally, and covers every
//! app state.
//!
//! `ScrollPosition` is a required component of `Node`, so tagging is all a
//! panel needs — there is nothing to insert alongside it.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

/// Default wheel speed: one `MouseScrollUnit::Line` detent ≈ this many
/// logical pixels, chosen so a detent advances roughly one card row.
pub const SCROLL_LINE_PX: f32 = 60.0;

/// Marker for a node that responds to the mouse wheel. The node must also
/// set `overflow: Overflow::scroll_y()`, or there is nothing to scroll.
///
/// ⚠ A scrollable that is a **flex item in a column container** also needs
/// `min_height: Val::Px(0.0)`. Flexbox's default `min-height: auto`
/// resolves to the content's height and outranks `max_height`, so the node
/// grows to fit its children, never overflows, and never scrolls no matter
/// what this handler writes.
#[derive(Component, Clone, Copy)]
pub struct Scrollable {
    /// Pixels advanced per wheel detent. [`SCROLL_LINE_PX`] unless a panel
    /// has a reason to differ (denser rows want less).
    pub line_px: f32,
}

impl Default for Scrollable {
    fn default() -> Self {
        Self { line_px: SCROLL_LINE_PX }
    }
}

impl Scrollable {
    /// A scrollable with a custom per-detent distance.
    #[allow(dead_code)]
    pub fn with_line_px(line_px: f32) -> Self {
        Self { line_px }
    }
}

/// Route this frame's wheel delta to whichever [`Scrollable`] the cursor is
/// over.
///
/// Both scroll units are handled: most desktop mice emit `Line(±1)` per
/// detent, while trackpads and Wayland surfaces emit `Pixel` deltas
/// directly. Deltas are aggregated into one scalar before the bounds check
/// so a flurry of events on a single frame doesn't re-run the hit test.
///
/// Only the lower bound is clamped — `ScrollPosition`'s docs state the
/// layout system normalises a position that a layout change has made
/// invalid, which covers the upper bound on the next frame.
pub fn handle_scroll(
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut scrollables: Query<(&ComputedNode, &UiGlobalTransform, &Scrollable, &mut ScrollPosition)>,
) {
    let mut lines = 0.0f32;
    let mut pixels = 0.0f32;
    for ev in wheel.read() {
        match ev.unit {
            MouseScrollUnit::Line => lines -= ev.y,
            MouseScrollUnit::Pixel => pixels -= ev.y,
        }
    }
    if lines == 0.0 && pixels == 0.0 {
        return;
    }
    let Ok(window) = windows.single() else { return };
    // Layout is in physical pixels (`ComputedNode::size`,
    // `UiGlobalTransform`), and so is a `Pixel` wheel delta; the scroll
    // offset is logical.
    let Some(cursor) = window.physical_cursor_position() else { return };
    let pixels = pixels / window.scale_factor();

    for (computed, transform, scrollable, mut scroll) in &mut scrollables {
        if !computed.contains_point(*transform, cursor) {
            continue;
        }
        // Per-panel line distance is applied here rather than at
        // aggregation time so nested scrollables with different speeds
        // stay independent.
        let delta_px = lines * scrollable.line_px + pixels;
        let new_y = (scroll.0.y + delta_px).max(0.0);
        if (new_y - scroll.0.y).abs() > f32::EPSILON {
            scroll.0.y = new_y;
        }
        // Topmost hovered scrollable consumes the tick, so nested
        // scrollables don't both move on one detent.
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::touch::TouchPhase;

    /// A panel laid out at logical (100, 50)-(300, 150) on a 2x display:
    /// `bevy_ui` keeps its size and position in physical pixels.
    fn panel(world: &mut World, logical_center: Vec2) -> Entity {
        world
            .spawn((
                Node::default(),
                Scrollable::default(),
                ComputedNode { size: Vec2::new(400.0, 200.0), inverse_scale_factor: 0.5, ..default() },
                UiGlobalTransform::from_translation(logical_center * 2.0),
            ))
            .id()
    }

    /// A wheel detent scrolls the panel under the cursor and no other. The
    /// hit test read the node's `GlobalTransform`, which a `bevy_ui` 0.19
    /// node doesn't have, so it matched nothing: the log, the zone browsers
    /// and the library search never scrolled.
    #[test]
    fn a_wheel_detent_scrolls_the_panel_under_the_cursor() {
        let mut world = World::new();
        world.init_resource::<Messages<MouseWheel>>();
        let mut window = Window::default();
        window.resolution.set_scale_factor_override(Some(2.0));
        window.set_cursor_position(Some(Vec2::new(150.0, 100.0)));
        world.spawn(window);
        let under = panel(&mut world, Vec2::new(200.0, 100.0));
        let elsewhere = panel(&mut world, Vec2::new(500.0, 300.0));
        world.write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window: Entity::PLACEHOLDER,
            phase: TouchPhase::Moved,
        });
        world.run_system_once(handle_scroll).unwrap();
        assert_eq!(world.get::<ScrollPosition>(under).unwrap().0.y, SCROLL_LINE_PX);
        assert_eq!(world.get::<ScrollPosition>(elsewhere).unwrap().0.y, 0.0);
    }
}
