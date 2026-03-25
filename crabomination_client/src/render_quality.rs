use bevy::anti_alias::smaa::SmaaPreset;
use bevy::prelude::*;

/// Message fired when the user selects a new quality preset from the menu.
#[derive(Message, Clone, Copy)]
pub struct ChangeQuality(pub RenderQuality);

/// Render quality preset, selectable from the in-game menu.
///
/// Controls shadow map resolution, anti-aliasing, and mesh detail.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // all variants are valid user-selectable quality levels
pub enum RenderQuality {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

impl RenderQuality {
    pub const ALL: [RenderQuality; 4] = [
        RenderQuality::Low,
        RenderQuality::Medium,
        RenderQuality::High,
        RenderQuality::Ultra,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Med",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }

    /// Shadow map resolution (pixels per side). Higher = sharper shadows.
    pub fn shadow_map_size(self) -> usize {
        match self {
            Self::Low => 512,
            Self::Medium => 2048,
            Self::High => 4096,
            Self::Ultra => 8192,
        }
    }

    /// SMAA anti-aliasing preset. `None` disables SMAA entirely (Low quality).
    pub fn smaa_preset(self) -> Option<SmaaPreset> {
        match self {
            Self::Low => None,
            Self::Medium => Some(SmaaPreset::Low),
            Self::High => Some(SmaaPreset::High),
            Self::Ultra => Some(SmaaPreset::Ultra),
        }
    }

    /// Number of arc segments per corner on card meshes. Higher = smoother corners.
    pub fn corner_segments(self) -> usize {
        match self {
            Self::Low => 3,
            Self::Medium => 5,
            Self::High => 8,
            Self::Ultra => 12,
        }
    }

    /// Ground plane mesh subdivisions. Higher = smoother lighting across the ground.
    pub fn ground_subdivisions(self) -> u32 {
        match self {
            Self::Low => 0,
            Self::Medium => 4,
            Self::High => 8,
            Self::Ultra => 10,
        }
    }
}
