use bevy::prelude::*;

use super::components::{CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH};

pub const HAND_CENTER_Z: f32 = 8.0; // how far toward the camera the hand sits
pub const HAND_Y: f32 = CARD_HEIGHT / 2.0; // centre of upright card sits at half its height
pub const HAND_CARD_SPACING: f32 = CARD_WIDTH * 0.7;
pub const HAND_FAN_ANGLE: f32 = 0.06; // radians of Z-rotation per card from center
pub const HAND_FAN_Y_DROP: f32 = 0.3; // Y drop per card away from center

// Tilt so the card's face normal is parallel to the camera's view direction.
// Camera at (0, 20, 12) looking at (0, 1, 0): forward = (0, -19, -12), angle = -atan2(12, 19) ≈ -0.56 rad around X.
const HAND_TILT_X: f32 = -0.56;

/// Returns the world-space transform a hand card should have at `slot` out of `total`.
pub fn hand_card_transform(slot: usize, total: usize) -> Transform {
    let center = (total as f32 - 1.0) / 2.0;
    let offset = slot as f32 - center;
    let x = offset * HAND_CARD_SPACING;
    let y = HAND_Y - offset.abs() * HAND_FAN_Y_DROP;
    // Lower slot index = closer to camera, so left cards overlap right cards.
    let z_offset = if total > 0 { (total - 1 - slot) as f32 } else { 0.0 };
    let z = HAND_CENTER_Z + z_offset * CARD_THICKNESS * 4.0;
    let rot_z = -offset * HAND_FAN_ANGLE;
    Transform::from_xyz(x, y, z)
        .with_rotation(Quat::from_rotation_x(HAND_TILT_X) * Quat::from_rotation_z(rot_z))
}
