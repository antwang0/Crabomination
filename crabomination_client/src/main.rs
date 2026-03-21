use std::f32::consts::PI;
use std::path::Path;

use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::{anti_alias::smaa::Smaa, color::palettes::basic::SILVER, prelude::*};

mod bot;
mod card;
mod config;
mod game;
mod scryfall;
mod systems;

use card::spawn_game_cards;
use config::GraphicsConfig;
use game::{build_game, BlockingState, BotTimer, GameLog, GameResource, GraveyardBrowserState, TargetingState};
use systems::animate::{
    adjust_animation_speed, animate_deck_shuffle, animate_draw_card, animate_flip,
    animate_hand_slide, animate_hover_lift, animate_play_card, animate_reveal_peek,
    animate_send_to_graveyard, animate_tap, AnimationSpeed,
};
use systems::game_ui::{
    auto_advance_human, bot_system, draw_attacker_overlays, draw_blocking_gizmos,
    handle_game_input, setup_game_hud, sync_game_visuals, trigger_reveal_animation,
    update_bot_text, update_game_log, update_hint, update_phase_chart, update_player_text,
    update_turn_text, AttackerGizmos, BlockingGizmos,
};
use systems::ui::{graveyard_browser, highlight_hovered_cards, peek_popup, pile_tooltip, reveal_popup, RevealPopupState};

/// All unique card names used in the game (for Scryfall image download).
const CARD_NAMES: &[&str] = &[
    // Lands
    "Mountain",
    "Plains",
    "Swamp",
    // Human (RW) cards
    "Savannah Lions",
    "White Knight",
    "Hopeful Eidolon",
    "Lightning Helix",
    "Goblin Guide",
    "Lightning Bolt",
    "Serra Angel",
    "Shivan Dragon",
    "Wrath of God",
    "Shock",
    // Bot (BR) cards
    "Hypnotic Specter",
    "Dark Ritual",
    "Terror",
    "Black Knight",
    "Sengir Vampire",
    "Terminate",
];

fn main() {
    let cfg = config::load();

    // Download any missing card images before Bevy starts.
    scryfall::ensure_card_images(CARD_NAMES, Path::new(&cfg.paths.asset_dir));

    let gfx = cfg.graphics;
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    file_path: cfg.paths.asset_dir,
                    ..default()
                }),
            MeshPickingPlugin,
        ))
        .init_gizmo_group::<BlockingGizmos>()
        .init_gizmo_group::<AttackerGizmos>()
        .add_systems(Startup, configure_gizmos)
        .insert_resource(DirectionalLightShadowMap { size: gfx.shadow_map_size })
        .insert_resource(gfx)
        .insert_resource(build_game())
        .insert_resource(GameLog::default())
        .insert_resource(BotTimer(Timer::from_seconds(0.4, TimerMode::Repeating)))
        .insert_resource(TargetingState::default())
        .insert_resource(BlockingState::default())
        .insert_resource(GraveyardBrowserState::default())
        .insert_resource(RevealPopupState::default())
        .insert_resource(AnimationSpeed::default())
        .add_systems(Startup, (setup, setup_game_hud))
        // Game logic: bot → auto-advance → player input (ordered)
        .add_systems(
            Update,
            (bot_system, auto_advance_human, handle_game_input).chain(),
        )
        // Visual sync (after game logic)
        .add_systems(Update, sync_game_visuals.after(handle_game_input))
        // HUD refresh (after game logic)
        .add_systems(
            Update,
            (
                update_turn_text,
                update_player_text,
                update_bot_text,
                update_hint,
                update_game_log,
                update_phase_chart,
            )
                .after(handle_game_input),
        )
        // Visual / animation systems
        .add_systems(
            Update,
            (
                highlight_hovered_cards,
                animate_hover_lift,
                peek_popup,
                graveyard_browser,
                pile_tooltip,
                reveal_popup,
                animate_flip,
                animate_deck_shuffle,
                animate_draw_card,
                animate_hand_slide,
                animate_play_card,
                animate_send_to_graveyard,
                animate_tap,
                animate_reveal_peek,
                trigger_reveal_animation,
                draw_blocking_gizmos,
                draw_attacker_overlays,
                adjust_animation_speed,
            ),
        )
        .run();
}

fn configure_gizmos(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<BlockingGizmos>();
    config.line.width = 4.0;
    let (config, _) = store.config_mut::<AttackerGizmos>();
    config.line.width = 3.0;
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    game: Res<GameResource>,
    gfx: Res<GraphicsConfig>,
) {
    spawn_game_cards(&mut commands, &mut meshes, &mut materials, &asset_server, &game);

    // Ambient fill light — softens harsh shadows.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: gfx.ambient_brightness,
        ..default()
    });

    // Key light (directional, with shadows).
    let radius = 14.0;
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            illuminance: gfx.key_light_illuminance,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 0.9 * radius,
            maximum_distance: 2.8 * radius,
            ..default()
        }
        .build(),
    ));

    // Fill light from the opposite side — further reduces shadow darkness.
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, -2.0, -PI / 6.)),
        DirectionalLight {
            shadows_enabled: false,
            illuminance: gfx.fill_light_illuminance,
            ..default()
        },
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 28.0, 18.0).looking_at(Vec3::ZERO, Vec3::Y),
        Smaa { preset: gfx.smaa_preset.to_bevy() },
    ));
}
