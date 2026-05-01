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
    pub card_id: CardId,
    pub kind: CounterType,
}

/// Geometry constants for coin layout in card-local space. Cards are
/// `CARD_WIDTH × CARD_HEIGHT` rectangles oriented with their local +Z
/// pointing toward the front face; once the card is laid flat on the
/// battlefield via `rotation_x(-π/2)` the +Z direction becomes world +Y
/// (above the table), so a coin sitting at local +Z naturally floats
/// above the card.
const COIN_RADIUS: f32 = 0.18;
const COIN_HEIGHT: f32 = 0.05;
const COIN_GAP_X: f32 = 0.08;
const COIN_GAP_Y: f32 = 0.03;
const COIN_BASE_Z: f32 = CARD_THICKNESS / 2.0 + 0.01; // just above the front face

pub fn init_counter_coin_assets(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let coin_mesh = meshes.add(Cylinder::new(COIN_RADIUS, COIN_HEIGHT));

    let mut mat = |color: Color, metallic: f32| -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.45,
            metallic,
            ..default()
        })
    };

    commands.insert_resource(CounterCoinAssets {
        coin_mesh,
        // +1/+1 → bright green; −1/−1 → muted red.
        plus_one_plus_one: mat(Color::srgb(0.24, 0.78, 0.32), 0.30),
        minus_one_minus_one: mat(Color::srgb(0.74, 0.22, 0.22), 0.30),
        // Loyalty → polished gold (high metallic for a coin look).
        loyalty: mat(Color::srgb(0.98, 0.82, 0.20), 0.85),
        charge: mat(Color::srgb(0.30, 0.80, 0.92), 0.45),
        stun: mat(Color::srgb(0.74, 0.36, 0.92), 0.30),
        time: mat(Color::srgb(0.94, 0.62, 0.22), 0.40),
        poison: mat(Color::srgb(0.32, 0.55, 0.18), 0.10),
        energy: mat(Color::srgb(0.30, 0.55, 0.95), 0.55),
        generic: mat(Color::srgb(0.70, 0.70, 0.78), 0.30),
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
                    let y = (COIN_HEIGHT + COIN_GAP_Y) * i as f32;
                    p_builder.spawn((
                        Mesh3d(assets.coin_mesh.clone()),
                        MeshMaterial3d(material_for(*kind, &assets)),
                        // Local space: card's surface lies in XY, +Z is
                        // out of the front face. After the parent's
                        // rotation_x(-π/2) the coins float above the
                        // card on the table.
                        Transform::from_xyz(x, y, COIN_BASE_Z)
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                        CounterCoin { card_id: p.id, kind: *kind },
                    ));
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
