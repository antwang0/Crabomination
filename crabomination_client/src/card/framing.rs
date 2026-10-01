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
    Spread, bf_card_transform, command_zone_card_transform, deck_position, exile_position,
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
/// `viewport` logical pixels, its HUD drawn at `ui_scale` (`UiScale`), with
/// its duel rows as far out as `spread` has grown them.
pub fn home_pose(n_seats: usize, viewport: Vec2, ui_scale: f32, spread: &Spread) -> Transform {
    fit_pose(n_seats, viewport, ui_scale, spread)
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

/// Right edge of the viewer's HUD panel in a viewport `w` UI px wide:
/// short of the opponent panel in the top-right corner. The panel's chips
/// wrap there (`game_ui::fit_player_hud_width`), so the rect [`hud_rects`]
/// frames the table against is the panel's real extent.
pub fn player_panel_width(w: f32) -> f32 {
    1090.0_f32.min(w - 470.0)
}

/// The persistent HUD panels the table must not run under, in logical px:
/// nominal sizes of the corner panels `game_ui::setup_game_hud` spawns.
/// The table is a trapezoid (its far edge is narrow), so it can sit between
/// the top corner panels while its wide near edge only has to clear the
/// action buttons.
///
/// The panels are laid out in UI px, which `UiScale` multiplies, so they are
/// placed in a viewport `ui_scale` times smaller and the rects scaled back
/// up: a UI at 150 % reserves panels half as large again.
pub fn hud_rects(viewport: Vec2, n_seats: usize, ui_scale: f32) -> [Rect; 6] {
    let w = viewport.x / ui_scale;
    // One 45 px row per opponent under a turn-order strip in a pod.
    let opp_panel_h = if n_seats > 2 { 30.0 + 45.0 * (n_seats - 1) as f32 } else { 70.0 };
    [
        // Player panel: the turn line and a row of stat chips, wrapping to a
        // second row at `player_panel_width`.
        Rect::new(0.0, 0.0, player_panel_width(w), 100.0),
        // The left control column under it: the phase chart, then the
        // action buttons (Pass, End Turn, Next Turn, Auto-pass, Undo with
        // its caption; the occasional ones are in the Esc menu). High on the
        // left edge, where the table is narrow in the view: in the near-left
        // corner these buttons cost a pod's cards 88 → 99 px at 1920x1080.
        Rect::new(0.0, 100.0, 146.0, 346.0),
        Rect::new(0.0, 346.0, 180.0, 562.0),
        // The prompt line under the buttons, four lines of it; a longer
        // prompt runs over the table while it's up. It costs a pod's cards
        // 47 → 46 px at 1280x720 and nothing at 1920x1080 or larger.
        Rect::new(0.0, 562.0, 180.0, 646.0),
        // Opponent panel, top-right, with the game log hanging under it.
        Rect::new(w - 460.0, 0.0, w, opp_panel_h),
        Rect::new(w - 292.0, opp_panel_h, w, opp_panel_h + 436.0),
    ]
    .map(|r| Rect { min: r.min * ui_scale, max: r.max * ui_scale })
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

/// The pose, looking down [`VIEW_DIRECTION`], that keeps the representative
/// board on screen and clear of the HUD and makes its hardest-to-read card
/// as large as it can be. For each look-at offset (a grid, in table units per
/// unit of distance) a bisection finds the closest distance that clears; the
/// offset whose smallest reference card (each seat's middle creature) comes
/// out largest wins. Taking the closest distance alone let the view slide
/// toward the viewer once the near corner had room, growing the viewer's
/// cards at the far seats' expense.
pub fn fit_pose(n_seats: usize, viewport: Vec2, ui_scale: f32, spread: &Spread) -> Transform {
    let back = VIEW_DIRECTION.normalize();
    let board = sample_board(n_seats, hand_zoom_for(viewport.y), spread);
    let cards = fit_cards(&board);
    let references: Vec<[Vec3; 8]> =
        board.iter().filter(|c| c.reference).map(|c| corners(&c.transform, 0.0)).collect();
    let hud = hud_rects(viewport, n_seats, ui_scale);
    let pose = |target: Vec3, d: f32| {
        Transform::from_translation(target + back * d).looking_at(target, Vec3::Y)
    };
    let smallest_card = |cam: &Transform| {
        let p = Projector::new(cam, viewport);
        references.iter().filter_map(|c| p.rect_of(c)).map(|r| r.width()).fold(f32::INFINITY, f32::min)
    };
    let mut best: Option<(f32, Transform)> = None;
    for i in -8..=8 {
        for j in -6..=6 {
            let nudge = Vec2::new(i as f32, j as f32) * 0.02;
            let target = |d: f32| Vec3::new(nudge.x * d, 0.0, nudge.y * d);
            let clears = |d: f32| clear(&cards, &pose(target(d), d), viewport, &hud);
            let (mut lo, mut hi) = (5.0_f32, 400.0_f32);
            if !clears(hi) {
                continue;
            }
            // 16 halvings of 395 units: six thousandths of a unit.
            for _ in 0..16 {
                let mid = 0.5 * (lo + hi);
                if clears(mid) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            let cam = pose(target(hi), hi);
            let score = smallest_card(&cam);
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, cam));
            }
        }
    }
    best.map_or_else(|| pose(Vec3::ZERO, 400.0), |(_, cam)| cam)
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

/// Where the 3-D stack hangs: a pile of camera-facing cards in the space
/// beside the table, the oldest item at the top and each newer one a step
/// lower and nearer the camera, so the item that resolves next is the one
/// wholly in view. Lying flat across the table's centre, where it was, a
/// spell covered the creatures it was aimed at.
#[derive(Clone, Copy, Debug)]
pub struct StackLane {
    /// World centre of the first (oldest) item's card.
    pub first: Vec3,
    /// World offset from one item to the next.
    pub step: Vec3,
    pub rotation: Quat,
    pub scale: f32,
    /// The window area (logical px) kept for the 2-D stack panel
    /// (`game_ui::place_stack_panel`), beside the area the pile was sized
    /// for three items deep: at its left, top-aligned with it, or — where
    /// that leaves the board clear — under it, right-aligned. Empty until a
    /// fit has placed it.
    pub panel: Rect,
}

impl Default for StackLane {
    /// The old placement, flat over the table's centre, until the first fit.
    fn default() -> Self {
        StackLane {
            first: Vec3::new(0.0, 0.8, 0.0),
            step: Vec3::X * (CARD_WIDTH + 0.5),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            scale: 1.0,
            panel: Rect::default(),
        }
    }
}

impl StackLane {
    /// The transform of stack item `idx` (0 = the bottom of the stack) in a
    /// stack `depth` items deep. Past [`LANE_ITEMS`] the steps tighten, to as
    /// little as [`LANE_MIN_STEP`] of one, so the item that resolves next
    /// still lands where the third would: the pile keeps to the window area
    /// the fit cleared for it, with the panel under it. Only a stack deeper
    /// than that runs on down.
    pub fn card(&self, idx: usize, depth: usize) -> Transform {
        let tighten = if depth as f32 > LANE_ITEMS {
            ((LANE_ITEMS - 1.0) / (depth as f32 - 1.0)).max(LANE_MIN_STEP)
        } else {
            1.0
        };
        Transform::from_translation(self.first + self.step * idx as f32 * tighten)
            .with_rotation(self.rotation)
            .with_scale(Vec3::splat(self.scale))
    }
}

/// Stack items the lane is sized to show before its pile runs on down.
const LANE_ITEMS: f32 = 3.0;
/// Share of a card's height each newer item sits below the last.
const LANE_STEP: f32 = 0.28;
/// The tightest a deep stack's steps get, as a share of [`LANE_STEP`]: each
/// older item still shows its name line.
const LANE_MIN_STEP: f32 = 0.4;
/// How far out the lane hangs, as a share of the camera's distance to the
/// table: well in front of every card on it, so nothing on the table draws
/// through the stack.
const LANE_DEPTH: f32 = 0.6;
/// The 2-D stack panel's width in UI px (`game_ui::setup_game_hud` pins it
/// to this), and its height at its tallest: [`STACK_PANEL_ROWS`] items, the
/// line counting the rest, and the footer.
pub const STACK_PANEL_WIDTH: f32 = 340.0;
const STACK_PANEL_HEIGHT: f32 = 215.0;
/// Items the panel lists, newest first (`game_ui::update_stack_panel`); a
/// line counts the older ones, whose names the pile still shows.
pub const STACK_PANEL_ROWS: usize = LANE_ITEMS as usize;
/// UI px between the panel and the lane.
pub const STACK_PANEL_GAP: f32 = 8.0;
/// Covered area (px²) that is a rounding sliver where the search's pixel
/// grid met a card edge, not a card the stack hides.
const SLIVER: f32 = 50.0;

fn area(r: Rect) -> f32 {
    if r.is_empty() { 0.0 } else { r.width() * r.height() }
}

/// The [`StackLane`] for the camera at `cam`. Of a grid of spots and card
/// sizes, the pile of three items that covers the least of the
/// representative board, inside the window and clear of the HUD; then one
/// whose 2-D panel clears the board, the HUD and the window's edge too; then
/// the larger card, the panel losing less, the spot further right, and the
/// one nearer the middle of the window's height. In a duel that is table the
/// board leaves empty, the panel beside the pile or under it; a pod's table
/// fills the window, and the pile takes its least-used corner, the panel
/// what it must. The panel was once placed off the pile alone, and lay over
/// the end of a duel's creature row (38,000 px² of the resting board at
/// 1920x1080).
pub fn stack_lane(n_seats: usize, viewport: Vec2, ui_scale: f32, cam: &Transform, spread: &Spread) -> StackLane {
    let projector = Projector::new(cam, viewport);
    let panel_size = Vec2::new(STACK_PANEL_WIDTH, STACK_PANEL_HEIGHT) * ui_scale;
    let gap = STACK_PANEL_GAP * ui_scale;
    // The lane is searched for in the right half of the window, its panel
    // reaching left of it: only the cards reaching into that can be covered.
    let reach = viewport.x * 0.5 - gap - panel_size.x;
    let board: Vec<Rect> = sample_board(n_seats, hand_zoom_for(viewport.y), spread)
        .iter()
        .filter_map(|c| projector.rect_of(&corners(&c.transform, c.height)))
        .filter(|r| r.max.x > reach)
        .collect();
    let hud = hud_rects(viewport, n_seats, ui_scale);
    let window = Rect::from_corners(Vec2::splat(MARGIN), viewport - MARGIN);
    let fits = |r: Rect| {
        window.contains(r.min) && window.contains(r.max) && hud.iter().all(|p| p.inflate(MARGIN).intersect(r).is_empty())
    };
    let covered = |r: Rect| -> f32 { board.iter().map(|c| area(c.intersect(r))).sum() };
    // What the panel costs, px²: the cards it covers, and what of it the HUD
    // hides or the window cuts off. Its height is for a deep stack, so on a
    // short window this only weighs against the cards, not rules it out.
    let lost = |r: Rect| -> f32 {
        let hidden: f32 = hud.iter().map(|p| area(p.inflate(MARGIN).intersect(r))).sum();
        covered(r) + hidden + area(r) - area(r.intersect(window))
    };
    let aspect = CARD_HEIGHT / CARD_WIDTH;
    // (pile's cover px², panel loses any, −width, panel's loss, −x,
    // off-centre) — lower is better.
    type Key = (f32, bool, f32, f32, f32, f32);
    let mut best: Option<(Key, Rect, Rect, f32)> = None;
    for width in [170.0, 150.0, 130.0, 110.0, 92.0, 76.0].map(|w| w * ui_scale) {
        let height = width * aspect * (1.0 + LANE_STEP * (LANE_ITEMS - 1.0));
        let mut x = viewport.x - MARGIN - width;
        while x >= viewport.x * 0.5 {
            let mut y = MARGIN;
            while y + height <= viewport.y - MARGIN {
                let lane = Rect::new(x, y, x + width, y + height);
                if fits(lane) {
                    let left = Rect::from_corners(Vec2::new(x - gap - panel_size.x, y), Vec2::new(x - gap, y + panel_size.y));
                    let below = Rect::from_corners(
                        Vec2::new(lane.max.x - panel_size.x, lane.max.y + gap),
                        Vec2::new(lane.max.x, lane.max.y + gap + panel_size.y),
                    );
                    // Under the pile, the panel hangs over the table's near
                    // side; where it would cover cards there it covered the
                    // most of them (12,000 → 44,000 px² at 1280x720), so it
                    // goes there only to clear the board.
                    let below = (lost(below) <= SLIVER).then_some(below);
                    for panel in std::iter::once(left).chain(below) {
                        let loss = lost(panel);
                        let off_centre = (lane.center().y - viewport.y * 0.5).abs();
                        let key = (covered(lane), loss > SLIVER, -width, loss, -x, off_centre);
                        if best.is_none_or(|(k, ..)| key < k) {
                            best = Some((key, lane, panel, width));
                        }
                    }
                }
                y += 32.0;
            }
            x -= 32.0;
        }
    }
    let Some((_, lane, panel, width)) = best else { return StackLane::default() };

    // The pile faces the camera at a fixed depth along its forward axis, so a
    // pixel offset is a world offset in the camera's plane.
    let forward = cam.forward();
    let depth = LANE_DEPTH * (cam.translation.y / -forward.y);
    let tan = (FOV_Y * 0.5).tan();
    let focal = viewport.y * 0.5 / tan;
    let at = |px: Vec2| {
        let ndc = Vec2::new(px.x / viewport.x * 2.0 - 1.0, 1.0 - px.y / viewport.y * 2.0);
        let local = Vec3::new(ndc.x * tan * viewport.x / viewport.y, ndc.y * tan, -1.0) * depth;
        cam.translation + cam.rotation * local
    };
    let card_px = Vec2::new(width, width * aspect);
    let first = at(lane.min + card_px * 0.5);
    let step_px = card_px.y * LANE_STEP;
    StackLane {
        first,
        // A step down the screen, and a hair nearer the camera so the newer
        // item draws over the older one.
        step: cam.rotation * Vec3::new(0.0, -step_px * depth / focal, 0.0) - *forward * 0.02,
        rotation: cam.rotation,
        scale: width * depth / focal / CARD_WIDTH,
        panel,
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
/// five in each opponent's. A duel whose creature rows have grown past six
/// groups is sampled with the rows it has ([`Spread`]).
const CREATURES: usize = 6;
const TAPPED: [usize; 2] = [1, 4];
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
    /// on-screen size the fit maximises (the smallest seat's) and the budget
    /// reports for its seat.
    pub reference: bool,
}

/// Every card of the representative board for `n_seats`, from seat 0's chair,
/// with each duel creature row at least as full as `spread` says it is.
pub fn sample_board(n_seats: usize, hand_zoom: f32, spread: &Spread) -> Vec<Placed> {
    let flat = |p: Vec3| {
        Transform::from_translation(p).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
    };
    let card = |seat, role, transform| Placed { seat, role, transform, height: 0.0, reference: false };
    let mut out = Vec::new();
    for seat in 0..n_seats {
        let creatures = CREATURES.max(spread.creatures(seat));
        // The middle untapped creature of the row: slot 2 of six.
        let reference = if creatures == CREATURES { 2 } else { creatures / 2 };
        for slot in 0..creatures {
            let tapped = TAPPED.contains(&slot) && slot != reference;
            let t = bf_card_transform(seat, 0, n_seats, slot, creatures, false, tapped);
            out.push(Placed { reference: slot == reference, ..card(seat, Role::Creature, t) });
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
        out.push(card(seat, Role::Pile, flat(graveyard_position(seat, 0, n_seats, spread))));
        // A commander pair, both cards — a 1v1 Commander game's too.
        for slot in 0..2 {
            out.push(card(seat, Role::Pile, command_zone_card_transform(seat, 0, n_seats, slot, 2, spread)));
        }
    }
    out.push(card(1, Role::Pile, flat(exile_position(n_seats, spread))));
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
    measure_spread(n_seats, viewport, cam, &Spread::default())
}

/// [`measure`] for a board whose duel creature rows are as `spread` has them.
#[cfg(test)]
pub fn measure_spread(n_seats: usize, viewport: Vec2, cam: &Transform, spread: &Spread) -> Budget {
    let board = sample_board(n_seats, hand_zoom_for(viewport.y), spread);
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

    /// The UI scale a window of this size gets at the default (Auto) size.
    fn auto(vp: Vec2) -> f32 {
        crate::theme::ui_scale_for(vp, 0)
    }

    fn table(pose: impl Fn(usize, Vec2) -> Transform) -> Vec<(usize, Vec2, Budget)> {
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
        println!("seats  window      viewer px  opp px  width  height  off-screen  hand∩lands sq px");
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

    /// The stack lane and its 2-D panel land in the window, clear of the HUD,
    /// and — in a duel, whose table leaves room beside it — clear of every
    /// card on the board, crowded rows included: measured by projecting the
    /// lane's own card transforms, so the pixel search and the world placement
    /// are checked against each other. A stack deeper than the lane was sized
    /// for keeps to the same area.
    #[test]
    fn the_stack_lane_hangs_clear_of_the_board() {
        for seats in [2, 4] {
            let spreads: &[usize] = if seats == 2 { &[6, 8, 10] } else { &[6] };
            for (w, h) in VIEWPORTS {
                for &groups in spreads {
                    let vp = Vec2::new(w, h);
                    let scale = auto(vp);
                    let spread = Spread::uniform(groups);
                    let cam = home_pose(seats, vp, scale, &spread);
                    let lane = stack_lane(seats, vp, scale, &cam, &spread);
                    let p = Projector::new(&cam, vp);
                    let at = format!("{seats} seats, {groups} groups, {vp}");
                    let pile = |depth: usize| -> Vec<Rect> {
                        (0..depth).map(|i| p.card_rect(&lane.card(i, depth), 0.0).expect("in front of the camera")).collect()
                    };
                    let cards = pile(3);
                    // Camera-facing: its projected width is the size the search chose.
                    assert!(cards[0].width() >= 70.0 * scale, "{at}: {}px", cards[0].width());
                    // Each newer item hangs a hair nearer the camera, a
                    // fraction of a pixel larger: the three-deep pile's own
                    // footprint, not the searched rect, is the bound.
                    let footprint = cards.iter().fold(cards[0], |acc, r| acc.union(*r));
                    for depth in 4..=6 {
                        for r in pile(depth) {
                            let bound = footprint.inflate(0.5);
                            assert!(bound.contains(r.min) && bound.contains(r.max),
                                "{at}: a {depth}-deep stack runs out of its lane ({r:?} in {footprint:?})");
                        }
                    }
                    let panel = lane.panel;
                    let size = Vec2::new(STACK_PANEL_WIDTH, STACK_PANEL_HEIGHT) * scale;
                    let gap = STACK_PANEL_GAP * scale;
                    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
                    assert!((panel.size() - size).length() < 0.01, "{at}: {panel:?}");
                    // The searched rect, from the oldest card, which lands
                    // exactly where the search put it.
                    let first = cards[0];
                    let bottom = first.min.y + first.height() * (1.0 + LANE_STEP * (LANE_ITEMS - 1.0));
                    let left = near(panel.max.x, first.min.x - gap) && near(panel.min.y, first.min.y);
                    let below = near(panel.max.x, first.max.x) && near(panel.min.y, bottom + gap);
                    assert!(left || below, "{at}: the panel {panel:?} isn't beside the pile {footprint:?}");
                    let window = Rect::from_corners(Vec2::ZERO, vp);
                    let hud = hud_rects(vp, seats, scale);
                    for r in &cards {
                        assert!(window.contains(r.min) && window.contains(r.max), "{at}: {r:?} off screen");
                        assert!(hud.iter().all(|h| h.intersect(*r).is_empty()), "{at}: {r:?} under the HUD");
                    }
                    let board: Vec<Rect> = sample_board(seats, hand_zoom_for(vp.y), &spread)
                        .iter()
                        .filter_map(|c| p.card_rect(&c.transform, c.height))
                        .collect();
                    let covered = |r: &Rect| -> f32 { board.iter().map(|c| area(c.intersect(*r))).sum() };
                    let under_hud: f32 = hud.iter().map(|h| area(h.intersect(panel))).sum();
                    let cut_off = area(panel) - area(panel.intersect(window));
                    assert!(under_hud == 0.0, "{at}: the stack panel {panel:?} runs under the HUD");
                    if seats == 2 {
                        let on_cards: f32 = cards.iter().map(covered).sum();
                        assert!(on_cards < SLIVER, "{at}: the stack covers {on_cards:.0} sq px of the board");
                        let on_cards = covered(&panel);
                        if vp.y >= 1080.0 {
                            assert!(on_cards + cut_off < SLIVER, "{at}: the stack panel loses {:.0} sq px", on_cards + cut_off);
                        } else {
                            // A 1280x720 duel has no room for the panel. It
                            // covers at most what it measured when this
                            // landed (18,763 px² at six groups); placed off
                            // the pile alone it covered 16,736 there, and at
                            // ten groups 12,395 and ran under the player
                            // panel. The bottom of its room, which a
                            // four-deep stack's count line fills, is off the
                            // window.
                            assert!(on_cards < 19_000.0, "{at}: the stack panel covers {on_cards:.0} sq px of the board");
                        }
                    }
                }
            }
        }
    }

    /// The layout budget: run with `--nocapture` for the table.
    #[test]
    fn budget() {
        print("legacy fixed camera", &table(|n, _| legacy_pose(n)));
        let fitted = table(|n, vp| home_pose(n, vp, auto(vp), &Spread::default()));
        print("fitted camera", &fitted);
        // Floors: what this layout measured when it landed, less a few
        // percent. Before the fit, the fixed camera gave 121 px (1v1) and
        // 80 px (pod) at 1920x1080, with the opponent's hand off the top
        // of the window, the viewer's hand over their own land row, and a
        // pod's far boards 3.5 cards wide.
        let floor = |seats: usize, vp: Vec2| match (seats, vp.x as u32, vp.y as u32) {
            (2, 1280, 720) => 73.0,
            (2, 1920, 1080) => 125.0,
            (2, 2560, 1080) => 131.0,
            // 3840x2160 draws the UI at 200 % (Auto), so its layout is
            // 1920x1080's doubled and so are its floors. At 100 % it was
            // 279 / 212 px, the HUD a quarter of the size it is at 1080p.
            (2, 3840, 2160) => 250.0,
            // Pods seat two to an edge, a table bound by width. With the
            // action buttons moved up under the phase chart (from the
            // near-left corner, where they cost 88 → 99 px here), the game
            // log is what binds a pod next (96/89 px without it) and the
            // phase chart a 1v1 (136/124 px).
            (_, 1280, 720) => 45.0,
            (_, 1920, 1080) => 95.0,
            (_, 2560, 1080) => 102.0,
            _ => 190.0,
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
                "{seats} seats at {vp}: the hand covers {:.0} sq px of the land row", b.hand_over_back_row,
            );
        }
    }

    /// A duel whose creature rows have grown keeps them on screen and clear
    /// of the HUD: the camera refits for them, pulling back only as far as it
    /// has to (run with `--nocapture` for the sizes). Six groups a seat is the
    /// resting board, which the rows keep the land row's width for.
    #[test]
    fn a_crowded_duel_stays_on_screen() {
        println!("crowded duel: viewer px / opp px per creature groups a seat");
        println!("window       6         8         10        12");
        for (w, h) in VIEWPORTS {
            let vp = Vec2::new(w, h);
            let mut line = format!("{w:>4}x{h:<4}  ");
            for groups in [6usize, 8, 10, 12] {
                let spread = Spread::uniform(groups);
                let cam = home_pose(2, vp, auto(vp), &spread);
                let cards = fit_cards(&sample_board(2, hand_zoom_for(h), &spread));
                assert!(clear(&cards, &cam, vp, &hud_rects(vp, 2, auto(vp))), "{groups} groups at {vp}");
                let b = measure_spread(2, vp, &cam, &spread);
                line.push_str(&format!("{:>4.0}/{:<4.0}  ", b.viewer_card_px, b.opp_card_px));
            }
            println!("{line}");
        }
    }

    /// The home pose keeps the whole board on screen and clear of the HUD.
    #[test]
    fn home_pose_clears_the_hud() {
        for seats in [2, 3, 4] {
            for (w, h) in VIEWPORTS {
                let vp = Vec2::new(w, h);
                let cards = fit_cards(&sample_board(seats, hand_zoom_for(h), &Spread::default()));
                assert!(
                    clear(&cards, &home_pose(seats, vp, auto(vp), &Spread::default()), vp, &hud_rects(vp, seats, auto(vp))),
                    "{seats} seats at {vp}",
                );
            }
        }
    }
}
