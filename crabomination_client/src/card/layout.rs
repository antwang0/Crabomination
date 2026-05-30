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
/// Hands larger than this clamp their total fan width to the 7-card width
/// and reduce per-card spacing proportionally.
pub const HAND_FAN_SOFT_CAP: usize = 7;

/// Effective per-card hand spacing. Equals `HAND_CARD_SPACING` for hands
/// up to `HAND_FAN_SOFT_CAP`; for larger hands, shrinks proportionally so
/// the total fan width stays at the soft-cap value.
fn hand_spacing(total: usize) -> f32 {
    if total <= HAND_FAN_SOFT_CAP {
        HAND_CARD_SPACING
    } else {
        HAND_CARD_SPACING * (HAND_FAN_SOFT_CAP as f32 - 1.0) / (total as f32 - 1.0)
    }
}

const HAND_CENTER_Z: f32 = 12.0;
const HAND_Y: f32 = CARD_HEIGHT / 2.0;
// Tilt so the card face points at the camera (0, 32, 14).
// Camera direction from scene: normalize(0, 32, 14) = (0, 0.916, 0.401).
// Face after rotation_x(θ) is (0, −sinθ, cosθ); solving: θ = −atan2(32, 14) ≈ −1.16.
const HAND_TILT_X: f32 = -1.16;

// ── Battlefield layout constants ─────────────────────────────────────────────

const BF_Y: f32 = 0.02;
/// Per-slot battlefield spacing. Sized to the longer axis (CARD_HEIGHT)
/// with extra headroom so adjacent **tapped** creatures don't visually
/// overlap — a tapped card rotates 90° and its long edge (CARD_HEIGHT)
/// becomes its width. With the previous `* 1.1` multiplier the gap
/// between two adjacent tapped cards was under half a unit and read as
/// overlapping at the typical camera angle.
const BF_CARD_SPACING: f32 = CARD_HEIGHT * 1.35;
/// Distance from center to creature row (viewer side; opponent is mirrored).
const BF_CREATURE_Z: f32 = 3.5;
/// Distance from center to land row.
const BF_LAND_Z: f32 = 8.5;

/// Effective per-card spacing for a battlefield row of `total` groups.
/// Caps at [`BF_CARD_SPACING`] for small rows but shrinks proportionally
/// when the row would otherwise extend past the deck/graveyard footprint
/// at |x| = [`DECK_X`]. Without this, a commander deck with 7+ distinct
/// lands pushes the outermost groups directly under the pile.
fn bf_spacing(total: usize) -> f32 {
    if total <= 1 {
        return BF_CARD_SPACING;
    }
    // Pile inner face sits at |x| = DECK_X - CARD_WIDTH/2. Add a small
    // visual margin so the row never butts up against the pile face,
    // and subtract another CARD_WIDTH/2 so the outermost group's edge
    // (not its center) stays inside that boundary.
    let max_outer_center = DECK_X - CARD_WIDTH - 0.3;
    let max_spacing = 2.0 * max_outer_center / (total as f32 - 1.0);
    BF_CARD_SPACING.min(max_spacing)
}

// ── Pile constants ───────────────────────────────────────────────────────────

// Pile X positions sit far enough from the centre that the rightmost
// land group (5 basics → group centre at ≈ 9.2 units from origin) still
// has clearance for a CARD_WIDTH/2 = 1.5-unit-wide card without
// poking into the deck/graveyard footprint. Previous 11.0 left the
// deck pile sitting on top of the leftmost land.
const DECK_X: f32 = 13.5;
const DECK_Z: f32 = 9.5;
const GRAVEYARD_X: f32 = 13.5;
const GRAVEYARD_Z: f32 = 4.0;
/// Command zone sits between the graveyard and the table edge,
/// closer to the center, so the commander is always visible.
/// Each card in the zone stacks slightly along Y for legibility.
const COMMAND_X: f32 = -11.0;
const COMMAND_Z: f32 = 4.0;

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

/// Transform for a card in `seat`'s command zone, slot `slot`. Cards
/// are face-up and tilted toward the camera (viewer-side) or
/// face-up but flipped for opponents (still legible from afar).
/// Multiple commanders stack with a small Y offset.
pub fn command_zone_card_transform(
    seat: usize,
    viewer: usize,
    n_seats: usize,
    slot: usize,
) -> Transform {
    let sign = z_sign(seat, viewer);
    let x = if is_viewer(seat, viewer) {
        COMMAND_X
    } else {
        -COMMAND_X + opp_x_offset(seat, viewer, n_seats)
    };
    let y = CARD_THICKNESS * (slot as f32) * 2.0;
    let z = sign * COMMAND_Z;
    let rot = if is_viewer(seat, viewer) {
        Quat::from_rotation_x(-FRAC_PI_2)
    } else {
        Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI)
    };
    Transform::from_xyz(x, y, z).with_rotation(rot)
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

/// How far (in Z) to pull the hand anchor toward the table center from
/// the hand fan's resting line, so combat lunges and targeting arrows
/// land just in front of the fan rather than buried under the cards.
const HAND_ANCHOR_PULLBACK: f32 = 2.5;

/// World anchor for `seat`'s on-table "presence" — the center of their
/// hand fan, pulled slightly toward the table center. Used as the 3-D
/// anchor that combat-lurch, attack-plan, and stack-spell arrows point
/// at (player *clicks* land on the 2-D HUD panel instead).
///
/// Pointing these at the hand — rather than a separate avatar disc —
/// ties "the player" to the cluster of cards already sitting at their
/// edge of the table: an attacker visibly lunges toward the defending
/// player's hand, and a removal spell's arrow sweeps to the same player
/// the 2-D panel click resolves to.
///
/// Y sits at table level; every current consumer overrides the height
/// (gizmo arrows raise it to ~0.2–0.3; the combat lurch keeps attackers
/// gliding flat at Y=0), so only X/Z are meaningful here.
pub fn player_hand_anchor(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    let x = opp_x_offset(seat, viewer, n_seats);
    let z = if is_viewer(seat, viewer) {
        HAND_CENTER_Z - HAND_ANCHOR_PULLBACK
    } else {
        -(HAND_CENTER_Z + 2.0) + HAND_ANCHOR_PULLBACK
    };
    Vec3::new(x, 0.01, z)
}

/// Width along X that a 7-card hand fan occupies (the soft cap). Used to
/// position the per-seat 3-D player-avatar disc just past the right
/// edge of the hand so it's clearly visible from the camera without
/// overlapping creature / land rows or the hand cards themselves.
const HAND_HALF_WIDTH: f32 = HAND_CARD_SPACING * ((HAND_FAN_SOFT_CAP as f32) - 1.0) / 2.0;

/// Resting position of `seat`'s 3-D player-avatar disc, just past the
/// right edge of their hand fan.
///
/// NOTE: transitional. The disc is being retired in favour of a 2-D HUD
/// panel; combat-lurch and targeting arrows already point at
/// [`player_hand_anchor`] instead. This is kept only so the disc (still
/// the keyboard-target carrier) has a stable home until it's removed.
pub fn player_target_zone_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    let x = opp_x_offset(seat, viewer, n_seats) + HAND_HALF_WIDTH + 1.2;
    let z = if is_viewer(seat, viewer) {
        HAND_CENTER_Z
    } else {
        -(HAND_CENTER_Z + 2.0)
    };
    Vec3::new(x, 0.01, z)
}

// ── Hand cards ───────────────────────────────────────────────────────────────

/// Hand-card transform for `seat`. Viewer's hand is face-up and fanned at the
/// front of the table; opponent hands are face-down and mirrored at the back.
///
/// `viewer_zoom` is a resolution-driven multiplier that enlarges the
/// viewer's hand on low-resolution displays so each card occupies a
/// roughly constant on-screen size. A zoom of 1.0 reproduces the
/// historical layout. The zoom scales: card mesh size, hand spacing,
/// fan Y drop, and a small pull toward the camera so the larger mesh
/// doesn't clip the table edge. Opponent hands ignore zoom — they're
/// face-down at the back of the table where size is fine.
pub fn hand_card_transform(
    seat: usize,
    viewer: usize,
    n_seats: usize,
    slot: usize,
    total: usize,
    viewer_zoom: f32,
) -> Transform {
    let center = (total as f32 - 1.0) / 2.0;
    let offset = slot as f32 - center;
    // Lower slot = closer to camera so left cards overlap right cards.
    let z_offset = if total > 0 {
        (total - 1).saturating_sub(slot) as f32
    } else {
        0.0
    };

    let spacing = hand_spacing(total);
    if is_viewer(seat, viewer) {
        let z = viewer_zoom;
        let x = offset * spacing * z;
        let y = HAND_Y * z - offset.abs() * HAND_FAN_Y_DROP * z;
        // Pull the hand a fraction closer to the camera as it scales up
        // so the (now larger) cards don't poke up into the battlefield.
        let world_z = HAND_CENTER_Z + (z - 1.0) * 1.5 + z_offset * CARD_THICKNESS * 4.0;
        let rot_z = -offset * HAND_FAN_ANGLE;
        Transform::from_xyz(x, y, world_z)
            .with_rotation(Quat::from_rotation_x(HAND_TILT_X) * Quat::from_rotation_z(rot_z))
            .with_scale(Vec3::splat(z))
    } else {
        let x = offset * spacing + opp_x_offset(seat, viewer, n_seats);
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
    let x = offset * bf_spacing(total) + opp_x_offset(seat, viewer, n_seats);

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

// ── Stack cards ──────────────────────────────────────────────────────────────

/// World transform for a card occupying slot `idx` of a stack of `total`
/// cards. Cards spread horizontally, centred on the table, hovering above
/// the battlefield and facing the camera.
pub fn stack_card_transform(idx: usize, total: usize) -> Transform {
    let spacing = CARD_WIDTH + 0.5;
    let total_width = (total.saturating_sub(1) as f32) * spacing;
    let x = (idx as f32) * spacing - total_width / 2.0;
    Transform::from_translation(Vec3::new(x, 0.8, 0.0))
        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
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
