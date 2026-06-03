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

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub paths: PathsConfig,
    pub graphics: GraphicsConfig,
}


#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Resource, Serialize, Deserialize)]
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
