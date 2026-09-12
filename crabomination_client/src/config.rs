#[cfg(not(target_arch = "wasm32"))]
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Parse a config document, falling back to defaults if it does not parse.
///
/// Split out of the loaders so the fallback is unit-testable without
/// touching the real config location. `label` only names the source in the
/// log line (a path natively, the storage key in the browser).
fn parse_or_default(text: &str, label: &str) -> Config {
    toml::from_str(text).unwrap_or_else(|e| {
        eprintln!(
            "config: {label} is invalid ({e}); using defaults. The file was \
             left as-is — fix or delete it to persist settings again."
        );
        Config::default()
    })
}

/// Returns the platform config file path:
/// - Windows:  %APPDATA%\crabomination\config.toml
/// - Linux:    ~/.config/crabomination/config.toml
/// - macOS:    ~/Library/Application Support/crabomination/config.toml
#[cfg(not(target_arch = "wasm32"))]
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crabomination")
        .join("config.toml")
}

/// Browser build: the whole TOML document lives under one localStorage key
/// (see `storage`); same format as the native file so a config is
/// copy-pasteable between the two.
#[cfg(target_arch = "wasm32")]
const CONFIG_STORAGE_KEY: &str = "config.toml";

/// Load config from localStorage, falling back to defaults. Unlike native
/// (which panics on a corrupt file the user hand-edited), a corrupt stored
/// value is discarded — there's no editor in the browser to fix it with.
#[cfg(target_arch = "wasm32")]
pub fn load() -> Config {
    match crate::storage::load(CONFIG_STORAGE_KEY) {
        Some(text) => parse_or_default(&text, CONFIG_STORAGE_KEY),
        None => Config::default(),
    }
}

/// Load config from the default location, writing defaults if the file is absent.
///
/// An unreadable or unparseable file falls back to defaults rather than
/// panicking: this file is meant to be hand-editable, and the game is the
/// only way most users would fix it — a mistyped key that refuses to launch
/// leaves them with no route back in. The broken file is deliberately left
/// on disk (not overwritten with defaults) so the edit can be recovered;
/// the next settings write is what replaces it.
#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Config {
    let path = config_path();
    if path.exists() {
        let Ok(text) = fs::read_to_string(&path).inspect_err(|e| {
            eprintln!("config: cannot read {} ({e}); using defaults", path.display());
        }) else {
            return Config::default();
        };
        parse_or_default(&text, &path.display().to_string())
    } else {
        let config = Config::default();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| eprintln!("Could not create config dir: {e}"));
        }
        let text = toml::to_string_pretty(&config).expect("config serialization failed");
        fs::write(&path, &text)
            .unwrap_or_else(|e| eprintln!("Could not write default config to {}: {e}", path.display()));
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub paths: PathsConfig,
    pub graphics: GraphicsConfig,
    pub gameplay: GameplayConfig,
}

/// The live, whole config kept as a resource so settings systems can
/// mutate one section and rewrite the file without losing the others.
/// Inserted before the menu/game plugins so `FromWorld`/seeding sees it.
#[derive(Resource)]
pub struct ConfigStore(pub Config);

/// Rewrite the config file with `config`. Failures are logged, never
/// fatal — losing a settings write shouldn't crash a running game.
#[cfg(target_arch = "wasm32")]
pub fn save(config: &Config) {
    match toml::to_string_pretty(config) {
        Ok(text) => {
            if !crate::storage::save(CONFIG_STORAGE_KEY, &text) {
                eprintln!("config: localStorage write failed");
            }
        }
        Err(e) => eprintln!("config: serialize failed: {e}"),
    }
}

/// Rewrite the config file with `config`. Failures are logged, never
/// fatal — losing a settings write shouldn't crash a running game.
#[cfg(not(target_arch = "wasm32"))]
pub fn save(config: &Config) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match toml::to_string_pretty(config) {
        Ok(text) => {
            if let Err(e) = fs::write(&path, text) {
                eprintln!("config: write {} failed: {e}", path.display());
            }
        }
        Err(e) => eprintln!("config: serialize failed: {e}"),
    }
}

/// Gameplay-feel options.
#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
#[serde(default)]
pub struct GameplayConfig {
    /// Sort your hand client-side (lands first, then by mana value, then
    /// name) instead of keeping draw order. Default: true.
    pub sort_hand: bool,
    /// Display name shown to other players. Empty = seed from the OS
    /// username. Persisted when edited in the menu.
    pub player_name: String,
    /// Last-used "join address" menu field. Empty = default.
    pub join_addr: String,
    /// Last-used decklist path for "Play Deck vs Bot". Empty = default.
    pub deck_path: String,
    /// Animation playback speed multiplier (the in-game `[` / `]` keys and
    /// slider persist here). Default: 1.0.
    pub animation_speed: f32,
    /// Phase-chart stop overrides for the viewer's own turns
    /// (step name → "always" / "skip"). Mirrors
    /// `systems::phase_bar::StopConfig.my`.
    pub stops_my: std::collections::HashMap<crabomination::game::TurnStep, crate::systems::phase_bar::StopMode>,
    /// Stop overrides for opponents' turns (`StopConfig.opp`).
    pub stops_opp: std::collections::HashMap<crabomination::game::TurnStep, crate::systems::phase_bar::StopMode>,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            sort_hand: true,
            player_name: String::new(),
            join_addr: String::new(),
            deck_path: String::new(),
            animation_speed: 1.0,
            stops_my: Default::default(),
            stops_opp: Default::default(),
        }
    }
}

/// Apply `f` to the **live** config and write the whole document back.
///
/// Every persistence path must go through the [`ConfigStore`] resource.
/// The previous `update()` helper re-read the file, edited that copy and
/// wrote it out, leaving `ConfigStore` — built once at startup and never
/// re-synced — stale. Because `persist_stops` / `persist_animation_speed`
/// / the settings menu all rewrite the *whole* document from the store,
/// the next one to fire silently reverted whatever `update()` had written:
/// typing a player name in the menu and then nudging animation speed
/// in-game restored the old name.
pub fn update_store(store: &mut ConfigStore, f: impl FnOnce(&mut Config)) {
    f(&mut store.0);
    save(&store.0);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    /// Directory used as Bevy's asset root and card image cache.
    ///
    /// An absolute path is used verbatim. A relative path (the default
    /// `"assets"`) is resolved by [`PathsConfig::resolved_asset_dir`] to a
    /// fixed location that does not depend on the current working directory.
    pub asset_dir: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self { asset_dir: "assets".to_string() }
    }
}

impl PathsConfig {
    /// Resolve [`asset_dir`](Self::asset_dir) to an absolute path.
    ///
    /// An absolute configured path is returned as-is. A relative path is
    /// anchored so the binary finds its assets regardless of where it is
    /// launched from:
    /// - **debug builds** anchor to the crate source dir
    ///   (`CARGO_MANIFEST_DIR`), so `cargo run` / `cargo dev` locate
    ///   `crabomination_client/assets` from any working directory;
    /// - **release builds** anchor to the executable's own directory, so a
    ///   shipped build just needs an `assets/` folder next to the binary.
    pub fn resolved_asset_dir(&self) -> PathBuf {
        let configured = Path::new(&self.asset_dir);
        if configured.is_absolute() {
            return configured.to_path_buf();
        }
        asset_base_dir().join(configured)
    }
}

/// Base directory that a relative `asset_dir` is resolved against.
#[cfg(debug_assertions)]
fn asset_base_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[cfg(not(debug_assertions))]
fn asset_base_dir() -> PathBuf {
    // Fall back to "." (current dir) only if the exe path is somehow
    // unavailable — practically never on supported platforms.
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Window mode for the primary window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowModeCfg {
    #[default]
    Windowed,
    Borderless,
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsConfig {
    /// Windowed or borderless-fullscreen. Default: windowed.
    pub window_mode: WindowModeCfg,
    /// Windowed-mode resolution (the restore size when un-maximized).
    pub window_width: u32,
    pub window_height: u32,
    /// Maximize the window on launch (legacy default). Picking an explicit
    /// resolution in the settings menu turns this off.
    pub maximize_on_launch: bool,
    /// Render quality preset (shadows, AA, mesh detail). Default: low.
    pub render_quality: crate::render_quality::RenderQuality,
    /// Shadow map resolution (must be a power of two). Default: 8192.
    pub shadow_map_size: usize,
    /// SMAA anti-aliasing preset: "off", "low", "medium", "high", "ultra". Default: "ultra".
    pub smaa_preset: SmaaPreset,
    /// Ambient light brightness. Default: 600.
    pub ambient_brightness: f32,
    /// Key directional light illuminance (lux). Default: 3500.
    pub key_light_illuminance: f32,
    /// Fill directional light illuminance (lux). Default: 1500.
    pub fill_light_illuminance: f32,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            window_mode: WindowModeCfg::default(),
            window_width: 1600,
            window_height: 1000,
            maximize_on_launch: true,
            render_quality: crate::render_quality::RenderQuality::default(),
            shadow_map_size: 8192,
            smaa_preset: SmaaPreset::Ultra,
            ambient_brightness: 600.0,
            key_light_illuminance: 3500.0,
            fill_light_illuminance: 1500.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmaaPreset {
    Low,
    Medium,
    High,
    #[default]
    Ultra,
}

impl SmaaPreset {
    // Kept for back-compat with existing config.toml files that still set
    // `graphics.smaa_preset`; the renderer now drives SMAA from
    // `RenderQuality`, so this method may be unused.
    #[allow(dead_code)]
    pub fn to_bevy(self) -> bevy::anti_alias::smaa::SmaaPreset {
        use bevy::anti_alias::smaa::SmaaPreset as B;
        match self {
            Self::Low    => B::Low,
            Self::Medium => B::Medium,
            Self::High   => B::High,
            Self::Ultra  => B::Ultra,
        }
    }
}


// ── Persistence systems ───────────────────────────────────────────────────────

use bevy::prelude::{DetectChanges, Res, ResMut};

/// Mirror phase-chart stop changes into the config file. Skips the
/// startup frame (`is_added`) and no-op writes.
pub fn persist_stops(
    stops: Res<crate::systems::phase_bar::StopConfig>,
    mut store: ResMut<ConfigStore>,
) {
    if !stops.is_changed() || stops.is_added() {
        return;
    }
    if store.0.gameplay.stops_my == stops.my && store.0.gameplay.stops_opp == stops.opp {
        return;
    }
    store.0.gameplay.stops_my = stops.my.clone();
    store.0.gameplay.stops_opp = stops.opp.clone();
    save(&store.0);
}

/// Mirror animation-speed changes (the `[` / `]` keys and the settings
/// slider) into the config file.
pub fn persist_animation_speed(
    speed: Res<crate::systems::animate::AnimationSpeed>,
    mut store: ResMut<ConfigStore>,
) {
    if !speed.is_changed() || speed.is_added() {
        return;
    }
    if (store.0.gameplay.animation_speed - speed.0).abs() < f32::EPSILON {
        return;
    }
    store.0.gameplay.animation_speed = speed.0;
    save(&store.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-edited config with a typo used to `panic!` out of `load()`,
    /// so the game refused to launch and the only UI that could have
    /// fixed the file was the one that wouldn't start.
    #[test]
    fn invalid_config_falls_back_to_defaults_instead_of_panicking() {
        let cfg = parse_or_default("this is not valid toml {{{", "test");
        let default = Config::default();
        assert_eq!(cfg.gameplay.sort_hand, default.gameplay.sort_hand);
        assert_eq!(cfg.graphics.window_width, default.graphics.window_width);
    }

    /// A key with the wrong *type* is the likeliest hand-edit slip
    /// (quoting a number), and takes the same path.
    #[test]
    fn wrong_typed_field_falls_back_to_defaults() {
        let cfg = parse_or_default("[graphics]\nwindow_width = \"1600\"\n", "test");
        assert_eq!(cfg.graphics.window_width, Config::default().graphics.window_width);
    }

    /// The fallback must not swallow a *valid* partial document — every
    /// section is `#[serde(default)]`, so absent keys keep their defaults
    /// while present ones are honoured.
    #[test]
    fn partial_config_keeps_present_values_and_defaults_the_rest() {
        let cfg = parse_or_default("[gameplay]\nplayer_name = \"Alice\"\n", "test");
        assert_eq!(cfg.gameplay.player_name, "Alice");
        assert_eq!(cfg.gameplay.animation_speed, Config::default().gameplay.animation_speed);
    }

    /// Every persistence path rewrites the *whole* document from
    /// `ConfigStore`, which is what made the stale-store bug destructive:
    /// an in-game animation-speed write reverted the menu's player name.
    /// That is only safe if a fully-populated config survives the
    /// serialize/deserialize round trip with no section dropped.
    #[test]
    fn whole_document_round_trips_with_every_section_populated() {
        let mut cfg = Config::default();
        cfg.paths.asset_dir = "/tmp/crab-assets".into();
        cfg.graphics.window_mode = WindowModeCfg::Borderless;
        cfg.graphics.window_width = 2560;
        cfg.graphics.maximize_on_launch = false;
        cfg.gameplay.player_name = "Alice".into();
        cfg.gameplay.join_addr = "10.0.0.2:7777".into();
        cfg.gameplay.deck_path = "decks/mono-red.txt".into();
        cfg.gameplay.animation_speed = 2.0;

        let back = parse_or_default(
            &toml::to_string_pretty(&cfg).expect("serialize"),
            "test",
        );

        assert_eq!(back.paths.asset_dir, "/tmp/crab-assets");
        assert_eq!(back.graphics.window_mode, WindowModeCfg::Borderless);
        assert_eq!(back.graphics.window_width, 2560);
        assert!(!back.graphics.maximize_on_launch);
        assert_eq!(back.gameplay.player_name, "Alice");
        assert_eq!(back.gameplay.join_addr, "10.0.0.2:7777");
        assert_eq!(back.gameplay.deck_path, "decks/mono-red.txt");
        assert_eq!(back.gameplay.animation_speed, 2.0);
    }

    #[test]
    fn gameplay_config_with_stops_roundtrips_through_toml() {
        use crate::systems::phase_bar::StopMode;
        use crabomination::game::TurnStep;
        let mut cfg = Config::default();
        cfg.gameplay.player_name = "Alice".into();
        cfg.gameplay.animation_speed = 1.5;
        cfg.gameplay.stops_my.insert(TurnStep::End, StopMode::Always);
        cfg.gameplay.stops_opp.insert(TurnStep::Upkeep, StopMode::Skip);
        let text = toml::to_string_pretty(&cfg).expect("serialize");
        let back: Config = toml::from_str(&text).expect("deserialize");
        assert_eq!(back.gameplay.player_name, "Alice");
        assert_eq!(back.gameplay.stops_my.get(&TurnStep::End), Some(&StopMode::Always));
        assert_eq!(back.gameplay.stops_opp.get(&TurnStep::Upkeep), Some(&StopMode::Skip));
        assert!((back.gameplay.animation_speed - 1.5).abs() < f32::EPSILON);
    }
}
