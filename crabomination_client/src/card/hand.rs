use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::components::{CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH};

use crabomination::{card::CardId, game::GameState};

pub const HAND_CENTER_Z: f32 = 12.0; // how far toward the camera the hand sits
pub const HAND_Y: f32 = CARD_HEIGHT / 2.0; // centre of upright card sits at half its height
pub const HAND_CARD_SPACING: f32 = CARD_WIDTH * 0.85;
pub const HAND_FAN_ANGLE: f32 = 0.06; // radians of Z-rotation per card from center
pub const HAND_FAN_Y_DROP: f32 = 0.3; // Y drop per card away from center

// Tilt so the card's face normal is parallel to the camera's view direction.
// Camera at (0, 28, 18) looking at (0, 0, 0): forward = (0, -28, -18), angle = -atan2(18, 28) ≈ -0.57 rad around X.
const HAND_TILT_X: f32 = -0.57;

/// Returns the world-space transform a hand card should have at `slot` out of `total`.
pub fn hand_card_transform(slot: usize, total: usize) -> Transform {
    let center = (total as f32 - 1.0) / 2.0;
    let offset = slot as f32 - center;
    let x = offset * HAND_CARD_SPACING;
    let y = HAND_Y - offset.abs() * HAND_FAN_Y_DROP;
    // Lower slot index = closer to camera, so left cards overlap right cards.
    let z_offset = if total > 0 { (total - 1).saturating_sub(slot) as f32 } else { 0.0 };
    let z = HAND_CENTER_Z + z_offset * CARD_THICKNESS * 4.0;
    let rot_z = -offset * HAND_FAN_ANGLE;
    Transform::from_xyz(x, y, z)
        .with_rotation(Quat::from_rotation_x(HAND_TILT_X) * Quat::from_rotation_z(rot_z))
}

/// Player 1 hand: face-down cards fanned at the far side (mirrored from player 0 hand).
pub fn p1_hand_card_transform(slot: usize, total: usize) -> Transform {
    let center = (total as f32 - 1.0) / 2.0;
    let offset = slot as f32 - center;
    let x = offset * HAND_CARD_SPACING;
    let y = HAND_Y + 3.0 - offset.abs() * HAND_FAN_Y_DROP; // extra lift so cards clear the table at the far camera angle
    // Mirror z_offset so rightmost card (slot 0) is closest to bot (matches human mirroring)
    let z_offset = if total > 0 { (total - 1).saturating_sub(slot) as f32 } else { 0.0 };
    let z = -(HAND_CENTER_Z + 2.0 + z_offset * CARD_THICKNESS * 4.0); // extra 2 units back
    let rot_z = offset * HAND_FAN_ANGLE; // mirror fan direction
    // Tilt toward camera from bot's side, flipped Z (PI) so back faces up
    Transform::from_xyz(x, y, z)
        .with_rotation(Quat::from_rotation_x(-HAND_TILT_X) * Quat::from_rotation_z(PI + rot_z))
}

// ── Battlefield layout ───────────────────────────────────────────────────────

const BF_Y: f32 = 0.02; // just above the ground plane
// Use CARD_HEIGHT as the spacing unit so tapped cards (rotated 90°, now CARD_HEIGHT wide along X) don't overlap.
const BF_CARD_SPACING: f32 = CARD_HEIGHT * 1.1;

// Rows must be > CARD_HEIGHT (~4.19) apart so flat cards don't overlap.
// Creature rows must each be > CARD_HEIGHT/2 (~2.1) from z=0 so opposing rows don't overlap.
// P0_CREATURE_Z=3.5: player 0 [1.4,5.6], player 1 [-5.6,-1.4] → 2.8-unit gap at center.
// P0_LAND_Z=8.5: land top at 6.4, creature bottom at 5.6 → 0.8-unit gap.
const P0_CREATURE_Z: f32 = 3.5;
const P0_LAND_Z: f32 = 8.5;

// Player 1 rows (mirrored).
const P1_CREATURE_Z: f32 = -3.5;
const P1_LAND_Z: f32 = -8.5;

/// How much each card in an identical-land stack is offset from the previous.
pub const LAND_STACK_OFFSET_X: f32 = 0.18;
// Each successive card (higher index = on top) shifts +Z, leaving the previous card's name
// strip (at the low-Z edge) uncovered and visible.
pub const LAND_STACK_OFFSET_Z: f32 = 0.35;

/// Returns `(group_slot, index_in_group, total_groups)` for a land card, grouping
/// identical lands (same name) into stacks.  Returns `None` if the card isn't found.
pub fn land_group_info(state: &GameState, owner: usize, card_id: CardId) -> Option<(usize, usize, usize)> {
    let lands: Vec<_> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == owner && c.definition.is_land())
        .collect();

    let target = lands.iter().find(|c| c.id == card_id)?;

    // Sorted unique names → stable group ordering
    let mut names: Vec<&'static str> = lands.iter().map(|c| c.definition.name).collect();
    names.sort_unstable();
    names.dedup();

    let group_slot = names.iter().position(|&n| n == target.definition.name)?;
    let index_in_group = lands
        .iter()
        .filter(|c| c.definition.name == target.definition.name)
        .position(|c| c.id == card_id)?;

    Some((group_slot, index_in_group, names.len()))
}

/// Returns the world-space transform for a battlefield card.
/// `is_land` selects the back row (lands) vs front row (creatures).
/// `is_bot` selects which side of the table.
/// `tapped` applies a 90° rotation for tapped cards.
pub fn bf_card_transform(slot: usize, total: usize, is_land: bool, is_bot: bool, tapped: bool) -> Transform {
    let center = (total as f32 - 1.0) / 2.0;
    let offset = slot as f32 - center;
    let x = offset * BF_CARD_SPACING;

    let z = match (is_bot, is_land) {
        (false, false) => P0_CREATURE_Z,
        (false, true) => P0_LAND_Z,
        (true, false) => P1_CREATURE_Z,
        (true, true) => P1_LAND_Z,
    };

    let base_rot = if is_bot {
        Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI)
    } else {
        Quat::from_rotation_x(-FRAC_PI_2)
    };

    let rot = if tapped {
        Quat::from_rotation_y(-FRAC_PI_2) * base_rot
    } else {
        base_rot
    };

    Transform::from_xyz(x, BF_Y, z).with_rotation(rot)
}
