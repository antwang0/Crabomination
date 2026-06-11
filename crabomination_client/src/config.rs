use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Returns the platform config file path:
/// - Windows:  %APPDATA%\crabomination\config.toml
/// - Linux:    ~/.config/crabomination/config.toml
/// - macOS:    ~/Library/Application Support/crabomination/config.toml
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crabomination")
        .join("config.toml")
}

/// Load config from the default location, writing defaults if the file is absent.
pub fn load() -> Config {
    let path = config_path();
    if path.exists() {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read config at {}: {e}", path.display()));
        toml::from_str(&text)
            .unwrap_or_else(|e| panic!("Invalid config at {}: {e}", path.display()))
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

/// Load the config, apply `f`, and write it back. Used by the menu to
/// persist edited fields (player name, join address, deck path).
pub fn update(f: impl FnOnce(&mut Config)) {
    let mut cfg = load();
    f(&mut cfg);
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match toml::to_string_pretty(&cfg) {
        Ok(text) => {
            if let Err(e) = fs::write(&path, &text) {
                eprintln!("Could not persist config to {}: {e}", path.display());
            }
        }
        Err(e) => eprintln!("Config serialization failed: {e}"),
    }
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

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsConfig {
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
