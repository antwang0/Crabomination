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

use std::collections::HashMap;

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
    // solid chips rather than washing out against bright card art. A
    // faint self-coloured emissive keeps them from going flat-black in
    // shadow; the bright outline ring (below) supplies the real contrast.
    let mut mat = |color: Color, metallic: f32| -> Handle<StandardMaterial> {
        let lin = color.to_linear();
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.40,
            metallic,
            emissive: LinearRgba::new(lin.red * 0.12, lin.green * 0.12, lin.blue * 0.12, 1.0),
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
