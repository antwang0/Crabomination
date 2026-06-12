#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::f32::consts::PI;

use bevy::asset::AssetApp;
use bevy::asset::io::{AssetSource, AssetSourceBuilder, AssetSourceId, ErasedAssetReader};
use bevy::image::{ImageFilterMode, ImageSamplerDescriptor};
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::post_process::bloom::Bloom;
use bevy::render::view::Hdr;
use bevy::{anti_alias::smaa::Smaa, color::palettes::basic::SILVER, prelude::*};

mod audit;
mod card;
mod config;
mod debug_export;
mod embedded_assets;
mod game;
mod menu;
mod net_plugin;
mod render_quality;
mod scryfall;
mod synthesized_cards;
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
use systems::animate::{
    adjust_animation_speed, animate_combat_lurch, animate_deck_shuffle, animate_draw_card,
    animate_flip, dispatch_animation_queue, animate_mdfc_flip, animate_hand_slide,
    animate_hover_lift, animate_play_card, animate_return_to_deck, animate_return_to_hand,
    animate_reveal_peek, animate_send_to_graveyard, animate_tap, update_combat_lurch_targets,
    AnimationSpeed,
};
use systems::game_ui::{
    apply_swap_front_material, auto_advance_p0, handle_ability_menu, handle_alt_cast_buttons,
    handle_auto_pass_toggle, handle_export_keypress, handle_game_input, poll_action_buttons,
    poll_player_chip_clicks,
    setup_game_hud,
    spawn_ability_menu, spawn_alt_cast_modal, sync_command_zone, sync_flipped_hand_cards,
    sync_game_visuals,
    handle_audit_buttons, handle_surrender_leave_buttons, pulse_urgent_pass_button,
    sync_audit_buttons, sync_player_hud_seat,
    sync_hint_chip_visibility, trigger_reveal_animation, update_attack_all_visibility,
    update_attack_button_label,
    animate_phase_banner, trigger_phase_banner, PhaseBannerTracker,
    animate_life_flash, trigger_life_flash, LifeFlashTracker,
    update_combat_preview_panel,
    position_log_below_opponents,
    update_log_text, update_mana_pips, update_opponent_panel_tint, update_opponent_stats_rows,
    update_hint, update_pass_button, update_phase_chart, update_player_chip_target_outline,
    update_phase_bar, update_player_stats_chips, update_stack_panel,
    handle_stack_resolve_button, update_turn_text,
    ButtonState, GameLogicSet,
};
use systems::gizmos::{
    draw_attack_plan_gizmos, draw_attacker_overlays, draw_blocking_gizmos,
    draw_legal_target_rings, draw_pt_modified_overlays, draw_stack_arrows, draw_target_arrow,
    AttackPlanGizmos, AttackerGizmos, BlockingGizmos, LegalTargetGizmos, PtModifiedGizmos,
    StackGizmos, TargetArrowGizmos,
};
use systems::quality::{
    handle_leave_game_button, handle_quality_buttons, handle_settings_toggle, handle_speed_slider,
    reset_esc_consumed, setup_quality_panel, sync_settings_visibility, update_speed_slider_visuals,
    EscConsumed, SettingsOpen,
};
use systems::ui::{
    exile_browser, graveyard_browser, graveyard_card_hover_name, highlight_hovered_cards, hover_card_preview,
    toggle_shortcut_help, update_castable_highlights, update_dying_highlights,
    update_activatable_highlights, peek_popup, pile_tooltip, reveal_popup, RevealPopupState,
};
use systems::decision_ui::{spawn_decision_ui, handle_scry_toggles, handle_scry_reorder, handle_trigger_reorder, handle_damage_order_reorder, handle_damage_assign_buttons, handle_search_select, handle_put_on_library_select, handle_put_on_library_hand_click, handle_discard_select, update_put_on_library_count_text, update_put_on_library_visuals, handle_choose_color_buttons, handle_name_card_buttons, handle_learn_buttons, handle_confirm, handle_mulligan_buttons, spawn_mode_pick_ui, handle_mode_pick_buttons, handle_optional_buttons, handle_choose_modes_toggle, handle_trigger_mode_buttons, handle_amount_buttons, handle_divide_damage_buttons, handle_creature_type_buttons, DecisionUiState};

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
    let asset_dir = cfg.paths.resolved_asset_dir();
    // A custom/fresh asset dir starts empty — materialize the embedded
    // core assets (font, cardback, table model) before anything reads them.
    embedded_assets::materialize_core_assets(&asset_dir);
    // Card-art prefetch runs on a background thread — launch never blocks
    // on the network. Missing images render as name placeholders (see
    // `CardPlaceholderReader`) and hot-swap to real art as downloads land
    // (`reload_completed_images`); the menu shows live progress.
    let image_prefetch = scryfall::ImagePrefetch::default();
    {
        let progress = image_prefetch.clone();
        let dir = asset_dir.clone();
        std::thread::spawn(move || {
            scryfall::ensure_card_images_with_progress(&specs, &dir, &progress);
        });
    }

    let gfx = cfg.graphics;
    let gameplay = cfg.gameplay;
    // Whole-config resource for the persistence systems (settings writes
    // rewrite the file without losing other sections). Cloned before the
    // sections move into their own resources below.
    let cfg_store = config::ConfigStore(config::Config {
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
    let (cfg_window_w, cfg_window_h) = (gfx.window_width, gfx.window_height);
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
                // that the corner panels overlap and the hand clips. (The
                // resolution-aware `UiScale` hook stays a no-op; see
                // `pick_ui_scale`.)
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
        ))
        .init_gizmo_group::<BlockingGizmos>()
        .init_gizmo_group::<AttackerGizmos>()
        .init_gizmo_group::<StackGizmos>()
        .init_gizmo_group::<PtModifiedGizmos>()
        .init_gizmo_group::<AttackPlanGizmos>()
        .init_gizmo_group::<LegalTargetGizmos>()
        .init_gizmo_group::<TargetArrowGizmos>()
        .init_gizmo_group::<crate::systems::impact::ImpactGizmos>()
        .add_systems(Startup, configure_gizmos)
        .insert_resource(DirectionalLightShadowMap { size: cfg_quality.shadow_map_size() })
        .insert_resource(gfx)
        .insert_resource(gameplay)
        .insert_resource(cfg_quality)
        .add_message::<ChangeQuality>()
        .insert_resource(GameLog::default())
        .insert_resource(PhaseBannerTracker::default())
        .insert_resource(LifeFlashTracker::default())
        .insert_resource(FastForward::default())
        .insert_resource(TargetingState::default())
        .insert_resource(game::LegalTargets::default())
        .insert_resource(game::PendingModalCast::default())
        .insert_resource(BlockingState::default())
        .insert_resource(AttackingState::default())
        .insert_resource(AltCastState::default())
        .insert_resource(game::SplitCastState::default())
        .insert_resource(game::PayTimesState::default())
        .insert_resource(FlippedHandCards::default())
        .insert_resource(CardNames::default())
        .insert_resource(GraveyardBrowserState::default())
        .insert_resource(RevealPopupState::default())
        .insert_resource(initial_anim_speed)
        .insert_resource(ButtonState::default())
        .insert_resource(HandZoom::default())
        .insert_resource(systems::kb_cursor::KeyboardCursor::default())
        .insert_resource(SettingsOpen::default())
        .insert_resource(EscConsumed::default())
        .insert_resource(audit::AuditTarget::default())
        .insert_resource(audit::AuditPoolFilter::default())
        .insert_resource(audit::ShowVerifiedCards::default())
        .insert_resource(audit::load_audited_cards())
        .insert_resource(DecisionUiState::default())
        .insert_resource(systems::debug_console::DebugConsoleState::default())
        .init_resource::<game::AbilityMenuState>()
        .init_resource::<systems::export_prompt::ExportPromptState>()
        .init_resource::<systems::game_ui::SurrenderConfirm>()
        .insert_resource(initial_stops)
        .insert_resource(menu::CliBootHint(load_state_arg))
        .insert_resource(menu::CliBootFormat(play_format_arg))
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
            (audit::handle_audit_picker, audit::handle_audit_scroll)
                .run_if(in_state(AppState::Audit)),
        )
        // Audit-mode HUD buttons: sync visibility every frame, handle
        // click → save + return to picker.
        .add_systems(
            Update,
            (sync_audit_buttons, handle_audit_buttons, handle_surrender_leave_buttons)
                .run_if(in_state(AppState::InGame)),
        )
        // Button polling runs first so handle_game_input can read latched state.
        // `poll_player_chip_clicks` must run after `poll_action_buttons` so it
        // can fill the chip slot that the action-button poll first cleared.
        .add_systems(
            Update,
            (poll_action_buttons, poll_player_chip_clicks)
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        // Esc / settings precedence chain. `reset_esc_consumed` clears
        // the cross-system flag, `handle_settings_toggle` claims Esc
        // when it acts on the modal, and `sync_settings_visibility`
        // flips the modal's `Display` on toggle. The kb-cursor input
        // system reads `EscConsumed` to know when to skip its own Esc
        // clear, so it must run after the toggle handler.
        .add_systems(
            Update,
            (
                reset_esc_consumed,
                handle_settings_toggle,
                sync_settings_visibility,
            )
                .chain()
                .before(systems::kb_cursor::handle_keyboard_cursor_input)
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
                .after(poll_action_buttons)
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
        // HUD refresh (after game logic)
        .add_systems(
            Update,
            (
                update_turn_text,
                update_phase_bar,
                update_player_stats_chips,
                update_mana_pips,
                update_opponent_stats_rows,
                update_opponent_panel_tint,
                position_log_below_opponents,
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
                adjust_animation_speed,
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
                toggle_shortcut_help,
                trigger_phase_banner,
                animate_phase_banner,
                trigger_life_flash,
                animate_life_flash,
                hover_card_preview,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Combat lurch: read attacker/blocker state, lerp Z forward.
        // Must run after animate_hover_lift since that system overwrites
        // transform.translation each frame from `base_translation`.
        .add_systems(
            Update,
            (update_combat_lurch_targets, animate_combat_lurch)
                .chain()
                .after(animate_hover_lift)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, animate_mdfc_flip.run_if(in_state(AppState::InGame)))
        .add_systems(Update, animate_return_to_hand.run_if(in_state(AppState::InGame)))
        // Pop the next queued animation onto an entity once it stops
        // animating, so chained transitions (e.g. play-then-tap on a
        // freshly-played land) play sequentially.
        .add_systems(Update, dispatch_animation_queue.run_if(in_state(AppState::InGame)))
        // Give every freshly-loaded card-face texture a mip chain so the
        // sampler's 16× anisotropy keeps text legible at the table's oblique
        // angle. Ungated: card images load during draft/menu as well as in a
        // match, and the system self-skips anything already mipmapped.
        .add_systems(Update, crate::card::mipmap::generate_card_mipmaps)
        // Counter coins (3-D cylinders on top of permanents).
        .add_systems(
            Update,
            crate::systems::counter_coins::sync_counter_coins
                .run_if(in_state(AppState::InGame)),
        )
        // Impact feedback: death bursts, damage sparks, life-loss vignette.
        .add_systems(
            Update,
            (
                crate::systems::impact::spawn_impact_effects,
                crate::systems::impact::spawn_mana_motes,
                crate::systems::impact::animate_impact_bursts,
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
        // Screen-space "<type> ×N" labels naming each counter and its count.
        .add_systems(
            Update,
            crate::systems::counter_coins::sync_counter_labels
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
        // Clarity batch: hover-preview for UI tiles (stack panel / log
        // lines), pulsing ring on the pending decision's source, and the
        // low-life danger frame.
        .add_systems(
            Update,
            (
                crate::systems::ui_card_hover::ui_card_hover_preview,
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
                crate::systems::camera_zoom::camera_zoom,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        // Battlefield-anchored gizmo overlays must run AFTER animate_combat_lurch,
        // which adds the lunge offset to each attacker/blocker's transform. If
        // they run before it (the default unordered placement), they read the
        // pre-lunge resting position and the overlay (e.g. the attacker swords
        // during DeclareBlockers) detaches from the card that has lunged forward.
        .add_systems(
            Update,
            (
                draw_blocking_gizmos,
                draw_attacker_overlays,
                draw_stack_arrows,
                draw_attack_plan_gizmos,
                draw_pt_modified_overlays,
                draw_legal_target_rings,
                draw_target_arrow,
            )
                .after(animate_combat_lurch)
                .run_if(in_state(AppState::InGame)),
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
                handle_auto_pass_toggle,
                systems::export_prompt::handle_export_prompt_input,
                systems::export_prompt::sync_export_prompt_ui,
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
        .init_resource::<systems::game_over::ActiveMatchKind>()
        // Defensive: the game-over systems read this as a required resource
        // every InGame frame. `start_net_session_from_menu` (re)sets it, but
        // init it too so no entry path can leave it missing.
        .init_resource::<systems::game_over::ActiveMatchFormat>()
        .init_resource::<systems::camera_zoom::CameraZoom>()
        .init_resource::<systems::camera_zoom::CameraHome>()
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
                handle_put_on_library_select,
                handle_put_on_library_hand_click,
                handle_discard_select,
                update_put_on_library_count_text,
                update_put_on_library_visuals,
                handle_confirm,
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
        // "Leave Game" button in the settings menu → back to main menu.
        .add_systems(
            Update,
            handle_leave_game_button.run_if(in_state(AppState::InGame)),
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
    let (config, _) = store.config_mut::<AttackPlanGizmos>();
    config.line.width = 4.0;
    let (config, _) = store.config_mut::<LegalTargetGizmos>();
    config.line.width = 5.0;
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
        Mesh3d(meshes.add(Plane3d::default().mesh().size(90.0, 90.0).subdivisions(quality.ground_subdivisions()))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        GroundPlane,
    ));

    let cam = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 32.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        quality.msaa(),
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
    // Contrast-adaptive sharpening — crisps up the (minified, tonemapped)
    // 3-D card faces. Gated off on Low like SMAA; see `RenderQuality::sharpening`.
    if let Some(cas) = quality.sharpening() {
        commands.entity(cam).insert(cas);
    }
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
            *mesh = Plane3d::default().mesh().size(90.0, 90.0)
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

/// Pick a hand-zoom factor from the primary window's logical height.
/// 1.0 at 1440p+, ramping up on smaller displays so the viewer's hand
/// stays roughly the same apparent size regardless of resolution.
/// Conservative curve — earlier values overflowed the play area into
/// the corner HUD panels at 1080p, so we keep the multiplier small
/// and only bump it for genuinely-small displays.
fn pick_hand_zoom(logical_height: f32) -> f32 {
    if logical_height >= 1080.0 {
        1.0
    } else if logical_height >= 800.0 {
        1.15
    } else {
        1.30
    }
}

/// Maximize the primary window on startup so the client opens sized to the
/// user's actual display instead of winit's fixed ~1280×720 default.
/// `set_maximized` records a request the winit backend applies on the next
/// frame, filling the monitor's work area — adapts to any resolution.
fn maximize_window(
    store: Option<Res<config::ConfigStore>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
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

/// 2-D UI scale tier. Currently a no-op — every tier returns `1.0` —
/// because earlier non-1.0 values grew the corner HUD panels enough
/// to cover the bottom-center hand area at 1080p. Kept as a stub so
/// re-enabling it is a one-line tuning change once the HUD layout
/// stops anchoring everything to the corners.
fn pick_ui_scale(_logical_height: f32) -> f32 {
    1.0
}

/// Drive Bevy's built-in `UiScale` from the primary window's logical
/// height. Currently a no-op (see `pick_ui_scale`) but the system
/// stays registered so re-enabling resolution-aware UI scaling
/// doesn't need to re-touch `main()`.
fn update_ui_scale_from_window(
    windows: Query<&Window>,
    mut scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else { return };
    let target = pick_ui_scale(window.height());
    if (scale.0 - target).abs() > 0.001 {
        scale.0 = target;
    }
}
