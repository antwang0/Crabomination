use bevy::prelude::*;

pub const CARD_WIDTH: f32 = 3.0;
pub const CARD_HEIGHT: f32 = CARD_WIDTH * 88.0 / 63.0;
const CARD_THICKNESS: f32 = 0.02;

#[derive(Component)]
pub struct Card;

/// Tracks an in-progress flip animation. `progress` goes from 0.0 to 1.0.
#[derive(Component)]
pub struct CardFlipAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_rotation: Quat,
    pub end_rotation: Quat,
    pub start_y: f32,
}

pub fn create_card_mesh() -> Mesh {
    Mesh::from(Rectangle::new(CARD_WIDTH, CARD_HEIGHT))
}

pub fn spawn_cards(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    let card_mesh = meshes.add(create_card_mesh());

    let card_definitions = vec![
        ("cards/black_lotus.png", "cards/cardback.png"),
    ];

    let spacing_x = CARD_WIDTH * 1.1;
    let spacing_z = CARD_HEIGHT * 1.1;

    for (i, (front_path, back_path)) in card_definitions.iter().enumerate() {
        let front_texture: Handle<Image> = asset_server.load(*front_path);
        let back_texture: Handle<Image> = asset_server.load(*back_path);

        let front_material = materials.add(StandardMaterial {
            base_color_texture: Some(front_texture),
            perceptual_roughness: 0.6,
            metallic: 0.2,
            ..default()
        });

        let back_material = materials.add(StandardMaterial {
            base_color_texture: Some(back_texture),
            perceptual_roughness: 0.6,
            metallic: 0.2,
            ..default()
        });

        let x = (i % 4) as f32 * spacing_x - 1.5;
        let z = (i / 4) as f32 * spacing_z - 1.0;

        commands.spawn((
            Transform::from_xyz(x, 0.01, z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            Visibility::default(),
            Card,
        )).with_children(|parent| {
            // Front face (facing +Z in local space)
            parent.spawn((
                Mesh3d(card_mesh.clone()),
                MeshMaterial3d(front_material),
                Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
            ));
            // Back face (facing -Z in local space, rotated 180° around Y)
            parent.spawn((
                Mesh3d(card_mesh.clone()),
                MeshMaterial3d(back_material),
                Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ));
        });
    }
}
