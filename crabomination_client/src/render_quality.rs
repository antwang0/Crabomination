use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::anti_alias::smaa::SmaaPreset;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
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
    #[default]
    Low,
    Medium,
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

    /// HDR + bloom post-processing for the main camera. `None` disables it
    /// entirely (Low quality — keeps the cheapest path free of the extra HDR
    /// render target and bloom mip chain, mirroring the SMAA gating).
    ///
    /// We deliberately do **not** use `Bloom::NATURAL`: it is energy-conserving
    /// with a zero threshold, so it blooms every pixel — including the
    /// normally-lit card art — and the whole table reads as a soft haze. Here
    /// we want only genuinely bright, over-bright (HDR > 1.0) highlights to
    /// glow: specular pops, and later the emissive overlays / counters. So we
    /// run an *additive* bloom behind a ~1.0 prefilter threshold, which leaves
    /// lit card midtones crisp and only adds glow on top of the brightest
    /// pixels. Any camera carrying a `Bloom` component must also carry the
    /// `Hdr` marker — see `setup_scene` / `apply_render_quality_change`.
    pub fn bloom(self) -> Option<Bloom> {
        match self {
            Self::Low => None,
            _ => Some(Bloom {
                intensity: 0.2,
                composite_mode: BloomCompositeMode::Additive,
                prefilter: BloomPrefilter { threshold: 1.0, threshold_softness: 0.5 },
                ..Bloom::NATURAL
            }),
        }
    }

    /// Contrast-adaptive sharpening for the main camera. `None` on Low.
    ///
    /// Card faces lie flat and minified at the table's oblique angle, so even
    /// with a mip chain + 16× anisotropy they read softer than the
    /// screen-space Alt-zoom popup (which samples mip 0 at ~1:1 and skips
    /// tonemapping entirely). A light CAS pass recovers edge contrast on the
    /// rendered 3-D output and visibly tightens card text. Gated off on Low
    /// because that tier has no MSAA/SMAA — sharpening aliased edges there
    /// would amplify shimmer rather than help.
    pub fn sharpening(self) -> Option<ContrastAdaptiveSharpening> {
        match self {
            Self::Low => None,
            _ => Some(ContrastAdaptiveSharpening {
                enabled: true,
                // Below the 0.6 default: enough to crisp up card text without
                // the ringing/halos a stronger pass leaves on high-contrast art.
                sharpening_strength: 0.5,
                denoise: false,
            }),
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

    /// MSAA sample count. Higher = smoother geometry edges at increased GPU cost.
    /// Low disables MSAA (SMAA post-process still applies on Medium+).
    pub fn msaa(self) -> Msaa {
        match self {
            Self::Low => Msaa::Off,
            Self::Medium => Msaa::Sample2,
            Self::High => Msaa::Sample4,
            Self::Ultra => Msaa::Sample4,
        }
    }
}
