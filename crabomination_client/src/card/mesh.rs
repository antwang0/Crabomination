use std::f32::consts::{FRAC_PI_2, PI};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy_mesh::{Indices, PrimitiveTopology};

use super::components::{CARD_HEIGHT, CARD_WIDTH};

pub const CORNER_RADIUS: f32 = 0.15;
pub const BORDER_WIDTH: f32 = 0.15;

/// Creates a rounded rectangle mesh lying in the XY plane, facing +Z.
pub fn create_rounded_rect_mesh(width: f32, height: f32, radius: f32, segments: usize) -> Mesh {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let perimeter = rounded_rect_perimeter(width, height, radius, segments);

    let vert_count = perimeter.len() + 1; // +1 for center
    let mut positions = Vec::with_capacity(vert_count);
    let mut normals = Vec::with_capacity(vert_count);
    let mut uvs = Vec::with_capacity(vert_count);

    // Center vertex
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 0.5]);

    // Perimeter vertices
    for p in &perimeter {
        positions.push([p.x, p.y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        // UV: map card space to 0..1
        let u = (p.x + hw) / width;
        let v = 1.0 - (p.y + hh) / height;
        uvs.push([u, v]);
    }

    // Fan triangles from center (index 0) to perimeter
    let perim_count = perimeter.len();
    let mut indices = Vec::with_capacity(perim_count * 3);
    for i in 0..perim_count {
        let current = (i + 1) as u32;
        let next = ((i + 1) % perim_count + 1) as u32;
        indices.push(0u32);
        indices.push(current);
        indices.push(next);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Creates a border ring mesh: the area between two rounded rectangles.
pub fn create_border_mesh(
    width: f32,
    height: f32,
    radius: f32,
    border: f32,
    segments: usize,
) -> Mesh {
    let outer = rounded_rect_perimeter(
        width + border * 2.0,
        height + border * 2.0,
        radius + border,
        segments,
    );
    let inner = rounded_rect_perimeter(width, height, radius, segments);
    let n = outer.len();

    let mut positions = Vec::with_capacity(n * 2);
    let mut normals = Vec::with_capacity(n * 2);
    let mut uvs = Vec::with_capacity(n * 2);

    for p in &outer {
        positions.push([p.x, p.y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.0, 0.0]);
    }
    for p in &inner {
        positions.push([p.x, p.y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.0, 0.0]);
    }

    let mut indices = Vec::with_capacity(n * 6);
    for i in 0..n {
        let next = (i + 1) % n;
        let o0 = i as u32;
        let o1 = next as u32;
        let i0 = (n + i) as u32;
        let i1 = (n + next) as u32;
        indices.extend_from_slice(&[o0, i0, o1, o1, i0, i1]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn rounded_rect_perimeter(width: f32, height: f32, radius: f32, segments: usize) -> Vec<Vec2> {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let r = radius.min(hw.min(hh));

    let corners = [
        (Vec2::new(hw - r, hh - r), 0.0),
        (Vec2::new(-hw + r, hh - r), FRAC_PI_2),
        (Vec2::new(-hw + r, -hh + r), PI),
        (Vec2::new(hw - r, -hh + r), 3.0 * FRAC_PI_2),
    ];

    let mut perimeter = Vec::new();
    for (center, start_angle) in &corners {
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = start_angle + t * FRAC_PI_2;
            perimeter.push(Vec2::new(
                center.x + r * angle.cos(),
                center.y + r * angle.sin(),
            ));
        }
    }
    perimeter
}

/// Convenience: card-sized rounded rect mesh.
pub fn card_mesh(meshes: &mut Assets<Mesh>, segments: usize) -> Handle<Mesh> {
    meshes.add(create_rounded_rect_mesh(CARD_WIDTH, CARD_HEIGHT, CORNER_RADIUS, segments))
}

/// Convenience: card border ring mesh.
pub fn card_border_mesh(meshes: &mut Assets<Mesh>, segments: usize) -> Handle<Mesh> {
    meshes.add(create_border_mesh(CARD_WIDTH, CARD_HEIGHT, CORNER_RADIUS, BORDER_WIDTH, segments))
}
