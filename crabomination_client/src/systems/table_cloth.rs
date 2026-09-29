//! The table's cloth: a woven felt texture and a pool of light over the
//! play area.
//!
//! The table was one flat colour to the horizon, with each seat's area a
//! flat tint of it, so it read as a void the cards floated over rather than
//! a surface they lay on. The ground and the seat tints now share a fine,
//! tileable felt texture (generated here, with its mip chain so it doesn't
//! shimmer at a distance), and their vertex colours carry a pool of light:
//! brightest over the boards, falling off past them, so the eye settles on
//! the play area and the table's edge recedes.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::mesh::{Indices, PrimitiveTopology};

/// World units one repeat of the felt covers.
const CLOTH_TILE: f32 = 4.0;
/// The felt texture's size in texels.
const CLOTH_SIZE: u32 = 256;
/// Light at the heart of the play area and out past its edge, as a
/// multiple of the table's colour.
const LIGHT_CENTRE: f32 = 1.18;
const LIGHT_EDGE: f32 = 0.55;
/// The pool of light's falloff, in multiples of the play area's half-size:
/// full light inside the first, the edge's beyond the second.
const FALLOFF: (f32, f32) = (0.5, 1.3);
/// Grid cells a table mesh is cut into along its longer side: enough for
/// the pool of light to fall off smoothly.
const TABLE_CELLS: u32 = 64;

/// A hash of a lattice point, 0-1.
fn hash(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = x.wrapping_mul(0x8da6_b343) ^ y.wrapping_mul(0xd816_3841) ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0xffff) as f32 / 65535.0
}

/// Value noise that repeats every `period` lattice cells, at (u, v) in
/// cells.
fn periodic_noise(u: f32, v: f32, period: u32, seed: u32) -> f32 {
    let (x0, y0) = (u.floor(), v.floor());
    let (fx, fy) = (u - x0, v - y0);
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let (sx, sy) = (smooth(fx), smooth(fy));
    let at = |dx: u32, dy: u32| {
        let x = (x0 as u32 + dx) % period;
        let y = (y0 as u32 + dy) % period;
        hash(x, y, seed)
    };
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * sx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * sx;
    top + (bottom - top) * sy
}

/// The felt's brightness (0-1) at texel (x, y): mottling, fibres and a
/// faint weave, repeating seamlessly across the texture's edges.
fn felt(x: u32, y: u32) -> f32 {
    let (u, v) = (x as f32 / CLOTH_SIZE as f32, y as f32 / CLOTH_SIZE as f32);
    let octave = |cells: u32, seed: u32| periodic_noise(u * cells as f32, v * cells as f32, cells, seed) - 0.5;
    let mottle = octave(4, 1) * 0.5 + octave(8, 2) * 0.25;
    let fibre = octave(64, 3) * 0.5 + octave(128, 4) * 0.35;
    let weave = (std::f32::consts::TAU * u * 64.0).sin() * (std::f32::consts::TAU * v * 64.0).sin();
    (0.9 + mottle * 0.12 + fibre * 0.16 + weave * 0.025).clamp(0.0, 1.0)
}

/// The felt texture: grey, to be tinted by each surface's colour, tiling,
/// with its mip chain.
pub fn cloth_texture() -> Image {
    let mut data = Vec::with_capacity((CLOTH_SIZE * CLOTH_SIZE * 4) as usize);
    for y in 0..CLOTH_SIZE {
        for x in 0..CLOTH_SIZE {
            let g = (felt(x, y) * 255.0).round() as u8;
            data.extend([g, g, g, 255]);
        }
    }
    let mut image = Image::new(
        Extent3d { width: CLOTH_SIZE, height: CLOTH_SIZE, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    crate::card::mipmap::generate_mipmaps(&mut image);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 16,
        ..default()
    });
    image
}

/// The light (a multiple of the surface's colour) at table point `at` (XZ)
/// over a play area centred on `area.center()` and `area.half_size()` out.
pub fn table_light(at: Vec2, area: Rect) -> f32 {
    let half = area.half_size().max(Vec2::splat(1.0));
    let d = ((at - area.center()) / half).length();
    let t = ((d - FALLOFF.0) / (FALLOFF.1 - FALLOFF.0)).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    LIGHT_CENTRE + (LIGHT_EDGE - LIGHT_CENTRE) * t
}

/// A flat piece of table covering `rect` (XZ, world units), lying at the
/// origin of its entity: the felt laid on in world units, so neighbouring
/// pieces' weave lines up, and the pool of light over `area` in its vertex
/// colours.
pub fn table_mesh(rect: Rect, area: Rect) -> Mesh {
    let size = rect.size();
    let long = size.x.max(size.y).max(1e-3);
    let cells = UVec2::new(
        ((TABLE_CELLS as f32 * size.x / long).ceil() as u32).max(1),
        ((TABLE_CELLS as f32 * size.y / long).ceil() as u32).max(1),
    );
    let centre = rect.center();
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut colours = Vec::new();
    for j in 0..=cells.y {
        for i in 0..=cells.x {
            let at = rect.min + size * Vec2::new(i as f32 / cells.x as f32, j as f32 / cells.y as f32);
            positions.push([at.x - centre.x, 0.0, at.y - centre.y]);
            uvs.push([at.x / CLOTH_TILE, at.y / CLOTH_TILE]);
            let light = table_light(at, area);
            colours.push([light, light, light, 1.0]);
        }
    }
    let row = cells.x + 1;
    let mut indices = Vec::with_capacity((cells.x * cells.y * 6) as usize);
    for j in 0..cells.y {
        for i in 0..cells.x {
            let a = j * row + i;
            // Counter-clockwise seen from above.
            indices.extend([a, a + row, a + 1, a + 1, a + row, a + row + 1]);
        }
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// The play area a table of `n_seats` lights: every seat's board, from
/// `layout::seat_board_outline`.
pub fn play_area(viewer: usize, n_seats: usize) -> Rect {
    (0..n_seats.max(1))
        .map(|seat| {
            let (min, max) = crate::card::layout::seat_board_outline(seat, viewer, n_seats);
            Rect::new(min.x, min.z, max.x, max.z)
        })
        .reduce(|a, b| a.union(b))
        .unwrap_or(Rect::new(-10.0, -8.0, 10.0, 8.0))
}

/// The felt texture every table surface shares.
#[derive(Resource, Clone)]
pub struct ClothTexture(pub Handle<Image>);

/// A table surface's material: `colour` felt, rough, with its pool of light
/// in the mesh's vertex colours.
pub fn cloth_material(colour: Color, cloth: &ClothTexture) -> StandardMaterial {
    StandardMaterial {
        base_color: colour,
        base_color_texture: Some(cloth.0.clone()),
        perceptual_roughness: 0.92,
        reflectance: 0.25,
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_felt_repeats_seamlessly() {
        // Across each edge, the texture runs on as if the next tile began:
        // the step over the seam is no bigger than a step inside.
        let step = |a: f32, b: f32| (a - b).abs();
        let mut worst_seam: f32 = 0.0;
        let mut worst_inside: f32 = 0.0;
        for i in 0..CLOTH_SIZE {
            worst_seam = worst_seam.max(step(felt(CLOTH_SIZE - 1, i), felt(0, i)));
            worst_seam = worst_seam.max(step(felt(i, CLOTH_SIZE - 1), felt(i, 0)));
            worst_inside = worst_inside.max(step(felt(CLOTH_SIZE / 2, i), felt(CLOTH_SIZE / 2 + 1, i)));
        }
        assert!(worst_seam <= worst_inside * 1.5 + 0.02, "seam {worst_seam} vs inside {worst_inside}");
    }

    #[test]
    fn the_light_pools_over_the_boards() {
        let area = Rect::new(-20.0, -14.0, 20.0, 14.0);
        assert_eq!(table_light(Vec2::ZERO, area), LIGHT_CENTRE);
        assert_eq!(table_light(Vec2::new(60.0, 0.0), area), LIGHT_EDGE);
        let halfway = table_light(Vec2::new(18.0, 0.0), area);
        assert!(halfway < LIGHT_CENTRE && halfway > LIGHT_EDGE, "{halfway}");
    }

    #[test]
    fn a_table_mesh_covers_its_rect_with_world_scaled_felt() {
        let rect = Rect::new(2.0, -3.0, 10.0, 5.0);
        let mesh = table_mesh(rect, rect);
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
            panic!("no uvs")
        };
        assert_eq!(uvs.first(), Some(&[2.0 / CLOTH_TILE, -3.0 / CLOTH_TILE]));
        assert_eq!(uvs.last(), Some(&[10.0 / CLOTH_TILE, 5.0 / CLOTH_TILE]));
    }
}
