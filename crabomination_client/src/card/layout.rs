//! Seat-keyed layout helpers. All position/transform functions take a
//! `viewer_seat` so they return anchors relative to the camera: the viewer's
//! own seat sits at the front of the table, opponents at the back.
//!
//! For 2-player games the layout matches the historical hardcoded P0/P1
//! constants exactly. For 3+ player games, opponents share the back of the
//! table and are spread along X.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use crabomination::card::CardId;

use super::components::{CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH};

// ── Hand layout constants ────────────────────────────────────────────────────

pub const HAND_CARD_SPACING: f32 = CARD_WIDTH * 0.85;
pub const HAND_FAN_ANGLE: f32 = 0.06;
pub const HAND_FAN_Y_DROP: f32 = 0.3;

const HAND_CENTER_Z: f32 = 12.0;
const HAND_Y: f32 = CARD_HEIGHT / 2.0;
// Tilt so the viewer's hand cards face the camera (camera at y=22, z=26).
const HAND_TILT_X: f32 = -0.57;

// ── Battlefield layout constants ─────────────────────────────────────────────

const BF_Y: f32 = 0.02;
const BF_CARD_SPACING: f32 = CARD_HEIGHT * 1.1;
/// Distance from center to creature row (viewer side; opponent is mirrored).
const BF_CREATURE_Z: f32 = 3.5;
/// Distance from center to land row.
const BF_LAND_Z: f32 = 8.5;

// ── Pile constants ───────────────────────────────────────────────────────────

const DECK_X: f32 = 11.0;
const DECK_Z: f32 = 9.5;
const GRAVEYARD_X: f32 = 11.0;
const GRAVEYARD_Z: f32 = 4.0;

pub const LAND_STACK_OFFSET_X: f32 = 0.18;
pub const LAND_STACK_OFFSET_Z: f32 = 0.35;

/// X spread per opponent slot when there are multiple opponents.
const OPP_X_SPREAD: f32 = 14.0;

// ── Seat helpers ─────────────────────────────────────────────────────────────

/// True if `seat` is the viewer (camera-side player).
pub fn is_viewer(seat: usize, viewer: usize) -> bool {
    seat == viewer
}

/// Sign that flips the Z axis between viewer (+1) and opponent (-1) seats.
fn z_sign(seat: usize, viewer: usize) -> f32 {
    if is_viewer(seat, viewer) { 1.0 } else { -1.0 }
}

/// Position of `seat` among the non-viewer seats and the total opponent count.
/// Returns `(0, 1)` for the viewer's own seat (treat as a single column).
fn opponent_index(seat: usize, viewer: usize, n_seats: usize) -> (usize, usize) {
    if is_viewer(seat, viewer) {
        return (0, 1);
    }
    let opps: Vec<usize> = (0..n_seats).filter(|s| *s != viewer).collect();
    let total = opps.len().max(1);
    let idx = opps.iter().position(|&s| s == seat).unwrap_or(0);
    (idx, total)
}

/// X offset spreading multiple opponents across the back row.
fn opp_x_offset(seat: usize, viewer: usize, n_seats: usize) -> f32 {
    if is_viewer(seat, viewer) {
        return 0.0;
    }
    let (idx, total) = opponent_index(seat, viewer, n_seats);
    if total <= 1 {
        return 0.0;
    }
    let center = (total as f32 - 1.0) / 2.0;
    (idx as f32 - center) * OPP_X_SPREAD
}

// ── Pile positions ───────────────────────────────────────────────────────────

/// Bottom-card position of `seat`'s deck pile.
pub fn deck_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    let sign = z_sign(seat, viewer);
    let x = if is_viewer(seat, viewer) {
        -DECK_X
    } else {
        DECK_X + opp_x_offset(seat, viewer, n_seats)
    };
    Vec3::new(x, 0.0, sign * DECK_Z)
}

/// Bottom-card position of `seat`'s graveyard pile.
pub fn graveyard_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    let sign = z_sign(seat, viewer);
    let x = if is_viewer(seat, viewer) {
        -GRAVEYARD_X
    } else {
        GRAVEYARD_X + opp_x_offset(seat, viewer, n_seats)
    };
    Vec3::new(x, 0.0, sign * GRAVEYARD_Z)
}

/// Rotation applied to a face-down card belonging to `seat` (deck pile,
/// opponent hand back, graveyard pile).
pub fn back_face_rotation(seat: usize, viewer: usize) -> Quat {
    if is_viewer(seat, viewer) {
        Quat::from_rotation_x(-FRAC_PI_2)
    } else {
        Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI)
    }
}

/// Center of `seat`'s clickable player-target zone (used for spell targeting).
pub fn player_target_zone_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    let sign = z_sign(seat, viewer);
    let x = opp_x_offset(seat, viewer, n_seats);
    Vec3::new(x, 0.01, sign * 6.0)
}

// ── Hand cards ───────────────────────────────────────────────────────────────

/// Hand-card transform for `seat`. Viewer's hand is face-up and fanned at the
/// front of the table; opponent hands are face-down and mirrored at the back.
pub fn hand_card_transform(
    seat: usize,
    viewer: usize,
    n_seats: usize,
    slot: usize,
    total: usize,
) -> Transform {
    let center = (total as f32 - 1.0) / 2.0;
    let offset = slot as f32 - center;
    // Lower slot = closer to camera so left cards overlap right cards.
    let z_offset = if total > 0 {
        (total - 1).saturating_sub(slot) as f32
    } else {
        0.0
    };

    if is_viewer(seat, viewer) {
        let x = offset * HAND_CARD_SPACING;
        let y = HAND_Y - offset.abs() * HAND_FAN_Y_DROP;
        let z = HAND_CENTER_Z + z_offset * CARD_THICKNESS * 4.0;
        let rot_z = -offset * HAND_FAN_ANGLE;
        Transform::from_xyz(x, y, z)
            .with_rotation(Quat::from_rotation_x(HAND_TILT_X) * Quat::from_rotation_z(rot_z))
    } else {
        let x = offset * HAND_CARD_SPACING + opp_x_offset(seat, viewer, n_seats);
        // Extra lift so cards clear the table at the far camera angle.
        let y = HAND_Y + 3.0 - offset.abs() * HAND_FAN_Y_DROP;
        let z = -(HAND_CENTER_Z + 2.0 + z_offset * CARD_THICKNESS * 4.0);
        let rot_z = offset * HAND_FAN_ANGLE;
        Transform::from_xyz(x, y, z)
            .with_rotation(Quat::from_rotation_x(-HAND_TILT_X) * Quat::from_rotation_z(PI + rot_z))
    }
}

// ── Battlefield cards ────────────────────────────────────────────────────────

/// Battlefield-card transform for `seat`. Lands in back row, creatures in
/// front row; viewer's rows are mirrored opposite the opponent's.
pub fn bf_card_transform(
    seat: usize,
    viewer: usize,
    n_seats: usize,
    slot: usize,
    total: usize,
    is_land: bool,
    tapped: bool,
) -> Transform {
    let center = (total as f32 - 1.0) / 2.0;
    let offset = slot as f32 - center;
    let x = offset * BF_CARD_SPACING + opp_x_offset(seat, viewer, n_seats);

    let row_offset = if is_land { BF_LAND_Z } else { BF_CREATURE_Z };
    let z = z_sign(seat, viewer) * row_offset;

    let base_rot = if is_viewer(seat, viewer) {
        Quat::from_rotation_x(-FRAC_PI_2)
    } else {
        Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI)
    };

    let rot = if tapped {
        Quat::from_rotation_y(-FRAC_PI_2) * base_rot
    } else {
        base_rot
    };

    Transform::from_xyz(x, BF_Y, z).with_rotation(rot)
}

/// Returns `(group_slot, index_in_group, total_groups)` grouping identical
/// lands by name. Seat-agnostic.
pub fn land_group_info_from_view(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    card_id: CardId,
) -> Option<(usize, usize, usize)> {
    let lands: Vec<_> = battlefield
        .iter()
        .filter(|c| c.owner == owner && c.is_land())
        .collect();

    let target = lands.iter().find(|c| c.id == card_id)?;
    let target_name = target.name.as_str();

    let mut names: Vec<&str> = lands.iter().map(|c| c.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();

    let group_slot = names.iter().position(|&n| n == target_name)?;
    let index_in_group = lands
        .iter()
        .filter(|c| c.name == target_name)
        .position(|c| c.id == card_id)?;

    Some((group_slot, index_in_group, names.len()))
}

/// World transform for a land card with stacking offset for identical lands.
pub fn land_card_transform(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    viewer: usize,
    n_seats: usize,
    card_id: CardId,
) -> Option<Transform> {
    let (group_slot, index, total_groups) =
        land_group_info_from_view(battlefield, owner, card_id)?;
    let base = bf_card_transform(owner, viewer, n_seats, group_slot, total_groups, true, false);
    // Stagger pulls subsequent cards toward the back of the row (toward the
    // owner's edge of the table) so each card's name strip stays visible.
    let sign = z_sign(owner, viewer);
    let stagger = Vec3::new(
        index as f32 * LAND_STACK_OFFSET_X * sign,
        index as f32 * CARD_THICKNESS * 1.5,
        index as f32 * LAND_STACK_OFFSET_Z * sign,
    );
    Some(Transform {
        translation: base.translation + stagger,
        rotation: base.rotation,
        scale: base.scale,
    })
}
