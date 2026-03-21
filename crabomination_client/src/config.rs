use std::fs;
use std::path::PathBuf;

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
pub struct Config {
    pub paths: PathsConfig,
    pub graphics: GraphicsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paths: PathsConfig::default(),
            graphics: GraphicsConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    /// Directory used as Bevy's asset root and card image cache.
    /// Defaults to the "assets" folder next to the executable.
    pub asset_dir: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self { asset_dir: "assets".to_string() }
    }
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
