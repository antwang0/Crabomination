//! 3-D counter-coin visualization on battlefield permanents.
//!
//! Each `CounterType` on a `PermanentView.counters` entry gets a small
//! coloured cylinder ("coin") spawned as a child of the card entity.
//! Multiple counters of the same type stack vertically; multiple types
//! sit side-by-side along the card's local X axis.
//!
//! Each frame we despawn-and-respawn the coins for any card whose
//! counter mix has changed, keyed by the same logic as the badge-removal
//! pass: cheap (≤ ~10 active permanents on a typical board) and avoids
//! the complexity of in-place reconciliation.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::{CardId, CounterType};

use crate::card::{BattlefieldCard, CARD_THICKNESS};
use crate::card::GameCardId;
use crate::net_plugin::CurrentView;

/// Shared cylinder mesh + per-counter-type material handles used to
/// spawn coin entities. Initialised on app startup alongside the card
/// mesh assets.
#[derive(Resource)]
pub struct CounterCoinAssets {
    pub coin_mesh: Handle<Mesh>,
    /// Slightly larger, flatter cylinder spawned just behind each coin
    /// to draw a bright rim ("outline") around it for contrast.
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
    pub plus_one_plus_one: Handle<StandardMaterial>,
    pub minus_one_minus_one: Handle<StandardMaterial>,
    pub loyalty: Handle<StandardMaterial>,
    pub charge: Handle<StandardMaterial>,
    pub stun: Handle<StandardMaterial>,
    pub time: Handle<StandardMaterial>,
    pub poison: Handle<StandardMaterial>,
    pub energy: Handle<StandardMaterial>,
    pub generic: Handle<StandardMaterial>,
}

/// Marker on each spawned coin-mesh entity. Stores which card it belongs
/// to (so the system can match against the engine state) and the kind so
/// stacking colours stay consistent across frames.
#[derive(Component)]
pub struct CounterCoin {
    #[allow(dead_code)]
    pub card_id: CardId,
    pub kind: CounterType,
}

/// Geometry constants for coin layout in card-local space. Cards are
/// `CARD_WIDTH × CARD_HEIGHT` rectangles oriented with their local +Z
/// pointing toward the front face; once the card is laid flat on the
/// battlefield via `rotation_x(-π/2)` the +Z direction becomes world +Y
/// (above the table), so a coin sitting at local +Z naturally floats
/// above the card.
const COIN_RADIUS: f32 = 0.24;
const COIN_HEIGHT: f32 = 0.07;
const COIN_GAP_X: f32 = 0.10;
const COIN_GAP_Y: f32 = 0.04;
const COIN_BASE_Z: f32 = CARD_THICKNESS / 2.0 + 0.05; // floats clearly above the front face
/// Outline ring: a touch wider than the coin and a touch shorter, so it
/// protrudes radially (a visible rim) without poking in front of the
/// coin's face (which would z-fight).
const OUTLINE_RADIUS: f32 = COIN_RADIUS + 0.055;
const OUTLINE_HEIGHT: f32 = COIN_HEIGHT * 0.7;
/// Vertical spacing between stacked coins of the *same* type. The coins
/// face the camera (their circular face lies in the card plane), so the
/// stack must step by roughly a coin *diameter* — stepping by the coin's
/// thickness (`COIN_HEIGHT`) made a 3-counter "tower" pile almost
/// entirely on top of itself. A hair under a full diameter leaves a
/// small rim of each lower coin visible so the stack is countable.
const COIN_STACK_STEP: f32 = COIN_RADIUS * 1.7 + COIN_GAP_Y;
/// Peak emissive magnitude (linear) for a coin fill. Pushed past the bloom
/// prefilter threshold (~1.0, see `RenderQuality::bloom`) so each coin glows
/// its own colour on HDR tiers. Normalised per-hue (see the `mat` closure) so
/// a dark-green +1/+1 and a bright-gold loyalty bloom at the same intensity.
/// On Low (no HDR/bloom) this is just a slightly brighter self-lit coin.
const COIN_GLOW: f32 = 1.7;

pub fn init_counter_coin_assets(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let coin_mesh = meshes.add(Cylinder::new(COIN_RADIUS, COIN_HEIGHT));
    let outline_mesh = meshes.add(Cylinder::new(OUTLINE_RADIUS, OUTLINE_HEIGHT));

    // Bright near-white rim so every coin's edge stands out crisply
    // regardless of the card art behind it. Built before the `mat`
    // closure below, which mutably borrows `materials` for its lifetime.
    let outline_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.96, 0.96, 0.98),
        perceptual_roughness: 0.5,
        metallic: 0.0,
        emissive: LinearRgba::new(0.30, 0.30, 0.34, 1.0),
        ..default()
    });

    // Deep, saturated coin fills — darker than before so they read as
    // solid chips rather than washing out against bright card art. Each fill
    // carries a self-coloured emissive normalised to `COIN_GLOW`, so on HDR
    // tiers the coin glows its own colour (and trips bloom) at a brightness
    // that's consistent across hues regardless of how dark the base fill is;
    // the bright outline ring (below) still supplies the hard edge contrast.
    let mut mat = |color: Color, metallic: f32| -> Handle<StandardMaterial> {
        let lin = color.to_linear();
        // Scale the hue so its brightest channel hits COIN_GLOW; keeps a deep
        // fill and a bright fill blooming at the same intensity.
        let peak = lin.red.max(lin.green).max(lin.blue).max(1e-4);
        let g = COIN_GLOW / peak;
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.40,
            metallic,
            emissive: LinearRgba::new(lin.red * g, lin.green * g, lin.blue * g, 1.0),
            ..default()
        })
    };

    commands.insert_resource(CounterCoinAssets {
        coin_mesh,
        outline_mesh,
        outline_material,
        // +1/+1 → deep green; −1/−1 → deep red.
        plus_one_plus_one: mat(Color::srgb(0.10, 0.45, 0.16), 0.30),
        minus_one_minus_one: mat(Color::srgb(0.50, 0.08, 0.08), 0.30),
        // Loyalty → dark gold (high metallic for a coin look).
        loyalty: mat(Color::srgb(0.62, 0.48, 0.06), 0.85),
        charge: mat(Color::srgb(0.10, 0.48, 0.60), 0.45),
        stun: mat(Color::srgb(0.44, 0.16, 0.60), 0.30),
        time: mat(Color::srgb(0.62, 0.36, 0.08), 0.40),
        poison: mat(Color::srgb(0.16, 0.34, 0.08), 0.10),
        energy: mat(Color::srgb(0.12, 0.30, 0.66), 0.55),
        generic: mat(Color::srgb(0.34, 0.34, 0.42), 0.30),
    });
}

fn material_for(kind: CounterType, assets: &CounterCoinAssets) -> Handle<StandardMaterial> {
    match kind {
        CounterType::PlusOnePlusOne => assets.plus_one_plus_one.clone(),
        CounterType::MinusOneMinusOne => assets.minus_one_minus_one.clone(),
        CounterType::Loyalty => assets.loyalty.clone(),
        CounterType::Charge => assets.charge.clone(),
        CounterType::Stun => assets.stun.clone(),
        CounterType::Time => assets.time.clone(),
        CounterType::Poison => assets.poison.clone(),
        CounterType::Energy => assets.energy.clone(),
        _ => assets.generic.clone(),
    }
}

/// Sort key for counter types — pins +1/+1 / −1/−1 to the front (most
/// commonly inspected during play), loyalty next, the rest in declaration
/// order. Keeps coin rows stable across frames.
fn sort_key(kind: CounterType) -> u8 {
    match kind {
        CounterType::PlusOnePlusOne => 0,
        CounterType::MinusOneMinusOne => 1,
        CounterType::Loyalty => 2,
        CounterType::Charge => 3,
        CounterType::Stun => 4,
        CounterType::Time => 5,
        CounterType::Poison => 6,
        CounterType::Energy => 7,
        _ => 8,
    }
}

/// Reconcile the 3-D counter coins with the engine's battlefield view.
/// On every frame we compute the *desired* coin set per card and the
/// *current* coin set from existing `CounterCoin` entities. Cards whose
/// signature changed (any counter added, removed, or its count moved) are
/// fully rebuilt — for everyone else we leave the coins alone so the
/// existing entities stay parented correctly through tap/play animations.
pub fn sync_counter_coins(
    mut commands: Commands,
    view: Res<CurrentView>,
    assets: Option<Res<CounterCoinAssets>>,
    bf_cards: Query<(Entity, &GameCardId), With<BattlefieldCard>>,
    existing: Query<(Entity, &ChildOf, &CounterCoin)>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(assets) = assets else { return };

    // Build CardId → battlefield-entity lookup.
    let mut card_entity: HashMap<CardId, Entity> = HashMap::new();
    for (e, gid) in &bf_cards {
        card_entity.insert(gid.0, e);
    }

    // Existing coin signature per parent entity, used to skip rebuilds when
    // the counter list hasn't changed.
    let mut existing_sig: HashMap<Entity, Vec<(CounterType, u32)>> = HashMap::new();
    let mut existing_entities: HashMap<Entity, Vec<Entity>> = HashMap::new();
    {
        // First pass: gather counts of (parent, kind).
        let mut accum: HashMap<(Entity, CounterType), u32> = HashMap::new();
        for (coin_e, child_of, coin) in &existing {
            *accum.entry((child_of.parent(), coin.kind)).or_default() += 1;
            existing_entities.entry(child_of.parent()).or_default().push(coin_e);
        }
        for ((parent, kind), n) in accum {
            existing_sig.entry(parent).or_default().push((kind, n));
        }
        // Sort each parent's signature for stable comparison.
        for sig in existing_sig.values_mut() {
            sig.sort_by_key(|(k, _)| sort_key(*k));
        }
    }

    let mut all_battlefield_entities: std::collections::HashSet<Entity> =
        std::collections::HashSet::new();

    for p in &cv.battlefield {
        let Some(&parent) = card_entity.get(&p.id) else { continue };
        all_battlefield_entities.insert(parent);

        // Build desired counter list (sorted by sort_key, dropping zeros).
        let mut desired: Vec<(CounterType, u32)> = p
            .counters
            .iter()
            .filter(|(_, n)| *n > 0)
            .map(|(k, n)| (*k, *n))
            .collect();
        desired.sort_by_key(|(k, _)| sort_key(*k));

        // Skip if the signature already matches.
        if existing_sig.get(&parent).map(|v| v.as_slice()) == Some(desired.as_slice()) {
            continue;
        }

        // Despawn old coins for this card and respawn the desired set.
        if let Some(coins) = existing_entities.remove(&parent) {
            for e in coins {
                commands.entity(e).despawn();
            }
        }

        if desired.is_empty() {
            continue;
        }

        commands.entity(parent).with_children(|p_builder| {
            // Lay out coin types along the card's local X axis (one
            // column per type), stacking duplicates upward in the
            // card's local Y axis so a creature with `+1/+1 ×3` shows
            // a 3-coin tower and not three separate columns.
            let n_types = desired.len() as f32;
            let row_width = n_types * (COIN_RADIUS * 2.0 + COIN_GAP_X) - COIN_GAP_X;
            let start_x = -row_width * 0.5 + COIN_RADIUS;
            for (col, (kind, count)) in desired.iter().enumerate() {
                let x = start_x + col as f32 * (COIN_RADIUS * 2.0 + COIN_GAP_X);
                // Cap the visible stack so a Cosmogoyf with 30 +1/+1's
                // doesn't tower off the table; a label could be added
                // later for "stack-of-N" cases.
                let n = (*count).min(8);
                for i in 0..n {
                    let y = COIN_STACK_STEP * i as f32;
                    let coin_rot = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
                    p_builder
                        .spawn((
                            Mesh3d(assets.coin_mesh.clone()),
                            MeshMaterial3d(material_for(*kind, &assets)),
                            // Local space: card's surface lies in XY, +Z
                            // is out of the front face. After the
                            // parent's rotation_x(-π/2) the coins float
                            // above the card on the table.
                            Transform::from_xyz(x, y, COIN_BASE_Z).with_rotation(coin_rot),
                            CounterCoin { card_id: p.id, kind: *kind },
                        ))
                        .with_children(|coin| {
                            // Outline rim as a child of the coin (so it
                            // shares the coin's despawn + isn't counted
                            // in the rebuild signature). Local -Y maps to
                            // world −Z under the coin's rotation_x(90°),
                            // tucking the rim just behind the coin face.
                            coin.spawn((
                                Mesh3d(assets.outline_mesh.clone()),
                                MeshMaterial3d(assets.outline_material.clone()),
                                Transform::from_xyz(0.0, -0.012, 0.0),
                                Pickable::IGNORE,
                            ));
                        });
                }
            }
        });
    }

    // Despawn any leftover coins whose parent no longer maps to a
    // battlefield card (e.g. the card was destroyed/exiled).
    for (parent, coin_entities) in existing_entities {
        if !all_battlefield_entities.contains(&parent) {
            for e in coin_entities {
                commands.entity(e).despawn();
            }
        }
    }
}

// ── Counter type+count labels ─────────────────────────────────────────────────
//
// The coins convey a counter's *kind* by colour, but a bare disc doesn't say
// what it is or how many there are (the stack is capped at 8 and never showed a
// number). A small screen-space label per counter type — "+1/+1 ×3", coloured
// to match the coin — makes each coin self-explanatory and the count exact.
// Mirrors `pt_label::sync_pt_labels`' reproject-and-reconcile pattern.

/// Below default-z UI so peek popups / tooltips / modals draw over it.
const COUNTER_LABEL_Z: i32 = -1;

/// Short, human-readable token naming a counter kind for the count label.
fn counter_token(kind: CounterType) -> &'static str {
    match kind {
        CounterType::PlusOnePlusOne => "+1/+1",
        CounterType::MinusOneMinusOne => "-1/-1",
        CounterType::MinusZeroMinusOne => "-0/-1",
        CounterType::MinusOneMinusZero => "-1/-0",
        CounterType::Loyalty => "Loyalty",
        CounterType::Charge => "Charge",
        CounterType::Stun => "Stun",
        CounterType::Time => "Time",
        CounterType::Poison => "Poison",
        CounterType::Energy => "Energy",
        CounterType::Lore => "Lore",
        CounterType::Fade => "Fade",
        CounterType::Level => "Level",
        CounterType::Experience => "XP",
        CounterType::Shield => "Shield",
        _ => "Counter",
    }
}

/// Bright, legible text colour matching each counter's coin fill (the coin
/// fills are deliberately dark for solidity; these are their readable twins).
fn counter_label_color(kind: CounterType) -> Color {
    match kind {
        CounterType::PlusOnePlusOne => Color::srgb(0.45, 0.95, 0.50),
        CounterType::MinusOneMinusOne
        | CounterType::MinusZeroMinusOne
        | CounterType::MinusOneMinusZero => Color::srgb(0.98, 0.48, 0.48),
        CounterType::Loyalty => Color::srgb(0.96, 0.82, 0.32),
        CounterType::Charge => Color::srgb(0.42, 0.85, 0.96),
        CounterType::Stun => Color::srgb(0.82, 0.56, 0.96),
        CounterType::Time => Color::srgb(0.96, 0.74, 0.42),
        CounterType::Poison => Color::srgb(0.56, 0.86, 0.42),
        CounterType::Energy => Color::srgb(0.46, 0.66, 0.98),
        _ => Color::srgb(0.86, 0.86, 0.92),
    }
}

/// Label text for a counter kind + count: "+1/+1 ×3", or just the token when
/// there's a single counter (the "×1" is noise).
fn counter_label_text(kind: CounterType, count: u32) -> String {
    let token = counter_token(kind);
    if count > 1 {
        format!("{token} ×{count}")
    } else {
        token.to_string()
    }
}

/// Project a card's coin-area world anchor to a viewport pixel, stacking each
/// counter type's label on its own line (`row`).
fn label_anchor(
    camera: &Camera,
    cam_xform: &GlobalTransform,
    world: Vec3,
    row: usize,
) -> Option<(f32, f32)> {
    camera
        .world_to_viewport(cam_xform, world)
        .ok()
        .map(|v| (v.x - 26.0, v.y - 12.0 + row as f32 * 20.0))
}

/// Screen-space "<type> ×N" label tied to a battlefield card's counter of a
/// given kind. One per (card, kind) so each type gets its own coloured line.
#[derive(Component)]
pub struct CounterLabel {
    pub card_id: CardId,
    pub kind: CounterType,
}

/// Reconcile counter labels with the engine view (see module note above).
#[allow(clippy::type_complexity)]
pub fn sync_counter_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<crate::theme::UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    mut labels: Query<(Entity, &CounterLabel, &mut Node, &mut Text, &mut TextColor)>,
) {
    let Some(cv) = &view.0 else {
        for (e, _, _, _, _) in &mut labels {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    // card_id → world anchor floating just above the card centre (by the coins).
    let anchor_local = Vec3::new(0.0, 0.0, COIN_BASE_Z + 0.1);
    let mut card_anchor: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        card_anchor.insert(gid.0, gtf.transform_point(anchor_local));
    }

    // desired (card, kind) → (count, row). Row orders multiple counter types
    // vertically in the same order as their coin columns (`sort_key`).
    let mut desired: HashMap<(CardId, CounterType), (u32, usize)> = HashMap::new();
    for p in &cv.battlefield {
        if !card_anchor.contains_key(&p.id) {
            continue;
        }
        let mut kinds: Vec<(CounterType, u32)> = p
            .counters
            .iter()
            .filter(|(_, n)| *n > 0)
            .map(|(k, n)| (*k, *n))
            .collect();
        kinds.sort_by_key(|(k, _)| sort_key(*k));
        for (row, (k, n)) in kinds.into_iter().enumerate() {
            desired.insert((p.id, k), (n, row));
        }
    }

    // Update existing labels; despawn any no longer present.
    let mut seen: HashSet<(CardId, CounterType)> = HashSet::new();
    for (e, label, mut node, mut text, mut color) in &mut labels {
        match desired.get(&(label.card_id, label.kind)) {
            Some(&(count, row)) => {
                seen.insert((label.card_id, label.kind));
                if let Some(world) = card_anchor.get(&label.card_id).copied()
                    && let Some((x, y)) = label_anchor(camera, cam_xform, world, row)
                {
                    node.display = Display::Flex;
                    node.left = Val::Px(x);
                    node.top = Val::Px(y);
                } else {
                    node.display = Display::None;
                }
                *text = Text::new(counter_label_text(label.kind, count));
                *color = TextColor(counter_label_color(label.kind));
            }
            None => {
                commands.entity(e).despawn();
            }
        }
    }

    // Spawn labels for newly-present (card, kind) pairs.
    for ((id, kind), (count, row)) in desired {
        if seen.contains(&(id, kind)) {
            continue;
        }
        let (left, top) = card_anchor
            .get(&id)
            .copied()
            .and_then(|world| label_anchor(camera, cam_xform, world, row))
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            CounterLabel { card_id: id, kind },
            Text::new(counter_label_text(kind, count)),
            ui_fonts.tf(14.0),
            TextColor(counter_label_color(kind)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(COUNTER_LABEL_Z),
            crate::systems::game_ui::InGameRoot,
        ));
    }
}
