use std::f32::consts::PI;

use bevy::prelude::*;

use super::components::{
    BackFaceMesh, Card, CardFrontTexture, CardHighlightAssets, CardHoverLift, CardMeshAssets,
    FrontFaceMesh,
    GraveyardPile, PileHovered, PlayerTargetZone, CARD_THICKNESS,
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

    let back_texture: Handle<Image> = asset_server.load("cards/cardback.png");
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

    // One clickable target zone per opponent. The viewer doesn't need a
    // self-target zone (spells targeting yourself land on your portrait via
    // a different code path).
    for seat in 0..n_seats {
        if seat == viewer_seat {
            continue;
        }
        let pos = player_target_zone_position(seat, viewer_seat, n_seats);
        commands
            .spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(8.0, 3.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.0, 0.0, 0.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(pos),
                Visibility::default(),
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
