use std::f32::consts::PI;

use bevy::prelude::*;

use super::components::{
    BackFaceMesh, Card, CardFrontTexture, CardHighlightAssets, CardHoverLift, CardMeshAssets,
    FrontFaceMesh,
    GraveyardPile, PileHovered, PlayerCrest, PlayerCrestRing, PlayerIcon, PlayerTargetZone,
    CARD_THICKNESS,
};
use super::layout::{back_face_rotation, graveyard_position, player_target_zone_position};
use super::mesh::{card_border_mesh, card_mesh};
use super::observers::{on_card_out, on_card_over, on_zone_out, on_zone_over};

use crate::game::GraveyardBrowserState;
use crate::scryfall;

/// Initialize shared mesh/material assets and spawn always-present scaffolding
/// (per-seat graveyard piles and per-opponent target zones). Per-card entities
/// are spawned on-demand by `sync_game_visuals` once a `ClientView` arrives.
///
/// `n_seats` and `viewer_seat` determine the per-seat layout: the viewer sits
/// at the front of the table; each opponent gets a graveyard pile and a
/// click-to-target zone at the back of the table.
pub fn init_shared_assets(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    segments: usize,
    n_seats: usize,
    viewer_seat: usize,
) {
    let card_mesh_handle = card_mesh(meshes, segments);
    let border_mesh_handle = card_border_mesh(meshes, segments);

    // Lives at the asset-dir root rather than under `cards/` because
    // `cards/` is gitignored (downloaded card art).
    let back_texture: Handle<Image> = asset_server.load("cardback.png");
    let back_material = materials.add(StandardMaterial {
        base_color_texture: Some(back_texture),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    let border_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.0),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.insert_resource(CardHighlightAssets {
        border_mesh: border_mesh_handle,
        border_material,
    });
    commands.insert_resource(CardMeshAssets {
        card_mesh: card_mesh_handle.clone(),
        back_material: back_material.clone(),
    });

    // One graveyard pile per seat. Hidden until non-empty.
    for seat in 0..n_seats {
        let pos = graveyard_position(seat, viewer_seat, n_seats);
        let rot = back_face_rotation(seat, viewer_seat);
        commands
            .spawn((
                Mesh3d(card_mesh_handle.clone()),
                MeshMaterial3d(back_material.clone()),
                Transform::from_translation(pos).with_rotation(rot),
                Visibility::Hidden,
                GraveyardPile { owner: seat },
                CardHoverLift {
                    current_lift: 0.0,
                    target_lift: 0.0,
                    base_translation: pos,
                },
            ))
            .observe(on_pile_over)
            .observe(on_pile_out)
            .observe(on_graveyard_click);
    }

    // Per-seat **player crest** — a small composite avatar sitting on
    // the table just past the right edge of each player's hand fan
    // (see `player_target_zone_position`).
    //
    // Anatomy:
    //   * **Halo ring** (bottom): a slightly larger, very flat cylinder
    //     under the main disc. Its material is mutated each frame by
    //     `update_player_crest_ring` to advertise seat state
    //     (yellow = legal click target, red = threatened, white =
    //     priority, gold = active player, dim ambient = idle).
    //   * **Disc**: the main coloured puck. Blue for the viewer, red for
    //     opponents. Carries the [`PlayerTargetZone`] click hit-region
    //     for *every* seat (including the viewer's own — legal for
    //     self-target spells like Healing Salve) and is the world-space
    //     anchor that attacker combat-lurch animations / stack arrows
    //     point at.
    //   * **Floating life numeral**: spawned separately by
    //     `setup_player_life_labels` as a UI text node, then re-projected
    //     to follow the disc's world position each frame.
    //
    // Sized larger than the historical disc (radius 1.8 vs. 1.4) so the
    // crest reads clearly from the slightly tilted top-down camera and
    // has room to anchor the life label.
    const CREST_DISC_RADIUS: f32 = 1.8;
    const CREST_DISC_HEIGHT: f32 = 0.30;
    const CREST_RING_RADIUS: f32 = 2.35;
    const CREST_RING_HEIGHT: f32 = 0.05;

    let disc_mesh = meshes.add(
        Cylinder::new(CREST_DISC_RADIUS, CREST_DISC_HEIGHT)
            .mesh()
            .resolution(48),
    );
    let ring_mesh = meshes.add(
        Cylinder::new(CREST_RING_RADIUS, CREST_RING_HEIGHT)
            .mesh()
            .resolution(64),
    );

    for seat in 0..n_seats {
        let pos = player_target_zone_position(seat, viewer_seat, n_seats);
        // Disc Y sits half its height above the table so the bottom face
        // rests on the ground plane; ring Y sits just below the disc.
        let disc_pos = Vec3::new(pos.x, CREST_DISC_HEIGHT * 0.5 + 0.01, pos.z);
        let ring_pos = Vec3::new(pos.x, CREST_RING_HEIGHT * 0.5 + 0.005, pos.z);

        let disc_base_color = if seat == viewer_seat {
            Color::srgb(0.28, 0.50, 0.90)
        } else {
            Color::srgb(0.85, 0.28, 0.28)
        };
        let disc_emissive = if seat == viewer_seat {
            LinearRgba::new(0.10, 0.18, 0.40, 1.0)
        } else {
            LinearRgba::new(0.40, 0.10, 0.10, 1.0)
        };
        let disc_mat = materials.add(StandardMaterial {
            base_color: disc_base_color,
            emissive: disc_emissive,
            perceptual_roughness: 0.55,
            metallic: 0.05,
            ..default()
        });

        // Ring starts at a dim neutral; `update_player_crest_ring`
        // overwrites both base_color and emissive each frame.
        let ring_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.20, 0.20, 0.24, 1.0),
            emissive: LinearRgba::new(0.05, 0.05, 0.07, 1.0),
            perceptual_roughness: 0.70,
            metallic: 0.0,
            ..default()
        });

        // Ring sits below the disc. Pickable::IGNORE so clicks pass
        // through to the disc itself (which is the canonical hit region).
        commands.spawn((
            Mesh3d(ring_mesh.clone()),
            MeshMaterial3d(ring_mat),
            Transform::from_translation(ring_pos),
            Visibility::default(),
            PlayerCrestRing { seat },
            Pickable::IGNORE,
        ));

        // Disc carries both legacy `PlayerIcon` (consumed by combat-
        // lurch / animation anchors that already look it up) and the
        // new `PlayerCrest` (single source of truth for "this is the
        // player's avatar entity"). `PlayerTargetZone` is now on
        // every seat — the viewer's disc accepts clicks for spells
        // that legally target you.
        commands
            .spawn((
                Mesh3d(disc_mesh.clone()),
                MeshMaterial3d(disc_mat),
                Transform::from_translation(disc_pos),
                Visibility::default(),
                PlayerIcon { seat },
                PlayerCrest { seat },
                PlayerTargetZone(seat),
            ))
            .observe(on_zone_over)
            .observe(on_zone_out);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_single_card(
    commands: &mut Commands,
    card_mesh: &Handle<Mesh>,
    front_material: Handle<StandardMaterial>,
    back_material: Handle<StandardMaterial>,
    transform: Transform,
    game_card_id: super::components::GameCardId,
    card_name: &str,
    base_pos: Vec3,
) -> Entity {
    commands
        .spawn((
            transform,
            Visibility::default(),
            Card,
            game_card_id,
            CardFrontTexture(scryfall::card_asset_path(card_name)),
            CardHoverLift {
                current_lift: 0.0,
                target_lift: 0.0,
                base_translation: base_pos,
            },
        ))
        .with_children(|parent| {
            // Front face (+Z)
            parent
                .spawn((
                    Mesh3d(card_mesh.clone()),
                    MeshMaterial3d(front_material),
                    Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
                    FrontFaceMesh,
                ))
                .observe(on_card_over)
                .observe(on_card_out);
            // Back face (-Z)
            parent
                .spawn((
                    Mesh3d(card_mesh.clone()),
                    MeshMaterial3d(back_material),
                    Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                        .with_rotation(Quat::from_rotation_y(PI)),
                    BackFaceMesh,
                ))
                .observe(on_card_over)
                .observe(on_card_out);
        })
        .id()
}

fn on_pile_over(ev: On<Pointer<Over>>, mut commands: Commands) {
    commands.entity(ev.entity).insert(PileHovered);
}

fn on_pile_out(ev: On<Pointer<Out>>, mut commands: Commands) {
    commands.entity(ev.entity).remove::<PileHovered>();
}

/// Click on any graveyard pile toggles the browser to that pile's owner.
fn on_graveyard_click(
    ev: On<Pointer<Click>>,
    piles: Query<&GraveyardPile>,
    mut browser: ResMut<GraveyardBrowserState>,
) {
    let Ok(pile) = piles.get(ev.entity) else { return };
    if browser.open && browser.owner == pile.owner {
        browser.open = false;
    } else {
        browser.open = true;
        browser.owner = pile.owner;
    }
}

pub fn card_front_material(
    name: &str,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
    let asset_path = scryfall::card_asset_path(name);
    let texture: Handle<Image> = asset_server.load(&asset_path);
    materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    })
}

/// Build a textured material for an MDFC back-face image. Loads from the
/// `_back`-suffixed asset path so stale front-face downloads with the
/// same name don't collide.
pub fn card_back_face_material(
    name: &str,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
    let asset_path = scryfall::card_back_face_asset_path(name);
    let texture: Handle<Image> = asset_server.load(&asset_path);
    materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    })
}
