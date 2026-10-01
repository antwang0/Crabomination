#![allow(clippy::too_many_arguments, clippy::type_complexity)]

// The bot seat runs the engine's search (`MctsBot`, 256 iterations), so this
// program allocates like the simulator binaries do and takes their allocator
// with it. A `#[global_allocator]` is a whole-program decision, which is why
// it is stated here rather than pulled in through the engine's own feature.
// See `crabomination_client/Cargo.toml` for the measurement.
#[cfg(all(feature = "mimalloc", not(target_arch = "wasm32")))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::f32::consts::PI;

use bevy::asset::AssetApp;
use bevy::asset::io::{AssetSource, AssetSourceBuilder, AssetSourceId, ErasedAssetReader};
use bevy::image::{ImageFilterMode, ImageSamplerDescriptor};
use bevy::light::{CascadeShadowConfig, DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::post_process::bloom::Bloom;
use bevy::camera::Hdr;
use bevy::{anti_alias::smaa::Smaa, prelude::*};

mod audit;
mod card;
mod config;
mod debug_export;
mod deck_import;
mod deck_picker;
#[cfg(not(target_arch = "wasm32"))]
mod embedded_assets;
mod game;
mod layout_harness;
mod mana_text;
mod menu;
mod net_plugin;
mod render_quality;
mod saved_decks;
mod scryfall;
mod storage;
mod synthesized_cards;
#[cfg(target_arch = "wasm32")]
mod ws_client;
mod systems;
mod theme;

use menu::{AppState, MenuPlugin, start_net_session_from_menu};
use net_plugin::SinglePlayerPlugin;

use card::{
    init_shared_assets, CardHighlightAssets, CardMeshAssets, HandZoom,
    CARD_HEIGHT, CARD_WIDTH, create_border_mesh, create_rounded_rect_mesh, BORDER_WIDTH, CORNER_RADIUS,
};
use render_quality::{ChangeQuality, RenderQuality};
use config::GraphicsConfig;
use game::{
    AltCastState, AttackingState, BlockingState, CardNames, FlippedHandCards, GameLog, GraveyardBrowserState,
    TargetingState,
};
use systems::game_ui::FastForward;
use systems::input_guard::text_input_active;
use systems::animate::{
    adjust_animation_speed, animate_combat_lurch, animate_draw_card,
    animate_mdfc_flip, animate_hand_slide,
    animate_hover_lift, animate_play_card, animate_return_to_deck, animate_return_to_hand,
    animate_reveal_peek, animate_send_to_graveyard, animate_tap, update_combat_lurch_targets,
    AnimationSpeed,
};
use systems::game_ui::{
    apply_swap_front_material, auto_advance_p0, cancel_pickers_on_escape,
    handle_ability_menu, handle_alt_cast_buttons,
    handle_hand_menu, spawn_hand_menu, HandMenuState,
    handle_auto_pass_toggle, handle_export_keypress, handle_game_input,
    handle_planar_die_keypress, handle_reveal_conspiracy_keypress, poll_action_buttons,
    poll_player_chip_clicks,
    setup_game_hud,
    spawn_ability_menu, spawn_alt_cast_modal, sync_command_zone, sync_flipped_hand_cards,
    sync_game_visuals,
    handle_audit_buttons, handle_surrender_button, pulse_urgent_pass_button,
    sync_audit_buttons, sync_player_hud_seat,
    sync_hint_chip_visibility, trigger_reveal_animation, update_attack_all_visibility,
    update_attack_button_label,
    animate_phase_banner, trigger_phase_banner, PhaseBannerTracker,
    record_life_history, sync_life_graph, toggle_life_graph, LifeHistory,
    update_combat_preview_panel,
    fit_player_hud_width, position_left_column_below_hud, position_log_below_opponents,
    update_log_text, update_mana_pips, update_opponent_panel_tint, update_opponent_stats_rows,
    update_hint, update_pass_button, update_phase_chart, update_player_chip_target_outline,
    update_player_stats_chips, update_stack_panel,
    handle_stack_resolve_button, update_turn_text,
    ButtonState, GameLogicSet,
};
use systems::gizmos::{
    draw_active_seat_glow, draw_attachment_tethers, draw_attack_plan_arrows,
    draw_block_arrows, draw_legal_target_rings, draw_stack_arrows, draw_target_arrow,
};
use systems::quality::{
    close_settings_on_esc, handle_leave_game_button, handle_quality_buttons, handle_speed_slider,
    handle_ui_size_button,
    open_settings_on_esc, setup_quality_panel, sync_settings_visibility,
    update_speed_slider_visuals, SettingsOpen,
};
use systems::ui::{
    exile_browser, graveyard_browser, graveyard_card_hover_name, graveyard_recast_click,
    highlight_hovered_cards, hover_card_preview,
    toggle_shortcut_help, update_castable_highlights, update_dying_highlights,
    update_activatable_highlights, peek_popup, pile_tooltip, reveal_popup, RevealPopupState,
};
use systems::decision_ui::{spawn_decision_ui, handle_scry_toggles, handle_scry_reorder, handle_trigger_reorder, handle_damage_order_reorder, handle_damage_assign_buttons, handle_search_select, handle_put_on_library_hand_click, handle_discard_select, update_put_on_library_count_text, update_put_on_library_visuals, handle_choose_color_buttons, handle_name_card_buttons, handle_learn_buttons, handle_confirm, handle_decision_cancel, handle_mulligan_buttons, spawn_mode_pick_ui, handle_mode_pick_buttons, handle_optional_buttons, handle_choose_modes_toggle, handle_trigger_mode_buttons, handle_amount_buttons, handle_divide_damage_buttons, handle_creature_type_buttons, handle_randomizer_buttons, handle_legend_keep_buttons, DecisionUiState};

/// Marks the decorative ground plane so quality changes can update its mesh.
#[derive(Component)]
struct GroundPlane;

/// Marks the primary 3D camera so quality changes can update its SMAA setting.
#[derive(Component)]
pub struct MainCamera;

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

    // CLI: `--play <format>` boots straight into a local-bot match of the
    // given format (e.g. `--play commander` for the 4-player FFA), skipping
    // the menu. Handy for verifying format-specific layouts.
    let play_format_arg: Option<menu::MatchFormat> = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|w| (w[0] == "--play").then(|| menu::MatchFormat::from_cli(&w[1])))
        .flatten();

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
        // SOS preparation cards: the cast copy appears on the stack under
        // the inset spell's own name, so prefetch it as a plain front —
        // Scryfall's `cards/named` resolves face names (returning the
        // full preparation card's art, which is the right image).
        if let Some(prep) = def.prepare_spell.as_ref() {
            fronts.insert(prep.name);
        }
    };
    let demo = crabomination::demo::build_demo_state();
    for player in &demo.players {
        for card in player.library.iter().chain(&player.hand).chain(&player.graveyard) {
            visit(&card.definition);
        }
    }
    // Every card the registry knows about — cube, SoS, the *full* STX
    // catalog, plus the xtra / Theros sets. This is a superset of every
    // deck *and* the audit catalog (and the Lessons sideboard), so
    // audit-mode cards and Learn-fetched Lessons (Academic Dispute, Anger,
    // …) get their art prefetched instead of rendering with the
    // missing-asset placeholder. Earlier this walked only cube + SoS, so
    // the many STX cards outside those pools had no image.
    for factory in crabomination::catalog::all_known_factories() {
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
        // Prismari Elemental (Artistic Process mode 2, Visionary's Dance,
        // Muse's Encouragement, the rest of the elemental_token() callers).
        // Was missing from the prefetch list, so the freshly-minted token
        // entered with a placeholder front face on the battlefield.
        "Elemental",
    ];

    let mut specs: Vec<CardImage> = fronts.into_iter().map(CardImage::Front).collect();
    specs.extend(
        mdfc_pairs
            .into_iter()
            .map(|(front, back)| CardImage::MdfcBack { front, back }),
    );
    specs.extend(token_names.iter().map(|name| CardImage::Token { name }));
    // Resolve once: a relative `asset_dir` is anchored to a fixed location
    // (crate source dir in debug, exe dir in release) so the prefetch cache
    // and Bevy's asset root agree regardless of the working directory.
    // (Browser: asset paths are URLs fetched relative to the page, so the
    // native filesystem anchoring doesn't apply — serve from "assets".)
    let asset_dir = if cfg!(target_arch = "wasm32") {
        std::path::PathBuf::from("assets")
    } else {
        cfg.paths.resolved_asset_dir()
    };
    // A custom/fresh asset dir starts empty — materialize the embedded
    // core assets (fonts, cardback) before anything reads them.
    #[cfg(not(target_arch = "wasm32"))]
    embedded_assets::materialize_core_assets(&asset_dir);
    // Token art fetched by type rather than name is often the wrong token
    // entirely; clear it once so the prefetch below fetches it by name.
    #[cfg(not(target_arch = "wasm32"))]
    scryfall::purge_type_matched_token_art(&asset_dir, token_names);
    // Card-art prefetch runs on a background thread — launch never blocks
    // on the network. Missing images render as name placeholders (see
    // `CardPlaceholderReader`) and hot-swap to real art as downloads land
    // (`reload_completed_images`); the menu shows live progress.
    let image_prefetch = scryfall::ImagePrefetch::default();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let progress = image_prefetch.clone();
        let dir = asset_dir.clone();
        // Cards the app renders jump this queue as they are asked for
        // (`scryfall::card_asset_path`), so a match never waits on unrelated art.
        std::thread::spawn(move || {
            scryfall::ensure_card_images_with_progress(&specs, &dir, &progress);
        });
    }
    // Browser: no prefetch — card art loads over HTTP through the asset
    // server (the browser cache is the disk cache). Mark the progress latch
    // done so the menu doesn't show a stuck download bar.
    #[cfg(target_arch = "wasm32")]
    {
        let _ = &specs;
        image_prefetch.finished.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    let gfx = cfg.graphics;
    let gameplay = cfg.gameplay;
    // Whole-config resource for the persistence systems (settings writes
    // rewrite the file without losing other sections). Cloned before the
    // sections move into their own resources below.
    let mut cfg_store = config::ConfigStore(config::Config {
        paths: cfg.paths.clone(),
        graphics: gfx.clone(),
        gameplay: gameplay.clone(),
    });
    let initial_stops = systems::phase_bar::StopConfig {
        my: gameplay.stops_my.clone(),
        opp: gameplay.stops_opp.clone(),
    };
    let initial_anim_speed = AnimationSpeed(gameplay.animation_speed.clamp(0.25, 4.0));
    let cfg_window_mode = gfx.window_mode;
    let harness = layout_harness::HarnessArgs::parse(&std::env::args().skip(1).collect::<Vec<_>>());
    // `--ui-size N` for this run only; nothing saves it unless a setting
    // is changed.
    if let Some(percent) = harness.ui_size {
        cfg_store.0.graphics.ui_size = percent;
    }
    // `--window WxH` pins the size (and skips `maximize_window`) so the
    // layout harness can render one aspect ratio after another.
    let (cfg_window_w, cfg_window_h) = harness.window.unwrap_or((gfx.window_width, gfx.window_height));
    let cfg_quality = gfx.render_quality;

    // Custom Default asset source: it wraps the normal file reader and
    // synthesizes a white name-placeholder PNG for any missing card image,
    // so art-less (synthesized / 404) cards need NO files on disk — the
    // prefetch above writes only real downloads. Must be registered before
    // AssetPlugin is added (sources are built at that point).
    let asset_dir_str = asset_dir.to_string_lossy().into_owned();
    let placeholder_font = std::sync::Arc::new(scryfall::load_placeholder_font(&asset_dir));
    let mut default_reader_factory = AssetSource::get_default_reader(asset_dir_str);

    App::new()
        .register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                Box::new(scryfall::CardPlaceholderReader::new(
                    default_reader_factory(),
                    placeholder_font.clone(),
                )) as Box<dyn ErasedAssetReader>
            }),
        )
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
                    file_path: asset_dir.to_string_lossy().into_owned(),
                    ..default()
                })
                // Open sized to the display rather than at winit's tiny
                // 1280×720 default: `maximize_window` (a Startup system)
                // maximizes the primary window to fill whatever monitor
                // it's on, so the full HUD — including the bottom hand fan,
                // which clips below ~800px tall — is always visible. The
                // resolution below is only the "restore" size if the user
                // un-maximizes. The HUD anchors fixed-pixel panels to all
                // four corners (turn/log ~280px right, player panel ~260px
                // left, stack ~420px centred) and the hand needs vertical
                // room, so the resize floor is raised to 1024×768 — below
                // that the corner panels overlap and the hand clips.
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: match cfg_window_mode {
                            config::WindowModeCfg::Windowed => bevy::window::WindowMode::Windowed,
                            config::WindowModeCfg::Borderless =>
                                bevy::window::WindowMode::BorderlessFullscreen(
                                    bevy::window::MonitorSelection::Primary,
                                ),
                        },
                        resolution: bevy::window::WindowResolution::new(
                            cfg_window_w, cfg_window_h,
                        ),
                        position: bevy::window::WindowPosition::Centered(
                            bevy::window::MonitorSelection::Primary,
                        ),
                        resize_constraints: bevy::window::WindowResizeConstraints {
                            min_width: 1024.0,
                            min_height: 768.0,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                }),
            MeshPickingPlugin,
        ))
        .insert_resource(cfg_store)
        .insert_resource(image_prefetch)
        .add_plugins((
            SinglePlayerPlugin,
            MenuPlugin,
            theme::UiFontsPlugin,
            systems::draft::DraftPlugin,
            systems::lobby_ui::LobbyUiPlugin,
            systems::game_ui::TableAwarenessPlugin,
        ))
        .insert_resource(DirectionalLightShadowMap { size: cfg_quality.shadow_map_size() })
        .insert_resource(gfx)
        .insert_resource(gameplay)
        .insert_resource(cfg_quality)
        .add_message::<ChangeQuality>()
        .insert_resource(GameLog::default())
        .insert_resource(PhaseBannerTracker::default())
        .init_resource::<systems::game_ui::life_ticker::LifeTicker>()
        .insert_resource(LifeHistory::default())
        .insert_resource(FastForward::default())
        .insert_resource(TargetingState::default())
        .insert_resource(game::LegalTargets::default())
        .insert_resource(game::PendingModalCast::default())
        .insert_resource(BlockingState::default())
        .insert_resource(AttackingState::default())
        .insert_resource(AltCastState::default())
        .insert_resource(game::SplitCastState::default())
        .insert_resource(game::PayTimesState::default())
        .insert_resource(game::SpreeCastState::default())
        .insert_resource(game::HelperTapState::default())
        .insert_resource(FlippedHandCards::default())
        .insert_resource(CardNames::default())
        .insert_resource(GraveyardBrowserState::default())
        .init_resource::<game::ExileBrowserState>()
        .insert_resource(RevealPopupState::default())
        .insert_resource(initial_anim_speed)
        .insert_resource(ButtonState::default())
        .insert_resource(HandZoom::default())
        .insert_resource(systems::kb_cursor::KeyboardCursor::default())
        .insert_resource(SettingsOpen::default())
        .insert_resource(systems::esc::EscFocus::default())
        .insert_resource(audit::AuditTarget::default())
        .insert_resource(audit::AuditPoolFilter::default())
        .insert_resource(audit::ShowVerifiedCards::default())
        .insert_resource(audit::load_audited_cards())
        .insert_resource(DecisionUiState::default())
        .init_resource::<systems::decision_ui::AutoOptionalAnswers>()
        .init_resource::<systems::commander_ui::CommanderDamageFlash>()
        .insert_resource(systems::debug_console::DebugConsoleState::default())
        .init_resource::<game::AbilityMenuState>()
        .init_resource::<HandMenuState>()
        .init_resource::<systems::export_prompt::ExportPromptState>()
        .init_resource::<systems::game_ui::SurrenderConfirm>()
        .insert_resource(initial_stops)
        .insert_resource(menu::CliBootHint(load_state_arg))
        .insert_resource(menu::CliBootFormat(play_format_arg))
        .insert_resource(harness.clone())
        .insert_resource(scryfall::CardArtDir(asset_dir.clone()))
        // Draw only when something changes; a harness screenshot run keeps
        // the continuous loop so its timed captures don't move.
        .add_plugins((
            systems::frame_pacing::FramePacingPlugin { continuous: harness.screenshot.is_some() },
            systems::perf_hud::PerfHudPlugin,
            systems::shadows::ShadowsPlugin,
        ))
        .init_resource::<layout_harness::ScreenshotClock>()
        .add_systems(
            Update,
            (
                layout_harness::open_settings_for_screenshot.run_if(in_state(AppState::InGame)),
                layout_harness::open_deck_picker_for_screenshot.run_if(in_state(AppState::Menu)),
                layout_harness::import_for_screenshot.run_if(in_state(AppState::Menu)),
                layout_harness::capture_screenshot,
                layout_harness::spawn_mana_gallery.run_if(in_state(AppState::InGame)),
                layout_harness::hover_card_for_screenshot.run_if(in_state(AppState::InGame)),
                layout_harness::zoom_on_card_for_screenshot
                    .after(crate::systems::camera_zoom::camera_zoom)
                    .run_if(in_state(AppState::InGame)),
            ),
        )
        .add_systems(
            PreUpdate,
            (
                layout_harness::inject_damage_for_screenshot,
                layout_harness::stage_combat_for_screenshot.run_if(in_state(AppState::InGame)),
                layout_harness::stage_decision_for_screenshot.run_if(in_state(AppState::InGame)),
                layout_harness::swing_life_for_screenshot.run_if(in_state(AppState::InGame)),
                layout_harness::fire_impacts_for_screenshot.run_if(in_state(AppState::InGame)),
            )
                .after(crate::net_plugin::poll_net),
        )
        // The stack panel sits beside the 3-D stack lane.
        .add_systems(Update, crate::systems::game_ui::place_stack_panel.run_if(in_state(AppState::InGame)))
        // Mana symbols in UI text, in every state (the draft shows costs too).
        // After `Update`, whose systems despawn the nodes it rebuilds (a
        // tooltip closing), and before the UI lays the new rows out.
        .add_systems(PostUpdate, mana_text::sync_mana_text.before(bevy::ui::UiSystems::Prepare))
        // A card focus dims fades its overlays with it, after their own
        // systems have painted them (`systems::focus`).
        .add_systems(PostUpdate, systems::focus::fade_card_overlays.before(bevy::ui::UiSystems::Prepare))
        .add_systems(PostUpdate, systems::ui::fade_in_hover_preview.before(bevy::ui::UiSystems::Prepare))
        // Combat, targeting and stack arrows, as geometry (`systems::arrows`).
        .init_resource::<systems::arrows::Arrows>()
        .add_systems(PostUpdate, systems::arrows::render_arrows)
        .init_resource::<systems::glow::Glow>()
        .add_systems(PostUpdate, systems::glow::render_glow)
        // A hovered battlefield card's tilt toward the camera is taken off
        // before `Update` and put back after it (`animate::HoverTilt`).
        .add_systems(First, systems::animate::untilt_hovered_cards)
        .add_systems(
            PostUpdate,
            systems::animate::tilt_hovered_cards.before(bevy::transform::TransformSystems::Propagate),
        )
        .add_systems(Startup, layout_harness::ignore_mouse_for_screenshot)
        .add_systems(Startup, setup)
        .add_systems(Startup, maximize_window)
        // Resolution-driven hand zoom + 2-D UI scale — both run every
        // frame but only write when the chosen tier flips, so they're
        // effectively free.
        .add_systems(
            Update,
            (update_hand_zoom_from_window, update_ui_scale_from_window),
        )
        // HUD scaffolding + network connection only spawn once the menu picks
        // a mode and we transition into the in-game state.
        .add_systems(
            OnEnter(AppState::InGame),
            (
                start_net_session_from_menu,
                setup_game_hud,
                setup_quality_panel,
            ),
        )
        // Phase-chart stop toggles (click a step row to cycle stop/skip).
        .add_systems(
            Update,
            systems::phase_bar::handle_phase_chart_clicks
                .run_if(in_state(AppState::InGame)),
        )
        // Main-menu Settings panel.
        .init_resource::<systems::settings_menu::MenuSettingsOpen>()
        .add_systems(
            Update,
            (
                systems::settings_menu::handle_settings_open,
                systems::settings_menu::handle_setting_rows,
                systems::settings_menu::sync_settings_panel,
                systems::settings_menu::update_setting_labels,
            )
                .chain()
                .run_if(in_state(AppState::Menu)),
        )
        // Settings persistence: mirror stop/animation-speed changes into
        // config.toml (each system early-outs on no change).
        .add_systems(
            Update,
            (config::persist_stops, config::persist_animation_speed),
        )
        // Hot-swap freshly downloaded card art over its placeholder.
        .add_systems(Update, scryfall::reload_completed_images)
        // Audit-mode card picker.
        .add_systems(OnEnter(AppState::Audit), audit::spawn_audit_picker)
        .add_systems(OnExit(AppState::Audit), audit::despawn_audit_picker)
        .add_systems(
            Update,
            audit::handle_audit_picker.run_if(in_state(AppState::Audit)),
        )
        // Mouse-wheel scrolling for every `Scrollable` panel, in every
        // state — the menu/audit/draft pickers and the in-game log,
        // browsers and decision grids all route through one handler.
        .add_systems(Update, systems::scroll::handle_scroll)
        // Audit-mode HUD buttons: sync visibility every frame, handle
        // click → save + return to picker.
        .add_systems(
            Update,
            (
                sync_audit_buttons,
                handle_audit_buttons,
                handle_surrender_button,
                crate::systems::game_ui::hide_actions_when_out,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Button polling runs first so handle_game_input can read latched state.
        // `poll_player_chip_clicks` must run after `poll_action_buttons` so it
        // can fill the chip slot that the action-button poll first cleared.
        .add_systems(
            Update,
            (poll_action_buttons, poll_player_chip_clicks, systems::drag_act::track_drag)
                .chain()
                .before(handle_game_input)
                .run_if(in_state(AppState::InGame)),
        )
        .init_resource::<systems::drag_act::DragAct>()
        // Escape arbitration. `compute_esc_focus` names the one surface
        // that owns this frame's press; it runs in `PreUpdate` so every
        // consumer is order-independent and nothing in the `Update` graph
        // has to move (see `systems::esc` for why a chained set cannot
        // work here — `cancel_pickers_on_escape` is pinned *after*
        // `handle_game_input` and the keyboard cursor *before* it, which
        // closes into a cycle).
        // `.after(InputSystems)` is load-bearing: `keyboard_input_system`
        // also runs in `PreUpdate`, so without it Bevy is free to schedule
        // this first and read *last* frame's `just_pressed` — Esc would
        // act a frame late, or on a press that had already been handled.
        .add_systems(
            PreUpdate,
            systems::esc::compute_esc_focus.after(bevy::input::InputSystems),
        )
        // The pause menu sits at both ends of the precedence: it closes
        // ahead of everything, and opens only on a press no open surface
        // wanted. Neither half needs an ordering constraint.
        .add_systems(
            Update,
            (close_settings_on_esc, open_settings_on_esc, sync_settings_visibility)
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        // Keyboard cursor: Tab/Arrows update `KeyboardCursor.selection`
        // before `handle_game_input` reads it as a fallback for clicks.
        .add_systems(
            Update,
            systems::kb_cursor::handle_keyboard_cursor_input
                .before(handle_game_input)
                .run_if(in_state(AppState::InGame)),
        )
        // Game logic: auto-advance → player input
        .add_systems(
            Update,
            (
                auto_advance_p0.in_set(GameLogicSet),
                handle_game_input.in_set(GameLogicSet).after(auto_advance_p0),
            )
                // Ordering against `poll_action_buttons` alone was not enough:
                // `.chain()` orders the two pollers against each other, but
                // left `poll_player_chip_clicks` and `handle_game_input`
                // mutually unordered. `poll_action_buttons` *clears* the chip
                // slot and `poll_player_chip_clicks` *fills* it, so on any
                // frame the scheduler ran the input system before the fill,
                // the click was read as `None` and dropped — clicking a
                // player's HUD box to target them did nothing. That box is the
                // only mouse affordance for a player target (the 3-D disc is
                // gone), so this took player targeting with it.
                .after(poll_player_chip_clicks)
                .run_if(in_state(AppState::InGame)),
        )
        // Visual sync (after game logic)
        .add_systems(
            Update,
            sync_game_visuals.after(GameLogicSet).run_if(in_state(AppState::InGame)),
        )
        // Keyboard selection marker — runs after sync so freshly-spawned
        // hand / bf entities exist; mirrors into `CardHovered` so the
        // existing hover highlight + hand-card lift react for free.
        .add_systems(
            Update,
            (
                systems::kb_cursor::apply_keyboard_selection,
                systems::kb_cursor::sync_kb_hover_marker,
            )
                .chain()
                .after(sync_game_visuals)
                .run_if(in_state(AppState::InGame)),
        )
        // MDFC flip sync — runs after visual sync so freshly-spawned hand
        // cards see their flipped state immediately on the next frame.
        .add_systems(
            Update,
            sync_flipped_hand_cards
                .after(sync_game_visuals)
                .run_if(in_state(AppState::InGame)),
        )
        // Command zone sync — spawns visuals for each card in any
        // player's command zone. Independent of the main game-visual
        // sync (no animation handoff with hand/battlefield).
        .add_systems(
            Update,
            sync_command_zone
                .after(sync_game_visuals)
                .run_if(in_state(AppState::InGame)),
        )
        // Life totals count to their new value, with a numeral beside the
        // seat's HUD row (`life_ticker`). The count starts before the rows
        // are rebuilt, so they're built on its first frame.
        .add_systems(
            Update,
            (
                systems::game_ui::life_ticker::track_life_changes
                    .before(update_player_stats_chips)
                    .before(update_opponent_stats_rows),
                systems::game_ui::life_ticker::tick_life_readouts,
                systems::game_ui::life_ticker::place_life_numerals,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(OnExit(AppState::InGame), systems::game_ui::life_ticker::reset_life_ticker)
        // HUD refresh (after game logic)
        .add_systems(
            Update,
            (
                update_turn_text,
                update_player_stats_chips,
                update_mana_pips,
                update_opponent_stats_rows,
                update_opponent_panel_tint,
                (position_log_below_opponents, position_left_column_below_hud),
                fit_player_hud_width,
                update_hint,
                update_phase_chart,
                update_log_text,
                handle_stack_resolve_button,
                update_stack_panel,
                update_combat_preview_panel,
                update_pass_button,
                pulse_urgent_pass_button,
                update_attack_all_visibility,
                update_attack_button_label,
                sync_player_hud_seat,
                update_player_chip_target_outline,
                sync_hint_chip_visibility,
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
                exile_browser,
                pile_tooltip,
                reveal_popup,
                animate_draw_card,
                animate_hand_slide,
                animate_play_card,
                animate_return_to_deck,
                animate_send_to_graveyard,
                animate_tap,
                animate_reveal_peek,
                trigger_reveal_animation,
                // Keyboard-only: `[` `]` `\` are typeable, so a text
                // surface owning the keyboard suppresses them.
                adjust_animation_speed.run_if(not(text_input_active)),
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Separate add_systems call to stay under Bevy's 20-tuple limit.
        .add_systems(
            Update,
            (
                update_castable_highlights,
                update_dying_highlights,
                update_activatable_highlights,
                // `?` is Shift+Slash — typeable, like the two above.
                toggle_shortcut_help.run_if(not(text_input_active)),
                trigger_phase_banner,
                animate_phase_banner,
                record_life_history,
                toggle_life_graph.run_if(not(text_input_active)),
                sync_life_graph,
                hover_card_preview,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Combat lurch: read attacker/blocker state, lerp Z forward.
        // Must run after animate_hover_lift since that system overwrites
        // transform.translation each frame from `base_translation`.
        .add_systems(
            Update,
            (update_combat_lurch_targets, animate_combat_lurch, systems::animate::animate_jolt)
                .chain()
                .after(animate_hover_lift)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, animate_mdfc_flip.run_if(in_state(AppState::InGame)))
        .add_systems(Update, animate_return_to_hand.run_if(in_state(AppState::InGame)))
        .add_systems(Update, systems::animate::animate_vanishing.run_if(in_state(AppState::InGame)))
        // Give every freshly-loaded card-face texture a mip chain so the
        // sampler's 16× anisotropy keeps text legible at the table's oblique
        // angle. Ungated: card images load during draft/menu as well as in a
        // match, and the system self-skips anything already mipmapped. In
        // `Last`: after the frame's asset events, before the render world
        // extracts, so a new image is held back until its chain is built.
        .add_systems(Last, crate::card::mipmap::generate_card_mipmaps)
        // Counter coins (3-D piles on top of permanents).
        .add_systems(
            Update,
            (
                crate::systems::counter_coins::sync_counter_coins,
                crate::systems::counter_coins::animate_coin_drops,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Impact feedback: deaths, damage sparks, digs, mana, life-loss
        // vignette.
        .add_systems(
            Update,
            (
                crate::systems::impact::spawn_impact_effects,
                crate::systems::impact::spawn_mana_motes,
                crate::systems::impact::animate_impacts,
                crate::systems::impact::animate_hit_vignettes,
                crate::systems::impact::animate_damage_numerals,
                crate::systems::impact::animate_mana_motes,
            )
                // After sync_game_visuals so freshly-cast spells have their
                // StackCard entity (mote destinations) and dying creatures
                // still have their battlefield entity (burst positions).
                .after(sync_game_visuals)
                .run_if(in_state(AppState::InGame)),
        )
        // Screen-space labels: each coin pile's count and kind; a changed
        // count (or P/T) swells for a moment.
        .add_systems(
            Update,
            (crate::systems::counter_coins::sync_counter_labels, crate::theme::animate_overlay_pulses)
                .run_if(in_state(AppState::InGame)),
        )
        // Alt-key tooltip with counter detail + modified P/T.
        .add_systems(
            Update,
            crate::systems::counter_tooltip::update_alt_tooltip
                .run_if(in_state(AppState::InGame)),
        )
        // Floating P/T badge over creatures whose stats differ from base.
        .add_systems(
            Update,
            crate::systems::pt_label::sync_pt_labels
                .run_if(in_state(AppState::InGame)),
        )
        // "×N" count chip over each cascaded token pile.
        .add_systems(
            Update,
            crate::systems::token_badge::sync_token_pile_badges
                .run_if(in_state(AppState::InGame)),
        )
        // CR 702.106 — the name a hidden-agenda conspiracy chose.
        .add_systems(
            Update,
            crate::systems::agenda_badge::sync_agenda_badges
                .run_if(in_state(AppState::InGame)),
        )
        // CR 903 — command-zone cost/tax chips, the castable ring on the
        // viewer's commander, lethal commander-damage warnings, and the HUD
        // commander-damage chip pulse.
        .add_systems(
            Update,
            (
                crate::systems::commander_ui::sync_command_zone_cost_badges,
                crate::systems::commander_ui::update_command_zone_castable_highlights,
                crate::systems::commander_ui::sync_lethal_commander_warnings,
                crate::systems::commander_ui::pulse_commander_damage_chips,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // "FREE" and right-click chips over hand cards.
        .add_systems(
            Update,
            crate::systems::hand_chips::sync_hand_chips
                .run_if(in_state(AppState::InGame)),
        )
        // CR 701.19 — shield chip over permanents that can regenerate.
        .add_systems(
            Update,
            crate::systems::regen_badge::sync_regen_badges
                .run_if(in_state(AppState::InGame)),
        )
        // Interdict / Hand to Hand — chip over permanents whose activated
        // abilities are switched off.
        .add_systems(
            Update,
            crate::systems::lock_badge::sync_lock_badges
                .run_if(in_state(AppState::InGame)),
        )
        // Clarity batch: pulsing ring on the pending decision's source, and
        // the low-life danger frame.
        .add_systems(
            Update,
            (
                crate::systems::gizmos::draw_decision_source_ring,
                crate::systems::ui::low_life_vignette,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Floating keyword-flag strip ("Fly DT LL") over creatures.
        .add_systems(
            Update,
            crate::systems::keyword_label::sync_keyword_labels
                .run_if(in_state(AppState::InGame)),
        )
        // Hold-Ctrl camera zoom onto the cursor / highlighted card.
        .add_systems(
            Update,
            (
                crate::systems::camera_zoom::adjust_camera_home_for_seats,
                crate::systems::camera_zoom::camera_focus_hotkeys,
                crate::systems::camera_zoom::camera_zoom,
                crate::systems::eliminated::sync_eliminated_shrouds,
                crate::systems::table_tint::sync_seat_tints,
                crate::systems::table_tint::sync_seat_name_plates,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        // "You're out" panel for a viewer knocked out of a game that goes on.
        .add_systems(
            Update,
            (
                crate::systems::eliminated::sync_out_panel,
                crate::systems::eliminated::handle_out_panel_buttons,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Battlefield-anchored overlays must run AFTER animate_combat_lurch,
        // which adds the lunge offset to each attacker/blocker's transform. If
        // they run before it (the default unordered placement), they read the
        // pre-lunge resting position and the overlay (e.g. a block arrow
        // during DeclareBlockers) detaches from the card that has lunged forward.
        .add_systems(
            Update,
            (
                draw_block_arrows,
                crate::systems::combat_badge::sync_combat_chips,
                crate::systems::focus::apply_focus_dim,
                draw_stack_arrows,
                draw_attack_plan_arrows,
                draw_legal_target_rings,
                draw_target_arrow,
                crate::systems::gizmos::draw_drag_arrow,
                draw_attachment_tethers,
                draw_active_seat_glow,
            )
                .after(animate_combat_lurch)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (graveyard_card_hover_name, graveyard_recast_click)
                .after(graveyard_browser)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                handle_export_keypress,
                handle_planar_die_keypress,
                handle_reveal_conspiracy_keypress,
                handle_auto_pass_toggle,
                systems::export_prompt::handle_export_prompt_input,
                systems::export_prompt::sync_export_prompt_ui,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .init_resource::<systems::chat::ChatInputState>()
        .add_systems(
            Update,
            (
                systems::chat::drain_chat_inbox,
                systems::chat::handle_chat_input,
                systems::chat::sync_chat_ui,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                systems::debug_console::toggle_debug_console,
                systems::debug_console::handle_debug_console_input,
                systems::debug_console::handle_debug_console_buttons,
                systems::debug_console::handle_debug_console_suggestions,
                systems::debug_console::sync_debug_console_ui,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .init_resource::<systems::game_over::AutoRematchState>()
        .init_resource::<systems::eliminated::OutPanelState>()
        .init_resource::<systems::match_stats::MatchStats>()
        .init_resource::<systems::game_over::ActiveMatchKind>()
        // Defensive: the game-over systems read this as a required resource
        // every InGame frame. `start_net_session_from_menu` (re)sets it, but
        // init it too so no entry path can leave it missing.
        .init_resource::<systems::game_over::ActiveMatchFormat>()
        .init_resource::<systems::camera_zoom::CameraZoom>()
        .init_resource::<systems::camera_zoom::CameraHome>()
        .init_resource::<systems::camera_zoom::CameraFocusSeat>()
        .add_systems(
            Update,
            (
                systems::match_stats::track_match_stats,
                systems::game_over::sync_game_over_modal,
                systems::game_over::handle_auto_rematch_focus,
                systems::game_over::handle_auto_rematch_keys,
                systems::game_over::handle_auto_rematch_set,
                systems::game_over::refresh_auto_rematch_text,
                systems::game_over::handle_rematch_button,
                systems::game_over::handle_new_game_button,
                systems::game_over::handle_export_state_button,
                systems::game_over::apply_auto_rematch_on_game_over,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            OnExit(AppState::InGame),
            (
                systems::game_over::cleanup_in_game_entities,
                net_plugin::teardown_net_session,
            ),
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
                handle_trigger_reorder,
                handle_damage_order_reorder,
                handle_damage_assign_buttons,
                handle_search_select,
                handle_put_on_library_hand_click,
                handle_discard_select,
                update_put_on_library_count_text,
                update_put_on_library_visuals,
                (handle_confirm, handle_decision_cancel).chain(),
                handle_mulligan_buttons,
                handle_choose_color_buttons,
                handle_name_card_buttons,
                handle_learn_buttons,
                // Resolution-time choice modals (modes / amounts / divided
                // damage / creature type) — independent handlers grouped to
                // stay inside Bevy's tuple-arity limit.
                (
                    handle_optional_buttons,
                    handle_choose_modes_toggle,
                    handle_trigger_mode_buttons,
                    handle_amount_buttons,
                    handle_divide_damage_buttons,
                    handle_creature_type_buttons,
                    handle_randomizer_buttons,
                    handle_legend_keep_buttons,
                ),
                spawn_mode_pick_ui,
                handle_mode_pick_buttons,
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
        // Esc closes any open cast-flow picker (see `cancel_pickers_on_escape`).
        // After the per-picker handlers, so a click and an Esc in the same
        // frame resolve as the click.
        .add_systems(
            Update,
            cancel_pickers_on_escape
                .after(handle_game_input)
                .run_if(in_state(AppState::InGame)),
        )
        // Hand play-option menu: every way a hand card can be played, so
        // no mechanic is shadowed by the right-click quick-play cascade.
        .add_systems(
            Update,
            (handle_hand_menu, spawn_hand_menu)
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
        // Split-card half picker (CR 709 / 702.102): right-click a split
        // hand card to cast its right half or the fused whole.
        .add_systems(
            Update,
            (
                systems::game_ui::handle_split_cast_buttons,
                systems::game_ui::spawn_split_cast_modal,
            )
                .chain()
                .after(handle_game_input)
                .run_if(in_state(AppState::InGame)),
        )
        // Squad / Replicate "pay N times" stepper (CR 702.157 / 702.107).
        .add_systems(
            Update,
            (
                systems::game_ui::handle_pay_times_buttons,
                systems::game_ui::spawn_pay_times_modal,
                systems::game_ui::handle_spree_cast_buttons,
                systems::game_ui::spawn_spree_cast_modal,
                systems::game_ui::handle_helper_tap_buttons,
                systems::game_ui::spawn_helper_tap_modal,
            )
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
        // "Leave Game" button in the settings menu → back to main menu; the
        // UI size button beside the quality row.
        .add_systems(
            Update,
            (handle_leave_game_button, handle_ui_size_button).run_if(in_state(AppState::InGame)),
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

/// Mip bias for the 3-D camera's texture sampling; see the camera in
/// [`setup`]. Half a level: sharper card text without the texture shimmer a
/// full level brings in motion.
const CARD_MIP_BIAS: f32 = -0.5;

/// The key light's shadow biases (world units; the normal bias is scaled by
/// the shadow-map texel size). See the key light in [`setup`].
const KEY_LIGHT_DEPTH_BIAS: f32 = 0.005;
const KEY_LIGHT_NORMAL_BIAS: f32 = 0.4;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
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
    crate::systems::counter_coins::init_counter_coin_assets(&mut commands, &mut materials);

    // Ambient fill light — softens harsh shadows.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: gfx.ambient_brightness,
        ..default()
    });

    // Key light (directional, with shadows). Its cascades are refit to the
    // table from the live camera (`systems::shadows`); the biases are small
    // because nothing that lies flat on the table casts, so the felt has
    // nothing to shadow itself with — Bevy's defaults (0.02 depth, 1.8
    // normal) lifted the shadow test above a card resting on the felt.
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: gfx.key_light_illuminance,
            shadow_depth_bias: KEY_LIGHT_DEPTH_BIAS,
            shadow_normal_bias: KEY_LIGHT_NORMAL_BIAS,
            ..default()
        },
        CascadeShadowConfig::default(),
        systems::shadows::KeyLight,
    ));

    // Fill light from the opposite side — further reduces shadow darkness.
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, -2.0, -PI / 6.)),
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: gfx.fill_light_illuminance,
            ..default()
        },
    ));

    // Ground plane. Dark slate keeps glare off the card faces and lets the
    // (lit) cards read as the brightest thing on the table. Felt, under a
    // pool of light over a duel's boards (`table_cloth`); a game's seat
    // tints lie over it with the pool fitted to their table.
    const TABLE_COLOR: Color = Color::srgb(0.20, 0.20, 0.23);
    let cloth = systems::table_cloth::ClothTexture(images.add(systems::table_cloth::cloth_texture()));
    let ground = Rect::from_center_size(Vec2::ZERO, Vec2::splat(90.0));
    commands.spawn((
        Mesh3d(meshes.add(systems::table_cloth::table_mesh(ground, systems::table_cloth::play_area(0, 2, &Default::default())))),
        MeshMaterial3d(materials.add(systems::table_cloth::cloth_material(TABLE_COLOR, &cloth))),
        GroundPlane,
        // It lies flat under everything; see `systems::shadows`.
        bevy::light::NotShadowCaster,
        // Nothing is picked on the bare felt, and its ~8 k triangles were
        // ray-tested against every frame the pointer was over the table.
        Pickable::IGNORE,
    ));
    commands.insert_resource(cloth);

    let cam = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 32.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        quality.msaa(),
        // Card art shows as authored; see `SCENE_TONEMAPPING`.
        SCENE_TONEMAPPING,
        // Sample textures half a mip level sharper. A creature on the table
        // covers ~125 px of a 745 px card image at 1080p, so the GPU sampled
        // around mip 2.6 — mostly the 93-texel-wide level, coarser than the
        // pixels it covers — which is why table cards read softer than the
        // Alt-zoom popup (mip ~1). The table looks down at 66°, so the 16x
        // anisotropic filter barely engages and can't make up for it.
        bevy::render::camera::MipBias(CARD_MIP_BIAS),
        MainCamera,
    )).id();
    // Only attach SMAA when the current quality preset asks for it. Low
    // quality returns `None` here, matching the runtime
    // `apply_render_quality_change` path.
    if let Some(preset) = quality.smaa_preset() {
        commands.entity(cam).insert(Smaa { preset });
    }
    // Bloom needs an HDR camera; the `Hdr` marker switches the camera to an
    // intermediate HDR render target. Both gate on the same Low-disabled
    // preset as SMAA so the cheapest path skips the extra target + mip chain.
    if let Some(bloom) = quality.bloom() {
        commands.entity(cam).insert((Hdr, bloom));
    }
    // Contrast-adaptive sharpening — crisps up the (minified)
    // 3-D card faces. Gated off on Low like SMAA; see `RenderQuality::sharpening`.
    if let Some(cas) = quality.sharpening() {
        commands.entity(cam).insert(cas);
    }
}

/// The main camera's tonemapping: none.
///
/// The card faces are **unlit** — their pixels are the authored sRGB art —
/// and they are most of the screen. Every filmic tonemapper is a curve for
/// HDR scenes that rolls off toward white, and on flat SDR art that is a
/// grey-and-wash: measured on Walking Ballista at the zoom pose against its
/// Scryfall image (Lab, text box / art), the source reads L* 82.5 / chroma
/// 5.2; the old default (`TonyMcMapface` plus a 0.9 post-saturation grade)
/// drew 71.4 / 3.9 — the white text box eleven points grey and a quarter of
/// the art's colour gone. `AgX`, `AcesFitted`, `BlenderFilmic` and
/// `SomewhatBoringDisplayTransform` all drew the text box at 73-74; none
/// drew 86.6 / 4.6, the art as authored (the small lift is the sharpening
/// pass). The lit felt and the chips sit well inside `[0, 1]`, and the
/// glows that are driven past it (the seat frame, bursts) clip toward their
/// hue rather than wash to cream; bloom still reads them from the HDR
/// target on the tiers that have it.
const SCENE_TONEMAPPING: bevy::core_pipeline::tonemapping::Tonemapping =
    bevy::core_pipeline::tonemapping::Tonemapping::None;

#[allow(clippy::too_many_arguments)]
fn apply_render_quality_change(
    mut messages: MessageReader<ChangeQuality>,
    mut quality: ResMut<RenderQuality>,
    card_assets: Option<Res<CardMeshAssets>>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shadow_map: ResMut<DirectionalLightShadowMap>,
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
        && let Some(mut mesh) = meshes.get_mut(&assets.card_mesh) {
            *mesh = create_rounded_rect_mesh(CARD_WIDTH, CARD_HEIGHT, CORNER_RADIUS, segments);
        }

    if let Some(assets) = &highlight_assets {
        for (handle, width) in [
            (&assets.border_mesh, BORDER_WIDTH),
            (&assets.hover_border_mesh, card::HOVER_BORDER_WIDTH),
            (&assets.dying_border_mesh, card::DYING_BORDER_WIDTH),
        ] {
            if let Some(mut mesh) = meshes.get_mut(handle) {
                *mesh = create_border_mesh(CARD_WIDTH, CARD_HEIGHT, CORNER_RADIUS, width, segments);
            }
        }
    }

    shadow_map.size = new_quality.shadow_map_size();

    if let Ok(cam) = camera_query.single() {
        commands.entity(cam).insert(new_quality.msaa());
        match new_quality.smaa_preset() {
            Some(preset) => { commands.entity(cam).insert(Smaa { preset }); }
            None => { commands.entity(cam).remove::<Smaa>(); }
        }
        // Add/remove HDR + bloom together; the `Hdr` marker toggles the
        // camera's intermediate HDR render target that bloom requires.
        match new_quality.bloom() {
            Some(bloom) => { commands.entity(cam).insert((Hdr, bloom)); }
            None => { commands.entity(cam).remove::<(Hdr, Bloom)>(); }
        }
        match new_quality.sharpening() {
            Some(cas) => { commands.entity(cam).insert(cas); }
            None => { commands.entity(cam).remove::<ContrastAdaptiveSharpening>(); }
        }
    }
}

/// Pick a hand-zoom factor from the primary window's logical height
/// (`framing::hand_zoom_for`, which the camera fit also sizes the hand by).
fn pick_hand_zoom(logical_height: f32) -> f32 {
    card::framing::hand_zoom_for(logical_height)
}

/// Maximize the primary window on startup so the client opens sized to the
/// user's actual display instead of winit's fixed ~1280×720 default.
/// `set_maximized` records a request the winit backend applies on the next
/// frame, filling the monitor's work area — adapts to any resolution.
fn maximize_window(
    store: Option<Res<config::ConfigStore>>,
    harness: Res<layout_harness::HarnessArgs>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if harness.window.is_some() {
        return;
    }
    // Skipped when the user picked an explicit resolution in Settings
    // (`maximize_on_launch` flips off there) or chose borderless mode.
    let g = store.as_ref().map(|s| &s.0.graphics);
    let maximize = g.is_none_or(|g| {
        g.maximize_on_launch && g.window_mode == config::WindowModeCfg::Windowed
    });
    if maximize && let Ok(mut window) = windows.single_mut() {
        window.set_maximized(true);
    }
}

/// Update `HandZoom` whenever the primary window's height crosses a
/// tier boundary. Runs every frame; only writes when the chosen tier
/// actually changes, so it doesn't churn change-detection on every
/// hand-card consumer.
fn update_hand_zoom_from_window(
    windows: Query<&Window>,
    mut zoom: ResMut<HandZoom>,
) {
    let Ok(window) = windows.single() else { return };
    let target = pick_hand_zoom(window.height());
    if (zoom.0 - target).abs() > 0.001 {
        zoom.0 = target;
    }
}

/// Drive Bevy's built-in `UiScale` from the Settings "UI size" and the
/// primary window's logical height (`theme::ui_scale_for`). It was pinned at
/// 1.0 because a larger UI grew the corner panels over the table; the
/// camera fit now reserves them at their scaled size
/// (`framing::hud_rects`), and everything placed at a projected or cursor
/// position divides the scale back out (`theme::project_to_ui`).
fn update_ui_scale_from_window(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    store: Option<Res<config::ConfigStore>>,
    mut scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else { return };
    let percent = store.map_or(0, |s| s.0.graphics.ui_size);
    let target = theme::ui_scale_for(window.size(), percent);
    if (scale.0 - target).abs() > 0.001 {
        scale.0 = target;
    }
}
