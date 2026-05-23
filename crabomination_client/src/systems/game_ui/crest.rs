//! Per-seat **player crest** — the 3-D disc + halo ring + floating
//! life numeral cluster that represents each player on the table.
//!
//! Sibling to `player_stats` (the 2-D HUD chips). The crest is the
//! *spatial* readout — anchored next to the player's seat in 3-D — and
//! is the single source of truth for player click-targeting. The 2-D
//! chip strips remain as a compact roster on the corners.
//!
//! Three systems:
//! * [`update_player_target_zone_material`] — subtle hover emphasis on
//!   the disc mesh itself.
//! * [`update_player_crest_ring`] — drives the halo ring's colour /
//!   emissive each frame to advertise state (targeting, threatened,
//!   active player).
//! * [`setup_player_life_labels`] (`OnEnter`) + [`update_player_crest_life_label`]
//!   (`Update`) — the screen-space life numeral that follows each
//!   crest's projected world position.

use bevy::prelude::*;
use crabomination::game::TurnStep;

use crate::card::{CardHovered, PlayerTargetZone};
use crate::game::TargetingState;
use crate::net_plugin::CurrentView;
use crate::theme::{self, UiFonts};

use super::InGameRoot;

/// Subtle hover emphasis on the player disc itself. The *loud* state
/// signal (targeting glow, threat warning, priority) lives on the
/// [`crate::card::PlayerCrestRing`] halo below the disc — see
/// [`update_player_crest_ring`]. This system only nudges the disc's
/// emissive when the pointer is over it so the player gets feedback
/// that the click region is responding, without competing with the
/// ring colours.
pub fn update_player_target_zone_material(
    mut materials: ResMut<Assets<StandardMaterial>>,
    zones: Query<
        (&MeshMaterial3d<StandardMaterial>, Has<CardHovered>),
        With<PlayerTargetZone>,
    >,
) {
    for (mat_handle, hovered) in &zones {
        let Some(mat) = materials.get_mut(&mat_handle.0) else { continue };
        let intensity = if hovered { 0.55 } else { 0.05 };
        mat.emissive = LinearRgba::new(intensity, intensity * 0.55, intensity * 0.55, 1.0);
    }
}

// ── Crest ring ───────────────────────────────────────────────────────────────
//
// `update_player_crest_ring` drives the halo material under each disc:
//   * **Targeting active** — yellow pulse on every seat (matches the
//     2-D chip border so 3-D and 2-D click regions share vocabulary).
//   * **Pending attacker waiting for defender** — yellow pulse on
//     *opponent* seats only.
//   * **Viewer threatened** (life ≤ 5 or lethal-on-board) — red glow
//     on the viewer's own ring as a peripheral-vision warning.
//   * **Active player** — soft gold ambient.
//   * **Otherwise** — dim neutral.

const CREST_RING_IDLE_BASE: Color = Color::srgba(0.18, 0.18, 0.22, 1.0);
const CREST_RING_IDLE_EMISSIVE: LinearRgba = LinearRgba::new(0.04, 0.04, 0.06, 1.0);
const CREST_RING_TARGET_BASE: Color = Color::srgb(1.0, 0.88, 0.0);
const CREST_RING_THREAT_BASE: Color = Color::srgb(1.0, 0.30, 0.30);
const CREST_RING_ACTIVE_BASE: Color = Color::srgb(1.0, 0.85, 0.55);

struct RingState {
    base: Color,
    base_mul: f32,
    emissive: f32,
}

fn ring_state_for_seat(
    seat: usize,
    your_seat: usize,
    cv: &crabomination::net::ClientView,
    targeting: &TargetingState,
    attacking: &crate::game::AttackingState,
    lethal_on_board: i32,
    pulse: f32,
) -> RingState {
    let attack_pick = cv.step == TurnStep::DeclareAttackers
        && cv.active_player == your_seat
        && cv.priority == your_seat
        && attacking.last_added.is_some();

    if targeting.active || (attack_pick && seat != your_seat) {
        return RingState {
            base: CREST_RING_TARGET_BASE,
            base_mul: 0.55 + 0.45 * pulse,
            emissive: 0.50 + 0.90 * pulse,
        };
    }

    let viewer_life = cv.players.iter().find(|p| p.seat == your_seat).map(|p| p.life).unwrap_or(20);
    let threatened = seat == your_seat
        && (viewer_life <= 5 || lethal_on_board >= viewer_life as i32);
    if threatened {
        return RingState {
            base: CREST_RING_THREAT_BASE,
            base_mul: 0.65 + 0.35 * pulse,
            emissive: 0.40 + 0.60 * pulse,
        };
    }

    if cv.active_player == seat {
        return RingState {
            base: CREST_RING_ACTIVE_BASE,
            base_mul: 0.55,
            emissive: 0.25,
        };
    }

    RingState {
        base: CREST_RING_IDLE_BASE,
        base_mul: 1.0,
        emissive: 0.0,
    }
}

pub fn update_player_crest_ring(
    view: Res<CurrentView>,
    targeting: Res<TargetingState>,
    attacking: Res<crate::game::AttackingState>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rings: Query<(&MeshMaterial3d<StandardMaterial>, &crate::card::PlayerCrestRing)>,
) {
    let Some(cv) = &view.0 else { return };

    let pulse = 0.5 + 0.5 * (time.elapsed_secs() * std::f32::consts::TAU * 0.8).sin().abs();

    use crabomination::card::Keyword;
    let lethal_on_board: i32 = cv
        .battlefield
        .iter()
        .filter(|c| {
            c.owner != cv.your_seat
                && c.is_creature()
                && !c.tapped
                && (!c.summoning_sick || c.keywords.contains(&Keyword::Haste))
                && !c.keywords.contains(&Keyword::Defender)
        })
        .map(|c| c.power.max(0))
        .sum();

    for (mat_handle, ring) in &rings {
        let Some(mat) = materials.get_mut(&mat_handle.0) else { continue };
        let st = ring_state_for_seat(
            ring.seat,
            cv.your_seat,
            cv,
            &targeting,
            &attacking,
            lethal_on_board,
            pulse,
        );
        let (r, g, b, _a) = {
            let s = st.base.to_srgba();
            (s.red * st.base_mul, s.green * st.base_mul, s.blue * st.base_mul, s.alpha)
        };
        mat.base_color = Color::srgba(r, g, b, 1.0);
        let e = st.emissive;
        let s = st.base.to_srgba();
        if e <= 0.0 {
            mat.emissive = CREST_RING_IDLE_EMISSIVE;
        } else {
            mat.emissive = LinearRgba::new(s.red * e, s.green * e, s.blue * e, 1.0);
        }
    }
}

// ── Floating life numeral ────────────────────────────────────────────────────

const CREST_LIFE_WORLD_Y: f32 = 1.4;

/// Spawn one absolute-positioned UI text node per seat on `OnEnter(InGame)`.
/// [`update_player_crest_life_label`] updates their `top`/`left`/text every
/// frame.
pub fn setup_player_life_labels(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<crate::card::PlayerLifeLabel>>,
) {
    if !existing.is_empty() {
        return;
    }
    const MAX_LABELS: usize = 8;
    for seat in 0..MAX_LABELS {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-1000.0),
                top: Val::Px(-1000.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            Pickable::IGNORE,
            crate::card::PlayerLifeLabel { seat },
            InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                ui_fonts.tf(22.0),
                TextColor(theme::ACCENT_GOLD),
                Pickable::IGNORE,
            ));
        });
    }
}

/// Project each crest's world position to viewport coords and reposition
/// the matching `PlayerLifeLabel` over it.
#[allow(clippy::type_complexity)]
pub fn update_player_crest_life_label(
    view: Res<CurrentView>,
    crests: Query<(&Transform, &crate::card::PlayerCrest)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    mut labels: Query<(
        &crate::card::PlayerLifeLabel,
        &mut Node,
        &mut BackgroundColor,
        &Children,
    )>,
    mut text_q: Query<(&mut Text, &mut TextColor)>,
) {
    let Some(cv) = &view.0 else {
        for (_, mut node, _, _) in &mut labels {
            node.display = Display::None;
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    let mut crest_pos: std::collections::HashMap<usize, Vec3> = std::collections::HashMap::new();
    for (xform, crest) in &crests {
        let mut p = xform.translation;
        p.y += CREST_LIFE_WORLD_Y;
        crest_pos.insert(crest.seat, p);
    }

    for (label, mut node, mut bg, children) in &mut labels {
        let Some(player) = cv.players.iter().find(|p| p.seat == label.seat) else {
            node.display = Display::None;
            continue;
        };
        let Some(world_pos) = crest_pos.get(&label.seat).copied() else {
            node.display = Display::None;
            continue;
        };

        let viewport = match camera.world_to_viewport(cam_xform, world_pos) {
            Ok(v) => v,
            Err(_) => {
                node.display = Display::None;
                continue;
            }
        };

        node.display = Display::Flex;
        node.left = Val::Px(viewport.x - 22.0);
        node.top = Val::Px(viewport.y - 18.0);

        let (life_color, bg_alpha) = match player.life {
            l if l <= 0 => (theme::TEXT_DANGER, 0.85),
            l if l <= 5 => (theme::TEXT_DANGER, 0.75),
            l if l <= 10 => (Color::srgb(1.0, 0.78, 0.40), 0.65),
            _ => (theme::ACCENT_GOLD, 0.65),
        };
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, bg_alpha));

        for child in children.iter() {
            if let Ok((mut t, mut c)) = text_q.get_mut(child) {
                t.0 = player.life.to_string();
                *c = TextColor(life_color);
            }
        }
    }
}
