//! Gizmo overlays drawn over the battlefield: blocking diamonds + assignment
//! arrows during the DeclareBlockers step, attacker swords during attacks, and
//! source→target arrows for items on the stack.

use std::collections::HashMap;

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::{Target, TurnStep};
use crabomination::net::StackItemView;

use crate::card::{BattlefieldCard, CardOwner, GameCardId, StackCard};
use crate::card::layout::player_target_zone_position;
use crate::game::BlockingState;
use crate::net_plugin::CurrentView;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BlockingGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AttackerGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct StackGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct PtModifiedGizmos;

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
            let color = if is_blocked { Color::srgb(0.0, 0.9, 0.3) } else { Color::srgb(1.0, 0.2, 0.2) };
            draw_diamond(&mut gizmos, pos, 1.1, color);
        }
    }

    if let Some(blocker_id) = blocking.selected_blocker
        && let Some(&pos) = positions.get(&blocker_id)
    {
        draw_diamond(&mut gizmos, pos, 1.1, Color::srgb(1.0, 0.88, 0.0));
        for &attacker_id in &attacking {
            let already_assigned = blocking.assignments.iter().any(|(_, a)| *a == attacker_id);
            if !already_assigned && let Some(&att_pos) = positions.get(&attacker_id) {
                gizmos.arrow(pos, att_pos, Color::srgba(1.0, 0.88, 0.0, 0.7)).with_tip_length(0.6);
            }
        }
    }

    for &(blocker_id, attacker_id) in &blocking.assignments {
        if let Some(&b_pos) = positions.get(&blocker_id)
            && let Some(&a_pos) = positions.get(&attacker_id)
        {
            gizmos.arrow(b_pos, a_pos, Color::srgb(0.0, 0.9, 0.3)).with_tip_length(0.6);
            draw_diamond(&mut gizmos, b_pos, 1.1, Color::srgb(0.0, 0.9, 0.3));
        }
    }
}

pub fn draw_attacker_overlays(
    view: Res<CurrentView>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    mut gizmos: Gizmos<AttackerGizmos>,
) {
    let Some(cv) = &view.0 else { return };
    let attacking: Vec<CardId> = cv.battlefield.iter().filter(|p| p.attacking).map(|p| p.id).collect();
    if attacking.is_empty() { return; }

    let mut positions: HashMap<CardId, Vec3> = HashMap::new();
    for (transform, gid) in &bf_cards {
        positions.insert(gid.0, transform.translation);
    }

    for attacker_id in attacking {
        if let Some(&pos) = positions.get(&attacker_id) {
            draw_crossed_swords(&mut gizmos, pos, Color::srgb(1.0, 0.35, 0.05));
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
    let color = Color::srgba(1.0, 0.6, 0.05, 0.9);

    for item in &cv.stack {
        let StackItemView::Known(known) = item else { continue };
        let source_id = known.source;
        let Some(ref target) = known.target else { continue };

        let from = stack_pos.get(&source_id).or_else(|| bf_pos.get(&source_id)).copied();
        let to = match target {
            Target::Permanent(id) => bf_pos.get(id).copied(),
            Target::Player(idx) => {
                if *idx < n_seats {
                    let mut p = player_target_zone_position(*idx, viewer, n_seats);
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
        let Some(&pos) = positions.get(&p.id) else { continue };
        let color = if dp > 0 && dt > 0 {
            Color::srgb(0.2, 0.95, 0.35)
        } else if dp < 0 || dt < 0 {
            Color::srgb(0.95, 0.25, 0.2)
        } else {
            Color::srgb(0.95, 0.85, 0.15)
        };
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
