//! Overlays over the battlefield: the combat, targeting and stack arrows
//! (pushed to `systems::arrows`, which draws them as shaded geometry), and the
//! gizmo rings, tethers and seat outline. The attacker and blocker marks are
//! chips (`systems::combat_badge`).

use std::collections::{HashMap, HashSet};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crabomination::card::CardId;
use crabomination::game::{AttackTarget, Target, TurnStep};
use crabomination::net::StackItemView;

use crate::card::{
    BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, CardHovered, GameCardId, HandCard, PlayerTargetZone,
    StackCard,
};
use crate::card::cover::{CardCover, CoverQuery};
use crate::card::layout::player_hand_anchor;
use crate::game::{AttackingState, BlockingState, TargetingState};
use crate::net_plugin::CurrentView;
use crate::systems::arrows::{ArrowKey, Arrows, Cue, End};
use crate::systems::game_ui::PlayerHudPanel;
use crate::systems::game_ui::table_awareness::seat_color;
use crate::MainCamera;

/// Scale a colour into the HDR range (linear space, alpha preserved) so it
/// exceeds the bloom prefilter threshold (~1.0, see `RenderQuality::bloom`)
/// and reads as emitted *light* rather than a flat line on HDR tiers. On Low
/// (no bloom) it just draws a brighter line, which is still perfectly legible.
fn glow(color: Color, intensity: f32) -> Color {
    let l = color.to_linear();
    LinearRgba::new(l.red * intensity, l.green * intensity, l.blue * intensity, l.alpha).into()
}

/// Standard glow strength for the gameplay-cue overlays. Picked so a fully
/// saturated cue (yellow/green/orange) lands comfortably past the bloom
/// threshold without becoming a fireball.
const CUE_GLOW: f32 = 2.6;

/// Where arrows meet the battlefield's cards. Cards overlap on a busy board
/// — a creature row wraps, a card lies over the one it is attached to — and
/// an arrow aimed at a covered card's centre landed on the card lying over
/// it, so a card is met in the middle of the part of it that shows
/// (`CardCover::visible_centre`), or at its combat chip.
#[derive(SystemParam)]
pub struct BoardSpots<'w, 's> {
    cards: Query<'w, 's, (Entity, &'static Transform, &'static GameCardId), With<BattlefieldCard>>,
    cover_cards: CoverQuery<'w, 's>,
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<MainCamera>>,
}

impl<'w, 's> BoardSpots<'w, 's> {
    /// This frame's spots.
    pub fn spots(&self) -> Spots<'_, 'w, 's> {
        let cover = self.camera.single().ok().map(|(_, eye)| CardCover::new(eye, &self.cover_cards));
        Spots { board: self, cover }
    }
}

/// [`BoardSpots`] with this frame's [`CardCover`].
pub struct Spots<'a, 'w, 's> {
    board: &'a BoardSpots<'w, 's>,
    cover: Option<CardCover>,
}

impl Spots<'_, '_, '_> {
    /// A battlefield card's entity and transform. Reads the local
    /// `Transform` (battlefield cards have no parent), so a card's combat
    /// lunge this frame is already in it.
    fn card(&self, id: CardId) -> Option<(Entity, GlobalTransform)> {
        self.board.cards.iter().find(|(_, _, g)| g.0 == id).map(|(e, t, _)| (e, GlobalTransform::from(*t)))
    }

    /// The middle of the part of the card that shows, `lift` above it.
    pub fn centre(&self, id: CardId, lift: f32) -> Option<Vec3> {
        let (e, t) = self.card(id)?;
        let local = self.cover.as_ref().map_or(Vec3::ZERO, |c| c.visible_centre(e, &t));
        Some(t.transform_point(local) + Vec3::Y * lift)
    }

    /// The middle of the part of the card that shows, `lift` off its face;
    /// the face's normal; and the card's scale.
    pub fn face(&self, id: CardId, lift: f32) -> Option<(Vec3, Vec3, f32)> {
        let (e, t) = self.card(id)?;
        let local = self.cover.as_ref().map_or(Vec3::ZERO, |c| c.visible_centre(e, &t));
        let normal = t.rotation() * Vec3::Z;
        Some((t.transform_point(local) + normal * lift, normal, t.scale().x))
    }

    /// Where the card's combat chip sits (`combat_badge::chip_local`),
    /// `lift` above it, and how far short of it an arrow stops to meet the
    /// chip's rim. The middle of what shows, with nothing to stop short of,
    /// when the chip has nowhere to go.
    pub fn chip(&self, id: CardId, lift: f32) -> Option<(Vec3, f32)> {
        let (e, t) = self.card(id)?;
        let (Some(cover), Ok((camera, eye))) = (&self.cover, self.board.camera.single()) else {
            return self.centre(id, lift).map(|at| (at, 0.0));
        };
        let project = |local: Vec3| camera.world_to_viewport(eye, t.transform_point(local)).ok();
        match crate::systems::combat_badge::chip_local(project, |local| cover.hides_local(e, &t, local)) {
            Some(local) => {
                let clearance = crate::systems::combat_badge::CHIP_CLEARANCE * CARD_WIDTH * t.scale().x;
                Some((t.transform_point(local) + Vec3::Y * lift, clearance))
            }
            None => self.centre(id, lift).map(|at| (at, 0.0)),
        }
    }
}

/// Faint tether lines linking each attached Aura / Equipment /
/// Fortification card to its host permanent, so the physical association
/// is readable on the board (previously tooltip-only).
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AttachmentGizmos;

/// Gold outline around the active player's board column, so "whose turn
/// is it" reads on the 3-D table itself — not just the HUD roster.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ActiveSeatGizmos;

/// Draw a glowing rectangle at table level around the active player's
/// column. Skipped when it's the viewer's own turn in 1v1 (the phase bar
/// already carries that signal); in a pod it always draws, since the
/// seat → column mapping is exactly what the eye has to find.
pub fn draw_active_seat_glow(
    view: Res<CurrentView>,
    time: Res<Time>,
    mut gizmos: Gizmos<ActiveSeatGizmos>,
) {
    let Some(cv) = &view.0 else { return };
    if cv.game_over.is_some() {
        return;
    }
    let n = cv.players.len();
    if n <= 2 && cv.active_player == cv.your_seat {
        return;
    }
    let (min, max) =
        crate::card::layout::seat_board_outline(cv.active_player, cv.your_seat, n);
    // Gentle breathing so the outline reads as "live" without pulsing
    // hard enough to pull the eye during someone else's long turn.
    let breathe = 0.75 + 0.25 * (time.elapsed_secs() * 1.6).sin();
    let color = glow(Color::srgba(0.95, 0.78, 0.30, 0.85), CUE_GLOW * breathe);
    let y = 0.04;
    let corners = [
        Vec3::new(min.x, y, min.z),
        Vec3::new(max.x, y, min.z),
        Vec3::new(max.x, y, max.z),
        Vec3::new(min.x, y, max.z),
    ];
    for i in 0..4 {
        gizmos.line(corners[i], corners[(i + 1) % 4], color);
    }
}

/// Draw a low-alpha tether from every attached permanent to its host. The
/// tether brightens when either end is hovered, so "what's enchanting
/// this?" is answerable by pointing at a card. Skipped for co-located
/// pairs, where the line would collapse to a dot.
pub fn draw_attachment_tethers(
    view: Res<CurrentView>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    hovered: Query<&GameCardId, (With<BattlefieldCard>, With<CardHovered>)>,
    mut gizmos: Gizmos<AttachmentGizmos>,
) {
    let Some(cv) = &view.0 else { return };

    let mut positions: HashMap<CardId, Vec3> = HashMap::new();
    for (transform, gid) in &bf_cards {
        // Slightly above the table so the line doesn't z-fight card faces.
        positions.insert(gid.0, transform.translation + Vec3::Y * 0.12);
    }
    let hovered: HashSet<CardId> = hovered.iter().map(|g| g.0).collect();

    for p in &cv.battlefield {
        let Some(host) = p.attached_to else { continue };
        let (Some(&from), Some(&to)) = (positions.get(&p.id), positions.get(&host)) else {
            continue;
        };
        if from.distance_squared(to) < 0.25 {
            continue;
        }
        let emphasized = hovered.contains(&p.id) || hovered.contains(&host);
        // Violet — distinct from the yellow/green/orange combat-cue
        // vocabulary; near-invisible at rest, glowing when hovered.
        let alpha = if emphasized { 0.9 } else { 0.28 };
        let intensity = if emphasized { CUE_GLOW } else { 1.0 };
        let color = glow(Color::srgba(0.72, 0.5, 0.95, alpha), intensity);
        // Lift the midpoint so the tether reads as a cord draped between
        // the two cards rather than a targeting arrow.
        let mid = (from + to) * 0.5 + Vec3::Y * 0.6;
        gizmos.line(from, mid, color);
        gizmos.line(mid, to, color);
    }
}

/// Project the cursor onto a horizontal plane at `plane_y` through the main
/// camera, for use as the live endpoint of the target-drag arrow. `None` when
/// there's no cursor in the window or the ray runs parallel to / away from the
/// plane.
fn cursor_on_plane(
    plane_y: f32,
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec3> {
    let cursor = windows.single().ok()?.cursor_position()?;
    let (camera, cam_xform) = camera_q.single().ok()?;
    let ray = camera.viewport_to_world(cam_xform, cursor).ok()?;
    let denom = ray.direction.y;
    if denom.abs() < 1e-5 {
        return None;
    }
    let t = (plane_y - ray.origin.y) / denom;
    (t >= 0.0).then(|| ray.origin + ray.direction * t)
}

/// Draw the live target-selection arrow: anchored at whatever is doing the
/// targeting (the spell in hand, the ability/equip source on the battlefield,
/// or — for stack-driven decision targets with no on-table source — the
/// viewer's own side of the table) and pointing at the cursor. When the cursor
/// is over a battlefield card or a player zone, the head snaps to it, matching
/// exactly what a click would resolve to.
#[allow(clippy::too_many_arguments)]
pub fn draw_target_arrow(
    targeting: Res<TargetingState>,
    view: Res<CurrentView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    board: BoardSpots,
    hand_cards: Query<(&Transform, &GameCardId), With<HandCard>>,
    hovered_bf: Query<&GameCardId, (With<CardHovered>, With<BattlefieldCard>)>,
    hovered_zone: Query<&PlayerTargetZone, With<CardHovered>>,
    chips: Query<(&Interaction, &PlayerHudPanel)>,
    mut arrows: ResMut<Arrows>,
) {
    if !targeting.active {
        return;
    }
    let Some(cv) = &view.0 else { return };
    let (viewer, n_seats) = (cv.your_seat, cv.players.len());

    // Held a touch off the table so the arrow floats clearly above the cards.
    const ARROW_Y: f32 = 0.4;

    let spots = board.spots();
    let bf_pos = |id: CardId| spots.centre(id, 0.0);
    let hand_pos =
        |id: CardId| hand_cards.iter().find(|(_, g)| g.0 == id).map(|(t, _)| t.translation);

    let mut source = targeting
        .pending_card_id
        .and_then(hand_pos)
        .or_else(|| targeting.pending_ability_source.and_then(bf_pos))
        .or_else(|| targeting.pending_equip_source.and_then(bf_pos))
        .unwrap_or_else(|| player_hand_anchor(viewer, viewer, n_seats));
    source.y = ARROW_Y;

    // A player can be targeted via their 3-D table zone *or* their 2-D HUD
    // chip (top-corner panels). Snap to the seat's table anchor for either so
    // the head doesn't dangle over the HUD when the cursor leaves the board.
    let hovered_chip_seat = chips.iter().find_map(|(i, panel)| {
        (matches!(i, Interaction::Hovered | Interaction::Pressed) && panel.seat < n_seats)
            .then_some(panel.seat)
    });

    let end = if let Ok(gid) = hovered_bf.single() {
        bf_pos(gid.0)
    } else if let Ok(zone) = hovered_zone.single() {
        Some(player_hand_anchor(zone.0, viewer, n_seats))
    } else if let Some(seat) = hovered_chip_seat {
        Some(player_hand_anchor(seat, viewer, n_seats))
    } else {
        cursor_on_plane(ARROW_Y, &windows, &camera_q)
    };
    let Some(mut end) = end else { return };
    end.y = ARROW_Y;

    // Skip degenerate arrows (cursor sitting on the source) so we don't draw a
    // zero-length stub with a stray arrowhead.
    if source.distance(end) < 0.3 {
        return;
    }

    let key = ArrowKey::new(Cue::TargetDrag, End::Cursor, End::Cursor);
    arrows.arrow(key, source, end, Color::srgb(1.0, 0.84, 0.1), 1.0);
}

/// A pulsing ring on each legal target of the spell or ability being aimed,
/// on the part of the card that shows.
pub fn draw_legal_target_rings(
    view: Res<CurrentView>,
    targeting: Res<crate::game::TargetingState>,
    legal: Res<crate::game::LegalTargets>,
    time: Res<Time>,
    board: BoardSpots,
    // A counterspell's legal targets are spells on the stack, which carry
    // `StackCard` rather than `BattlefieldCard` — ringing only the
    // battlefield left "counter target spell" with no visible target.
    stack_cards: Query<(&Transform, &GameCardId), (With<StackCard>, Without<BattlefieldCard>)>,
    mut arrows: ResMut<Arrows>,
) {
    // Pulse rings whenever any targeting flow has populated `legal`.
    // Cast-time targeting uses the catalog evaluator in
    // `legal_target_filter::enumerate_for_cast`; decision-driven targets
    // come from the engine's `Decision::ChooseTarget.legal`. Empty
    // `legal.permanents` (callers that opted out, or filters the client
    // evaluator can't pin down) draws nothing — the cursor falls back to
    // the older "highlight everything clickable" behaviour.
    if !targeting.active || legal.permanents.is_empty() {
        return;
    }
    if view.0.is_none() {
        return;
    }
    // The ring breathes in and out.
    let pulse = 0.55 + 0.45 * (time.elapsed_secs() * 4.0).sin().abs();
    let colour = Color::srgba(1.0, 0.84, 0.1, pulse);
    let spots = board.spots();
    for &id in &legal.permanents {
        let key = ArrowKey::new(Cue::LegalTarget, End::Card(id), End::Card(id));
        if let Some((at, normal, scale)) = spots.face(id, 0.2) {
            arrows.ring(key, at, 1.2 * scale, normal, colour);
        } else if let Some((t, _)) = stack_cards.iter().find(|(_, g)| g.0 == id) {
            // A spell in the stack lane stands facing the camera; its ring
            // lies on its face.
            let normal = t.rotation * Vec3::Z;
            arrows.ring(key, t.translation + normal * 0.2, 1.2 * t.scale.x, normal, colour);
        }
    }
}

/// Blocks as arrows from each blocker to the attacker it blocks. Declared
/// blocks draw for every seat for as long as combat lasts; while the viewer
/// picks blocks, their plan draws too, and the blocker they have picked up
/// reaches toward each attacker it could still take. The attackers' and
/// blockers' own marks are chips (`combat_badge`).
pub fn draw_block_arrows(
    view: Res<CurrentView>,
    blocking: Res<BlockingState>,
    board: BoardSpots,
    mut arrows: ResMut<Arrows>,
) {
    let Some(cv) = &view.0 else { return };
    // Declared blocks are drawn for *every* seat, for as long as combat lasts.
    // Previously the whole system bailed unless the viewer was the defender in
    // the Declare Blockers step, and it drew only `blocking.assignments` — the
    // client's own in-progress selection, which the pass handler drains the
    // moment blocks are submitted. So the attacker never saw who was blocking
    // what, and the defender lost the picture exactly when it mattered: the
    // combat-trick window and the damage step. `blocking_attackers` is the
    // server's declared assignment and is on the wire for every permanent.
    let in_combat = matches!(
        cv.step,
        TurnStep::DeclareBlockers | TurnStep::FirstStrikeDamage | TurnStep::CombatDamage
    );
    if !in_combat {
        return;
    }
    let picking_blocks = cv.step == TurnStep::DeclareBlockers && cv.declares_blocks(cv.your_seat);

    // Chip to chip: the blocker's 🛡 to the attacker's ⚔.
    let spots = board.spots();
    let mut arrow = |cue: Cue, blocker: CardId, attacker: CardId, colour: Color, weight: f32| {
        if let (Some((from, clear_from)), Some((to, clear_to))) = (spots.chip(blocker, 0.15), spots.chip(attacker, 0.15)) {
            let key = ArrowKey::new(cue, End::Card(blocker), End::Card(attacker));
            arrows.arrow_trimmed(key, from, to, colour, weight, [clear_from, clear_to]);
        }
    };

    // Declared blocks, straight from the server. One arrow per (blocker,
    // attacker) pair, so a multi-block blocker fans out to each attacker it
    // was assigned to and a gang block shows every arm.
    for perm in &cv.battlefield {
        for &attacker in &perm.blocking_attackers {
            arrow(Cue::Block, perm.id, attacker, Color::srgb(0.15, 0.85, 1.0), 0.9);
        }
    }

    if !picking_blocks {
        return;
    }
    for &(blocker, attacker) in &blocking.assignments {
        arrow(Cue::BlockPlan, blocker, attacker, Color::srgb(0.0, 0.9, 0.3), 0.9);
    }
    // The picked-up blocker's options: thin, so a board of attackers
    // doesn't bury the plan under candidates.
    if let Some(blocker) = blocking.selected_blocker {
        for attacker in cv.battlefield.iter().filter(|p| p.attacking).map(|p| p.id) {
            if !blocking.assignments.iter().any(|(_, a)| *a == attacker) {
                arrow(Cue::BlockCandidate, blocker, attacker, Color::srgba(1.0, 0.84, 0.1, 0.85), 0.6);
            }
        }
    }
}

/// Arrows from each spell or ability on the stack to its targets. A spell
/// hangs in the stack lane beside the table (`framing::stack_lane`), facing
/// the camera; its arrow leaves from the card's bottom edge, in front of the
/// face, so the card doesn't hide where the arrow starts.
pub fn draw_stack_arrows(
    view: Res<CurrentView>,
    stack_cards: Query<(&Transform, &GameCardId), With<StackCard>>,
    board: BoardSpots,
    mut arrows: ResMut<Arrows>,
) {
    let Some(cv) = &view.0 else { return };
    if cv.stack.is_empty() { return; }

    // (where an arrow from it starts, where one at it ends)
    let mut stack_pos: HashMap<CardId, (Vec3, Vec3)> = HashMap::new();
    for (t, gid) in &stack_cards {
        let front = t.rotation * Vec3::Z * 0.1;
        let bottom = t.transform_point(Vec3::new(0.0, -CARD_HEIGHT * 0.42, 0.0));
        stack_pos.insert(gid.0, (bottom + front, t.translation + front * 3.0));
    }
    let spots = board.spots();

    let viewer = cv.your_seat;
    let n_seats = cv.players.len();
    let orange = Color::srgb(1.0, 0.6, 0.05);

    for item in &cv.stack {
        let StackItemView::Known(known) = item else { continue };
        let source_id = known.source;

        let from = stack_pos.get(&source_id).map(|p| p.0).or_else(|| spots.centre(source_id, 0.2));
        // A target can be a battlefield permanent, a player zone, or
        // another spell on the stack (counter magic).
        let resolve = |target: &Target| match target {
            Target::Permanent(id) => spots
                .centre(*id, 0.2)
                .or_else(|| stack_pos.get(id).map(|p| p.1))
                .map(|at| (End::Card(*id), at)),
            Target::Player(idx) => (*idx < n_seats).then(|| {
                let mut p = player_hand_anchor(*idx, viewer, n_seats);
                p.y = 0.2;
                (End::Seat(*idx), p)
            }),
        };

        let Some(from) = from else { continue };
        let source = End::Card(source_id);
        if let Some((end, to)) = known.target.as_ref().and_then(resolve) {
            arrows.arrow(ArrowKey::new(Cue::Stack, source, end), from, to, orange, 1.0);
        }
        // Slots 1+ — every extra target of a multi-target spell, a little
        // thinner so the primary stays legible when a spell fans out
        // (Forked Bolt, Cone of Flame).
        for t in &known.additional_targets {
            if let Some((end, to)) = resolve(t) {
                let key = ArrowKey::new(Cue::StackExtra, source, end);
                arrows.arrow(key, from, to, Color::srgba(1.0, 0.45, 0.05, 0.8), 0.75);
            }
        }
    }
}

/// Where an attack aimed at `target` lands: the defending player's spot at
/// the edge of their board, or the planeswalker or battle's card.
fn attack_end(target: AttackTarget, spots: &Spots, viewer: usize, n_seats: usize) -> Option<(End, Vec3)> {
    match target {
        AttackTarget::Player(seat) => {
            let mut p = player_hand_anchor(seat, viewer, n_seats);
            p.y = 0.3;
            Some((End::Seat(seat), p))
        }
        // A Siege battle is attacked like a planeswalker — the arrow points
        // at its battlefield position (CR 508.1).
        AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => {
            spots.centre(id, 0.18).map(|p| (End::Card(id), p))
        }
    }
}

/// Render the viewer's in-progress attack plan: an arrow from each chosen
/// attacker to its target (player disc or opp planeswalker card), and a ring
/// on each player the plan aims at. Active only during the viewer's own
/// DeclareAttackers step with priority — outside that window the plan is
/// stale and the resource is cleared by the input handler.
pub fn draw_attack_plan_arrows(
    view: Res<CurrentView>,
    attacking: Res<AttackingState>,
    board: BoardSpots,
    mut arrows: ResMut<Arrows>,
) {
    let Some(cv) = &view.0 else { return };
    if cv.step != TurnStep::DeclareAttackers
        || cv.active_player != cv.your_seat
        || cv.priority != cv.your_seat
    {
        return;
    }
    if attacking.plan.is_empty() {
        return;
    }

    let viewer = cv.your_seat;
    let n_seats = cv.players.len();
    let yellow = Color::srgb(1.0, 0.84, 0.1);
    let spots = board.spots();

    for (attacker, target) in &attacking.plan {
        let Some((from, clear)) = spots.chip(*attacker, 0.18) else {
            continue;
        };
        // In a pod each arrow takes its defending seat's identity colour
        // (the HUD avatar's), so "which opponent is this aimed at" reads off
        // the arrow; 1v1 keeps the classic yellow. The pending attacker (next
        // defender click rebinds it) draws at half alpha.
        let base = match crate::systems::game_ui::table_awareness::plan_defender(cv, *target) {
            Some(d) if n_seats > 2 => seat_color(d),
            _ => yellow,
        };
        let colour = if attacking.last_added == Some(*attacker) { base.with_alpha(0.5) } else { base };
        // The attacker itself is marked by its combat chip (`combat_badge`,
        // which includes planned attackers), so here we only draw the arrow
        // to its chosen defender.
        let Some((end, to)) = attack_end(*target, &spots, viewer, n_seats) else { continue };
        let key = ArrowKey::new(Cue::AttackPlan, End::Card(*attacker), end);
        arrows.arrow_trimmed(key, from, to, colour, 1.0, [clear, 0.0]);
        // A ring where the arrowheads of a multi-attacker plan converge, so
        // the defending player's spot is unmistakable.
        if let End::Seat(_) = end {
            arrows.ring(ArrowKey::new(Cue::Defender, end, end), to, 1.1, Vec3::Y, base);
        }
    }
}

/// After the declaration, keep one arrow per attacker pointing at what it
/// is attacking (CR 508.1b — the view's `attack_target`), coloured by the
/// defending seat. In a pod the combat chips alone don't say *who* is under
/// fire, and the defenders each need to know which attackers are theirs to
/// block. 1v1 skips it: there is only one place an attack can go.
pub fn draw_declared_attack_arrows(
    view: Res<CurrentView>,
    attacking: Res<AttackingState>,
    board: BoardSpots,
    mut arrows: ResMut<Arrows>,
) {
    let Some(cv) = &view.0 else { return };
    let n_seats = cv.players.len();
    if n_seats <= 2 || cv.game_over.is_some() {
        return;
    }
    if !cv.battlefield.iter().any(|c| c.attacking && c.attack_target.is_some()) {
        return;
    }
    let viewer = cv.your_seat;
    let spots = board.spots();
    for c in cv.battlefield.iter().filter(|c| c.attacking) {
        // The plan overlay already draws this attacker.
        if attacking.contains(c.id) {
            continue;
        }
        let (Some(target), Some((from, clear))) = (c.attack_target, spots.chip(c.id, 0.18)) else {
            continue;
        };
        let Some((end, to)) = attack_end(target, &spots, viewer, n_seats) else { continue };
        let base = c.defending_player.map(seat_color).unwrap_or(Color::srgb(1.0, 0.35, 0.05));
        let key = ArrowKey::new(Cue::Attack, End::Card(c.id), end);
        arrows.arrow_trimmed(key, from, to, base.with_alpha(0.8), 0.85, [clear, 0.0]);
    }
}

/// Pulsing cyan ring on the battlefield permanent whose pending
/// decision is waiting on the viewer — "the game is waiting HERE". Most
/// decisions carry their source's CardId; when that source is a live
/// battlefield permanent, ring it so the modal's context is findable on
/// the board at a glance.
pub fn draw_decision_source_ring(
    view: Res<CurrentView>,
    time: Res<Time>,
    board: BoardSpots,
    mut arrows: ResMut<Arrows>,
) {
    use crabomination::net::DecisionWire as D;
    let Some(cv) = &view.0 else { return };
    let Some(pd) = &cv.pending_decision else { return };
    if pd.acting_player != cv.your_seat {
        return;
    }
    let source = match pd.decision.as_ref() {
        Some(
            D::ChooseMode { source, .. }
            | D::ChooseModes { source, .. }
            | D::ChooseColor { source, .. }
            | D::OptionalTrigger { source, .. }
            | D::ChooseCreatureType { source, .. }
            | D::NameCard { source, .. }
            | D::ChooseAmount { source, .. }
            | D::DivideDamage { source, .. }
            | D::ChooseCards { source, .. }
            | D::ChooseTarget { source, .. },
        ) => *source,
        Some(D::AssignCombatDamage { attacker, .. } | D::CombatDamageOrder { attacker, .. }) => {
            *attacker
        }
        _ => return,
    };
    let Some((at, normal, scale)) = board.spots().face(source, 0.2) else { return };
    let pulse = 0.55 + 0.45 * (time.elapsed_secs() * 4.0).sin().abs();
    let key = ArrowKey::new(Cue::DecisionSource, End::Card(source), End::Card(source));
    arrows.ring(key, at, 1.35 * scale, normal, Color::srgba(0.0, 0.9, 1.0, pulse));
}
