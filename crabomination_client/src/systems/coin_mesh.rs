//! The counter coin's mesh: a clay poker chip, turned from a half-profile.
//!
//! A plain cylinder read as a flat disc of paint: no edge caught the light,
//! a pile of them was one striped column, and the rim that kept a dark coin
//! apart from dark card art was a second, larger cylinder behind it. The
//! chip has a rounded edge, a raised rim around a recessed cream face (the
//! label the count is printed on, in the chip's own deep colour), a dark
//! groove around that face and cream spots around the edge that wrap over
//! the rim, as on a casino chip. A white count on the coloured face read
//! poorly, the cream spots crowding its glyphs. The colours are baked
//! into the vertices, so the spots are geometry rather than a decal fighting
//! the surface for depth, and every kind's chip shares one material
//! ([`chip_material`]).
//!
//! The chip is turned about +Y, `height` thick and centred on the origin,
//! like Bevy's `Cylinder`.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy_mesh::{Indices, PrimitiveTopology};

/// Columns the profile is turned through.
const COLUMNS: usize = 64;
/// Edge spots: every `SPOT_EVERY` columns, the first `SPOT_WIDTH` are cream —
/// eight spots, each a third of the gap between them.
const SPOT_EVERY: usize = 8;
const SPOT_WIDTH: usize = 3;
/// Edges in each rounded quarter of the rim.
const ARC_STEPS: usize = 4;
/// The cream of the face and the edge spots.
pub const INLAY: Color = Color::srgb(0.93, 0.90, 0.82);

/// Where on the chip an edge of the profile lies, which decides its colour.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Band {
    /// The recessed face the count is printed on: cream.
    Face,
    /// The wall from the face up to the rim: a groove darker than the body.
    Frame,
    /// The inner half of the rim: the body.
    Rim,
    /// The outer rim, the rounded edges and the side: the body, with spots.
    Edge,
    /// The underside, never seen: the body.
    Base,
}

/// One edge of the half-profile, in (radius, height), with the surface
/// normal at each end — the same at both for a flat face, turning with the
/// arc on the rounded edge.
struct ProfileEdge {
    from: Vec2,
    to: Vec2,
    n_from: Vec2,
    n_to: Vec2,
    band: Band,
}

/// The half-profile, from the centre of the face out over the rim, down the
/// side and back in under the base.
fn profile(radius: f32, height: f32) -> Vec<ProfileEdge> {
    let top = height / 2.0;
    let recess = height * 0.18;
    let bevel = height * 0.3;
    let flat = |from: Vec2, to: Vec2, band: Band| {
        let along = to - from;
        let n = Vec2::new(-along.y, along.x).normalize();
        ProfileEdge { from, to, n_from: n, n_to: n, band }
    };
    let arc = |centre: Vec2, from_angle: f32, to_angle: f32, band: Band| {
        (0..ARC_STEPS).map(move |i| {
            let a0 = from_angle + (to_angle - from_angle) * i as f32 / ARC_STEPS as f32;
            let a1 = from_angle + (to_angle - from_angle) * (i + 1) as f32 / ARC_STEPS as f32;
            let (n0, n1) = (Vec2::from_angle(a0), Vec2::from_angle(a1));
            ProfileEdge { from: centre + n0 * bevel, to: centre + n1 * bevel, n_from: n0, n_to: n1, band }
        })
    };
    let face = radius * 0.60;
    let frame = radius * 0.66;
    let spots_from = radius * 0.80;
    let mut edges = vec![
        flat(Vec2::new(0.0, top - recess), Vec2::new(face, top - recess), Band::Face),
        flat(Vec2::new(face, top - recess), Vec2::new(frame, top), Band::Frame),
        flat(Vec2::new(frame, top), Vec2::new(spots_from, top), Band::Rim),
        flat(Vec2::new(spots_from, top), Vec2::new(radius - bevel, top), Band::Edge),
    ];
    edges.extend(arc(Vec2::new(radius - bevel, top - bevel), FRAC_PI_2, 0.0, Band::Edge));
    edges.push(flat(Vec2::new(radius, top - bevel), Vec2::new(radius, -top + bevel), Band::Edge));
    edges.extend(arc(Vec2::new(radius - bevel, -top + bevel), 0.0, -FRAC_PI_2, Band::Edge));
    edges.push(flat(Vec2::new(radius - bevel, -top), Vec2::new(0.0, -top), Band::Base));
    edges
}

/// The chip for a counter kind whose body is `body`.
pub fn chip_mesh(radius: f32, height: f32, body: Color) -> Mesh {
    let body = body.to_linear();
    let groove = LinearRgba { red: body.red * 0.45, green: body.green * 0.45, blue: body.blue * 0.45, ..body };
    let inlay = INLAY.to_linear();
    let colour = |band: Band, column: usize| {
        let spot = column % SPOT_EVERY < SPOT_WIDTH;
        match band {
            Band::Face => inlay,
            Band::Frame => groove,
            Band::Edge if spot => inlay,
            Band::Rim | Band::Edge | Band::Base => body,
        }
        .to_f32_array()
    };

    let edges = profile(radius, height);
    let quads = COLUMNS * edges.len();
    let mut positions = Vec::with_capacity(quads * 4);
    let mut normals = Vec::with_capacity(quads * 4);
    let mut colours = Vec::with_capacity(quads * 4);
    let mut indices = Vec::with_capacity(quads * 6);
    // A point of the profile turned to `angle`: (radius, height) → XYZ.
    let turn = |p: Vec2, angle: f32| [p.x * angle.cos(), p.y, p.x * angle.sin()];
    for column in 0..COLUMNS {
        let (a0, a1) = (TAU * column as f32 / COLUMNS as f32, TAU * (column + 1) as f32 / COLUMNS as f32);
        for edge in &edges {
            // Each column owns its vertices, so a spot's colour stops at
            // its edge instead of bleeding into the next column.
            let base = positions.len() as u32;
            for (p, n, a) in [
                (edge.from, edge.n_from, a0),
                (edge.to, edge.n_to, a0),
                (edge.to, edge.n_to, a1),
                (edge.from, edge.n_from, a1),
            ] {
                positions.push(turn(p, a));
                normals.push(turn(n, a));
                colours.push(colour(edge.band, column));
            }
            // Counter-clockwise seen from outside the chip.
            indices.extend([base, base + 3, base + 1, base + 1, base + 3, base + 2]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// The material every chip shares: white, so the vertex colours are the
/// chip's colours, with the soft sheen of a clay chip.
pub fn chip_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.42,
        metallic: 0.0,
        reflectance: 0.45,
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_mesh::VertexAttributeValues;

    fn attribute(mesh: &Mesh, id: bevy_mesh::MeshVertexAttribute) -> Vec<Vec3> {
        match mesh.attribute(id) {
            Some(VertexAttributeValues::Float32x3(v)) => v.iter().map(|&p| Vec3::from(p)).collect(),
            other => panic!("unexpected attribute {other:?}"),
        }
    }

    #[test]
    fn every_triangle_faces_the_way_its_normals_do() {
        let mesh = chip_mesh(0.34, 0.06, Color::srgb(0.2, 0.4, 0.2));
        let positions = attribute(&mesh, Mesh::ATTRIBUTE_POSITION);
        let normals = attribute(&mesh, Mesh::ATTRIBUTE_NORMAL);
        let Some(Indices::U32(indices)) = mesh.indices() else { panic!("no indices") };
        let mut checked = 0;
        for tri in indices.chunks(3) {
            let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| i as usize);
            let facing = (positions[b] - positions[a]).cross(positions[c] - positions[a]);
            // The triangles at the face's centre have no area.
            if facing.length() < 1e-9 {
                continue;
            }
            let smooth = normals[a] + normals[b] + normals[c];
            assert!(facing.dot(smooth) > 0.0, "triangle {tri:?} is wound inside out");
            checked += 1;
        }
        assert!(checked > COLUMNS * 10);
    }

    #[test]
    fn the_chip_fits_its_cylinder() {
        let (radius, height) = (0.34, 0.06);
        for p in attribute(&chip_mesh(radius, height, Color::WHITE), Mesh::ATTRIBUTE_POSITION) {
            assert!(Vec2::new(p.x, p.z).length() <= radius + 1e-5, "{p}");
            assert!(p.y.abs() <= height / 2.0 + 1e-5, "{p}");
        }
    }

    #[test]
    fn the_spots_are_cream_and_the_rest_of_the_edge_is_the_body() {
        let body = Color::srgb(0.2, 0.4, 0.2);
        let mesh = chip_mesh(0.34, 0.06, body);
        let Some(VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("no colours")
        };
        let per_column = colours.len() / COLUMNS;
        // The side of column 0 (a spot) and of column SPOT_WIDTH (a gap).
        let side = 4 + ARC_STEPS;
        assert_eq!(colours[side * 4], INLAY.to_linear().to_f32_array());
        assert_eq!(colours[SPOT_WIDTH * per_column + side * 4], body.to_linear().to_f32_array());
    }
}
