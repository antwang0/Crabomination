//! Hold-Ctrl camera zoom.
//!
//! While either Ctrl key is held, the main camera dollies in on the
//! part of the board under the cursor — or onto the battlefield card the
//! cursor/keyboard is highlighting — keeping the same viewing angle but
//! roughly halving the distance. Releasing Ctrl lerps the camera back to
//! its home pose. This is purely a client-side view convenience;
//! it touches nothing but the camera `Transform`.
//!
//! The focus point under the cursor is raycast against the table plane
//! (`y = 0`) using the *home* camera pose, not the live (moving) one —
//! that decouples the cursor→world mapping from the camera's own motion,
//! so the focus follows the cursor smoothly instead of chasing itself.

use bevy::prelude::*;

use crate::MainCamera;
use crate::card::{BattlefieldCard, CardHovered};
use crate::systems::kb_cursor::KeyboardSelected;

/// Fraction of the home distance used when zoomed (smaller = closer).
const CAM_ZOOM_SCALE: f32 = 0.45;
/// Lerp rate toward the target pose (per second, exponential approach).
const CAM_LERP_SPEED: f32 = 7.0;

/// The camera's resting pose, refit by [`adjust_camera_home_for_seats`]
/// whenever the seat count or the window size changes
/// ([`crate::card::framing::home_pose`]: the closest pose that keeps the
/// table on screen and clear of the HUD).
#[derive(Resource)]
pub struct CameraHome {
    pub pose: Transform,
    /// The table point the home pose looks at.
    pub target: Vec3,
    /// `(seats, logical window size)` the pose was fit for.
    fitted_for: Option<(usize, UVec2)>,
}

impl Default for CameraHome {
    fn default() -> Self {
        let pose = crate::card::framing::legacy_pose(2);
        Self { pose, target: Vec3::ZERO, fitted_for: None }
    }
}

/// Refit the home pose for the current seat count and window size. The fit
/// is a few milliseconds and only runs when either changes.
pub fn adjust_camera_home_for_seats(
    view: Res<crate::net_plugin::CurrentView>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut home: ResMut<CameraHome>,
) {
    let Some(cv) = &view.0 else { return };
    let Ok(window) = windows.single() else { return };
    let size = window.resolution.size();
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }
    let key = (cv.players.len(), size.as_uvec2());
    if home.fitted_for == Some(key) {
        return;
    }
    let pose = crate::card::framing::home_pose(key.0, size);
    // The fit looks down the pose's forward axis at the table plane.
    let forward = pose.forward();
    let target = pose.translation + forward * (-pose.translation.y / forward.y);
    *home = CameraHome { pose, target, fitted_for: Some(key) };
}

/// Seat the camera is parked on via the seat-focus hotkeys (`1`–`6`).
/// `None` = the normal home pose. Pressing the focused seat's digit again
/// (or Escape) returns home.
#[derive(Resource, Default)]
pub struct CameraFocusSeat(pub Option<usize>);

/// Digit hotkeys park the camera over a seat's shoulder — `1` is seat 0,
/// `2` seat 1, and so on — so far-edge boards in a multiplayer pod can be
/// inspected up close (their cards read upright from behind their edge).
/// Suppressed while any text surface owns the keyboard.
pub fn camera_focus_hotkeys(
    keyboard: Res<ButtonInput<KeyCode>>,
    view: Res<crate::net_plugin::CurrentView>,
    text_input: crate::systems::input_guard::TextInputGuard,
    esc: Res<crate::systems::esc::EscFocus>,
    mut focus: ResMut<CameraFocusSeat>,
) {
    let Some(cv) = &view.0 else {
        focus.0 = None;
        return;
    };
    let n = cv.players.len();
    // A stale focus (seat count shrank between games) snaps home.
    if focus.0.is_some_and(|s| s >= n) {
        focus.0 = None;
    }
    if text_input.typing() {
        return;
    }
    const DIGITS: [KeyCode; 6] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ];
    for (seat, key) in DIGITS.iter().enumerate() {
        if seat < n && keyboard.just_pressed(*key) {
            focus.0 = if focus.0 == Some(seat) { None } else { Some(seat) };
        }
    }
    // `EscSurface::CameraFocus` is last in the precedence order, just
    // ahead of the unclaimed-press fallback that opens the pause menu.
    // `compute_esc_focus` only names it when a seat is actually focused,
    // so an unfocused camera cannot swallow that press.
    if esc.owns(crate::systems::esc::EscSurface::CameraFocus) {
        focus.0 = None;
    }
}

/// Over-the-shoulder pose for a focused seat: hovering behind that seat's
/// table edge, looking down at the middle of their board rows.
fn focus_pose(seat: usize, viewer: usize, n_seats: usize) -> Transform {
    let spot = crate::card::layout::seat_spot(seat, viewer, n_seats);
    let look_at = Vec3::new(spot.board_center, 0.0, spot.z_sign * 6.0);
    let pos = look_at + Vec3::new(0.0, 20.0, spot.z_sign * 14.0);
    Transform::from_translation(pos).looking_at(look_at, Vec3::Y)
}

/// Last computed focus point, held across frames so the zoom stays put
/// when the cursor briefly leaves the table (e.g. drifts over a UI
/// panel) while Ctrl is still down.
#[derive(Resource)]
pub struct CameraZoom {
    pub focus: Vec3,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self { focus: Vec3::ZERO }
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_zoom(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    windows: Query<&Window>,
    mut zoom: ResMut<CameraZoom>,
    hovered_bf: Query<&GlobalTransform, (With<BattlefieldCard>, With<CardHovered>)>,
    // The keyboard-selected card (WASD/arrow navigation). Preferred over the
    // mouse hover so the two don't fight: `sync_kb_hover_marker` mirrors
    // `KeyboardSelected` into `CardHovered`, so while navigating with the
    // keyboard *two* cards can carry `CardHovered` (the kb card + whatever
    // the mouse happens to be over). Picking the kb card deterministically
    // keeps the zoom focus from stuttering between them.
    kb_selected_bf: Query<&GlobalTransform, (With<BattlefieldCard>, With<KeyboardSelected>)>,
    home_pose: Res<CameraHome>,
    focus_seat: Res<CameraFocusSeat>,
    view: Res<crate::net_plugin::CurrentView>,
    mut camera: Query<(&mut Transform, &Camera), With<MainCamera>>,
) {
    let Ok((mut cam_xform, camera)) = camera.single_mut() else { return };

    let home = home_pose.pose;
    // The home pose's offset from the point it looks at: a zoom keeps the
    // angle and scales the distance.
    let home_offset = home.translation - home_pose.target;

    let ctrl_held =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    let target = if ctrl_held {
        // Focus priority: the keyboard-selected card (stable while using
        // WASD) → the mouse-hovered card → the cursor raycast onto the
        // table plane through the *home* camera pose.
        if let Some(card) = kb_selected_bf.iter().next() {
            zoom.focus = card.translation();
        } else if let Some(card) = hovered_bf.iter().next() {
            zoom.focus = card.translation();
        } else if let Some(point) = cursor_on_table(&windows, camera, &home) {
            zoom.focus = point;
        }
        // (else: keep the previously stored focus)
        let pos = zoom.focus + home_offset * CAM_ZOOM_SCALE;
        Transform::from_translation(pos).looking_at(zoom.focus, Vec3::Y)
    } else if let (Some(seat), Some(cv)) = (focus_seat.0, view.0.as_ref()) {
        // Seat focus (hotkeys 1-6): hold-Ctrl inspection still wins above.
        focus_pose(seat, cv.your_seat, cv.players.len())
    } else {
        home
    };

    // Exponential approach so it eases in/out and is frame-rate stable.
    let t = (CAM_LERP_SPEED * time.delta_secs()).clamp(0.0, 1.0);
    cam_xform.translation = cam_xform.translation.lerp(target.translation, t);
    cam_xform.rotation = cam_xform.rotation.slerp(target.rotation, t);
}

/// Raycast the cursor against the table plane (`y = 0`) using the fixed
/// `home` camera pose. Returns the world point under the cursor, or
/// `None` if there's no cursor or the ray is parallel to / behind the
/// plane.
fn cursor_on_table(
    windows: &Query<&Window>,
    camera: &Camera,
    home: &Transform,
) -> Option<Vec3> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let home_global = GlobalTransform::from(*home);
    let ray = camera.viewport_to_world(&home_global, cursor).ok()?;
    let dy = ray.direction.y;
    if dy.abs() < 1.0e-5 {
        return None;
    }
    let dist = -ray.origin.y / dy;
    (dist > 0.0).then(|| ray.get_point(dist))
}
