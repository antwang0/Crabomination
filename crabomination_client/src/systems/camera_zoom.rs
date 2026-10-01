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
/// whenever the seat count, the window size, the UI size or a duel's grown
/// creature rows change ([`crate::card::framing::home_pose`]: the closest
/// pose that keeps the table on screen and clear of the HUD).
#[derive(Resource)]
pub struct CameraHome {
    pub pose: Transform,
    /// The table point the home pose looks at.
    pub target: Vec3,
    /// Where the 3-D stack hangs for this pose (`framing::stack_lane`):
    /// beside the table, so a spell doesn't cover what it targets.
    pub stack_lane: crate::card::framing::StackLane,
    /// What the pose was fit for.
    fitted_for: Option<FitKey>,
    /// A refit for a new spread, running off the main thread.
    pending: Option<(FitKey, bevy::tasks::Task<Fit>)>,
    /// Fits for this window's spreads so far: a crowded duel goes back and
    /// forth between a few.
    cache: Vec<(FitKey, Fit)>,
}

/// `(seats, logical window size, UiScale bits, spread)`.
type FitKey = (usize, UVec2, u32, crate::card::Spread);

/// A fitted home: pose, the table point it looks at, and the stack lane.
#[derive(Clone, Copy)]
struct Fit {
    pose: Transform,
    target: Vec3,
    stack_lane: crate::card::framing::StackLane,
}

impl Fit {
    fn compute(key: FitKey) -> Self {
        let (n, size, scale, spread) = key;
        let (size, scale) = (size.as_vec2(), f32::from_bits(scale));
        let pose = crate::card::framing::home_pose(n, size, scale, &spread);
        // The fit looks down the pose's forward axis at the table plane.
        let forward = pose.forward();
        let target = pose.translation + forward * (-pose.translation.y / forward.y);
        let stack_lane = crate::card::framing::stack_lane(n, size, scale, &pose, &spread);
        Fit { pose, target, stack_lane }
    }
}

impl Default for CameraHome {
    fn default() -> Self {
        let pose = crate::card::framing::legacy_pose(2);
        Self { pose, target: Vec3::ZERO, stack_lane: Default::default(), fitted_for: None, pending: None, cache: Vec::new() }
    }
}

impl CameraHome {
    fn apply(&mut self, key: FitKey, fit: Fit) {
        self.pose = fit.pose;
        self.target = fit.target;
        self.stack_lane = fit.stack_lane;
        self.fitted_for = Some(key);
    }
}

/// Refit the home pose for the current seat count, window size and UI size
/// (the HUD panels it keeps the table clear of scale with it), and for a
/// duel's creature rows once one grows past the land row's width
/// (`layout::Spread`): the camera eases out to keep a crowded board on
/// screen and back as it thins. A fit is ~0.1 s in an unoptimized client,
/// so one for a new spread runs on a task pool thread (the camera eases
/// there once it lands); a new window or table fits at once, as the frame
/// it's for has nothing to show until it does.
pub fn adjust_camera_home_for_seats(
    view: Res<crate::net_plugin::CurrentView>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut home: ResMut<CameraHome>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    let Some(cv) = &view.0 else { return };
    let Ok(window) = windows.single() else { return };
    let size = window.resolution.size();
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }
    let n = cv.players.len();
    let spread = crate::card::Spread::of(&cv.battlefield, n);
    let key: FitKey = (n, size.as_uvec2(), ui_scale.0.to_bits(), spread);
    // A landed refit is kept, and applied if it's still the one wanted.
    if let Some((for_key, task)) = home.pending.as_mut()
        && let Some(fit) = bevy::tasks::block_on(bevy::tasks::futures_lite::future::poll_once(task))
    {
        let for_key = *for_key;
        home.pending = None;
        home.cache.push((for_key, fit));
        if for_key == key {
            home.apply(key, fit);
        }
    }
    if home.fitted_for == Some(key) {
        return;
    }
    let same_table = home.fitted_for.is_some_and(|(n0, s0, u0, _)| (n0, s0, u0) == (key.0, key.1, key.2));
    if !same_table {
        home.cache.clear();
        home.pending = None;
        let fit = Fit::compute(key);
        home.cache.push((key, fit));
        home.apply(key, fit);
        return;
    }
    if let Some(&(_, fit)) = home.cache.iter().find(|(k, _)| *k == key) {
        home.apply(key, fit);
        return;
    }
    if home.pending.as_ref().is_none_or(|(k, _)| *k != key) {
        let task = bevy::tasks::AsyncComputeTaskPool::get().spawn(async move { Fit::compute(key) });
        home.pending = Some((key, task));
    }
    // The frame loop idles on a still table; keep it polling.
    redraw.write(bevy::window::RequestRedraw);
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
    let (look_at, eye) = crate::card::layout::seat_focus(seat, viewer, n_seats);
    Transform::from_translation(eye).looking_at(look_at, Vec3::Y)
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

    // Exponential approach so it eases in/out and is frame-rate stable. It
    // never quite lands, so it snaps once it is within a hair, and a camera
    // at rest is not written at all: a moved camera re-projects every card
    // overlay, relays the UI out and keeps the reactive frame loop awake.
    if cam_xform.translation == target.translation && cam_xform.rotation == target.rotation {
        return;
    }
    let t = (CAM_LERP_SPEED * crate::systems::animate::anim_dt(&time)).clamp(0.0, 1.0);
    let translation = cam_xform.translation.lerp(target.translation, t);
    let rotation = cam_xform.rotation.slerp(target.rotation, t);
    let settled = translation.distance_squared(target.translation) < 1e-8
        && rotation.angle_between(target.rotation) < 1e-5;
    cam_xform.translation = if settled { target.translation } else { translation };
    cam_xform.rotation = if settled { target.rotation } else { rotation };
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
