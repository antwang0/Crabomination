use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::components::{
    Card, CardFrontTexture, CardHighlightAssets, CardHoverLift, DeckCard, CARD_THICKNESS,
    DECK_CARD_Y_STEP, DECK_POSITION, DECK_SIZE,
};
use super::mesh::{card_border_mesh, card_mesh};
use super::observers::{on_card_out, on_card_over};

const DECK_CARD_FRONTS: &[&str] = &[
    "cards/black_lotus.png",
    "cards/sheoldred_the_apocalypse.png",
];

pub fn spawn_deck(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    let card_mesh_handle = card_mesh(meshes);

    let back_texture: Handle<Image> = asset_server.load("cards/cardback.png");
    let back_material_shared = materials.add(StandardMaterial {
        base_color_texture: Some(back_texture),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    for i in 0..DECK_SIZE {
        let y = i as f32 * DECK_CARD_Y_STEP + 0.01;
        let front_path = DECK_CARD_FRONTS[i % DECK_CARD_FRONTS.len()];
        let front_texture: Handle<Image> = asset_server.load(front_path);
        let front_material = materials.add(StandardMaterial {
            base_color_texture: Some(front_texture),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        });

        commands
            .spawn((
                Transform::from_xyz(DECK_POSITION.x, y, DECK_POSITION.z)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2) * Quat::from_rotation_z(PI)),
                Visibility::default(),
                Card,
                DeckCard { index: i },
                CardFrontTexture(front_path.to_string()),
                CardHoverLift {
                    current_lift: 0.0,
                    target_lift: 0.0,
                    base_translation: Vec3::new(DECK_POSITION.x, y, DECK_POSITION.z),
                },
            ))
            .with_children(|parent| {
                // Front face (+Z)
                parent
                    .spawn((
                        Mesh3d(card_mesh_handle.clone()),
                        MeshMaterial3d(front_material),
                        Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
                    ))
                    .observe(on_card_over)
                    .observe(on_card_out);
                // Back face (-Z)
                parent
                    .spawn((
                        Mesh3d(card_mesh_handle.clone()),
                        MeshMaterial3d(back_material_shared.clone()),
                        Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                            .with_rotation(Quat::from_rotation_y(PI)),
                    ))
                    .observe(on_card_over)
                    .observe(on_card_out);
            });
    }
}

pub fn spawn_cards(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    use std::f32::consts::FRAC_PI_2;

    use super::components::{CARD_HEIGHT, CARD_WIDTH};

    let card_mesh_handle = card_mesh(meshes);
    let border_mesh_handle = card_border_mesh(meshes);

    let border_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.0),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.insert_resource(CardHighlightAssets {
        border_mesh: border_mesh_handle,
        border_material: border_material.clone(),
    });

    let card_definitions = vec![
        ("cards/black_lotus.png", "cards/cardback.png"),
        ("cards/sheoldred_the_apocalypse.png", "cards/cardback.png"),
    ];

    let spacing_x = CARD_WIDTH * 1.1;
    let spacing_z = CARD_HEIGHT * 1.1;

    for (i, (front_path, back_path)) in card_definitions.iter().enumerate() {
        let front_texture: Handle<Image> = asset_server.load(*front_path);
        let back_texture: Handle<Image> = asset_server.load(*back_path);

        let front_material = materials.add(StandardMaterial {
            base_color_texture: Some(front_texture),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        });

        let back_material = materials.add(StandardMaterial {
            base_color_texture: Some(back_texture),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        });

        let x = (i % 4) as f32 * spacing_x - 1.5;
        let z = (i / 4) as f32 * spacing_z - 1.0;

        commands
            .spawn((
                Transform::from_xyz(x, 0.01, z)
                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                Visibility::default(),
                Card,
                CardFrontTexture(front_path.to_string()),
                CardHoverLift {
                    current_lift: 0.0,
                    target_lift: 0.0,
                    base_translation: Vec3::new(x, 0.01, z),
                },
            ))
            .with_children(|parent| {
                // Front face (facing +Z in local space)
                parent
                    .spawn((
                        Mesh3d(card_mesh_handle.clone()),
                        MeshMaterial3d(front_material),
                        Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
                    ))
                    .observe(on_card_over)
                    .observe(on_card_out);
                // Back face (facing -Z, rotated 180° around Y)
                parent
                    .spawn((
                        Mesh3d(card_mesh_handle.clone()),
                        MeshMaterial3d(back_material),
                        Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                            .with_rotation(Quat::from_rotation_y(PI)),
                    ))
                    .observe(on_card_over)
                    .observe(on_card_out);
            });
    }
}
