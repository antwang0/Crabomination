//! On-load mipmap generation for card-face textures.
//!
//! Cards lie flat on the table and are viewed at a steep, foreshortened
//! angle, so their textures are heavily minified — especially opponent
//! boards and cards at the back of the table. Bevy's PNG loader produces
//! a single mip level, which makes the configured 16× anisotropic filter
//! (see `ImagePlugin` in `main.rs`) a no-op: with no mip chain to sample
//! from, minified card text shimmers and aliases into mush.
//!
//! There is no built-in CPU mipmap generator in this Bevy version, so we
//! build one ourselves: whenever a `cards/…` (or the shared `cardback`)
//! `Image` finishes loading, we downsample it into a full mip chain and
//! hand the packed levels back to the asset. wgpu uploads the packed mips
//! straight from `Image::data` (`create_texture_with_data`), so this is
//! all that's needed for the sampler's mip + anisotropy settings to start
//! doing their job.

use bevy::asset::AssetEvent;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

/// True for the RGBA8 formats the PNG loader emits — the only formats our
/// byte-level box filter knows how to downsample.
fn is_rgba8(format: TextureFormat) -> bool {
    matches!(format, TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm)
}

/// Decode an 8-bit sRGB-encoded channel to linear light in `0.0..=1.0`.
fn srgb_to_linear(byte: u8) -> f32 {
    let s = byte as f32 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// Encode a linear-light value back to an 8-bit sRGB channel (rounded).
fn linear_to_srgb(linear: f32) -> u8 {
    let l = linear.clamp(0.0, 1.0);
    let s = if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0 + 0.5) as u8
}

/// Whether `path` names a card-face texture we want mipmapped (the per-card
/// `cards/<name>.png` images plus the shared card back).
fn is_card_texture(path: &str) -> bool {
    path.starts_with("cards/") || path == "cardback.png"
}

/// Generate a full mip chain for an RGBA8 [`Image`] in place, each level a
/// 2×2 downsample of the one above. No-op if the image already carries mips,
/// isn't RGBA8, or has no CPU-side data.
///
/// For sRGB-encoded images (the card faces) the RGB channels are decoded to
/// **linear light**, averaged there, then re-encoded — a gamma-correct box
/// filter. Averaging the raw sRGB bytes instead (the naive path) biases mid
/// mips toward a muddy mid-gray and visibly softens dark card text against
/// light frames; doing it in linear light keeps edge contrast, which is what
/// the table card actually samples at its oblique angle. Alpha is already
/// linear, so it (and genuinely-linear `Rgba8Unorm` images) box-averages
/// directly.
pub fn generate_mipmaps(image: &mut Image) -> bool {
    if image.texture_descriptor.mip_level_count > 1 {
        return false;
    }
    let format = image.texture_descriptor.format;
    if !is_rgba8(format) {
        return false;
    }
    let width = image.texture_descriptor.size.width;
    let height = image.texture_descriptor.size.height;
    if width <= 1 && height <= 1 {
        return false;
    }
    let Some(base) = image.data.as_ref() else {
        return false;
    };
    if base.len() != (width as usize) * (height as usize) * 4 {
        return false;
    }

    let srgb = format == TextureFormat::Rgba8UnormSrgb;
    // 256-entry sRGB→linear decode table, built once per image. Cheaper than
    // calling `powf` four times per channel per output texel.
    let decode: [f32; 256] = std::array::from_fn(|i| srgb_to_linear(i as u8));

    /// Average a single channel across a 2×2 block, in linear light when
    /// `srgb` (RGB channels) and directly otherwise (alpha / linear formats).
    #[inline]
    fn avg(samples: [u8; 4], srgb: bool, decode: &[f32; 256]) -> u8 {
        if srgb {
            let lin = samples.iter().map(|&b| decode[b as usize]).sum::<f32>() / 4.0;
            linear_to_srgb(lin)
        } else {
            let sum: u32 = samples.iter().map(|&b| b as u32).sum();
            ((sum + 2) / 4) as u8
        }
    }

    let mut packed = base.clone();
    let mut prev = base.clone();
    let (mut pw, mut ph) = (width, height);
    let mut levels = 1u32;

    while pw > 1 || ph > 1 {
        let nw = (pw / 2).max(1);
        let nh = (ph / 2).max(1);
        let mut next = vec![0u8; (nw as usize) * (nh as usize) * 4];
        for y in 0..nh {
            for x in 0..nw {
                // 2×2 source block, clamped at odd edges so a non-power-of-two
                // dimension's final column/row still averages real pixels.
                let sx0 = (x * 2).min(pw - 1);
                let sx1 = (x * 2 + 1).min(pw - 1);
                let sy0 = (y * 2).min(ph - 1);
                let sy1 = (y * 2 + 1).min(ph - 1);
                let texel = |sx: u32, sy: u32, c: u32| -> u8 {
                    prev[((sy * pw + sx) * 4 + c) as usize]
                };
                for c in 0..4 {
                    let block = [
                        texel(sx0, sy0, c),
                        texel(sx1, sy0, c),
                        texel(sx0, sy1, c),
                        texel(sx1, sy1, c),
                    ];
                    // Channel 3 is alpha — always linear, never gamma-decoded.
                    let chan_srgb = srgb && c < 3;
                    next[((y * nw + x) * 4 + c) as usize] = avg(block, chan_srgb, &decode);
                }
            }
        }
        packed.extend_from_slice(&next);
        prev = next;
        pw = nw;
        ph = nh;
        levels += 1;
    }

    image.texture_descriptor.mip_level_count = levels;
    image.data = Some(packed);
    true
}

/// Watch for freshly-loaded card-face images and give each a mip chain.
///
/// Guarded by `generate_mipmaps`'s `mip_level_count > 1` early-out, so the
/// `Modified` event our own mutation emits is read back and skipped without
/// reprocessing — no loop, no dedupe bookkeeping needed.
pub fn generate_card_mipmaps(
    mut events: MessageReader<AssetEvent<Image>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        let (AssetEvent::Added { id } | AssetEvent::LoadedWithDependencies { id }) = event else {
            continue;
        };
        // Cheap immutable pre-check: only `get_mut` (which would fire a
        // redundant `Modified`) once we know there's real work to do.
        let needs = images
            .get(*id)
            .is_some_and(|img| img.texture_descriptor.mip_level_count == 1);
        if !needs {
            continue;
        }
        let is_card = asset_server
            .get_path(*id)
            .is_some_and(|p| is_card_texture(&p.path().to_string_lossy()));
        if !is_card {
            continue;
        }
        if let Some(image) = images.get_mut(*id) {
            generate_mipmaps(image);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::Extent3d;

    fn solid_rgba8(w: u32, h: u32) -> Image {
        let mut img = Image::new_fill(
            Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2,
            &[200, 100, 50, 255],
            TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        );
        img.texture_descriptor.mip_level_count = 1;
        img
    }

    #[test]
    fn builds_full_chain_and_is_idempotent() {
        let mut img = solid_rgba8(64, 64);
        assert!(generate_mipmaps(&mut img));
        // floor(log2(64)) + 1 = 7 levels.
        assert_eq!(img.texture_descriptor.mip_level_count, 7);
        // Packed data = sum of every level's byte footprint.
        let mut expected = 0usize;
        let (mut w, mut h) = (64u32, 64u32);
        loop {
            expected += (w * h * 4) as usize;
            if w == 1 && h == 1 {
                break;
            }
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
        assert_eq!(img.data.as_ref().unwrap().len(), expected);
        // A solid fill must average to (within sRGB round-trip rounding) the
        // same colour at every level.
        for (got, want) in img.data.as_ref().unwrap()[..4].iter().zip([200, 100, 50, 255]) {
            assert!((*got as i32 - want as i32).abs() <= 1, "got {got}, want ~{want}");
        }
        // Second pass is a no-op.
        assert!(!generate_mipmaps(&mut img));
    }

    #[test]
    fn srgb_average_is_gamma_correct() {
        // A 2×1 black/white sRGB image: gamma-correct downsampling must land
        // near the linear midpoint (~188), NOT the naive sRGB midpoint (~128).
        let mut img = Image::new(
            Extent3d { width: 2, height: 1, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2,
            vec![0, 0, 0, 255, 255, 255, 255, 255],
            TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        );
        img.texture_descriptor.mip_level_count = 1;
        assert!(generate_mipmaps(&mut img));
        // Mip 1 is a single texel, packed right after the 2-texel base.
        let mip1 = &img.data.as_ref().unwrap()[8..12];
        assert!(mip1[0] >= 180 && mip1[0] <= 196, "expected ~188, got {}", mip1[0]);
        assert_eq!(mip1[3], 255); // alpha averaged linearly: (255+255)/2
    }

    #[test]
    fn non_power_of_two_terminates() {
        let mut img = solid_rgba8(745, 1040);
        assert!(generate_mipmaps(&mut img));
        assert_eq!(img.texture_descriptor.mip_level_count, 11); // floor(log2(1040)) + 1
    }
}
