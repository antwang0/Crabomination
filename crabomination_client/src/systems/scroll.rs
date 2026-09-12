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
use bevy::ui::ComputedNode;

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
    mut scrollables: Query<(&ComputedNode, &GlobalTransform, &Scrollable, &mut ScrollPosition)>,
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
    let Some(cursor) = window.cursor_position() else { return };

    for (computed, gtf, scrollable, mut scroll) in &mut scrollables {
        // `ComputedNode::size()` is in logical pixels; the global
        // translation is the node's centre in screen space.
        let half = computed.size() * 0.5;
        let center = gtf.translation().truncate();
        let inside = cursor.x >= center.x - half.x
            && cursor.x <= center.x + half.x
            && cursor.y >= center.y - half.y
            && cursor.y <= center.y + half.y;
        if !inside {
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
