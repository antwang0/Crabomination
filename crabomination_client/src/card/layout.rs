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

/// The viewer's hand line. Far enough in front of the back row that the
/// fan's top edge (with its lower half off the bottom of the window, see
/// `framing::HAND_VISIBLE`) clears the land row instead of covering it.
const HAND_CENTER_Z: f32 = 14.0;
/// A far-edge opponent's face-down fan (3+ players), past their land row.
const FAR_HAND_Z: f32 = 14.0;
const HAND_Y: f32 = CARD_HEIGHT / 2.0;
// Tilt so the card face points at the camera (0, 32, 14).
// Camera direction from scene: normalize(0, 32, 14) = (0, 0.916, 0.401).
// Face after rotation_x(θ) is (0, −sinθ, cosθ); solving: θ = −atan2(32, 14) ≈ −1.16.
const HAND_TILT_X: f32 = -1.16;

// ── Battlefield layout constants ─────────────────────────────────────────────

const BF_Y: f32 = 0.02;
/// Per-slot height ramp along a battlefield row.
///
/// A row wider than its column compresses below `CARD_WIDTH` (a 1v1 land
/// row does this from eight groups on, and any row does it once its cards
/// are tapped and rotated 90°). Two overlapping quads at the *same* Y are
/// coplanar, and the depth buffer then flickers between them frame to
/// frame — the "glitching, intersecting" board. Ramping Y by slot makes
/// the overlap a clean shingle instead: strictly ordered in depth, and at
/// one card-thickness per slot the lift is invisible from the camera's
/// grazing angle even across a dozen slots.
const BF_SLOT_Y_STEP: f32 = CARD_THICKNESS;
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

// Row spacing lives in `bf_spacing_col`; the 1v1 bound comes from
// `board_half_for` — pile inner face at |x| = DECK_X − CARD_WIDTH/2 with a
// small margin so rows never butt up against the pile face.

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
/// Exile is a single shared zone (CR 406.2), so it gets one pile rather
/// than one per seat: the right-hand edge at the table's midline, between
/// the two seats' pile strips and clear of both.
const EXILE_X: f32 = 13.5;
const EXILE_Z: f32 = 0.0;
/// Command zone sits between the graveyard and the table edge,
/// closer to the center, so the commander is always visible.
/// Each card in the zone stacks slightly along Y for legibility.
const COMMAND_X: f32 = -11.0;
const COMMAND_Z: f32 = 4.0;
/// Multiplayer command-zone depth: the outermost slot of the pile strip,
/// past the deck (z 9.5) with clearance for the deck card (4.2/2) plus the
/// far-edge commander at 1.3× scale (5.46/2) and margin, beside the hand
/// line — the fan never reaches the strip's X.
const COMMAND_Z_MULTI: f32 = 15.0;

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

// ── Pod seating (3-4 players) ────────────────────────────────────────────────
//
// A pod sits two to a long edge, the way four people share a rectangular
// table: the viewer front-left, then clockwise the seat across on the left,
// the one across on the right, and the one beside the viewer on the right
// (three players make a triangle: the viewer front-centre, two across).
// Every seat lays out in its own frame exactly as the viewer does on the
// near edge of a 1v1 table — creature and land rows toward the centre,
// graveyard and deck in a strip on its left, command zone and (for
// opponents) the face-down hand in a strip on its right — and the frame
// turns that layout to face the table centre from the seat's edge.
//
// It replaces the viewer alone on the near edge with all three opponents
// across the far one: three far boards a few cards wide, a camera that had
// to frame the whole far edge, and empty table either side of the viewer.
// Seating players around all four edges measured the same card size (the
// HUD's side columns bound both at ~90 px at 1920x1080) but put two boards
// sideways between those columns; two to a side keeps every card upright.

/// Where a pod seat sits: the rotation about Y that turns the viewer's
/// near-edge layout to face the table centre from this seat's edge, how far
/// that edge's centre line sits from the table centre, and where along the
/// edge the seat's column is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeatFrame {
    pub yaw: f32,
    pub offset: f32,
    /// Half-length of the seat's board along its edge.
    pub half: f32,
    /// Table-plane shift of the whole seat along its edge.
    pub shift: Vec3,
}

impl SeatFrame {
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw)
    }

    /// A seat-local point (the seat's edge toward +Z, the table centre
    /// toward −Z, as for the viewer) on the table.
    pub fn point(&self, local: Vec3) -> Vec3 {
        self.rotation() * (local + Vec3::Z * self.offset) + self.shift
    }

    pub fn place(&self, local: Transform) -> Transform {
        Transform {
            translation: self.point(local.translation),
            rotation: self.rotation() * local.rotation,
            scale: local.scale,
        }
    }

    /// The seat-local point at table point `world` ([`Self::point`]'s inverse).
    #[cfg(test)]
    fn local(&self, world: Vec3) -> Vec3 {
        self.rotation().inverse() * (world - self.shift) - Vec3::Z * self.offset
    }

    /// Seat-local X of the pile strips either side of the board.
    fn strip_x(&self) -> f32 {
        self.half + CARD_WIDTH * 0.5
    }
}

/// Half-length of a pod seat's board: five groups a row before a row wraps.
const POD_HALF: f32 = 10.3;
/// Centre of a pod seat's column: its board plus a pile strip either side.
const POD_COLUMN_X: f32 = POD_HALF + CARD_WIDTH + 0.2;
/// How far each edge of a pod table sits back from the centre line. A pod
/// is bound by width (the HUD's side columns), so it has height to spare,
/// and the gap gives a wrapped creature row a whole card's depth instead of
/// the shingle a 1v1 board settles for.
const POD_EDGE_GAP: f32 = CARD_HEIGHT * 0.75;
/// Step between the cards of an opponent's face-down hand, toward its owner.
const POD_HAND_STEP: f32 = 0.2;

/// The frame of `seat` in a 3-4 player pod; `None` for other table sizes.
pub fn pod_frame(seat: usize, viewer: usize, n_seats: usize) -> Option<SeatFrame> {
    if !(3..=4).contains(&n_seats) {
        return None;
    }
    // Clockwise from the viewer: `(yaw, column)`, column −1 left, +1 right.
    let k = (seat + n_seats - viewer % n_seats) % n_seats;
    let (yaw, column) = match (n_seats, k) {
        (4, 0) => (0.0, -1.0),
        (4, 1) => (PI, -1.0),
        (4, 2) => (PI, 1.0),
        (4, _) => (0.0, 1.0),
        (_, 0) => (0.0, 0.0),
        (_, 1) => (PI, -1.0),
        _ => (PI, 1.0),
    };
    Some(SeatFrame {
        yaw,
        offset: POD_EDGE_GAP,
        half: POD_HALF,
        shift: Vec3::X * column * POD_COLUMN_X,
    })
}

/// Rotation turning a seat-local offset to the table: the pod frame's, or a
/// half turn for the far edge of the two-edge layouts.
fn seat_rotation(seat: usize, viewer: usize, n_seats: usize) -> Quat {
    match pod_frame(seat, viewer, n_seats) {
        Some(f) => f.rotation(),
        None if seat_spot(seat, viewer, n_seats).z_sign < 0.0 => Quat::from_rotation_y(PI),
        None => Quat::IDENTITY,
    }
}

/// Where the camera parks to look at `seat`'s board from behind its edge
/// (the seat-focus hotkeys): `(look_at, eye)`.
pub fn seat_focus(seat: usize, viewer: usize, n_seats: usize) -> (Vec3, Vec3) {
    let (look_at, back) = match pod_frame(seat, viewer, n_seats) {
        Some(f) => (f.point(Vec3::new(0.0, 0.0, 6.0)), f.rotation() * Vec3::new(0.0, 20.0, 14.0)),
        None => {
            let spot = seat_spot(seat, viewer, n_seats);
            (
                Vec3::new(spot.board_center, 0.0, spot.z_sign * 6.0),
                Vec3::new(0.0, 20.0, spot.z_sign * 14.0),
            )
        }
    };
    (look_at, look_at + back)
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
/// Half-width of a 5-6 player table, split into columns along both edges.
/// (3-4 player pods lay out by seat frame instead, see [`pod_frame`].) The
/// camera has to fit the whole far edge, so this trades far boards' width
/// against card size; 27 measured best when 4-player pods used this layout
/// (`framing` budget): 30 gave 87 px cards at 1920x1080, 24 the same cards
/// on narrower boards.
const MULTI_HALF_X: f32 = 27.0;
/// Gap kept between adjacent columns so neighbouring boards don't touch.
const COL_MARGIN: f32 = 0.6;
/// Outer strip (X) of each multiplayer column reserved for the deck /
/// graveyard / command piles, so they never sit on top of the board.
const PILE_STRIP: f32 = CARD_WIDTH * 1.4;
/// Far-edge pile row (3+ players): a far seat's deck, graveyard, command
/// zone and face-down hand sit in one row *behind* its land row, so the
/// whole column goes to the board. A far column is a third of the table,
/// and a pile strip beside the board took 4.2 of its 9.4-unit half-width:
/// six creatures overlapped in a 3.5-card-wide row. The row's centre line
/// clears a wrapped second land row (`BF_LAND_Z` + `row_wrap_dz(2)` +
/// half a card = 12.0) by the scaled command card's half-height.
const FAR_PILE_Z: f32 = 14.6;
/// Slot pitch along the far pile row.
const FAR_PILE_STEP: f32 = CARD_WIDTH * 1.2;
/// A far seat's command card is drawn larger: "what's their commander?" is
/// the first thing a pod player looks up, from the far end of the table.
const FAR_COMMAND_SCALE: f32 = 1.15;

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

/// Number of seats placed on the viewer's (front) edge. Pods of 3-4 keep
/// the viewer alone in front — a face-down opponent hand is a poor use of
/// the closest, largest screen area, and the viewer's own board gets the
/// full table width. Larger pods fall back to two-per-side (ceil(n/2))
/// so the far edge doesn't shrink columns below a playable width.
fn front_count(n: usize) -> usize {
    if n <= 4 { 1 } else { n.div_ceil(2) }
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
    if !front {
        // Far seats keep their piles in a row behind the board.
        return SeatSpot {
            z_sign: -1.0,
            col_center: center,
            half_width: half,
            board_center: center,
            board_half: half.clamp(CARD_WIDTH, BOARD_HALF_CAP),
        };
    }
    // Shift the board inward off the outer pile strip (only when the column is
    // off-centre — a lone centred column keeps its board centred).
    let shift = if center.abs() > 0.1 { center.signum() * PILE_STRIP } else { 0.0 };
    let board_half = (half - PILE_STRIP).clamp(CARD_WIDTH, BOARD_HALF_CAP);
    // A lone column on its edge (the 3-4 player viewer) gets the whole
    // MULTI_HALF_X, but its board is capped at BOARD_HALF_CAP — clamp the
    // pile strip to sit just outside the *board*, not out at the table
    // edge where the piles would leave the camera frame.
    let half_width = half.min(board_half + PILE_STRIP);
    SeatSpot {
        z_sign: if front { 1.0 } else { -1.0 },
        col_center: center,
        half_width,
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

/// True when `seat` keeps its piles in the far pile row (3+ players, far
/// edge) rather than a strip beside its board.
fn in_far_pile_row(seat: usize, viewer: usize, n_seats: usize) -> bool {
    n_seats > 2 && seat_spot(seat, viewer, n_seats).z_sign < 0.0
}

/// Centre of slot `i` of a far seat's pile row, counted from the column's
/// outer edge inward: 0 deck, 1 graveyard, 2 command zone, 3 hand.
fn far_pile_slot(seat: usize, viewer: usize, n_seats: usize, i: usize) -> Vec3 {
    let spot = seat_spot(seat, viewer, n_seats);
    let out = outward_sign(&spot, seat, viewer);
    let x = spot.col_center + out * (spot.half_width - CARD_WIDTH * 0.5 - i as f32 * FAR_PILE_STEP);
    Vec3::new(x, 0.0, -FAR_PILE_Z)
}


// ── Pile positions ───────────────────────────────────────────────────────────

/// Bottom-card position of `seat`'s deck pile.
pub fn deck_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    if let Some(f) = pod_frame(seat, viewer, n_seats) {
        return f.point(Vec3::new(-f.strip_x(), 0.0, DECK_Z));
    }
    if n_seats <= 2 {
        let sign = z_sign(seat, viewer);
        let x = if is_viewer(seat, viewer) {
            -DECK_X
        } else {
            DECK_X + opp_x_offset(seat, viewer, n_seats)
        };
        return Vec3::new(x, 0.0, sign * DECK_Z);
    }
    if in_far_pile_row(seat, viewer, n_seats) {
        return far_pile_slot(seat, viewer, n_seats, 0);
    }
    let spot = seat_spot(seat, viewer, n_seats);
    Vec3::new(pile_x(seat, viewer, &spot, CARD_WIDTH * 0.5), 0.0, spot.z_sign * DECK_Z)
}

/// Bottom-card position of `seat`'s graveyard pile.
pub fn graveyard_position(seat: usize, viewer: usize, n_seats: usize) -> Vec3 {
    if let Some(f) = pod_frame(seat, viewer, n_seats) {
        return f.point(Vec3::new(-f.strip_x(), 0.0, GRAVEYARD_Z));
    }
    if n_seats <= 2 {
        let sign = z_sign(seat, viewer);
        let x = if is_viewer(seat, viewer) {
            -GRAVEYARD_X
        } else {
            GRAVEYARD_X + opp_x_offset(seat, viewer, n_seats)
        };
        return Vec3::new(x, 0.0, sign * GRAVEYARD_Z);
    }
    if in_far_pile_row(seat, viewer, n_seats) {
        return far_pile_slot(seat, viewer, n_seats, 1);
    }
    let spot = seat_spot(seat, viewer, n_seats);
    Vec3::new(pile_x(seat, viewer, &spot, CARD_WIDTH * 0.5), 0.0, spot.z_sign * GRAVEYARD_Z)
}

/// Bottom-card position of the shared exile pile. Exile isn't owned by a
/// seat, so this takes no `seat` — one pile holds every exiled card and
/// clicking it opens the browser.
pub fn exile_position(n_seats: usize) -> Vec3 {
    if n_seats <= 2 {
        return Vec3::new(EXILE_X, 0.0, EXILE_Z);
    }
    if let Some(f) = pod_frame(0, 0, n_seats) {
        // The viewer's right strip, behind their command zone.
        return f.point(Vec3::new(f.strip_x(), 0.0, DECK_Z));
    }
    // 5-6 players share both edges: just past the table's right edge.
    Vec3::new(MULTI_HALF_X + CARD_WIDTH * 0.6, 0.0, EXILE_Z)
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
    if let Some(f) = pod_frame(seat, viewer, n_seats) {
        // The seat's right strip, level with its graveyard across the board.
        return f.place(
            Transform::from_xyz(f.strip_x(), y, GRAVEYARD_Z).with_rotation(face_rotation(1.0)),
        );
    }
    if n_seats <= 2 {
        let sign = z_sign(seat, viewer);
        let x = if is_viewer(seat, viewer) {
            COMMAND_X
        } else {
            -COMMAND_X + opp_x_offset(seat, viewer, n_seats)
        };
        return Transform::from_xyz(x, y, sign * COMMAND_Z).with_rotation(face_rotation(sign));
    }
    if in_far_pile_row(seat, viewer, n_seats) {
        let p = far_pile_slot(seat, viewer, n_seats, 2);
        return Transform::from_xyz(p.x, y, p.z)
            .with_rotation(face_rotation(-1.0))
            .with_scale(Vec3::splat(FAR_COMMAND_SCALE));
    }
    let spot = seat_spot(seat, viewer, n_seats);
    let x = pile_x(seat, viewer, &spot, CARD_WIDTH * 0.5);
    // Several seats share this edge (5-6 players): the third slot of the
    // outer pile strip, past the deck toward the player (graveyard z 4.0 →
    // deck z 9.5 → command z 15.0).
    let z = spot.z_sign * COMMAND_Z_MULTI;
    Transform::from_xyz(x, y, z).with_rotation(face_rotation(spot.z_sign))
}

/// Rotation applied to a face-down card belonging to `seat` (deck pile,
/// opponent hand back, graveyard pile). Keyed on the seat's table edge, so
/// far-edge piles face that player while front-edge piles face the camera.
pub fn back_face_rotation(seat: usize, viewer: usize, n_seats: usize) -> Quat {
    match pod_frame(seat, viewer, n_seats) {
        Some(f) => f.rotation() * face_rotation(1.0),
        None => face_rotation(seat_spot(seat, viewer, n_seats).z_sign),
    }
}

/// How far (in Z) to pull the hand anchor toward the table center from
/// the hand fan's resting line, so combat lunges and targeting arrows
/// land just in front of the fan rather than buried under the cards.
const HAND_ANCHOR_PULLBACK: f32 = 2.5;
/// 1v1 opponent hand: the first card's centre sits this far past the table's
/// pile line, and each further card steps this far again.
const OPP_HAND_INSET: f32 = CARD_WIDTH * 1.2;
const OPP_HAND_STEP: f32 = CARD_WIDTH * 0.18;

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
    if let Some(f) = pod_frame(seat, viewer, n_seats) {
        let z = if seat == viewer { HAND_CENTER_Z } else { FAR_HAND_Z } - HAND_ANCHOR_PULLBACK;
        return f.point(Vec3::new(0.0, 0.01, z));
    }
    let spot = seat_spot(seat, viewer, n_seats);
    let z = if spot.z_sign > 0.0 {
        HAND_CENTER_Z - HAND_ANCHOR_PULLBACK
    } else {
        -FAR_HAND_Z + HAND_ANCHOR_PULLBACK
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
        // A pod seat's frame places the fan under its own board (the
        // viewer's frame is unrotated, so this only moves it).
        let seat_origin = pod_frame(seat, viewer, n_seats)
            .map_or(Vec3::X * spot.board_center, |f| f.point(Vec3::ZERO));
        let x = offset * spacing * z + seat_origin.x;
        let y = HAND_Y * z - offset.abs() * HAND_FAN_Y_DROP * z;
        // Pull the hand a fraction closer to the camera as it scales up
        // so the (now larger) cards don't poke up into the battlefield.
        let world_z = HAND_CENTER_Z + (z - 1.0) * 1.5 + z_offset * CARD_THICKNESS * 4.0 + seat_origin.z;
        let rot_z = -offset * HAND_FAN_ANGLE;
        Transform::from_xyz(x, y, world_z)
            .with_rotation(Quat::from_rotation_x(HAND_TILT_X) * Quat::from_rotation_z(rot_z))
            .with_scale(Vec3::splat(z))
    } else if n_seats <= 2 {
        // 1v1: the opponent's hand is a count, not information, so it sits
        // as a tight face-down spread beside their deck, outboard of the
        // pile strip, rather than as a full fan past their land row. The
        // window is short of height, not width: the fan cost the table
        // ~5 units of depth (and ran under the HUD's top band), where the
        // spread costs a few units of width the window has spare.
        // Across from their piles (the viewer-left / opponent-right
        // diagonal), where the viewer's own deck sits on the near edge: the
        // corner beyond the piles is under the HUD's game log.
        let x = -(DECK_X + OPP_HAND_INSET + slot as f32 * OPP_HAND_STEP);
        let z = spot.z_sign * DECK_Z;
        Transform::from_xyz(x, slot as f32 * CARD_THICKNESS * 2.0, z)
            .with_rotation(face_rotation(spot.z_sign))
    } else if let Some(f) = pod_frame(seat, viewer, n_seats) {
        // An opponent's hand: a face-down stack in the seat's right strip,
        // each card a little further toward its owner so the count reads.
        f.place(
            Transform::from_xyz(
                f.strip_x(),
                slot as f32 * CARD_THICKNESS * 2.0,
                DECK_Z + slot as f32 * POD_HAND_STEP,
            )
            .with_rotation(face_rotation(1.0)),
        )
    } else if in_far_pile_row(seat, viewer, n_seats) {
        // A far seat's hand: a compact face-down spread in its pile row,
        // stepping inward from the slot after the command zone.
        let base = far_pile_slot(seat, viewer, n_seats, 3);
        let inward = -outward_sign(&spot, seat, viewer);
        Transform::from_xyz(
            base.x + inward * slot as f32 * OPP_HAND_STEP,
            slot as f32 * CARD_THICKNESS * 2.0,
            base.z,
        )
        .with_rotation(face_rotation(-1.0))
    } else {
        // Opponent fans are informational (face-down backs), so compact
        // them to their own column: in a pod, neighbouring back-edge seats
        // are close enough that the full 7-card fan width bleeds into the
        // next player's hand. 1v1 keeps the historical spacing.
        let spacing = if n_seats > 2 && total > 1 {
            spacing.min(2.0 * spot.board_half / (total as f32 - 1.0))
        } else {
            spacing
        };
        let x = offset * spacing + spot.board_center;
        // Extra lift so cards clear the table at the far camera angle.
        let y = HAND_Y + 3.0 - offset.abs() * HAND_FAN_Y_DROP;
        // Far edge fans away from the camera (−Z), the viewer's edge toward it
        // (+Z); cards stack toward their owner's side of the table either way.
        let base_z = spot.z_sign * FAR_HAND_Z;
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

/// Below this per-group spacing the front row stops compressing and wraps
/// into a second row instead — tighter than this and cards read as a
/// single overlapped smear (a card is `CARD_WIDTH` = 3.0 wide).
const MIN_WRAP_SPACING: f32 = CARD_WIDTH * 1.15;
/// Innermost z a battlefield row centre may take: half a card off the table
/// centre, so the row's near edge stops exactly at z = 0.
///
/// Rows are mirrored per seat (`spot.z_sign`), so anything that crosses the
/// centre line lands in the *opponent's* half — and when both players wrap,
/// the two inner rows occupy the same volume. The old fixed
/// `ROW_WRAP_DZ = CARD_HEIGHT * 1.1` put the second row's centre at
/// z = −1.11 and its near edge at −3.21, more than three units into enemy
/// territory. Reported as boards overlapping "especially when both players
/// have a lot of creatures".
const BF_ROW_MIN_Z: f32 = CARD_HEIGHT * 0.5;

/// Z step between wrapped front rows, chosen so `rows` rows fit between
/// [`BF_CREATURE_Z`] and [`BF_ROW_MIN_Z`] — on the owner's side of the table,
/// and clear of the land row behind them.
///
/// The band is only about half a card deep, so wrapped rows *shingle* rather
/// than clear one another. That is the same trade the per-slot Y ramp already
/// makes for a compressed row (see [`BF_SLOT_Y_STEP`]): a strictly-ordered
/// overlap reads far better than two boards fighting for the same space.
fn row_wrap_dz(rows: usize) -> f32 {
    if rows <= 1 {
        return 0.0;
    }
    (BF_CREATURE_Z - BF_ROW_MIN_Z).max(0.0) / (rows as f32 - 1.0)
}
/// Wrapped rows are capped at two; past that the rows compress again —
/// a third row would cross into the opposite edge's territory.
const MAX_WRAP_ROWS: usize = 2;

/// `(rows, per_row)` for a front row of `total` groups in a column of
/// usable half-width `half`: one row while every group keeps at least
/// [`MIN_WRAP_SPACING`], then balanced rows up to [`MAX_WRAP_ROWS`].
fn front_row_shape(total: usize, half: f32) -> (usize, usize) {
    if total <= 1 {
        return (1, total.max(1));
    }
    let max_outer_center = (half - CARD_WIDTH * 0.5 - 0.3).max(0.0);
    let cap = ((2.0 * max_outer_center / MIN_WRAP_SPACING).floor() as usize + 1).max(1);
    if total <= cap {
        (1, total)
    } else {
        let rows = total.div_ceil(cap).min(MAX_WRAP_ROWS);
        (rows, total.div_ceil(rows))
    }
}

/// The seat's usable board half-width for battlefield rows.
fn board_half_for(spot: &SeatSpot, n_seats: usize) -> f32 {
    if n_seats <= 2 {
        // Matches the historical [`bf_spacing`] bound (max outer centre at
        // DECK_X − CARD_WIDTH − 0.3).
        DECK_X - CARD_WIDTH * 0.5
    } else {
        spot.board_half
    }
}

/// Battlefield-card transform for `seat`. Back row holds lands and support
/// permanents; the front row holds creatures and *wraps* into a second row
/// toward the table centre instead of compressing into an unreadable smear.
/// Each seat's rows sit on their table edge, centred on their column and
/// clamped to it so neighbouring boards don't overlap.
pub fn bf_card_transform(
    seat: usize,
    viewer: usize,
    n_seats: usize,
    slot: usize,
    total: usize,
    back_row: bool,
    tapped: bool,
) -> Transform {
    let tap = |rot: Quat| if tapped { Quat::from_rotation_y(-FRAC_PI_2) * rot } else { rot };
    if let Some(f) = pod_frame(seat, viewer, n_seats) {
        let (x, z) = row_position(slot, total, back_row, f.half, f.offset);
        let local = Transform::from_xyz(x, BF_Y + slot as f32 * BF_SLOT_Y_STEP, z)
            .with_rotation(tap(face_rotation(1.0)));
        return f.place(local);
    }
    let spot = seat_spot(seat, viewer, n_seats);
    let half = board_half_for(&spot, n_seats);
    let x_base = if n_seats <= 2 {
        opp_x_offset(seat, viewer, n_seats)
    } else {
        spot.board_center
    };
    let (x, row_z) = row_position(slot, total, back_row, half, 0.0);
    let z = spot.z_sign * row_z;
    Transform::from_xyz(x_base + x, BF_Y + slot as f32 * BF_SLOT_Y_STEP, z)
        .with_rotation(tap(face_rotation(spot.z_sign)))
}

/// Seat-local `(x, z)` of slot `slot` of a row of `total` groups on a board
/// of half-width `half`: X from the board's centre, Z out from the table
/// centre toward the seat. `inward_room` is how much further than
/// [`BF_ROW_MIN_Z`] a wrapped creature row may reach toward the centre line
/// (a pod's edge gap), up to a whole card's depth per row.
fn row_position(slot: usize, total: usize, back_row: bool, half: f32, inward_room: f32) -> (f32, f32) {
    let (rows, per_row) = front_row_shape(total, half);
    let row = (slot / per_row.max(1)).min(rows - 1);
    let col = slot - row * per_row;
    // The last row takes the remainder; earlier rows are full.
    let row_count = if row + 1 == rows { total - row * per_row } else { per_row };
    let offset = col as f32 - (row_count as f32 - 1.0) / 2.0;
    let x = offset * bf_spacing_col(row_count, half);
    if back_row {
        // The back row used to compress without limit: eight land groups in
        // 1v1 already sat 2.91 apart against a 3.0-wide card, and twenty sat
        // 1.07 apart — an unreadable smear. It wraps on the same
        // `MIN_WRAP_SPACING` rule as the front row, stepping *outward* (away
        // from the table centre) so it never reaches into the creature band.
        (x, BF_LAND_Z + row as f32 * row_wrap_dz(rows))
    } else {
        let dz = if rows <= 1 {
            0.0
        } else {
            ((BF_CREATURE_Z - BF_ROW_MIN_Z + inward_room) / (rows as f32 - 1.0))
                .min(CARD_HEIGHT + 0.3)
        };
        (x, BF_CREATURE_Z - row as f32 * dz)
    }
}

/// True if this permanent lays out in the back row (lands + support).
/// The front row holds the permanents that fight or get fought over —
/// creatures *and planeswalkers* — while mana rocks, enchantments, and
/// other noncreature permanents sit alongside the lands, matching how
/// players physically arrange busy commander boards.
///
/// A planeswalker parked in the land row read as a land: it sat behind
/// the creatures, away from the combat line it is attacked across, and it
/// inflated the land row's group count (which is what compresses that row
/// into an overlapping smear). Planeswalkers are permanents you point
/// attackers and burn at, so they belong on the front line.
pub fn in_back_row(c: &crabomination::net::PermanentView) -> bool {
    c.is_land() || !(c.is_creature() || c.is_planeswalker())
}

/// Returns `(group_slot, index_in_group, total_groups)` for a back-row
/// permanent. Lands come first, grouped by name (identical basics stack);
/// support permanents (nonland noncreature) follow — token piles group by
/// visual identity (same key as the creature row, so the ×N badge anchors
/// correctly), nontokens each get their own group. Seat-agnostic.
pub fn back_row_group_info_from_view(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    card_id: CardId,
) -> Option<(usize, usize, usize)> {
    let backs: Vec<_> = battlefield
        .iter()
        .filter(|c| c.owner == owner && in_back_row(c))
        .collect();
    backs.iter().find(|c| c.id == card_id)?;

    // Land name groups, sorted for a stable order.
    let mut land_names: Vec<&str> =
        backs.iter().filter(|c| c.is_land()).map(|c| c.name.as_str()).collect();
    land_names.sort_unstable();
    land_names.dedup();

    // Support groups in first-appearance order after the lands.
    #[derive(PartialEq)]
    enum Key<'a> {
        Token { name: &'a str, power: i32, toughness: i32, tapped: bool },
        Single(CardId),
    }
    fn support_key<'a>(c: &'a crabomination::net::PermanentView) -> Key<'a> {
        if c.is_token {
            Key::Token { name: c.name.as_str(), power: c.power, toughness: c.toughness, tapped: c.tapped }
        } else {
            Key::Single(c.id)
        }
    }
    let mut support_groups: Vec<(Key<'_>, usize)> = Vec::new();
    let mut found: Option<(usize, usize)> = None;
    for c in backs.iter().filter(|c| !c.is_land()) {
        let k = support_key(c);
        let gi = match support_groups.iter().position(|(gk, _)| *gk == k) {
            Some(i) => {
                support_groups[i].1 += 1;
                i
            }
            None => {
                support_groups.push((k, 1));
                support_groups.len() - 1
            }
        };
        if c.id == card_id {
            found = Some((land_names.len() + gi, support_groups[gi].1 - 1));
        }
    }
    let total_groups = land_names.len() + support_groups.len();
    if let Some((slot, index)) = found {
        return Some((slot, index, total_groups));
    }

    // The card is a land: name-group lookup.
    let target = backs.iter().find(|c| c.id == card_id)?;
    let group_slot = land_names.iter().position(|&n| n == target.name.as_str())?;
    let index_in_group = backs
        .iter()
        .filter(|c| c.is_land() && c.name == target.name)
        .position(|c| c.id == card_id)?;
    Some((group_slot, index_in_group, total_groups))
}

/// Grouping for the front (creature) row: identical tokens (same name,
/// computed P/T, and tapped state) collapse into one cascaded pile; every
/// nontoken permanent is its own group. Returns `(group_slot,
/// index_in_group, total_groups)` for `card_id`, in first-appearance order
/// — the same shape `back_row_group_info_from_view` returns for the back
/// row. Lands and noncreature, nonplaneswalker permanents live in the back
/// row (see [`in_back_row`]).
pub fn creature_group_info_from_view(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    card_id: CardId,
) -> Option<(usize, usize, usize)> {
    // A pile key: tokens group by visual identity, nontokens by id (so
    // two Grizzly Bears cards still sit apart — only tokens pile up).
    #[derive(PartialEq)]
    enum Key<'a> {
        Token { name: &'a str, power: i32, toughness: i32, tapped: bool },
        Single(CardId),
    }
    let mut groups: Vec<(Key<'_>, usize)> = Vec::new();
    let mut found: Option<(usize, usize)> = None;
    for c in battlefield.iter().filter(|c| c.owner == owner && !in_back_row(c)) {
        let k = if c.is_token {
            Key::Token { name: c.name.as_str(), power: c.power, toughness: c.toughness, tapped: c.tapped }
        } else {
            Key::Single(c.id)
        };
        let gi = match groups.iter().position(|(gk, _)| *gk == k) {
            Some(i) => {
                groups[i].1 += 1;
                i
            }
            None => {
                groups.push((k, 1));
                groups.len() - 1
            }
        };
        if c.id == card_id {
            found = Some((gi, groups[gi].1 - 1));
        }
    }
    found.map(|(slot, index)| (slot, index, groups.len()))
}

/// World transform for a creature-row permanent: identical tokens cascade
/// into one pile (same stagger as the land stacks, so the board reads
/// consistently), nontokens get their own slot exactly as before.
pub fn creature_card_transform(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    viewer: usize,
    n_seats: usize,
    card_id: CardId,
    tapped: bool,
) -> Option<Transform> {
    let (group_slot, index, total_groups) =
        creature_group_info_from_view(battlefield, owner, card_id)?;
    let base =
        bf_card_transform(owner, viewer, n_seats, group_slot, total_groups, false, tapped);
    let stagger = seat_rotation(owner, viewer, n_seats)
        * Vec3::new(
            index as f32 * LAND_STACK_OFFSET_X,
            index as f32 * CARD_THICKNESS * 1.5,
            index as f32 * LAND_STACK_OFFSET_Z,
        );
    Some(Transform {
        translation: base.translation + stagger,
        rotation: base.rotation,
        scale: base.scale,
    })
}

/// Table-level outline of a seat's board area — `(min, max)` corners at
/// y = 0, spanning the column width and both battlefield rows plus a small
/// margin. Used by the active-seat glow and the eliminated-player shroud.
pub fn seat_board_outline(seat: usize, viewer: usize, n_seats: usize) -> (Vec3, Vec3) {
    if let Some(f) = pod_frame(seat, viewer, n_seats) {
        let half = f.half + 0.8;
        // A wrapped creature row may use the edge gap, up to the centre line.
        let near = BF_ROW_MIN_Z - CARD_HEIGHT * 0.5 - f.offset;
        let far = BF_LAND_Z + CARD_HEIGHT * 0.5 + 0.4;
        let a = f.point(Vec3::new(-half, 0.0, near));
        let b = f.point(Vec3::new(half, 0.0, far));
        return (a.min(b), a.max(b));
    }
    let spot = seat_spot(seat, viewer, n_seats);
    let half = board_half_for(&spot, n_seats) + 0.8;
    let x_base = if n_seats <= 2 {
        opp_x_offset(seat, viewer, n_seats)
    } else {
        spot.board_center
    };
    // From the innermost a wrapped front row may reach out to the far side
    // of the land row. `BF_ROW_MIN_Z` is that inner bound by construction, so
    // the outline now stops at the table centre instead of reaching across it.
    let z_near = spot.z_sign * (BF_ROW_MIN_Z - CARD_HEIGHT * 0.5);
    let z_far = spot.z_sign * (BF_LAND_Z + CARD_HEIGHT * 0.5 + 0.4);
    (
        Vec3::new(x_base - half, 0.0, z_near.min(z_far)),
        Vec3::new(x_base + half, 0.0, z_near.max(z_far)),
    )
}

// ── Seat regions (table tint) ────────────────────────────────────────────────

/// How far a seat region runs out from the table centre: past everything the
/// camera frames, to the ground plane's edge (`main::setup`, 90 × 90).
const REGION_REACH: f32 = 45.0;
/// Plain table left between two seats' regions, so the boundary reads.
pub const REGION_SEAM: f32 = 0.4;

/// The part of the table (XZ) that belongs to `seat`: its half in 1v1, its
/// quadrant in a 4-player pod (half the far edge each at 3 players; the near
/// edge is the viewer's), its column in 5-6. Regions run from the table's
/// centre lines out past the frame and stop [`REGION_SEAM`]/2 short of each
/// other. `systems::table_tint` shades each in its seat colour.
pub fn seat_region(seat: usize, viewer: usize, n_seats: usize) -> Rect {
    let r = REGION_REACH;
    let s = REGION_SEAM * 0.5;
    // X span for a seat sitting left (−1), centre (0) or right (+1).
    let lateral = |side: f32| -> (f32, f32) {
        if side < 0.0 {
            (-r, -s)
        } else if side > 0.0 {
            (s, r)
        } else {
            (-r, r)
        }
    };
    let (x0, x1, near) = if let Some(f) = pod_frame(seat, viewer, n_seats) {
        let side = if f.shift.x.abs() < 0.1 { 0.0 } else { f.shift.x.signum() };
        let (x0, x1) = lateral(side);
        (x0, x1, f.yaw.abs() < 1.0)
    } else if n_seats <= 2 {
        (-r, r, is_viewer(seat, viewer))
    } else {
        // Columns along each edge; the outermost reach the table's side.
        let spot = seat_spot(seat, viewer, n_seats);
        let (_, col, cols) = seat_slot(seat, viewer, n_seats);
        let col_w = 2.0 * MULTI_HALF_X / cols.max(1) as f32;
        let x0 = if col == 0 { -r } else { spot.col_center - col_w * 0.5 + s };
        let x1 = if col + 1 == cols { r } else { spot.col_center + col_w * 0.5 - s };
        (x0, x1, spot.z_sign > 0.0)
    };
    let (z0, z1) = if near { (s, r) } else { (-r, -s) };
    Rect::new(x0, z0, x1, z1)
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

/// World transform for a back-row card (lands and support permanents),
/// with stacking offsets for identical lands and token piles.
pub fn back_row_card_transform(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    viewer: usize,
    n_seats: usize,
    card_id: CardId,
) -> Option<Transform> {
    let (group_slot, index, total_groups) =
        back_row_group_info_from_view(battlefield, owner, card_id)?;
    let base = bf_card_transform(owner, viewer, n_seats, group_slot, total_groups, true, false);
    // Stagger pulls subsequent cards toward the back of the row (toward the
    // owner's edge of the table) so each card's name strip stays visible.
    let stagger = seat_rotation(owner, viewer, n_seats)
        * Vec3::new(
            index as f32 * LAND_STACK_OFFSET_X,
            index as f32 * CARD_THICKNESS * 1.5,
            index as f32 * LAND_STACK_OFFSET_Z,
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
    fn four_player_pod_sits_two_to_an_edge() {
        // Clockwise from the viewer at front-left: far-left, far-right,
        // front-right — for every viewer, since seating is viewer-relative.
        for viewer in 0..4 {
            let at = |k: usize| {
                let f = pod_frame((viewer + k) % 4, viewer, 4).unwrap();
                (f.yaw.abs() > 1.0, f.shift.x.signum())
            };
            assert_eq!(at(0), (false, -1.0), "the viewer is front-left");
            assert_eq!(at(1), (true, -1.0), "the next seat is across on the left");
            assert_eq!(at(2), (true, 1.0), "then across on the right");
            assert_eq!(at(3), (false, 1.0), "then beside the viewer on the right");
        }
    }
    #[test]
    fn six_player_falls_back_to_two_per_side() {
        let fronts = (0..6).filter(|&s| seat_spot(s, 0, 6).z_sign > 0.0).count();
        assert_eq!(fronts, 3, "5+ player pods split seats across both edges");
    }

    #[test]
    fn pod_piles_sit_beside_their_board() {
        // Every pod seat's deck and graveyard sit in the strip just past its
        // board's edge — not out at a table edge the camera has to reach.
        for n in [3usize, 4] {
            for s in 0..n {
                let f = pod_frame(s, 0, n).unwrap();
                for pile in [deck_position(s, 0, n), graveyard_position(s, 0, n)] {
                    let x = f.local(pile).x.abs();
                    assert!(
                        x >= f.half + CARD_WIDTH * 0.5 - 0.01 && x <= f.half + CARD_WIDTH + 0.01,
                        "n={n} seat {s}: a pile at local x={x} isn't beside the board",
                    );
                }
            }
        }
    }
    #[test]
    fn four_player_columns_dont_overlap_and_stay_on_table() {
        // No two seats' boards overlap, and every board stays on the table.
        for n in [3usize, 4, 5, 6] {
            let boards: Vec<Rect> = (0..n).map(|s| board_footprint(s, 0, n)).collect();
            for (i, a) in boards.iter().enumerate() {
                assert!(a.min.x >= -MULTI_HALF_X - 0.01 && a.max.x <= MULTI_HALF_X + 0.01, "n={n} seat {i} off the table");
                for (j, b) in boards.iter().enumerate().skip(i + 1) {
                    assert!(a.intersect(*b).is_empty(), "n={n}: seats {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
    }

    /// Where `seat`'s battlefield cards can lie on the table (XZ): its rows'
    /// card edges, from the innermost wrapped creature row out to a wrapped
    /// second land row.
    fn board_footprint(seat: usize, viewer: usize, n: usize) -> Rect {
        let far = BF_LAND_Z + row_wrap_dz(2) + CARD_HEIGHT * 0.5;
        if let Some(f) = pod_frame(seat, viewer, n) {
            let edge = f.half - 0.3;
            let a = f.point(Vec3::new(-edge, 0.0, -f.offset));
            let b = f.point(Vec3::new(edge, 0.0, far));
            return Rect::from_corners(Vec2::new(a.x, a.z), Vec2::new(b.x, b.z));
        }
        let sp = seat_spot(seat, viewer, n);
        let (z0, z1) = if sp.z_sign > 0.0 { (0.0, far) } else { (-far, 0.0) };
        Rect::new(sp.board_center - sp.board_half, z0, sp.board_center + sp.board_half, z1)
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
    fn three_player_puts_both_opponents_across() {
        // A triangle: the viewer front-centre, the next seat across on the
        // left, the last across on the right.
        let f = |s: usize| pod_frame(s, 0, 3).unwrap();
        assert_eq!((f(0).yaw, f(0).shift.x), (0.0, 0.0));
        assert!(f(1).yaw.abs() > 1.0 && f(1).shift.x < 0.0);
        assert!(f(2).yaw.abs() > 1.0 && f(2).shift.x > 0.0);
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
    fn seat_regions_tile_the_table_without_overlap() {
        // Each seat's tint region holds its board and piles, and no two
        // regions touch (a seam of plain table runs between them).
        for n in [2usize, 3, 4, 5, 6] {
            let regions: Vec<Rect> = (0..n).map(|s| seat_region(s, 0, n)).collect();
            for (i, a) in regions.iter().enumerate() {
                for (j, b) in regions.iter().enumerate().skip(i + 1) {
                    assert!(a.intersect(*b).is_empty(), "n={n}: regions {i} and {j} overlap");
                }
                let xz = |p: Vec3| Vec2::new(p.x, p.z);
                if n > 2 {
                    let board = board_footprint(i, 0, n);
                    assert!(
                        a.contains(board.min + Vec2::splat(REGION_SEAM))
                            && a.contains(board.max - Vec2::splat(REGION_SEAM)),
                        "n={n}: seat {i}'s board isn't in its region",
                    );
                }
                for pile in [deck_position(i, 0, n), graveyard_position(i, 0, n)] {
                    assert!(a.contains(xz(pile)), "n={n}: seat {i}'s pile {pile} is outside its region");
                }
            }
        }
    }

    #[test]
    fn board_area_clears_the_pile_strip() {
        // Piles never sit on a battlefield: no seat's deck, graveyard,
        // command card or face-down hand overlaps any seat's board, wrapped
        // rows included. Near-edge seats keep a strip beside their board;
        // far seats keep a row behind it, so the check is on footprints,
        // not on X alone.
        let rect = |c: Vec3, w: f32, h: f32| Rect::from_center_size(Vec2::new(c.x, c.z), Vec2::new(w, h));
        for n in [3usize, 4, 5, 6] {
            let boards: Vec<Rect> = (0..n).map(|s| board_footprint(s, 0, n)).collect();
            for s in 0..n {
                let cmd = command_zone_card_transform(s, 0, n, 0);
                let mut piles = vec![
                    ("deck", rect(deck_position(s, 0, n), CARD_WIDTH, CARD_HEIGHT)),
                    ("graveyard", rect(graveyard_position(s, 0, n), CARD_WIDTH, CARD_HEIGHT)),
                    ("command", rect(cmd.translation, CARD_WIDTH * cmd.scale.x, CARD_HEIGHT * cmd.scale.x)),
                ];
                if s != 0 {
                    for slot in 0..7 {
                        let h = hand_card_transform(s, 0, n, slot, 7, 1.0);
                        if h.translation.y < 1.0 {
                            // A flat stack or spread; a raised fan is above
                            // the table, not on it. A card lying across the
                            // table (a side seat) swaps its footprint.
                            let (w, d) = if (h.rotation * Vec3::Y).x.abs() > 0.5 {
                                (CARD_HEIGHT, CARD_WIDTH)
                            } else {
                                (CARD_WIDTH, CARD_HEIGHT)
                            };
                            piles.push(("hand", rect(h.translation, w, d)));
                        }
                    }
                }
                piles.push(("exile", rect(exile_position(n), CARD_WIDTH, CARD_HEIGHT)));
                for (what, r) in &piles {
                    for (b, board) in boards.iter().enumerate() {
                        assert!(
                            board.intersect(*r).is_empty(),
                            "n={n}: seat {s}'s {what} at {:?} sits on seat {b}'s board {board:?}",
                            r.center(),
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn small_front_row_keeps_single_row_geometry() {
        // 4 creature groups in 1v1: everyone on the historical creature line,
        // full spacing, no wrap.
        for slot in 0..4 {
            let t = bf_card_transform(0, 0, 2, slot, 4, false, false);
            assert!((t.translation.z - BF_CREATURE_Z).abs() < 1e-4, "no wrap for small rows");
        }
        let a = bf_card_transform(0, 0, 2, 0, 4, false, false).translation.x;
        let b = bf_card_transform(0, 0, 2, 1, 4, false, false).translation.x;
        assert!((b - a - BF_CARD_SPACING).abs() < 1e-4, "full spacing for small rows");
    }

    #[test]
    fn crowded_front_row_wraps_toward_centre() {
        // A pod board with 8 creature groups must wrap into two rows instead
        // of compressing below MIN_WRAP_SPACING, the wrap stepping toward the
        // table centre — a whole card deep, since a pod has the edge gap for
        // it — and every group staying on its own board.
        let (seat, n, total) = (2, 4, 8); // far-right in a 4-player pod
        let f = pod_frame(seat, 0, n).unwrap();
        let local: Vec<Vec3> = (0..total)
            .map(|s| f.local(bf_card_transform(seat, 0, n, s, total, false, false).translation))
            .collect();
        let mut rows: Vec<f32> = local.iter().map(|p| p.z).collect();
        rows.sort_by(f32::total_cmp);
        rows.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
        assert_eq!(rows.len(), 2, "8 groups form two rows, got local z rows {rows:?}");
        assert!(rows[0] < BF_CREATURE_Z, "the wrap steps toward the centre: {rows:?}");
        assert!(rows[1] - rows[0] >= CARD_HEIGHT, "wrapped rows don't overlap: {rows:?}");
        for (s, p) in local.iter().enumerate() {
            assert!(p.x.abs() + CARD_WIDTH * 0.5 <= f.half + 0.01, "slot {s} leaks off its board");
        }
        // Wrapping must beat single-row compression decisively.
        let single_row = bf_spacing_col(total, f.half);
        for (a, b) in [(0usize, 1usize), (4, 5)] {
            let gap = (local[b].x - local[a].x).abs();
            assert!(gap >= single_row * 1.8, "wrapped spacing {gap} vs compressed {single_row}");
        }
    }
    #[test]
    fn crowded_rows_shingle_instead_of_z_fighting() {
        // Regression: eight land groups in 1v1 compress below CARD_WIDTH,
        // so adjacent cards overlap. At a shared Y they are coplanar and
        // the depth buffer flickers. Every slot must sit at its own height.
        let half = board_half_for(&seat_spot(0, 0, 2), 2);
        assert!(
            bf_spacing_col(8, half) < CARD_WIDTH,
            "precondition: eight back-row groups really do overlap in 1v1",
        );
        let ys: Vec<f32> =
            (0..8).map(|s| bf_card_transform(0, 0, 2, s, 8, true, false).translation.y).collect();
        for pair in ys.windows(2) {
            assert!(pair[1] > pair[0], "slots must be strictly ordered in height: {ys:?}");
        }
        // The ramp is a depth tiebreak, not a visible lift: the whole row
        // stays flat against the table.
        assert!(ys[7] - ys[0] < 0.3, "shingle ramp must stay imperceptible, got {ys:?}");
    }

    /// The back row wraps outward rather than compressing without limit.
    ///
    /// It used to stay on one z line whatever the count: eight land groups in
    /// 1v1 already sat 2.91 apart against a 3.0-wide card, and twenty sat 1.07
    /// apart — the "overlapping permanents" report. It now wraps on the same
    /// `MIN_WRAP_SPACING` rule as the front row, and steps *away* from the
    /// table centre so it never reaches into the creature band.
    #[test]
    fn back_row_wraps_outward_instead_of_smearing() {
        // Small rows keep the historical single-line geometry.
        for slot in 0..6 {
            let t = bf_card_transform(0, 0, 2, slot, 6, true, false);
            assert!((t.translation.z - BF_LAND_Z).abs() < 1e-4, "6 groups stay on one line");
        }

        // A count that used to smear now forms two rows with readable gaps.
        let total = 10;
        let xs: Vec<f32> = (0..total)
            .map(|s| bf_card_transform(0, 0, 2, s, total, true, false).translation.x)
            .collect();
        let mut rows: Vec<f32> = (0..total)
            .map(|s| bf_card_transform(0, 0, 2, s, total, true, false).translation.z)
            .collect();
        rows.sort_by(f32::total_cmp);
        rows.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
        assert_eq!(rows.len(), 2, "10 land groups wrap, got rows {rows:?}");
        assert!(
            (xs[1] - xs[0]).abs() >= CARD_WIDTH,
            "and the wrapped rows are wide enough to read: {:.2} vs card width {CARD_WIDTH}",
            (xs[1] - xs[0]).abs(),
        );
        // Outward: every land row is at or beyond the historical land line,
        // so the wrap can never collide with the creatures in front of it.
        for z in &rows {
            assert!(*z >= BF_LAND_Z - 1e-4, "land rows step away from centre, got {rows:?}");
        }
    }

    /// No battlefield row may cross the table centre.
    ///
    /// Rows are mirrored per seat, so a row that crosses z = 0 lands in the
    /// opponent's half — and when both players wrap, the two inner rows fight
    /// for the same volume. The old fixed wrap step put the second creature
    /// row's near edge at z = −3.21, three units deep into enemy territory:
    /// the boards visibly intersected once both sides had seven-plus groups.
    #[test]
    fn wrapped_rows_stay_on_their_own_side_of_the_table() {
        let half_depth = CARD_HEIGHT * 0.5;
        // Pods: however crowded, no seat's rows cross the table's centre
        // line into the opposite edge's half.
        for n in [3usize, 4] {
            for total in 1..=24usize {
                for back_row in [false, true] {
                    for seat in 0..n {
                        let f = pod_frame(seat, 0, n).unwrap();
                        let sign = if f.yaw.abs() > 1.0 { -1.0 } else { 1.0 };
                        for slot in 0..total {
                            let z = bf_card_transform(seat, 0, n, slot, total, back_row, false).translation.z;
                            assert!(
                                sign * z - half_depth >= -1e-3,
                                "n={n} seat {seat} slot {slot}/{total} crosses the centre line",
                            );
                        }
                    }
                }
            }
        }
        for total in 1..=24usize {
            for back_row in [false, true] {
                for seat in 0..2 {
                    for slot in 0..total {
                        let t = bf_card_transform(seat, 0, 2, slot, total, back_row, false);
                        let sign = if seat == 0 { 1.0 } else { -1.0 };
                        let near_edge = sign * t.translation.z - half_depth;
                        assert!(
                            near_edge >= -1e-4,
                            "seat {seat} slot {slot}/{total} (back={back_row}) reaches \
                             {near_edge:.3} past the table centre",
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn front_row_shape_balances_rows() {
        // Capacity math: rows never exceed MAX_WRAP_ROWS and always cover total.
        for total in 1..40usize {
            for half in [4.0f32, 6.0, 10.0, 13.0] {
                let (rows, per_row) = front_row_shape(total, half);
                assert!((1..=MAX_WRAP_ROWS).contains(&rows));
                assert!(rows * per_row >= total, "shape must cover all groups");
                assert!(per_row * (rows - 1) < total, "no empty trailing row");
            }
        }
    }

    #[test]
    fn opponent_hands_stay_inside_their_columns() {
        // Regression: with the historical 7-card fan width, adjacent
        // back-edge opponents' hands overlapped in a 4-player pod. Every
        // card of a big hand must stay within its seat's column. In 1v1 the
        // hand is a compact spread past the table's pile line, clear of
        // every board row.
        for n in [3usize, 4] {
            for seat in 1..n {
                let spot = seat_spot(seat, 0, n);
                for slot in 0..10 {
                    let x = hand_card_transform(seat, 0, n, slot, 10, 1.0).translation.x;
                    assert!(
                        (x - spot.board_center).abs() <= spot.board_half + CARD_WIDTH / 2.0 + 0.01,
                        "n={n} seat={seat} slot={slot}: hand card at x={x} leaves its column",
                    );
                }
            }
        }
        // The outer edge of the 1v1 rows: their centres stop at
        // DECK_X − CARD_WIDTH/2 (`board_half_for`).
        let row_edge = DECK_X;
        let mut last: Option<f32> = None;
        for slot in 0..10 {
            let t = hand_card_transform(1, 0, 2, slot, 10, 1.0).translation;
            assert!(t.x.abs() - CARD_WIDTH * 0.5 >= row_edge, "1v1 slot {slot} at x={} over a row", t.x);
            assert!(t.z < 0.0, "the opponent's hand stays on their half");
            if let Some(prev) = last {
                assert!((t.x - prev).abs() >= OPP_HAND_STEP - 1e-4, "cards step apart so the count reads");
            }
            last = Some(t.x);
        }
    }

    #[test]
    fn command_zone_clears_deck_and_graveyard_piles() {
        // Regression: after the pile-strip half_width clamp, the multiplayer
        // command zone (board outer-front corner at hand depth) landed under
        // the deck pile. Footprints may not intersect for any seat. The
        // command card scales up to 1.3× on the far edge — use its scaled
        // extents.
        for n in [3usize, 4, 5, 6] {
            for s in 0..n {
                let cmd = command_zone_card_transform(s, 0, n, 0);
                let c = cmd.translation;
                let (cw, ch) =
                    (CARD_WIDTH * cmd.scale.x, CARD_HEIGHT * cmd.scale.x);
                for pile in [deck_position(s, 0, n), graveyard_position(s, 0, n)] {
                    let overlap_x = (pile.x - c.x).abs() < (CARD_WIDTH + cw) / 2.0;
                    let overlap_z = (pile.z - c.z).abs() < (CARD_HEIGHT + ch) / 2.0;
                    assert!(
                        !(overlap_x && overlap_z),
                        "n={n} seat={s}: command zone ({:.1},{:.1}) under a pile ({:.1},{:.1})",
                        c.x, c.z, pile.x, pile.z,
                    );
                }
            }
        }
    }
    #[test]
    fn seat_slot_is_viewer_relative() {
        // The same clockwise shape regardless of which seat is the viewer.
        for v in 0..4 {
            let (front, col, cols) = seat_slot(v, v, 4);
            assert!(front && col == 0 && cols == 1, "viewer is always front, alone");
        }
        for v in 0..6 {
            let (front, col, cols) = seat_slot(v, v, 6);
            assert!(front && col == 0 && cols == 3, "6p viewer is front-left of three");
        }
    }
}
