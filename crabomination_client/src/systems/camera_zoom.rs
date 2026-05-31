//! Hold-Ctrl camera zoom.
//!
//! While either Ctrl key is held, the main camera dollies in on the
//! part of the board under the cursor — or onto the battlefield card the
//! cursor/keyboard is highlighting — keeping the same viewing angle but
//! roughly halving the distance. Releasing Ctrl lerps the camera back to
//! its fixed home pose. This is purely a client-side view convenience;
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

/// Fixed "home" camera position (matches the spawn in `main::setup`).
const CAM_HOME_POS: Vec3 = Vec3::new(0.0, 32.0, 14.0);
/// Fraction of the home distance used when zoomed (smaller = closer).
const CAM_ZOOM_SCALE: f32 = 0.45;
/// Lerp rate toward the target pose (per second, exponential approach).
const CAM_LERP_SPEED: f32 = 7.0;

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
    mut camera: Query<(&mut Transform, &Camera), With<MainCamera>>,
) {
    let Ok((mut cam_xform, camera)) = camera.single_mut() else { return };

    let home = Transform::from_translation(CAM_HOME_POS).looking_at(Vec3::ZERO, Vec3::Y);

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
        let pos = zoom.focus + CAM_HOME_POS * CAM_ZOOM_SCALE;
        Transform::from_translation(pos).looking_at(zoom.focus, Vec3::Y)
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
