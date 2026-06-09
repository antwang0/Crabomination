//! Transient "impact" feedback for high-drama moments: a radial burst when a
//! creature dies, a sharp spark where damage lands, and a red edge-vignette
//! when the viewer loses life.
//!
//! All three are intentionally over-bright (colours in the HDR > 1.0 range)
//! so they bloom on the Medium+ tiers — the death burst and damage spark in
//! particular read as a flash of light rather than a flat gizmo line. They
//! self-despawn on a short timer and are tagged `InGameRoot`, so leaving the
//! match cleans them up with the rest of the HUD.

use std::collections::{HashMap, HashSet};
use std::f32::consts::TAU;

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::mana::Color as ManaColor;
use crabomination::net::{GameEventWire, StackItemView};

use crate::card::layout::player_hand_anchor;
use crate::card::{BattlefieldCard, GameCardId, StackCard};
use crate::net_plugin::{CurrentView, LatestServerEvents};
use crate::systems::game_ui::InGameRoot;
use crate::theme::{self, UiFonts};
use crate::MainCamera;

/// Gizmo group for the 3-D bursts/sparks. Registered in `main.rs`.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ImpactGizmos;

/// An expanding, fading ring-plus-spokes drawn in the XZ plane at `pos`.
/// `color` is already HDR-bright; it dims out over the burst's life.
#[derive(Component)]
pub struct ImpactBurst {
    pos: Vec3,
    age: f32,
    ttl: f32,
    color: Color,
    start_r: f32,
    end_r: f32,
    spokes: usize,
}

/// A red screen-edge flash shown when the viewer takes life loss. Driven via
/// the node's border (centre stays clear), fading on a short timer.
#[derive(Component)]
pub struct HitVignette {
    age: f32,
    ttl: f32,
    peak_alpha: f32,
}

/// A floating "−N" numeral that rises and fades off a creature that was just
/// dealt damage. Screen-space after spawn (no per-frame card tracking), the
/// same pattern as the HUD life-flash numerals.
#[derive(Component)]
pub struct DamageNumeral {
    remaining: f32,
    total: f32,
    /// Spawn-time `top` in px; the numeral rises from here as it fades.
    base_top: f32,
}

/// A glowing mana-coloured mote that arcs from a player's land row toward the
/// table centre as they tap for mana — one per `ManaAdded`, so a multi-pip
/// cast reads as mana converging into the spell.
#[derive(Component)]
pub struct ManaMote {
    start: Vec3,
    end: Vec3,
    age: f32,
    ttl: f32,
    color: Color,
}

// ── Tuning ─────────────────────────────────────────────────────────────────

const DEATH_TTL: f32 = 0.55;
const DEATH_START_R: f32 = 0.3;
const DEATH_END_R: f32 = 2.3;
const DEATH_SPOKES: usize = 8;

const SPARK_TTL: f32 = 0.3;
const SPARK_START_R: f32 = 0.15;
const SPARK_END_R: f32 = 0.95;
const SPARK_SPOKES: usize = 6;

/// Raise the burst slightly off the table so it reads above the card faces and
/// any counter coins sitting on them.
const BURST_Y: f32 = 0.25;

const VIGNETTE_TTL: f32 = 0.7;
/// Border thickness (px) of the edge flash — fat enough to register in
/// peripheral vision without crowding the corner HUD panels.
const VIGNETTE_BORDER: f32 = 48.0;

const DMG_NUMERAL_SECS: f32 = 0.9;
/// How far (px) the damage numeral floats upward over its lifetime.
const DMG_NUMERAL_RISE: f32 = 36.0;

const MANA_MOTE_TTL: f32 = 0.55;
/// Peak arc height of a travelling mana mote.
const MANA_MOTE_LIFT: f32 = 1.6;

/// Bright red (HDR) for the death burst — blooms, then fades to nothing.
fn death_color() -> Color {
    LinearRgba::new(3.2, 0.25, 0.1, 1.0).into()
}

/// Hot, near-white spark for damage landing.
fn spark_color() -> Color {
    LinearRgba::new(3.0, 2.2, 1.2, 1.0).into()
}

/// HDR mana-pip colour for a travelling mote; `None` is colorless.
fn mana_color(color: Option<ManaColor>) -> Color {
    let (r, g, b) = match color {
        Some(ManaColor::White) => (2.7, 2.6, 2.1),
        Some(ManaColor::Blue) => (0.3, 1.4, 3.0),
        Some(ManaColor::Black) => (1.5, 0.9, 1.9),
        Some(ManaColor::Red) => (3.0, 0.7, 0.4),
        Some(ManaColor::Green) => (0.5, 2.6, 0.8),
        None => (1.8, 1.8, 2.1),
    };
    LinearRgba::new(r, g, b, 1.0).into()
}

// ── Spawning ───────────────────────────────────────────────────────────────

/// Read the latest server-event batch and spawn the matching impact effects.
/// `events.0` is repopulated every frame by `poll_net` (PreUpdate) and is
/// non-empty only on a batch frame, so a plain emptiness check both gates the
/// work and prevents re-spawning across frames.
pub fn spawn_impact_effects(
    mut commands: Commands,
    events: Res<LatestServerEvents>,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GlobalTransform, &GameCardId)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    existing_vignettes: Query<Entity, With<HitVignette>>,
) {
    if events.0.is_empty() {
        return;
    }
    let Some(cv) = &view.0 else { return };

    let pos_of = |id: CardId| -> Option<Vec3> {
        cards.iter().find(|(_, g)| g.0 == id).map(|(t, _)| t.translation())
    };

    for ev in &events.0 {
        match ev {
            GameEventWire::CreatureDied { card_id } => {
                if let Some(p) = pos_of(*card_id) {
                    spawn_burst(
                        &mut commands,
                        p,
                        death_color(),
                        DEATH_TTL,
                        DEATH_START_R,
                        DEATH_END_R,
                        DEATH_SPOKES,
                    );
                }
            }
            GameEventWire::DamageDealt { to_card, to_player, amount } => {
                let card_world = to_card.and_then(&pos_of);
                // Spark at the struck permanent, else at the damaged player's
                // table anchor. (Unblocked combat damage and direct burn both
                // surface as a player hit here.)
                let at = card_world.or_else(|| {
                    to_player
                        .map(|seat| player_hand_anchor(seat, cv.your_seat, cv.players.len()))
                });
                if let Some(p) = at {
                    spawn_burst(
                        &mut commands,
                        p,
                        spark_color(),
                        SPARK_TTL,
                        SPARK_START_R,
                        SPARK_END_R,
                        SPARK_SPOKES,
                    );
                }
                // Floating "−N" on a struck creature, so the player reads how
                // much it took. Player damage already surfaces via the
                // life-loss flash numeral + vignette, so only creatures here.
                if *amount > 0
                    && let Some(world) = card_world
                    && let Ok((camera, cam_xform)) = camera_q.single()
                    && let Ok(screen) =
                        camera.world_to_viewport(cam_xform, world + Vec3::Y * 0.6)
                {
                    spawn_damage_numeral(&mut commands, &ui_fonts, *amount, screen);
                }
            }
            GameEventWire::LifeLost { player, amount } if *player == cv.your_seat => {
                // Replace any in-flight vignette so rapid hits don't stack into
                // an opaque red frame; scale the flash with the hit size.
                for e in &existing_vignettes {
                    commands.entity(e).despawn();
                }
                let peak_alpha = (0.18 + 0.05 * *amount as f32).min(0.6);
                spawn_vignette(&mut commands, peak_alpha);
            }
            _ => {}
        }
    }
}

/// Spawn a mana mote for each `ManaAdded` / `ColorlessManaAdded` in the latest
/// batch, arcing from one of the producing player's actually-tapped lands to
/// the spell (or ability) it's paying for on the stack.
///
/// Runs after `sync_game_visuals` so the freshly-cast spell's `StackCard`
/// entity already exists to aim at. When the stack is empty (mana floated with
/// nothing to feed) no motes fire — they'd otherwise sail at an empty table.
pub fn spawn_mana_motes(
    mut commands: Commands,
    events: Res<LatestServerEvents>,
    view: Res<CurrentView>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    stack_cards: Query<(&Transform, &GameCardId), With<StackCard>>,
    // Lands that were tapped as of the previous frame, so we can tell which
    // ones *newly* tapped in the current batch — `ManaAdded` carries no source
    // permanent, but the lands that just tapped this batch are what produced
    // the mana. Kept across frames; the view only changes on a batch.
    mut prev_tapped: Local<HashSet<CardId>>,
) {
    let Some(cv) = &view.0 else {
        prev_tapped.clear();
        return;
    };

    let land_pos: HashMap<CardId, Vec3> =
        bf_cards.iter().map(|(t, g)| (g.0, t.translation)).collect();
    let current_tapped: HashSet<CardId> = cv
        .battlefield
        .iter()
        .filter(|p| p.is_land() && p.tapped)
        .map(|p| p.id)
        .collect();

    if !events.0.is_empty() {
        let stack_pos = |id: CardId| {
            stack_cards.iter().find(|(_, g)| g.0 == id).map(|(t, _)| t.translation)
        };
        // Destination: the spell cast this batch → the top stack item → table
        // centre (only if something is on the stack). Empty stack ⇒ skip.
        let cast_id = events.0.iter().find_map(|e| match e {
            GameEventWire::SpellCast { card_id, .. } => Some(*card_id),
            _ => None,
        });
        let dest = cast_id
            .and_then(stack_pos)
            .or_else(|| match cv.stack.last() {
                Some(StackItemView::Known(k)) => stack_pos(k.source),
                _ => None,
            })
            .or_else(|| (!cv.stack.is_empty()).then_some(Vec3::new(0.0, 0.9, 0.0)));

        if let Some(mut dest) = dest {
            dest.y += 0.2;

            // Source pools per controller: the lands that *newly* tapped this
            // batch (the ones that produced this mana), with all of a player's
            // tapped lands as a fallback if we somehow saw no fresh tap (e.g.
            // mana from a non-land source).
            let mut newly: HashMap<usize, Vec<Vec3>> = HashMap::new();
            let mut all_tapped: HashMap<usize, Vec<Vec3>> = HashMap::new();
            for p in &cv.battlefield {
                if !p.is_land() || !p.tapped {
                    continue;
                }
                let Some(&pos) = land_pos.get(&p.id) else { continue };
                all_tapped.entry(p.controller).or_default().push(pos);
                if !prev_tapped.contains(&p.id) {
                    newly.entry(p.controller).or_default().push(pos);
                }
            }

            let mut next_src: HashMap<usize, usize> = HashMap::new();
            for ev in &events.0 {
                let (player, color, source) = match ev {
                    GameEventWire::ManaAdded { player, color, source } => {
                        (*player, Some(*color), *source)
                    }
                    GameEventWire::ColorlessManaAdded { player, source } => {
                        (*player, None, *source)
                    }
                    _ => continue,
                };
                // Prefer the exact producing permanent the engine now reports;
                // fall back to a newly-tapped (else any) land of the player for
                // sourceless mana (rituals, devotion / X-cost effects).
                let mut start = match source.and_then(|sid| land_pos.get(&sid).copied()) {
                    Some(pos) => pos,
                    None => {
                        let sources = newly
                            .get(&player)
                            .filter(|v| !v.is_empty())
                            .or_else(|| all_tapped.get(&player).filter(|v| !v.is_empty()));
                        let Some(sources) = sources else { continue };
                        let slot = next_src.entry(player).or_default();
                        let pos = sources[*slot % sources.len()];
                        *slot += 1;
                        pos
                    }
                };
                start.y += 0.3;
                commands.spawn((
                    ManaMote {
                        start,
                        end: dest,
                        age: 0.0,
                        ttl: MANA_MOTE_TTL,
                        color: mana_color(color),
                    },
                    InGameRoot,
                ));
            }
        }
    }

    *prev_tapped = current_tapped;
}

#[allow(clippy::too_many_arguments)]
fn spawn_burst(
    commands: &mut Commands,
    pos: Vec3,
    color: Color,
    ttl: f32,
    start_r: f32,
    end_r: f32,
    spokes: usize,
) {
    commands.spawn((
        ImpactBurst { pos: pos + Vec3::Y * BURST_Y, age: 0.0, ttl, color, start_r, end_r, spokes },
        InGameRoot,
    ));
}

fn spawn_damage_numeral(commands: &mut Commands, fonts: &UiFonts, amount: u32, screen: Vec2) {
    let base_top = screen.y;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(base_top),
                // Nudge left so the numeral sits roughly centred over the hit.
                left: Val::Px(screen.x - 14.0),
                ..default()
            },
            Pickable::IGNORE,
            InGameRoot,
            DamageNumeral { remaining: DMG_NUMERAL_SECS, total: DMG_NUMERAL_SECS, base_top },
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(format!("-{amount}")),
                fonts.tf(26.0),
                TextColor(theme::TEXT_DANGER),
                Pickable::IGNORE,
            ));
        });
}

fn spawn_vignette(commands: &mut Commands, peak_alpha: f32) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            border: UiRect::all(Val::Px(VIGNETTE_BORDER)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.9, 0.08, 0.08, 0.0)),
        // Above the board / HUD so the edge flash is never occluded; ignore
        // picking so it can't eat clicks.
        GlobalZIndex(100),
        Pickable::IGNORE,
        InGameRoot,
        HitVignette { age: 0.0, ttl: VIGNETTE_TTL, peak_alpha },
    ));
}

// ── Animation ──────────────────────────────────────────────────────────────

/// Expand each burst's ring (ease-out) while dimming its colour to zero, then
/// despawn it when elapsed.
pub fn animate_impact_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut bursts: Query<(Entity, &mut ImpactBurst)>,
    mut gizmos: Gizmos<ImpactGizmos>,
) {
    for (entity, mut burst) in &mut bursts {
        burst.age += time.delta_secs();
        if burst.age >= burst.ttl {
            commands.entity(entity).despawn();
            continue;
        }
        let t = (burst.age / burst.ttl).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - t) * (1.0 - t); // ease-out expansion
        let r = burst.start_r + (burst.end_r - burst.start_r) * eased;
        // Fade brightness (and alpha) linearly to zero; multiplying the HDR
        // colour keeps it blooming early and gone late.
        let col = faded(burst.color, 1.0 - t);

        draw_ring(&mut gizmos, burst.pos, r, col);
        for i in 0..burst.spokes {
            let a = (i as f32 / burst.spokes as f32) * TAU;
            let dir = Vec3::new(a.cos(), 0.0, a.sin());
            gizmos.line(burst.pos + dir * (r * 0.45), burst.pos + dir * r, col);
        }
    }
}

/// Pulse the hit vignette: a fast rise, then a fade over the remainder.
pub fn animate_hit_vignettes(
    mut commands: Commands,
    time: Res<Time>,
    mut vignettes: Query<(Entity, &mut HitVignette, &mut BorderColor)>,
) {
    for (entity, mut v, mut border) in &mut vignettes {
        v.age += time.delta_secs();
        if v.age >= v.ttl {
            commands.entity(entity).despawn();
            continue;
        }
        let t = v.age / v.ttl;
        // Rise over the first 20%, ease back down over the rest.
        let shape = if t < 0.2 { t / 0.2 } else { (1.0 - t) / 0.8 };
        let alpha = shape.clamp(0.0, 1.0) * v.peak_alpha;
        *border = BorderColor::all(Color::srgba(0.9, 0.08, 0.08, alpha));
    }
}

/// Float each damage numeral upward and fade it out, despawning when elapsed.
pub fn animate_damage_numerals(
    mut commands: Commands,
    time: Res<Time>,
    mut numerals: Query<(Entity, &mut DamageNumeral, &mut Node, &Children)>,
    mut texts: Query<&mut TextColor>,
) {
    for (entity, mut numeral, mut node, children) in &mut numerals {
        numeral.remaining -= time.delta_secs();
        if numeral.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = (numeral.remaining / numeral.total).clamp(0.0, 1.0); // 1 → 0
        node.top = Val::Px(numeral.base_top - (1.0 - frac) * DMG_NUMERAL_RISE);
        // Hold full opacity, then ease out over the final 50%.
        let alpha = (frac / 0.5).min(1.0);
        for child in children.iter() {
            if let Ok(mut tc) = texts.get_mut(child) {
                let c = tc.0.to_srgba();
                tc.0 = Color::srgba(c.red, c.green, c.blue, alpha);
            }
        }
    }
}

/// Fly each mana mote along its arc and fade it out, despawning when elapsed.
pub fn animate_mana_motes(
    mut commands: Commands,
    time: Res<Time>,
    mut motes: Query<(Entity, &mut ManaMote)>,
    mut gizmos: Gizmos<ImpactGizmos>,
) {
    for (entity, mut mote) in &mut motes {
        mote.age += time.delta_secs();
        if mote.age >= mote.ttl {
            commands.entity(entity).despawn();
            continue;
        }
        let t = (mote.age / mote.ttl).clamp(0.0, 1.0);
        let mut pos = mote.start.lerp(mote.end, t);
        pos.y += (t * TAU * 0.5).sin() * MANA_MOTE_LIFT; // half-sine arc
        let col = faded(mote.color, 1.0 - t);
        draw_ring(&mut gizmos, pos, 0.16, col);
        // A short cross through the centre gives the mote a brighter core than
        // the ring alone.
        gizmos.line(pos - Vec3::X * 0.12, pos + Vec3::X * 0.12, col);
        gizmos.line(pos - Vec3::Z * 0.12, pos + Vec3::Z * 0.12, col);
    }
}

fn faded(color: Color, k: f32) -> Color {
    let l = color.to_linear();
    LinearRgba::new(l.red * k, l.green * k, l.blue * k, l.alpha * k).into()
}

fn draw_ring(gizmos: &mut Gizmos<ImpactGizmos>, center: Vec3, r: f32, color: Color) {
    const SEGMENTS: usize = 28;
    for i in 0..SEGMENTS {
        let a0 = (i as f32 / SEGMENTS as f32) * TAU;
        let a1 = ((i + 1) as f32 / SEGMENTS as f32) * TAU;
        let p0 = center + Vec3::new(a0.cos() * r, 0.0, a0.sin() * r);
        let p1 = center + Vec3::new(a1.cos() * r, 0.0, a1.sin() * r);
        gizmos.line(p0, p1, color);
    }
}
