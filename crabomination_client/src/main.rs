use std::f32::consts::PI;
use std::path::Path;

use bevy::image::{ImageFilterMode, ImageSamplerDescriptor};
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::{anti_alias::smaa::Smaa, color::palettes::basic::SILVER, prelude::*};

mod card;
mod config;
mod debug_export;
mod game;
mod menu;
mod net_plugin;
mod render_quality;
mod scryfall;
mod systems;

use menu::{AppState, MenuPlugin, start_net_session_from_menu};
use net_plugin::SinglePlayerPlugin;

use card::{
    init_shared_assets, CardHighlightAssets, CardMeshAssets,
    CARD_HEIGHT, CARD_WIDTH, create_border_mesh, create_rounded_rect_mesh, BORDER_WIDTH, CORNER_RADIUS,
};
use render_quality::{ChangeQuality, RenderQuality};
use config::GraphicsConfig;
use game::{
    AltCastState, BlockingState, CardNames, FlippedHandCards, GameLog, GraveyardBrowserState,
    TargetingState,
};
use systems::game_ui::FastForward;
use systems::animate::{
    adjust_animation_speed, animate_deck_shuffle, animate_draw_card, animate_flip,
    dispatch_animation_queue,
    animate_mdfc_flip,
    animate_hand_slide, animate_hover_lift, animate_play_card, animate_return_to_deck,
    animate_return_to_hand, animate_reveal_peek, animate_send_to_graveyard, animate_tap,
    AnimationSpeed,
};
use systems::game_ui::{
    apply_swap_front_material, auto_advance_p0, handle_ability_menu, handle_alt_cast_buttons,
    handle_export_keypress, handle_game_input, poll_action_buttons, setup_game_hud,
    spawn_ability_menu, spawn_alt_cast_modal, sync_flipped_hand_cards, sync_game_visuals,
    trigger_reveal_animation, update_attack_all_visibility, update_log_text, update_p1_text,
    update_hint, update_pass_button, update_phase_chart, update_player_text,
    update_stack_panel, update_turn_text, ButtonState, GameLogicSet,
};
use systems::gizmos::{
    draw_attacker_overlays, draw_blocking_gizmos, draw_pt_modified_overlays,
    draw_stack_arrows, AttackerGizmos, BlockingGizmos, PtModifiedGizmos, StackGizmos,
};
use systems::quality::{
    handle_quality_buttons, handle_speed_slider, setup_quality_panel,
    update_speed_slider_visuals,
};
use systems::ui::{
    graveyard_browser, graveyard_card_hover_name, highlight_hovered_cards, peek_popup,
    pile_tooltip, reveal_popup, RevealPopupState,
};
use systems::decision_ui::{spawn_decision_ui, handle_scry_toggles, handle_scry_reorder, handle_search_select, handle_put_on_library_select, handle_put_on_library_hand_click, handle_discard_select, update_put_on_library_count_text, update_put_on_library_visuals, handle_choose_color_buttons, handle_confirm, handle_mulligan_buttons, DecisionUiState};

/// Marks the decorative ground plane so quality changes can update its mesh.
#[derive(Component)]
struct GroundPlane;

/// Marks the primary 3D camera so quality changes can update its SMAA setting.
#[derive(Component)]
struct MainCamera;

fn main() {
    let cfg = config::load();

    // CLI: `--load-state <path>` boots straight into inspection mode on
    // the named debug snapshot, skipping the menu. Anything else (no
    // args, unknown flags) falls through to the normal menu flow.
    let load_state_arg: Option<std::path::PathBuf> = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|w| (w[0] == "--load-state").then(|| std::path::PathBuf::from(&w[1])));

    // Preload card images for every card the player could possibly see —
    // the demo (Modern) decks and the full cube card universe. Cube
    // matches roll a random deck per seat *after* startup, so we can't
    // narrow the prefetch to "cards in this match"; the full union keeps
    // Scryfall fetches off the critical path of gameplay.
    //
    // For each MDFC we record both faces _and the front→back link_ so
    // the prefetcher can query Scryfall by the front name with
    // `face=back`. Querying by back name alone 404s for most MDFCs.
    use scryfall::CardImage;
    use std::collections::HashSet;
    let mut fronts: HashSet<&'static str> = HashSet::new();
    let mut mdfc_pairs: HashSet<(&'static str, &'static str)> = HashSet::new();
    let mut visit = |def: &crabomination::card::CardDefinition| {
        fronts.insert(def.name);
        if let Some(back) = def.back_face.as_ref() {
            mdfc_pairs.insert((def.name, back.name));
        }
    };
    let demo = crabomination::demo::build_demo_state();
    for player in &demo.players {
        for card in player.library.iter().chain(&player.hand).chain(&player.graveyard) {
            visit(&card.definition);
        }
    }
    for factory in crabomination::cube::all_cube_cards() {
        visit(&factory());
    }
    for factory in crabomination::sos_mode::all_sos_cards() {
        visit(&factory());
    }
    // Token names: created mid-game by `Effect::CreateToken` factories,
    // never present in deck definitions, so the catalog walk above
    // doesn't see them. They use a separate Scryfall query path
    // (`is:token+t:<name>`) since the bare type name doesn't resolve
    // on `cards/named`.
    let token_names: &[&'static str] = &[
        // Cube / demo tokens.
        "Bird", "Citizen", "Faerie", "Giant",
        "Clue", "Treasure", "Food", "Blood",
        // Cube-extras created by Soldier / Beast / Construct / Elephant /
        // Cat token cards (Raise the Alarm, Beast Within, Karn Scion of
        // Urza, Mascot Exhibition).
        "Soldier", "Beast", "Construct", "Elephant", "Cat",
        // Strixhaven / SoS-specific tokens. Without these the client
        // prefetch never downloads art for Fractal Anomaly's payload, the
        // Inkling tokens that Eager Glyphmage and friends produce, the
        // Pest tokens (Pest Mascot, Pest Summoning), or the R/W Spirit
        // tokens (Antiquities on the Loose, Group Project), so they
        // render with a missing-asset placeholder in the 3-D view.
        "Fractal", "Inkling", "Pest", "Spirit",
    ];

    let mut specs: Vec<CardImage> = fronts.into_iter().map(CardImage::Front).collect();
    specs.extend(
        mdfc_pairs
            .into_iter()
            .map(|(front, back)| CardImage::MdfcBack { front, back }),
    );
    specs.extend(token_names.iter().map(|name| CardImage::Token { name }));
    scryfall::ensure_card_images(&specs, Path::new(&cfg.paths.asset_dir));

    let gfx = cfg.graphics;
    App::new()
        // DefaultPlugins must come first — its StatesPlugin is what makes
        // `init_state::<AppState>()` work for MenuPlugin.
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor {
                        // 16x anisotropic filtering — keeps card text sharp at oblique angles.
                        anisotropy_clamp: 16,
                        mag_filter: ImageFilterMode::Linear,
                        min_filter: ImageFilterMode::Linear,
                        mipmap_filter: ImageFilterMode::Linear,
                        ..ImageSamplerDescriptor::default()
                    },
                })
                .set(AssetPlugin {
                    file_path: cfg.paths.asset_dir,
                    ..default()
                }),
            MeshPickingPlugin,
        ))
        .add_plugins((SinglePlayerPlugin, MenuPlugin))
        .init_gizmo_group::<BlockingGizmos>()
        .init_gizmo_group::<AttackerGizmos>()
        .init_gizmo_group::<StackGizmos>()
        .init_gizmo_group::<PtModifiedGizmos>()
        .add_systems(Startup, configure_gizmos)
        .insert_resource(DirectionalLightShadowMap { size: gfx.shadow_map_size })
        .insert_resource(gfx)
        .insert_resource(RenderQuality::default())
        .add_message::<ChangeQuality>()
        .insert_resource(GameLog::default())
        .insert_resource(FastForward::default())
        .insert_resource(TargetingState::default())
        .insert_resource(BlockingState::default())
        .insert_resource(AltCastState::default())
        .insert_resource(FlippedHandCards::default())
        .insert_resource(CardNames::default())
        .insert_resource(GraveyardBrowserState::default())
        .insert_resource(RevealPopupState::default())
        .insert_resource(AnimationSpeed::default())
        .insert_resource(ButtonState::default())
        .insert_resource(DecisionUiState::default())
        .init_resource::<game::AbilityMenuState>()
        .init_resource::<systems::export_prompt::ExportPromptState>()
        .insert_resource(menu::CliBootHint(load_state_arg))
        .add_systems(Startup, setup)
        // HUD scaffolding + network connection only spawn once the menu picks
        // a mode and we transition into the in-game state.
        .add_systems(
            OnEnter(AppState::InGame),
            (start_net_session_from_menu, setup_game_hud, setup_quality_panel),
        )
        // Button polling runs first so handle_game_input can read latched state.
        .add_systems(Update, poll_action_buttons.run_if(in_state(AppState::InGame)))
        // Game logic: auto-advance → player input
        .add_systems(
            Update,
            (
                auto_advance_p0.in_set(GameLogicSet),
                handle_game_input.in_set(GameLogicSet).after(auto_advance_p0),
            )
                .after(poll_action_buttons)
                .run_if(in_state(AppState::InGame)),
        )
        // Visual sync (after game logic)
        .add_systems(
            Update,
            sync_game_visuals.after(GameLogicSet).run_if(in_state(AppState::InGame)),
        )
        // MDFC flip sync — runs after visual sync so freshly-spawned hand
        // cards see their flipped state immediately on the next frame.
        .add_systems(
            Update,
            sync_flipped_hand_cards
                .after(sync_game_visuals)
                .run_if(in_state(AppState::InGame)),
        )
        // HUD refresh (after game logic)
        .add_systems(
            Update,
            (
                update_turn_text,
                update_player_text,
                update_p1_text,
                update_hint,
                update_phase_chart,
                update_log_text,
                update_stack_panel,
                update_pass_button,
                update_attack_all_visibility,
            )
                .after(handle_game_input)
                .run_if(in_state(AppState::InGame)),
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
                draw_stack_arrows,
                adjust_animation_speed,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Separate add_systems call to stay under Bevy's 20-tuple limit.
        .add_systems(Update, animate_mdfc_flip.run_if(in_state(AppState::InGame)))
        .add_systems(Update, animate_return_to_hand.run_if(in_state(AppState::InGame)))
        // Pop the next queued animation onto an entity once it stops
        // animating, so chained transitions (e.g. play-then-tap on a
        // freshly-played land) play sequentially.
        .add_systems(Update, dispatch_animation_queue.run_if(in_state(AppState::InGame)))
        // Counter coins (3-D cylinders on top of permanents).
        .add_systems(
            Update,
            crate::systems::counter_coins::sync_counter_coins
                .run_if(in_state(AppState::InGame)),
        )
        // Alt-key tooltip with counter detail + modified P/T.
        .add_systems(
            Update,
            crate::systems::counter_tooltip::update_alt_tooltip
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            draw_pt_modified_overlays.run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            graveyard_card_hover_name
                .after(graveyard_browser)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                handle_export_keypress,
                systems::export_prompt::handle_export_prompt_input,
                systems::export_prompt::sync_export_prompt_ui,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .init_resource::<systems::game_over::AutoRematchState>()
        .init_resource::<systems::game_over::ActiveMatchKind>()
        .add_systems(
            Update,
            (
                systems::game_over::sync_game_over_modal,
                systems::game_over::handle_auto_rematch_focus,
                systems::game_over::handle_auto_rematch_keys,
                systems::game_over::handle_auto_rematch_set,
                systems::game_over::refresh_auto_rematch_text,
                systems::game_over::handle_rematch_button,
                systems::game_over::handle_new_game_button,
                systems::game_over::apply_auto_rematch_on_game_over,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            OnExit(AppState::InGame),
            systems::game_over::cleanup_in_game_entities,
        )
        // Run after sync_game_visuals so SwapFrontMaterial markers
        // queued during the hand→battlefield transition land before
        // the next frame's render.
        .add_systems(
            Update,
            apply_swap_front_material
                .after(sync_game_visuals)
                .run_if(in_state(AppState::InGame)),
        )
        // Decision UI: spawn modal when pending, handle interactions, submit answer.
        .add_systems(
            Update,
            (
                spawn_decision_ui,
                handle_scry_toggles,
                handle_scry_reorder,
                handle_search_select,
                handle_put_on_library_select,
                handle_put_on_library_hand_click,
                handle_discard_select,
                update_put_on_library_count_text,
                update_put_on_library_visuals,
                handle_confirm,
                handle_mulligan_buttons,
                handle_choose_color_buttons,
            )
                .chain()
                .after(handle_game_input)
                .run_if(in_state(AppState::InGame)),
        )
        // Ability menu: handle clicks first, then (re)spawn menu to reflect new state.
        .add_systems(
            Update,
            (handle_ability_menu, spawn_ability_menu)
                .chain()
                .after(handle_game_input)
                .run_if(in_state(AppState::InGame)),
        )
        // Alt-cast (pitch) modal: pick a pitch card after right-clicking
        // a hand card with `has_alternative_cost`.
        .add_systems(
            Update,
            (handle_alt_cast_buttons, spawn_alt_cast_modal)
                .chain()
                .after(handle_game_input)
                .run_if(in_state(AppState::InGame)),
        )
        // Quality menu: buttons send event, system applies all quality changes
        .add_systems(
            Update,
            (handle_quality_buttons, apply_render_quality_change)
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        // Animation-speed slider: drag to set, label/fill mirror state.
        .add_systems(
            Update,
            (handle_speed_slider, update_speed_slider_visuals)
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

fn configure_gizmos(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<BlockingGizmos>();
    config.line.width = 4.0;
    let (config, _) = store.config_mut::<AttackerGizmos>();
    config.line.width = 3.0;
    let (config, _) = store.config_mut::<StackGizmos>();
    config.line.width = 3.0;
    let (config, _) = store.config_mut::<PtModifiedGizmos>();
    config.line.width = 4.0;
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    gfx: Res<GraphicsConfig>,
    quality: Res<RenderQuality>,
) {
    // Card meshes are spawned by `sync_game_visuals` from the first ClientView
    // that arrives. Here we just initialize shared mesh/material assets and
    // the always-present scaffolding: per-seat graveyard piles and one
    // click-to-target zone per opponent. The demo state defines the seat
    // count; for true multiplayer this will eventually come from the lobby.
    let demo = crabomination::demo::build_demo_state();
    let n_seats = demo.players.len();
    let viewer_seat = 0;
    init_shared_assets(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        quality.corner_segments(),
        n_seats,
        viewer_seat,
    );
    crate::systems::counter_coins::init_counter_coin_assets(
        &mut commands,
        &mut meshes,
        &mut materials,
    );

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
        Transform::from_xyz(0.0, 32.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        Smaa { preset: gfx.smaa_preset.to_bevy() },
        quality.msaa(),
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
        commands.entity(cam).insert(new_quality.msaa());
        match new_quality.smaa_preset() {
            Some(preset) => { commands.entity(cam).insert(Smaa { preset }); }
            None => { commands.entity(cam).remove::<Smaa>(); }
        }
    }
}
