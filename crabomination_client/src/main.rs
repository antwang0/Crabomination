use std::f32::consts::PI;
use std::path::Path;

use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::{anti_alias::smaa::Smaa, color::palettes::basic::SILVER, prelude::*};

mod bot;
mod card;
mod config;
mod game;
mod render_quality;
mod scryfall;
mod systems;

use card::{
    spawn_game_cards, CardHighlightAssets, CardMeshAssets,
    CARD_HEIGHT, CARD_WIDTH, create_border_mesh, create_rounded_rect_mesh, BORDER_WIDTH, CORNER_RADIUS,
};
use render_quality::{ChangeQuality, RenderQuality};
use config::GraphicsConfig;
use game::{build_game, BlockingState, MulliganState, P1Timer, GameLog, GameResource, GraveyardBrowserState, TargetingState};
use systems::animate::{
    adjust_animation_speed, animate_deck_shuffle, animate_draw_card, animate_flip,
    animate_hand_slide, animate_hover_lift, animate_play_card, animate_return_to_deck,
    animate_reveal_peek, animate_send_to_graveyard, animate_tap, AnimationSpeed,
};
use systems::game_ui::{
    auto_advance_p0, p1_system, draw_attacker_overlays, draw_blocking_gizmos,
    handle_game_input, mulligan_system, poll_action_buttons, setup_game_hud, setup_mulligan_ui,
    setup_quality_panel, handle_quality_buttons, sync_game_visuals, trigger_reveal_animation,
    update_mulligan_ui, update_p1_text, update_game_log, update_hint, update_phase_chart,
    update_player_text, update_turn_text, AttackerGizmos, BlockingGizmos, ButtonState,
};
use systems::ui::{graveyard_browser, highlight_hovered_cards, peek_popup, pile_tooltip, reveal_popup, RevealPopupState};
use systems::decision_ui::{spawn_decision_ui, handle_scry_toggles, handle_confirm, DecisionUiState};

/// Marks the decorative ground plane so quality changes can update its mesh.
#[derive(Component)]
struct GroundPlane;

/// Marks the primary 3D camera so quality changes can update its SMAA setting.
#[derive(Component)]
struct MainCamera;

/// All unique card names used in the game (for Scryfall image download).
const CARD_NAMES: &[&str] = &[
    // Lands
    "Island",
    "Mountain",
    "Plains",
    "Swamp",
    // Power / Moxen
    "Black Lotus",
    "Mox Emerald",
    "Mox Jet",
    "Mox Pearl",
    "Mox Ruby",
    "Mox Sapphire",
    "Sol Ring",
    // Player 0 (UW) creatures
    "Birds of Paradise",
    "Mahamoti Djinn",
    "Savannah Lions",
    "Serra Angel",
    "White Knight",
    // Player 0 (UW) spells
    "Ancestral Recall",
    "Brainstorm",
    "Counterspell",
    "Force of Will",
    "Opt",
    "Preordain",
    "Swords to Plowshares",
    "Wrath of God",
    // Player 1 (B) creatures
    "Black Knight",
    "Hypnotic Specter",
    "Juzám Djinn",
    // Player 1 (B) spells
    "Dark Ritual",
    "Demonic Tutor",
    "Hymn to Tourach",
    "Reanimate",
    "Terminate",
    "Terror",
    "Wheel of Fortune",
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
        .insert_resource(RenderQuality::default())
        .add_message::<ChangeQuality>()
        .insert_resource(build_game())
        .insert_resource(GameLog::default())
        .insert_resource(P1Timer(Timer::from_seconds(0.4, TimerMode::Repeating)))
        .insert_resource(TargetingState::default())
        .insert_resource(BlockingState::default())
        .insert_resource(GraveyardBrowserState::default())
        .insert_resource(MulliganState::default())
        .insert_resource(RevealPopupState::default())
        .insert_resource(AnimationSpeed::default())
        .insert_resource(ButtonState::default())
        .insert_resource(DecisionUiState::default())
        .add_systems(Startup, (setup, setup_game_hud, setup_mulligan_ui, setup_quality_panel))
        // Button polling runs first so handle_game_input can read latched state.
        .add_systems(Update, poll_action_buttons)
        // Game logic: mulligan → player 1 AI → auto-advance → player input (ordered)
        .add_systems(
            Update,
            (mulligan_system, p1_system, auto_advance_p0, handle_game_input)
                .chain()
                .after(poll_action_buttons),
        )
        // Visual sync (after game logic)
        .add_systems(Update, sync_game_visuals.after(handle_game_input))
        // HUD refresh (after game logic)
        .add_systems(
            Update,
            (
                update_turn_text,
                update_player_text,
                update_p1_text,
                update_hint,
                update_game_log,
                update_phase_chart,
                update_mulligan_ui,
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
                animate_return_to_deck,
                animate_send_to_graveyard,
                animate_tap,
                animate_reveal_peek,
                trigger_reveal_animation,
                draw_blocking_gizmos,
                draw_attacker_overlays,
                adjust_animation_speed,
            ),
        )
        // Decision UI: spawn modal when pending, handle interactions, submit answer.
        .add_systems(Update, (spawn_decision_ui, handle_scry_toggles, handle_confirm).chain().after(handle_game_input))
        // Quality menu: buttons send event, system applies all quality changes
        .add_systems(Update, (handle_quality_buttons, apply_render_quality_change).chain())
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
    quality: Res<RenderQuality>,
) {
    spawn_game_cards(&mut commands, &mut meshes, &mut materials, &asset_server, &game, quality.corner_segments());

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
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(quality.ground_subdivisions()))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        GroundPlane,
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 28.0, 18.0).looking_at(Vec3::ZERO, Vec3::Y),
        Smaa { preset: gfx.smaa_preset.to_bevy() },
        MainCamera,
    ));
}

#[allow(clippy::too_many_arguments)]
fn apply_render_quality_change(
    mut messages: MessageReader<ChangeQuality>,
    mut quality: ResMut<RenderQuality>,
    card_assets: Option<Res<CardMeshAssets>>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shadow_map: ResMut<DirectionalLightShadowMap>,
    ground_query: Query<&Mesh3d, With<GroundPlane>>,
    camera_query: Query<Entity, With<MainCamera>>,
    mut commands: Commands,
) {
    let Some(msg) = messages.read().last() else { return };
    let new_quality = msg.0;
    if new_quality == *quality {
        return;
    }
    *quality = new_quality;

    let segments = new_quality.corner_segments();

    if let Some(assets) = &card_assets
        && let Some(mesh) = meshes.get_mut(&assets.card_mesh) {
            *mesh = create_rounded_rect_mesh(CARD_WIDTH, CARD_HEIGHT, CORNER_RADIUS, segments);
        }

    if let Some(assets) = &highlight_assets
        && let Some(mesh) = meshes.get_mut(&assets.border_mesh) {
            *mesh = create_border_mesh(CARD_WIDTH, CARD_HEIGHT, CORNER_RADIUS, BORDER_WIDTH, segments);
        }

    if let Ok(ground_mesh) = ground_query.single()
        && let Some(mesh) = meshes.get_mut(&ground_mesh.0) {
            *mesh = Plane3d::default().mesh().size(50.0, 50.0)
                .subdivisions(new_quality.ground_subdivisions())
                .into();
        }

    shadow_map.size = new_quality.shadow_map_size();

    if let Ok(cam) = camera_query.single() {
        match new_quality.smaa_preset() {
            Some(preset) => { commands.entity(cam).insert(Smaa { preset }); }
            None => { commands.entity(cam).remove::<Smaa>(); }
        }
    }
}
