//! Gizmo overlays drawn over the battlefield: blocking diamonds + assignment
//! arrows during the DeclareBlockers step, attacker swords during attacks, and
//! source→target arrows for items on the stack.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crabomination::card::CardId;
use crabomination::game::{AttackTarget, Target, TurnStep};
use crabomination::net::StackItemView;

use crate::card::{
    BattlefieldCard, CardHovered, CardOwner, GameCardId, HandCard, PlayerTargetZone, StackCard,
};
use crate::card::layout::player_hand_anchor;
use crate::game::{AttackingState, BlockingState, TargetingState};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::PlayerHudPanel;
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

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BlockingGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AttackerGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct StackGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct PtModifiedGizmos;

/// Overlay drawn while the viewer is building their attack plan during
/// their own DeclareAttackers step. One diamond per selected attacker,
/// one arrow per attacker → target (player zone or planeswalker).
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AttackPlanGizmos;

/// Overlay drawn around battlefield permanents enumerated as legal
/// targets for a server `Decision::ChooseTarget` while the viewer is
/// picking. Pulses the same yellow used elsewhere in the targeting
/// vocabulary (player chip outline, blocker selection diamond).
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct LegalTargetGizmos;

/// Overlay for the interactive "drag" arrow that runs from the targeting
/// source to the cursor (or the hovered target it snaps to) while the viewer
/// is picking a spell/ability target.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct TargetArrowGizmos;

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
/// exactly what a click would resolve to. Glows like the other cues, so it
/// reads as a beam of light on HDR tiers.
#[allow(clippy::too_many_arguments)]
pub fn draw_target_arrow(
    targeting: Res<TargetingState>,
    view: Res<CurrentView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    hand_cards: Query<(&Transform, &GameCardId), With<HandCard>>,
    hovered_bf: Query<&GameCardId, (With<CardHovered>, With<BattlefieldCard>)>,
    hovered_zone: Query<&PlayerTargetZone, With<CardHovered>>,
    chips: Query<(&Interaction, &PlayerHudPanel)>,
    mut gizmos: Gizmos<TargetArrowGizmos>,
) {
    if !targeting.active {
        return;
    }
    let Some(cv) = &view.0 else { return };
    let (viewer, n_seats) = (cv.your_seat, cv.players.len());

    // Held a touch off the table so the arrow floats clearly above the cards.
    const ARROW_Y: f32 = 0.4;

    let bf_pos = |id: CardId| bf_cards.iter().find(|(_, g)| g.0 == id).map(|(t, _)| t.translation);
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

    let color = glow(Color::srgb(1.0, 0.88, 0.0), CUE_GLOW);
    gizmos.arrow(source, end, color).with_tip_length(0.8);
}

pub fn draw_legal_target_rings(
    view: Res<CurrentView>,
    targeting: Res<crate::game::TargetingState>,
    legal: Res<crate::game::LegalTargets>,
    time: Res<Time>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    mut gizmos: Gizmos<LegalTargetGizmos>,
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
    let pulse = 0.55 + 0.45 * (time.elapsed_secs() * 4.0).sin().abs();
    // Pulse drives both hue brightness and the HDR glow, so the ring visibly
    // breathes light in and out as it pulses.
    let color = glow(Color::srgb(pulse, pulse * 0.88, 0.0), 3.0);
    for (t, gid) in &bf_cards {
        if !legal.permanents.contains(&gid.0) {
            continue;
        }
        let center = t.translation + Vec3::Y * 0.2;
        let n = 28;
        let r = 1.2;
        for i in 0..n {
            let a0 = (i as f32) / (n as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32) / (n as f32) * std::f32::consts::TAU;
            let p0 = center + Vec3::new(a0.cos() * r, 0.0, a0.sin() * r);
            let p1 = center + Vec3::new(a1.cos() * r, 0.0, a1.sin() * r);
            gizmos.line(p0, p1, color);
        }
    }
}

pub fn draw_blocking_gizmos(
    view: Res<CurrentView>,
    blocking: Res<BlockingState>,
    bf_cards: Query<(&Transform, &GameCardId, &CardOwner), With<BattlefieldCard>>,
    mut gizmos: Gizmos<BlockingGizmos>,
) {
    let Some(cv) = &view.0 else { return };
    // Active player is some opponent (any non-viewer seat) declaring attackers
    // we may need to block.
    if cv.step != TurnStep::DeclareBlockers || cv.active_player == cv.your_seat { return; }

    let mut positions: HashMap<CardId, Vec3> = HashMap::new();
    for (transform, gid, _) in &bf_cards {
        positions.insert(gid.0, transform.translation + Vec3::Y * 0.15);
    }

    let attacking: Vec<CardId> = cv.battlefield.iter().filter(|p| p.attacking).map(|p| p.id).collect();

    for &attacker_id in &attacking {
        let is_blocked = blocking.assignments.iter().any(|(_, a)| *a == attacker_id);
        if let Some(&pos) = positions.get(&attacker_id) {
            let base = if is_blocked { Color::srgb(0.0, 0.9, 0.3) } else { Color::srgb(1.0, 0.2, 0.2) };
            draw_diamond(&mut gizmos, pos, 1.1, glow(base, CUE_GLOW));
        }
    }

    if let Some(blocker_id) = blocking.selected_blocker
        && let Some(&pos) = positions.get(&blocker_id)
    {
        draw_diamond(&mut gizmos, pos, 1.1, glow(Color::srgb(1.0, 0.88, 0.0), CUE_GLOW));
        for &attacker_id in &attacking {
            let already_assigned = blocking.assignments.iter().any(|(_, a)| *a == attacker_id);
            if !already_assigned && let Some(&att_pos) = positions.get(&attacker_id) {
                gizmos.arrow(pos, att_pos, glow(Color::srgba(1.0, 0.88, 0.0, 0.7), CUE_GLOW)).with_tip_length(0.6);
            }
        }
    }

    for &(blocker_id, attacker_id) in &blocking.assignments {
        if let Some(&b_pos) = positions.get(&blocker_id)
            && let Some(&a_pos) = positions.get(&attacker_id)
        {
            let green = glow(Color::srgb(0.0, 0.9, 0.3), CUE_GLOW);
            gizmos.arrow(b_pos, a_pos, green).with_tip_length(0.6);
            draw_diamond(&mut gizmos, b_pos, 1.1, green);
        }
    }
}

pub fn draw_attacker_overlays(
    view: Res<CurrentView>,
    attack_plan: Res<AttackingState>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    mut gizmos: Gizmos<AttackerGizmos>,
) {
    let Some(cv) = &view.0 else { return };
    // Creatures the engine already reports as attacking…
    let mut attacking: HashSet<CardId> =
        cv.battlefield.iter().filter(|p| p.attacking).map(|p| p.id).collect();
    // …plus the viewer's in-progress attack plan, so a creature shows its
    // swords the moment it's selected during DeclareAttackers — not only after
    // the attack is submitted (Attack All / Confirm).
    if cv.step == TurnStep::DeclareAttackers && cv.active_player == cv.your_seat {
        attacking.extend(attack_plan.plan.iter().map(|(atk, _)| *atk));
    }
    if attacking.is_empty() { return; }

    let mut positions: HashMap<CardId, Vec3> = HashMap::new();
    for (transform, gid) in &bf_cards {
        positions.insert(gid.0, transform.translation);
    }

    for attacker_id in attacking {
        if let Some(&pos) = positions.get(&attacker_id) {
            draw_crossed_swords(&mut gizmos, pos, glow(Color::srgb(1.0, 0.35, 0.05), CUE_GLOW));
        }
    }
}

pub fn draw_stack_arrows(
    view: Res<CurrentView>,
    stack_cards: Query<(&Transform, &GameCardId), With<StackCard>>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    mut gizmos: Gizmos<StackGizmos>,
) {
    let Some(cv) = &view.0 else { return };
    if cv.stack.is_empty() { return; }

    let mut stack_pos: HashMap<CardId, Vec3> = HashMap::new();
    for (t, gid) in &stack_cards {
        stack_pos.insert(gid.0, t.translation + Vec3::Y * 0.2);
    }
    let mut bf_pos: HashMap<CardId, Vec3> = HashMap::new();
    for (t, gid) in &bf_cards {
        bf_pos.insert(gid.0, t.translation + Vec3::Y * 0.2);
    }

    let viewer = cv.your_seat;
    let n_seats = cv.players.len();
    let color = glow(Color::srgba(1.0, 0.6, 0.05, 0.9), CUE_GLOW);

    for item in &cv.stack {
        let StackItemView::Known(known) = item else { continue };
        let source_id = known.source;
        let Some(ref target) = known.target else { continue };

        let from = stack_pos.get(&source_id).or_else(|| bf_pos.get(&source_id)).copied();
        let to = match target {
            Target::Permanent(id) => bf_pos.get(id).copied(),
            Target::Player(idx) => {
                if *idx < n_seats {
                    let mut p = player_hand_anchor(*idx, viewer, n_seats);
                    p.y = 0.2;
                    Some(p)
                } else {
                    None
                }
            }
        };

        if let (Some(from), Some(to)) = (from, to) {
            gizmos.arrow(from, to, color).with_tip_length(0.7);
        }
    }
}

/// Draw a colored ring above any battlefield creature whose computed P/T
/// differs from the card's printed (base) P/T. Green when buffed, red when
/// debuffed, yellow on a mixed change. Helps the player notice that a
/// creature has counters, a Giant Growth pump, an anthem effect, etc.,
/// even though the 3D card face still shows the printed values.
pub fn draw_pt_modified_overlays(
    view: Res<CurrentView>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    mut gizmos: Gizmos<PtModifiedGizmos>,
) {
    let Some(cv) = &view.0 else { return };
    let mut positions: HashMap<CardId, Vec3> = HashMap::new();
    for (t, gid) in &bf_cards {
        positions.insert(gid.0, t.translation);
    }
    for p in &cv.battlefield {
        if !p.card_types.contains(&crabomination::card::CardType::Creature) { continue; }
        let dp = p.power - p.base_power;
        let dt = p.toughness - p.base_toughness;
        if dp == 0 && dt == 0 { continue; }
        // Skip the ring when the P/T delta is fully explained by visible
        // +1/+1 / -1/-1 counter coins on the card. The coins already
        // communicate the modification, and the ring on top of them looks
        // like a redundant tint over the card.
        let plus = p
            .counters
            .iter()
            .find_map(|(k, n)| (*k == crabomination::card::CounterType::PlusOnePlusOne).then_some(*n))
            .unwrap_or(0) as i32;
        let minus = p
            .counters
            .iter()
            .find_map(|(k, n)| (*k == crabomination::card::CounterType::MinusOneMinusOne).then_some(*n))
            .unwrap_or(0) as i32;
        let counter_delta = plus - minus;
        if dp == counter_delta && dt == counter_delta { continue; }
        let Some(&pos) = positions.get(&p.id) else { continue };
        let base = if dp > 0 && dt > 0 {
            Color::srgb(0.2, 0.95, 0.35)
        } else if dp < 0 || dt < 0 {
            Color::srgb(0.95, 0.25, 0.2)
        } else {
            Color::srgb(0.95, 0.85, 0.15)
        };
        // Gentler glow than the action cues — this is a passive "stats
        // changed" marker, not something the player must act on.
        let color = glow(base, 1.8);
        let center = pos + Vec3::Y * 0.18;
        let n = 24;
        let r = 1.05;
        for i in 0..n {
            let a0 = (i as f32) / (n as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32) / (n as f32) * std::f32::consts::TAU;
            let p0 = center + Vec3::new(a0.cos() * r, 0.0, a0.sin() * r);
            let p1 = center + Vec3::new(a1.cos() * r, 0.0, a1.sin() * r);
            gizmos.line(p0, p1, color);
        }
    }
}

/// Draw a crossed-swords (⚔) symbol in the XZ plane at `pos`, slightly raised.
fn draw_crossed_swords(gizmos: &mut Gizmos<AttackerGizmos>, pos: Vec3, color: Color) {
    let y = pos.y + 0.18;
    let r: f32 = 0.7;
    let g: f32 = 0.22;
    let gf: f32 = 0.35;

    let s1_handle = Vec3::new(pos.x - r, y, pos.z - r);
    let s1_tip = Vec3::new(pos.x + r, y, pos.z + r);
    let s1_guard = s1_handle.lerp(s1_tip, gf);
    let gd1 = Vec3::new(1.0, 0.0, -1.0).normalize() * g;
    gizmos.line(s1_handle, s1_tip, color);
    gizmos.line(s1_guard - gd1, s1_guard + gd1, color);

    let s2_handle = Vec3::new(pos.x + r, y, pos.z - r);
    let s2_tip = Vec3::new(pos.x - r, y, pos.z + r);
    let s2_guard = s2_handle.lerp(s2_tip, gf);
    let gd2 = Vec3::new(1.0, 0.0, 1.0).normalize() * g;
    gizmos.line(s2_handle, s2_tip, color);
    gizmos.line(s2_guard - gd2, s2_guard + gd2, color);
}

fn draw_diamond(gizmos: &mut Gizmos<BlockingGizmos>, pos: Vec3, r: f32, color: Color) {
    let n = pos + Vec3::Z * r;
    let s = pos - Vec3::Z * r;
    let e = pos + Vec3::X * r;
    let w = pos - Vec3::X * r;
    gizmos.line(n, e, color);
    gizmos.line(e, s, color);
    gizmos.line(s, w, color);
    gizmos.line(w, n, color);
}

/// Render the viewer's in-progress attack plan: a yellow diamond on each
/// chosen attacker and an arrow from that attacker to its target (player
/// disc or opp planeswalker card). Active only during the viewer's own
/// DeclareAttackers step with priority — outside that window the plan is
/// stale and the resource is cleared by the input handler.
pub fn draw_attack_plan_gizmos(
    view: Res<CurrentView>,
    attacking: Res<AttackingState>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    mut gizmos: Gizmos<AttackPlanGizmos>,
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
    let yellow = glow(Color::srgb(1.0, 0.88, 0.0), CUE_GLOW);
    let pending = glow(Color::srgba(1.0, 0.88, 0.0, 0.5), CUE_GLOW);

    let mut positions: HashMap<CardId, Vec3> = HashMap::new();
    for (t, gid) in &bf_cards {
        positions.insert(gid.0, t.translation + Vec3::Y * 0.18);
    }

    for (attacker, target) in &attacking.plan {
        let Some(&from) = positions.get(attacker) else {
            continue;
        };
        let color = if attacking.last_added == Some(*attacker) {
            pending
        } else {
            yellow
        };
        // The attacker itself is marked by its crossed-swords overlay (see
        // `draw_attacker_overlays`, which now includes planned attackers), so
        // here we only draw the targeting arrow to its chosen defender.
        let to = match target {
            AttackTarget::Player(seat) => {
                let mut p = player_hand_anchor(*seat, viewer, n_seats);
                p.y = 0.3;
                Some(p)
            }
            AttackTarget::Planeswalker(pw_id) => positions.get(pw_id).copied(),
        };
        if let Some(to) = to {
            gizmos.arrow(from, to, color).with_tip_length(0.7);
        }
    }
}
