//! Board framing: where the camera sits, and what that buys on screen.
//!
//! [`home_pose`] is the camera's resting pose — the one `camera_zoom` eases
//! back to. [`measure`] projects a representative busy board (the same
//! shape as `layout_harness::fixture_state`) through that pose with the
//! engine's real layout functions and reports how large its cards land on
//! screen, how much of the window the table uses, and whether anything
//! falls off it. The `budget` test pins those numbers, so a layout change
//! that shrinks the cards or pushes the board off screen fails a test
//! rather than waiting for someone to notice.

use bevy::prelude::*;

use super::components::{CARD_HEIGHT, CARD_WIDTH, pile_height};
use super::layout::{
    bf_card_transform, command_zone_card_transform, deck_position, exile_position,
    graveyard_position, hand_card_transform,
};

/// Vertical field of view: Bevy's `PerspectiveProjection` default, which the
/// main camera never overrides.
pub const FOV_Y: f32 = std::f32::consts::FRAC_PI_4;
const NEAR: f32 = 0.1;

/// The fixed poses the camera used before it was fitted to the window.
pub fn legacy_pose(n_seats: usize) -> Transform {
    let pos = if n_seats > 2 { Vec3::new(0.0, 46.0, 24.0) } else { Vec3::new(0.0, 32.0, 14.0) };
    Transform::from_translation(pos).looking_at(Vec3::ZERO, Vec3::Y)
}

/// The direction the home pose looks from (table point → camera): the 1v1
/// camera's 66° pitch, for every table size. The viewer's hand is tilted to
/// face it (`layout::HAND_TILT_X`), and pods sit two to an edge like a
/// wider 1v1 table; the old pod pitch (62°, from (0, 46, 24)) put the hand
/// over the viewer's land row.
const VIEW_DIRECTION: Vec3 = Vec3::new(0.0, 32.0, 14.0);

/// The camera's resting pose for a table of `n_seats` in a window of
/// `viewport` logical pixels.
pub fn home_pose(n_seats: usize, viewport: Vec2) -> Transform {
    fit_pose(n_seats, viewport)
}

/// The viewer's hand scale for a window `logical_height` px tall: larger
/// on small displays so hand cards stay a readable size.
pub fn hand_zoom_for(logical_height: f32) -> f32 {
    if logical_height >= 1080.0 {
        1.0
    } else if logical_height >= 800.0 {
        1.15
    } else {
        1.30
    }
}

/// The persistent HUD panels the table must not run under, in logical px:
/// nominal sizes of the corner panels `game_ui::setup_game_hud` spawns.
/// The table is a trapezoid (its far edge is narrow), so it can sit between
/// the top corner panels while its wide near edge only has to clear the
/// action buttons.
pub fn hud_rects(viewport: Vec2, n_seats: usize) -> [Rect; 5] {
    let (w, h) = (viewport.x, viewport.y);
    // One 45 px row per opponent under a turn-order strip in a pod.
    let opp_panel_h = if n_seats > 2 { 30.0 + 45.0 * (n_seats - 1) as f32 } else { 70.0 };
    [
        // Player panel: turn line, phase strip and a row of stat chips.
        Rect::new(0.0, 0.0, 1090.0_f32.min(w - 470.0), 100.0),
        // Phase chart under it.
        Rect::new(0.0, 100.0, 146.0, 346.0),
        // Action buttons, bottom-left.
        Rect::new(0.0, h - 296.0, 180.0, h),
        // Opponent panel, top-right, with the game log hanging under it.
        Rect::new(w - 460.0, 0.0, w, opp_panel_h),
        Rect::new(w - 292.0, opp_panel_h, w, opp_panel_h + 436.0),
    ]
}

/// Share of a hand card (from its top edge) that must stay on screen; the
/// rest may run off the bottom of the window, as it always has.
pub const HAND_VISIBLE: f32 = 0.5;
/// Clearance kept between the table and a HUD panel or the window edge.
const MARGIN: f32 = 6.0;

/// True when every card of `board` seen through `cam` sits inside the
/// window and clear of the HUD (the viewer's hand only needs its top
/// [`HAND_VISIBLE`] on screen).
/// `(corners, is the viewer's hand)` per card, outermost first so a pose
/// that fails is usually rejected by its first few cards.
fn fit_cards(board: &[Placed]) -> Vec<([Vec3; 8], bool)> {
    let mut cards: Vec<_> = board
        .iter()
        .map(|c| (corners(&c.transform, c.height), c.role == Role::Hand && c.seat == 0))
        .collect();
    let reach = |c: &[Vec3; 8]| c.iter().map(|p| p.x.abs().max(p.z.abs())).fold(0.0, f32::max);
    cards.sort_by(|a, b| reach(&b.0).total_cmp(&reach(&a.0)));
    cards
}

fn clear(cards: &[([Vec3; 8], bool)], cam: &Transform, viewport: Vec2, hud: &[Rect]) -> bool {
    let window = Rect::from_corners(Vec2::splat(MARGIN), viewport - MARGIN);
    let projector = Projector::new(cam, viewport);
    cards.iter().all(|(corners, viewer_hand)| {
        let Some(mut r) = projector.rect_of(corners) else { return false };
        if *viewer_hand {
            r.max.y = r.min.y + r.height() * HAND_VISIBLE;
        }
        window.contains(r.min)
            && window.contains(r.max)
            && hud.iter().all(|p| p.inflate(MARGIN).intersect(r).is_empty())
    })
}

/// The closest pose, looking down [`VIEW_DIRECTION`], that keeps
/// the representative board on screen and clear of the HUD. At each
/// distance the look-at point is tried centred first, then nudged in
/// widening steps; a bisection on the distance keeps the closest distance
/// some nudge clears.
pub fn fit_pose(n_seats: usize, viewport: Vec2) -> Transform {
    let back = VIEW_DIRECTION.normalize();
    let cards = fit_cards(&sample_board(n_seats, hand_zoom_for(viewport.y)));
    let hud = hud_rects(viewport, n_seats);
    let pose = |target: Vec3, d: f32| {
        Transform::from_translation(target + back * d).looking_at(target, Vec3::Y)
    };
    // Nudges in table units per unit of distance, nearest first.
    let mut nudges: Vec<Vec2> = Vec::new();
    for i in -8..=8 {
        for j in -6..=6 {
            nudges.push(Vec2::new(i as f32 * 0.02, j as f32 * 0.02));
        }
    }
    nudges.sort_by(|a, b| a.length().total_cmp(&b.length()));
    let placement = |d: f32| {
        nudges.iter().map(|n| Vec3::new(n.x * d, 0.0, n.y * d)).find(|t| {
            clear(&cards, &pose(*t, d), viewport, &hud)
        })
    };
    let (mut lo, mut hi) = (5.0_f32, 400.0_f32);
    let mut best = (Vec3::ZERO, hi);
    // 18 halvings of 395 units: 1.5 thousandths of a unit.
    for _ in 0..18 {
        let mid = 0.5 * (lo + hi);
        match placement(mid) {
            Some(t) => {
                best = (t, mid);
                hi = mid;
            }
            None => lo = mid,
        }
    }
    pose(best.0, best.1)
}

/// World → logical-pixel projection for a camera at `cam`: the same
/// reverse-Z infinite perspective Bevy's camera uses, built once per pose.
#[derive(Clone, Copy)]
pub struct Projector {
    clip_from_world: Mat4,
    viewport: Vec2,
}

impl Projector {
    pub fn new(cam: &Transform, viewport: Vec2) -> Self {
        let view = cam.to_matrix().inverse();
        let proj = Mat4::perspective_infinite_reverse_rh(FOV_Y, viewport.x / viewport.y, NEAR);
        Projector { clip_from_world: proj * view, viewport }
    }

    /// `None` behind the camera.
    pub fn project(&self, p: Vec3) -> Option<Vec2> {
        let clip = self.clip_from_world * p.extend(1.0);
        if clip.w <= 0.0 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        Some(Vec2::new((ndc.x + 1.0) * 0.5 * self.viewport.x, (1.0 - ndc.y) * 0.5 * self.viewport.y))
    }

    /// Screen-space bounding box of a card face placed at `card`, lifted
    /// by `height` for a pile.
    #[cfg(test)]
    pub fn card_rect(&self, card: &Transform, height: f32) -> Option<Rect> {
        self.rect_of(&corners(card, height))
    }

    /// Screen-space bounding box of world points.
    pub fn rect_of(&self, points: &[Vec3]) -> Option<Rect> {
        let mut rect: Option<Rect> = None;
        for p in points {
            let p = self.project(*p)?;
            rect = Some(rect.map_or(Rect::from_center_size(p, Vec2::ZERO), |r| r.union_point(p)));
        }
        rect
    }
}

/// World corners of a card face placed at `card` (the mesh is a
/// `CARD_WIDTH` × `CARD_HEIGHT` rectangle in its local XY plane), and of the
/// same face lifted by `height` for a pile.
fn corners(card: &Transform, height: f32) -> [Vec3; 8] {
    let (hw, hh) = (CARD_WIDTH * 0.5, CARD_HEIGHT * 0.5);
    let face = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
        .map(|(x, y)| card.transform_point(Vec3::new(x, y, 0.0)));
    let mut out = [Vec3::ZERO; 8];
    for i in 0..4 {
        out[i] = face[i];
        out[i + 4] = face[i] + Vec3::Y * height;
    }
    out
}

/// What a card on the table is, for the budget's bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Creature,
    BackRow,
    Hand,
    Pile,
}

/// Groups per seat in the representative board: the harness fixture's six
/// creatures (two tapped) and seven back-row groups (five land names, a
/// mana rock, an enchantment), with seven cards in the viewer's hand and
/// five in each opponent's.
const CREATURES: usize = 6;
const TAPPED: [usize; 2] = [1, 4];
const _: () = assert!(TAPPED[0] != 2 && TAPPED[1] != 2, "slot 2 is the reference card");
const BACK_GROUPS: usize = 7;
const VIEWER_HAND: usize = 7;
const OPP_HAND: usize = 5;
const LIBRARY: usize = 99;

/// One card of the representative board.
#[derive(Clone, Copy, Debug)]
pub struct Placed {
    pub seat: usize,
    pub role: Role,
    pub transform: Transform,
    /// Pile height above the table (0 for a single card).
    pub height: f32,
    /// An untapped creature in the middle of its row — the card whose
    /// on-screen size the budget reports for its seat.
    #[cfg_attr(not(test), allow(dead_code))]
    pub reference: bool,
}

/// Every card of the representative board for `n_seats`, from seat 0's chair.
pub fn sample_board(n_seats: usize, hand_zoom: f32) -> Vec<Placed> {
    let flat = |p: Vec3| {
        Transform::from_translation(p).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
    };
    let card = |seat, role, transform| Placed { seat, role, transform, height: 0.0, reference: false };
    let mut out = Vec::new();
    for seat in 0..n_seats {
        for slot in 0..CREATURES {
            let tapped = TAPPED.contains(&slot);
            let t = bf_card_transform(seat, 0, n_seats, slot, CREATURES, false, tapped);
            // Slot 2 is the middle untapped creature of a six-card row.
            out.push(Placed { reference: slot == 2, ..card(seat, Role::Creature, t) });
        }
        for slot in 0..BACK_GROUPS {
            let t = bf_card_transform(seat, 0, n_seats, slot, BACK_GROUPS, true, false);
            out.push(card(seat, Role::BackRow, t));
        }
        let (hand, zoom) = if seat == 0 { (VIEWER_HAND, hand_zoom) } else { (OPP_HAND, 1.0) };
        for slot in 0..hand {
            out.push(card(seat, Role::Hand, hand_card_transform(seat, 0, n_seats, slot, hand, zoom)));
        }
        out.push(Placed {
            height: pile_height(LIBRARY),
            ..card(seat, Role::Pile, flat(deck_position(seat, 0, n_seats)))
        });
        out.push(card(seat, Role::Pile, flat(graveyard_position(seat, 0, n_seats))));
        if n_seats > 2 {
            out.push(card(seat, Role::Pile, command_zone_card_transform(seat, 0, n_seats, 0)));
        }
    }
    out.push(card(1, Role::Pile, flat(exile_position(n_seats))));
    out
}

/// What the representative board looks like through `cam` in a window of
/// `viewport` logical pixels.
#[cfg(test)]
#[derive(Clone, Debug)]
pub struct Budget {
    /// On-screen width of the viewer's middle untapped creature.
    pub viewer_card_px: f32,
    /// The smallest on-screen width of any opponent's untapped creature.
    pub opp_card_px: f32,
    /// Share of the window's width and height that bounding box spans.
    pub width_used: f32,
    pub height_used: f32,
    /// Cards with any corner outside the window.
    pub off_screen: usize,
    /// Screen area (px²) the viewer's hand covers of the viewer's back row.
    pub hand_over_back_row: f32,
}

#[cfg(test)]
pub fn measure(n_seats: usize, viewport: Vec2, cam: &Transform) -> Budget {
    let board = sample_board(n_seats, hand_zoom_for(viewport.y));
    let window = Rect::from_corners(Vec2::ZERO, viewport);
    let mut content: Option<Rect> = None;
    let mut off_screen = 0;
    let mut viewer_card_px = 0.0;
    let mut opp_card_px = f32::INFINITY;
    let mut hand = Vec::new();
    let mut back = Vec::new();
    let projector = Projector::new(cam, viewport);
    for c in &board {
        let Some(r) = projector.card_rect(&c.transform, c.height) else {
            off_screen += 1;
            continue;
        };
        content = Some(content.map_or(r, |acc| acc.union(r)));
        if !window.contains(r.min) || !window.contains(r.max) {
            off_screen += 1;
        }
        match (c.role, c.seat) {
            (Role::Creature, 0) if c.reference => viewer_card_px = r.width(),
            (Role::Creature, _) if c.reference => opp_card_px = opp_card_px.min(r.width()),
            (Role::Hand, 0) => hand.push(r),
            (Role::BackRow, 0) => back.push(r),
            _ => {}
        }
    }
    let content = content.unwrap_or(window);
    let hand_over_back_row = hand
        .iter()
        .flat_map(|h| back.iter().map(move |b| h.intersect(*b)))
        .filter(|r| !r.is_empty())
        .map(|r| r.width() * r.height())
        .sum();
    Budget {
        viewer_card_px,
        opp_card_px: if opp_card_px.is_finite() { opp_card_px } else { 0.0 },
        width_used: content.width() / viewport.x,
        height_used: content.height() / viewport.y,
        off_screen,
        hand_over_back_row,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORTS: [(f32, f32); 4] = [(1280.0, 720.0), (1920.0, 1080.0), (2560.0, 1080.0), (3840.0, 2160.0)];

    fn table(pose: fn(usize, Vec2) -> Transform) -> Vec<(usize, Vec2, Budget)> {
        let mut rows = Vec::new();
        for seats in [2, 4] {
            for (w, h) in VIEWPORTS {
                let vp = Vec2::new(w, h);
                rows.push((seats, vp, measure(seats, vp, &pose(seats, vp))));
            }
        }
        rows
    }

    fn print(label: &str, rows: &[(usize, Vec2, Budget)]) {
        println!("{label}");
        println!("seats  window      viewer px  opp px  width  height  off-screen  hand∩lands px²");
        for (seats, vp, b) in rows {
            println!(
                "{seats}      {:>4}x{:<4}  {:>8.0}  {:>6.0}  {:>4.0}%  {:>5.0}%  {:>10}  {:>13.0}",
                vp.x, vp.y, b.viewer_card_px, b.opp_card_px, b.width_used * 100.0,
                b.height_used * 100.0, b.off_screen, b.hand_over_back_row,
            );
        }
    }

    /// The projection matches Bevy's own camera math.
    #[test]
    fn projection_matches_bevy() {
        assert_eq!(FOV_Y, PerspectiveProjection::default().fov);
        let cam = legacy_pose(2);
        let centre = Projector::new(&cam, Vec2::new(1920.0, 1080.0)).project(Vec3::ZERO).unwrap();
        assert!((centre - Vec2::new(960.0, 540.0)).length() < 0.5, "{centre}");
    }

    /// The layout budget: run with `--nocapture` for the table.
    #[test]
    fn budget() {
        print("legacy fixed camera", &table(|n, _| legacy_pose(n)));
        let fitted = table(home_pose);
        print("fitted camera", &fitted);
        // Floors: what this layout measured when it landed, less a few
        // percent. Before the fit, the fixed camera gave 121 px (1v1) and
        // 80 px (pod) at 1920x1080, with the opponent's hand off the top
        // of the window, the viewer's hand over their own land row, and a
        // pod's far boards 3.5 cards wide.
        let floor = |seats: usize, vp: Vec2| match (seats, vp.x as u32, vp.y as u32) {
            (2, 1280, 720) => 72.0,
            (2, 1920, 1080) => 120.0,
            (2, 2560, 1080) => 120.0,
            (2, 3840, 2160) => 260.0,
            // Pods seat two to an edge. That table is bound by width, and
            // the HUD's action-button column is what binds it: without that
            // column these read 99/84 at 1920x1080 (the game log is next,
            // 92/87 without it).
            (_, 1280, 720) => 44.0,
            (_, 1920, 1080) => 83.0,
            (_, 2560, 1080) => 105.0,
            _ => 205.0,
        };
        for (seats, vp, b) in &fitted {
            assert!(
                b.viewer_card_px >= floor(*seats, *vp),
                "{seats} seats at {vp}: viewer cards shrank to {:.0} px", b.viewer_card_px,
            );
            // Only the viewer's hand runs off the window (its lower half).
            assert!(b.off_screen <= VIEWER_HAND, "{seats} seats at {vp}: {} cards off screen", b.off_screen);
            // The hand no longer covers the land row: under a tenth of the
            // row's area (it covered 55% at 1920x1080 before the hand moved).
            let row_area = BACK_GROUPS as f32 * b.viewer_card_px.powi(2) * CARD_HEIGHT / CARD_WIDTH;
            assert!(
                b.hand_over_back_row < 0.1 * row_area,
                "{seats} seats at {vp}: the hand covers {:.0} px² of the land row", b.hand_over_back_row,
            );
        }
    }

    /// The home pose keeps the whole board on screen and clear of the HUD.
    #[test]
    fn home_pose_clears_the_hud() {
        for seats in [2, 3, 4] {
            for (w, h) in VIEWPORTS {
                let vp = Vec2::new(w, h);
                let cards = fit_cards(&sample_board(seats, hand_zoom_for(h)));
                assert!(
                    clear(&cards, &home_pose(seats, vp), vp, &hud_rects(vp, seats)),
                    "{seats} seats at {vp}",
                );
            }
        }
    }
}
