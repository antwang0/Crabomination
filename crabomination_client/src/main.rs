use std::f32::consts::PI;

use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::{anti_alias::smaa::Smaa, color::palettes::basic::SILVER, prelude::*};

mod card;
mod systems;

use card::{spawn_cards, spawn_deck};
use systems::animate::{
    animate_deck_shuffle, animate_draw_card, animate_flip, animate_hand_slide, animate_hover_lift,
};
use systems::input::{
    rebalance_hand, trigger_draw, trigger_rotate, trigger_shuffle, DrawButton, RotateButton,
    ShuffleButton,
};
use systems::ui::{highlight_hovered_cards, peek_popup};

const LIGHT_SHADOW_MAP_SIZE: usize = 8192;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshPickingPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                trigger_rotate,
                trigger_shuffle,
                trigger_draw,
                highlight_hovered_cards,
                animate_hover_lift,
                rebalance_hand,
                peek_popup,
                animate_flip,
                animate_deck_shuffle,
                animate_draw_card,
                animate_hand_slide,
            ),
        )
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
    spawn_deck(&mut commands, &mut meshes, &mut materials, &asset_server);

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

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Smaa {
            preset: bevy::anti_alias::smaa::SmaaPreset::Ultra,
        },
    ));

    // UI: Rotate Button
    commands
        .spawn((
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
        ))
        .with_children(|parent| {
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

    // UI: Shuffle Button
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(220.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Button,
            ShuffleButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Shuffle Deck (S)"),
                TextFont {
                    font: asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });

    // UI: Draw Button
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(440.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Button,
            DrawButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Draw Card (D)"),
                TextFont {
                    font: asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}
