use std::f32::consts::PI;

use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::{anti_alias::smaa::Smaa, color::palettes::basic::SILVER, prelude::*};

mod card;
use card::{spawn_cards, Card, CardFlipAnimation, CARD_WIDTH};

const LIGHT_SHADOW_MAP_SIZE: usize = 8192;
const FLIP_SPEED: f32 = 1.5;

#[derive(Component)]
struct RotateButton;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .add_systems(Startup, setup)
        .add_systems(Update, (trigger_rotate, animate_flip))
        .insert_resource(DirectionalLightShadowMap {
            size: LIGHT_SHADOW_MAP_SIZE,
        })
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    spawn_cards(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Light
    let radius = 10.0;
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 4500.0,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 0.9 * radius,
            maximum_distance: 2.8 * radius,
            ..default()
        }
        .build(),
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Smaa {
            preset: bevy::anti_alias::smaa::SmaaPreset::Ultra,
        },
    ));

    // UI: Rotate Button
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(20.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        Button,
        RotateButton,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Rotate Card (R)"),
            TextFont {
                font: asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf"),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    });
}

fn trigger_rotate(
    mut commands: Commands,
    cards: Query<(Entity, &Transform), (With<Card>, Without<CardFlipAnimation>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<RotateButton>)>,
) {
    let mut should_rotate = keyboard.just_pressed(KeyCode::KeyR);

    for interaction in &button_query {
        if *interaction == Interaction::Pressed {
            should_rotate = true;
        }
    }

    if should_rotate {
        for (entity, transform) in &cards {
            let start = transform.rotation;
            let end = Quat::from_rotation_z(PI) * start;
            commands.entity(entity).insert(CardFlipAnimation {
                progress: 0.0,
                speed: FLIP_SPEED,
                start_rotation: start,
                end_rotation: end,
                start_y: transform.translation.y,
            });
        }
    }
}

fn animate_flip(
    mut commands: Commands,
    time: Res<Time>,
    mut cards: Query<(Entity, &mut Transform, &mut CardFlipAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * anim.speed;

        if anim.progress >= 1.0 {
            transform.rotation = anim.end_rotation;
            transform.translation.y = anim.start_y;
            commands.entity(entity).remove::<CardFlipAnimation>();
        } else {
            let t = ease_in_out(anim.progress);
            transform.rotation = anim.start_rotation.slerp(anim.end_rotation, t);

            let flip_angle = t * PI;
            let y_offset = (CARD_WIDTH / 2.0) * flip_angle.sin().abs();
            transform.translation.y = anim.start_y + y_offset;
        }
    }
}

fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
