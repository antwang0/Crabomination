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

// ── Two-per-side seating (3+ players) ──────────────────────────────────────────
//
// For 3+ players the table is rectangular with players on the two long
// edges: the viewer's edge (front, +Z) and the far edge (back, −Z). Seats
// fill columns left→right along each edge, clockwise from the viewer at
// front-left, so a 4-player pod reads as a clean 2-and-2 (viewer & V+3 in
// front, V+1 & V+2 across). Each seat owns a column whose width shrinks with
// the number of players sharing that edge, so boards and piles never overlap
// a neighbour. 1- and 2-player games bypass all of this (see [`seat_spot`]).

/// Usable half-width (X) of the whole table for 1-/2-player games — the
/// distance the piles sit at today.
const TABLE_HALF_X: f32 = DECK_X;
/// Wider half-width used for 3+ player tables, so two columns per edge each
/// get room for a board, hand, and a pile strip without the front hands
/// overlapping at the centre. Paired with a pulled-back multiplayer camera
/// (`CAM_HOME_POS_MULTI` in `systems::camera_zoom`).
const MULTI_HALF_X: f32 = 30.0;
/// Gap kept between adjacent columns so neighbouring boards don't touch.
const COL_MARGIN: f32 = 0.6;
/// Outer strip (X) of each multiplayer column reserved for the deck /
/// graveyard / command piles, so they never sit on top of the board.
const PILE_STRIP: f32 = CARD_WIDTH * 1.4;
/// Largest board half-width any single seat gets (keeps a lone back-edge
/// player in a 3-player game from spreading absurdly wide).
const BOARD_HALF_CAP: f32 = 13.0;

/// Resolved table placement for a seat: which long edge and where along it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeatSpot {
    /// +1 for the viewer's (front) edge, −1 for the far (back) edge.
    pub z_sign: f32,
    /// World-X centre of this seat's full column (used to place the outer
    /// pile strip).
    pub col_center: f32,
    /// Usable half-width (X) of the full column.
    pub half_width: f32,
    /// X centre of the *inner* board area (column minus the outer pile strip),
    /// where the battlefield and hand sit.
    pub board_center: f32,
    /// Half-width (X) of the inner board area.
    pub board_half: f32,
}

/// Number of seats placed on the viewer's (front) edge: ceil(n/2), so the
/// viewer's side never has fewer players than the far side.
fn front_count(n: usize) -> usize {
    n.div_ceil(2)
}

/// `(is_front, column_index, columns_on_that_edge)` for `seat` under the
/// clockwise 2-per-side rule. `k = (seat − viewer) mod n` is the seat's
/// clockwise distance from the viewer (0 = viewer, front-left).
fn seat_slot(seat: usize, viewer: usize, n_seats: usize) -> (bool, usize, usize) {
    let n = n_seats.max(1);
    let k = (seat + n - (viewer % n)) % n;
    let fc = front_count(n);
    let bc = n - fc;
    if k == 0 {
        (true, 0, fc) // viewer — front edge, leftmost column
    } else if k <= bc {
        (false, k - 1, bc) // the seats just after the viewer fill the far edge
    } else {
        // The remaining seats wrap back onto the front edge, to the
        // viewer's right (columns 1..fc).
        (true, k - bc, fc)
    }
}

/// Centre-X and usable half-width for column `col` of `cols` evenly dividing
/// the multiplayer table width.
fn col_geom(col: usize, cols: usize) -> (f32, f32) {
    let cols_f = cols.max(1) as f32;
    let col_w = 2.0 * MULTI_HALF_X / cols_f;
    let center = (col as f32 - (cols_f - 1.0) / 2.0) * col_w;
    let half = (col_w / 2.0 - COL_MARGIN).max(CARD_WIDTH * 0.5);
    (center, half)
}

/// Resolve a seat to its table placement. 1- and 2-player games keep the
/// historical layout exactly (viewer front-centre full width, opponent back-
/// centre); 3+ players use the two-per-side rectangular seating, reserving an
/// outer pile strip so the board area stays clear.
pub fn seat_spot(seat: usize, viewer: usize, n_seats: usize) -> SeatSpot {
    if n_seats <= 2 {
        return SeatSpot {
            z_sign: z_sign(seat, viewer),
            col_center: 0.0,
            half_width: TABLE_HALF_X,
            board_center: 0.0,
            board_half: TABLE_HALF_X,
        };
    }
    let (front, col, cols) = seat_slot(seat, viewer, n_seats);
    let (center, half) = col_geom(col, cols);
    // Shift the board inward off the outer pile strip (only when the column is
    // off-centre — a lone centred column keeps its board centred).
    let shift = if center.abs() > 0.1 { center.signum() * PILE_STRIP } else { 0.0 };
    let board_half = (half - PILE_STRIP).clamp(CARD_WIDTH, BOARD_HALF_CAP);
    SeatSpot {
        z_sign: if front { 1.0 } else { -1.0 },
        col_center: center,
        half_width: half,
        board_center: center - shift,
        board_half,
    }
}

/// Outward X sign for a seat's column — points away from the table centre
/// (toward the nearer table edge), so piles tuck to the outside of the
/// column and leave the inner board area clear. Centre columns (col_center
/// ≈ 0) fall back to the legacy viewer-left / opponent-right diagonal.
fn outward_sign(spot: &SeatSpot, seat: usize, viewer: usize) -> f32 {
    if spot.col_center.abs() > 0.1 {
        spot.col_center.signum()
    } else if is_viewer(seat, viewer) {
        // Centre column (1-/2-player): keep the legacy viewer-left /
        // opponent-right diagonal.
        -1.0
    } else {
        1.0
    }
}

/// Face rotation for a card sitting on the edge with the given `z_sign`:
/// front-edge cards face the camera; far-edge cards are additionally flipped
/// 180° about Z so their art reads from that player's side of the table.
fn face_rotation(z_sign: f32) -> Quat {
    if z_sign > 0.0 {
        Quat::from_rotation_x(-FRAC_PI_2)
    } else {
        Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI)
    }
}

/// X of a pile tucked just inside the outer edge of `seat`'s column.
fn pile_x(seat: usize, viewer: usize, spot: &SeatSpot, inset: f32) -> f32 {
    spot.col_center + outward_sign(spot, seat, viewer) * (spot.half_width - inset)
}

// ── Pile positions ───────────────────────────────────────────────────────────

/// Bottom-card position of `seat`'s deck pile.
pub fn deck_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    if n_seats <= 2 {
        let sign = z_sign(seat, viewer);
        let x = if is_viewer(seat, viewer) {
            -DECK_X
        } else {
            DECK_X + opp_x_offset(seat, viewer, n_seats)
        };
        return Vec3::new(x, 0.0, sign * DECK_Z);
    }
    let spot = seat_spot(seat, viewer, n_seats);
    Vec3::new(pile_x(seat, viewer, &spot, CARD_WIDTH * 0.5), 0.0, spot.z_sign * DECK_Z)
}

/// Bottom-card position of `seat`'s graveyard pile.
pub fn graveyard_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    if n_seats <= 2 {
        let sign = z_sign(seat, viewer);
        let x = if is_viewer(seat, viewer) {
            -GRAVEYARD_X
        } else {
            GRAVEYARD_X + opp_x_offset(seat, viewer, n_seats)
        };
        return Vec3::new(x, 0.0, sign * GRAVEYARD_Z);
    }
    let spot = seat_spot(seat, viewer, n_seats);
    Vec3::new(pile_x(seat, viewer, &spot, CARD_WIDTH * 0.5), 0.0, spot.z_sign * GRAVEYARD_Z)
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
    let y = CARD_THICKNESS * (slot as f32) * 2.0;
    if n_seats <= 2 {
        let sign = z_sign(seat, viewer);
        let x = if is_viewer(seat, viewer) {
            COMMAND_X
        } else {
            -COMMAND_X + opp_x_offset(seat, viewer, n_seats)
        };
        return Transform::from_xyz(x, y, sign * COMMAND_Z).with_rotation(face_rotation(sign));
    }
    let spot = seat_spot(seat, viewer, n_seats);
    // Sits at the player's outer-front corner — in the gap just outside the
    // board, pulled toward the player (near hand depth) — so the commander is
    // prominent and clear of both the board rows and the hand fan.
    let out = outward_sign(&spot, seat, viewer);
    let x = spot.board_center + out * (spot.board_half + CARD_WIDTH);
    let z = spot.z_sign * (HAND_CENTER_Z - 2.0);
    Transform::from_xyz(x, y, z).with_rotation(face_rotation(spot.z_sign))
}

/// Rotation applied to a face-down card belonging to `seat` (deck pile,
/// opponent hand back, graveyard pile). Keyed on the seat's table edge, so
/// far-edge piles face that player while front-edge piles face the camera.
pub fn back_face_rotation(seat: usize, viewer: usize, n_seats: usize) -> Quat {
    face_rotation(seat_spot(seat, viewer, n_seats).z_sign)
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
    let spot = seat_spot(seat, viewer, n_seats);
    let z = if spot.z_sign > 0.0 {
        HAND_CENTER_Z - HAND_ANCHOR_PULLBACK
    } else {
        -(HAND_CENTER_Z + 2.0) + HAND_ANCHOR_PULLBACK
    };
    Vec3::new(spot.board_center, 0.01, z)
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
    let spot = seat_spot(seat, viewer, n_seats);
    if is_viewer(seat, viewer) {
        let z = viewer_zoom;
        // Shift the whole fan to the viewer's board centre (0 in 1-/2-player
        // games, off to the front-left edge in a 3+ player pod).
        let x = offset * spacing * z + spot.board_center;
        let y = HAND_Y * z - offset.abs() * HAND_FAN_Y_DROP * z;
        // Pull the hand a fraction closer to the camera as it scales up
        // so the (now larger) cards don't poke up into the battlefield.
        let world_z = HAND_CENTER_Z + (z - 1.0) * 1.5 + z_offset * CARD_THICKNESS * 4.0;
        let rot_z = -offset * HAND_FAN_ANGLE;
        Transform::from_xyz(x, y, world_z)
            .with_rotation(Quat::from_rotation_x(HAND_TILT_X) * Quat::from_rotation_z(rot_z))
            .with_scale(Vec3::splat(z))
    } else {
        let x = offset * spacing + spot.board_center;
        // Extra lift so cards clear the table at the far camera angle.
        let y = HAND_Y + 3.0 - offset.abs() * HAND_FAN_Y_DROP;
        // Far edge fans away from the camera (−Z), the viewer's edge toward it
        // (+Z); cards stack toward their owner's side of the table either way.
        let base_z = if spot.z_sign > 0.0 { HAND_CENTER_Z + 2.0 } else { -(HAND_CENTER_Z + 2.0) };
        let z = base_z + spot.z_sign * (z_offset * CARD_THICKNESS * 4.0);
        let tilt = if spot.z_sign > 0.0 { HAND_TILT_X } else { -HAND_TILT_X };
        let rot_z = offset * HAND_FAN_ANGLE;
        Transform::from_xyz(x, y, z)
            .with_rotation(Quat::from_rotation_x(tilt) * Quat::from_rotation_z(PI + rot_z))
    }
}

// ── Battlefield cards ────────────────────────────────────────────────────────

/// Per-card battlefield spacing for a row of `total` groups confined to a
/// column of half-width `half`. Like [`bf_spacing`] but clamps to the seat's
/// own column (3+ player seating) instead of the whole table.
fn bf_spacing_col(total: usize, half: f32) -> f32 {
    if total <= 1 {
        return BF_CARD_SPACING;
    }
    let max_outer_center = (half - CARD_WIDTH * 0.5 - 0.3).max(0.0);
    let max_spacing = 2.0 * max_outer_center / (total as f32 - 1.0);
    BF_CARD_SPACING.min(max_spacing)
}

/// Battlefield-card transform for `seat`. Lands in back row, creatures in
/// front row; each seat's rows sit on their table edge, centred on their
/// column and clamped to it so neighbouring boards don't overlap.
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
    let spot = seat_spot(seat, viewer, n_seats);
    let x = if n_seats <= 2 {
        offset * bf_spacing(total) + opp_x_offset(seat, viewer, n_seats)
    } else {
        spot.board_center + offset * bf_spacing_col(total, spot.board_half)
    };

    let row_offset = if is_land { BF_LAND_Z } else { BF_CREATURE_Z };
    let z = spot.z_sign * row_offset;

    let base_rot = face_rotation(spot.z_sign);
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
    let sign = seat_spot(owner, viewer, n_seats).z_sign;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_player_layout_is_unchanged() {
        // Viewer front-centre, opponent back-centre, diagonal piles — the
        // historical 1v1 geometry must be byte-for-byte preserved.
        assert_eq!(seat_spot(0, 0, 2).z_sign, 1.0);
        assert_eq!(seat_spot(1, 0, 2).z_sign, -1.0);
        assert_eq!(seat_spot(0, 0, 2).col_center, 0.0);
        assert_eq!(deck_position(0, 0, 2), Vec3::new(-DECK_X, 0.0, DECK_Z));
        assert_eq!(deck_position(1, 0, 2), Vec3::new(DECK_X, 0.0, -DECK_Z));
        assert_eq!(graveyard_position(0, 0, 2), Vec3::new(-GRAVEYARD_X, 0.0, GRAVEYARD_Z));
    }

    #[test]
    fn four_player_seats_two_and_two() {
        // viewer=0: front = {0, 3}, back = {1, 2} (clockwise from front-left).
        let front: Vec<bool> = (0..4).map(|s| seat_spot(s, 0, 4).z_sign > 0.0).collect();
        assert_eq!(front, vec![true, false, false, true]);
        // Exactly two seats per edge.
        assert_eq!(front.iter().filter(|f| **f).count(), 2);
    }

    #[test]
    fn four_player_columns_dont_overlap_and_stay_on_table() {
        // Group seats by edge; within an edge no two columns may overlap, and
        // every column must stay inside the table half-width.
        for &front in &[true, false] {
            let cols: Vec<(f32, f32)> = (0..4)
                .map(|s| seat_spot(s, 0, 4))
                .filter(|sp| (sp.z_sign > 0.0) == front)
                .map(|sp| (sp.col_center, sp.half_width))
                .collect();
            for (c, h) in &cols {
                assert!(
                    c.abs() + h <= MULTI_HALF_X + 0.01,
                    "column center {c} ± {h} runs off the table",
                );
            }
            for i in 0..cols.len() {
                for j in (i + 1)..cols.len() {
                    let (ci, hi) = cols[i];
                    let (cj, hj) = cols[j];
                    let gap = (ci - cj).abs();
                    assert!(gap >= hi + hj, "columns {ci} and {cj} overlap (gap {gap})");
                }
            }
        }
    }

    #[test]
    fn viewer_always_on_front_edge() {
        for n in 2..=6 {
            for v in 0..n {
                assert!(seat_spot(v, v, n).z_sign > 0.0, "viewer must sit front (n={n}, v={v})");
            }
        }
    }

    #[test]
    fn three_player_puts_viewer_and_one_opp_in_front() {
        // n=3: front = {viewer, V+2}, back = {V+1}.
        let fronts: Vec<bool> = (0..3).map(|s| seat_spot(s, 0, 3).z_sign > 0.0).collect();
        assert_eq!(fronts, vec![true, false, true]);
    }

    #[test]
    fn piles_stay_on_table_for_four_players() {
        for s in 0..4 {
            let d = deck_position(s, 0, 4);
            let g = graveyard_position(s, 0, 4);
            assert!(d.x.abs() <= MULTI_HALF_X + 0.01, "deck off table: {}", d.x);
            assert!(g.x.abs() <= MULTI_HALF_X + 0.01, "graveyard off table: {}", g.x);
        }
    }

    #[test]
    fn board_area_clears_the_pile_strip() {
        // Each seat's board must stay inboard of its outer pile strip so the
        // deck / graveyard / command piles never sit on the battlefield.
        for s in 0..4 {
            let sp = seat_spot(s, 0, 4);
            let board_outer = sp.board_center.abs() + sp.board_half;
            let pile_inner = deck_position(s, 0, 4).x.abs() - CARD_WIDTH * 0.5;
            assert!(
                board_outer <= pile_inner + 0.01,
                "seat {s}: board outer {board_outer} overlaps pile strip inner {pile_inner}",
            );
        }
    }

    #[test]
    fn seat_slot_is_viewer_relative() {
        // The same clockwise shape regardless of which seat is the viewer.
        for v in 0..4 {
            let (front, col, cols) = seat_slot(v, v, 4);
            assert!(front && col == 0 && cols == 2, "viewer is always front-left");
        }
    }
}
