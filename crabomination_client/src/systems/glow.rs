//! Light on the table — the flash, shockwave and embers of a death, the
//! sparks where damage lands, mana flying to its spell, the glow round the
//! active seat's board — drawn as soft, additive geometry.
//!
//! These were gizmo lines: one-pixel rings and spokes that read as a
//! wireframe next to the shaded arrows, chips and cards. Here each is a soft
//! shape whose colour runs out to nothing at its edges, blended by adding
//! its light to what lies under it — so effects that overlap brighten
//! rather than cover each other — and bright past 1, so it blooms on the
//! tiers that have bloom.
//!
//! Drawing is immediate-mode, like [`Arrows`](super::arrows::Arrows): a
//! system pushes this frame's [`Light`]s into [`Glow`], and [`render_glow`]
//! builds them into one mesh in `PostUpdate`. Additive light doesn't care
//! what order it is drawn in, so one mesh holds all of it.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};

use crate::MainCamera;
use crate::menu::AppState;

/// One shape of light. Colours are linear and may run past 1 (HDR); a
/// shape is brightest where its colour is given and fades to nothing at its
/// edges.
#[derive(Clone, Debug)]
pub enum Light {
    /// A ball of light `radius` across, turned to face the camera.
    Orb { at: Vec3, radius: f32, colour: LinearRgba },
    /// A pool of light lying flat on the table.
    Pool { at: Vec3, radius: f32, colour: LinearRgba },
    /// A ring of light flat on the table: brightest at `radius`, fading
    /// over `width` inside it and a third of that outside.
    Wave { at: Vec3, radius: f32, width: f32, colour: LinearRgba },
    /// A line of light through `points`, head first, turned to face the
    /// camera: `half` wide at its head, narrowing and fading to its tail.
    Trail { points: Vec<Vec3>, half: f32, colour: LinearRgba },
    /// A band of light lying flat round the rectangle `min`-`max` (XZ) at
    /// height `y`: brightest on the rectangle's edge, spilling `half`
    /// inside it and half that outside.
    Frame { min: Vec2, max: Vec2, y: f32, half: f32, colour: LinearRgba },
}

/// This frame's light. Push with [`Glow::push`]; [`render_glow`] draws and
/// empties it.
#[derive(Resource, Default)]
pub struct Glow(Vec<Light>);

impl Glow {
    pub fn push(&mut self, light: Light) {
        self.0.push(light);
    }

    pub fn extend(&mut self, lights: impl IntoIterator<Item = Light>) {
        self.0.extend(lights);
    }
}

/// Segments round an orb or a pool, and round a wave.
const ORB_SEGMENTS: usize = 20;
const WAVE_SEGMENTS: usize = 64;
/// An orb's or pool's brightness out from its centre, as (share of the
/// radius, share of its colour): a hot centre, a soft falloff.
const ORB_PROFILE: [(f32, f32); 4] = [(0.0, 1.0), (0.22, 0.6), (0.5, 0.2), (1.0, 0.0)];

/// Vertices of this frame's light, ready to become a mesh.
#[derive(Default)]
struct Geometry {
    positions: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

fn lit(colour: LinearRgba, k: f32) -> [f32; 4] {
    [colour.red * k, colour.green * k, colour.blue * k, 1.0]
}

impl Geometry {
    /// Add `rows` rows of `columns` vertices each, `vertex(row, column)`
    /// giving each one's place and colour, and stitch neighbouring rows
    /// into quads — the last row back to the first when `wrap`.
    fn strip(&mut self, rows: usize, columns: usize, wrap: bool, mut vertex: impl FnMut(usize, usize) -> (Vec3, [f32; 4])) {
        let first = self.positions.len() as u32;
        for r in 0..rows {
            for c in 0..columns {
                let (at, colour) = vertex(r, c);
                self.positions.push(at.to_array());
                self.colours.push(colour);
            }
        }
        let joined = if wrap { rows } else { rows.saturating_sub(1) };
        for r in 0..joined {
            let (a, b) = (r, (r + 1) % rows);
            let v = |row: usize, col: usize| first + (row * columns + col) as u32;
            for c in 0..columns - 1 {
                self.indices.extend([v(a, c), v(a, c + 1), v(b, c), v(a, c + 1), v(b, c + 1), v(b, c)]);
            }
        }
    }

    /// Light round `centre` in the plane of `u` and `v`, `profile` giving
    /// its brightness at each distance out.
    fn radial(&mut self, centre: Vec3, (u, v): (Vec3, Vec3), segments: usize, profile: &[(f32, f32)], colour: LinearRgba) {
        self.strip(segments, profile.len(), true, |r, c| {
            let angle = TAU * r as f32 / segments as f32;
            let (radius, k) = profile[c];
            (centre + (u * angle.cos() + v * angle.sin()) * radius, lit(colour, k))
        });
    }

    fn add(&mut self, light: &Light, eye: Vec3, (right, up): (Vec3, Vec3)) {
        match light {
            Light::Orb { at, radius, colour } => {
                let profile = ORB_PROFILE.map(|(r, k)| (r * radius, k));
                self.radial(*at, (right, up), ORB_SEGMENTS, &profile, *colour);
            }
            Light::Pool { at, radius, colour } => {
                let profile = ORB_PROFILE.map(|(r, k)| (r * radius, k));
                self.radial(*at, (Vec3::X, Vec3::Z), ORB_SEGMENTS, &profile, *colour);
            }
            Light::Wave { at, radius, width, colour } => {
                let profile = [
                    ((radius - width).max(0.0), 0.0),
                    ((radius - width * 0.35).max(0.0), 0.35),
                    (*radius, 1.0),
                    (radius + width / 3.0, 0.0),
                ];
                self.radial(*at, (Vec3::X, Vec3::Z), WAVE_SEGMENTS, &profile, *colour);
            }
            Light::Trail { points, half, colour } => {
                if points.len() < 2 {
                    return;
                }
                let last = (points.len() - 1) as f32;
                self.strip(points.len(), 3, false, |r, c| {
                    let at = points[r];
                    let along = if r + 1 < points.len() { at - points[r + 1] } else { points[r - 1] - at };
                    let to_eye = (eye - at).normalize_or(Vec3::Y);
                    let side = along.cross(to_eye).normalize_or(Vec3::X);
                    // Head (row 0) to tail: narrowing, and fading out.
                    let t = r as f32 / last;
                    let width = half * (1.0 - 0.7 * t);
                    let offset = c as f32 - 1.0;
                    let k = if c == 1 { (1.0 - t) * (1.0 - t) } else { 0.0 };
                    (at + side * width * offset, lit(*colour, k))
                });
            }
            Light::Frame { min, max, y, half, colour } => {
                // Out from the rectangle's edge: spill outside, the bright
                // edge, spill inside.
                let profile = [(half / 2.0, 0.0), (half * 0.06, 1.0), (-half * 0.06, 1.0), (-half, 0.0)];
                self.strip(4, profile.len(), true, |r, c| {
                    let (d, k) = profile[c];
                    let (lo, hi) = (*min - Vec2::splat(d), *max + Vec2::splat(d));
                    let corner = [lo, Vec2::new(hi.x, lo.y), hi, Vec2::new(lo.x, hi.y)][r];
                    (Vec3::new(corner.x, *y, corner.y), lit(*colour, k))
                });
            }
        }
    }

    fn into_mesh(self) -> Mesh {
        let normals = vec![[0.0, 1.0, 0.0]; self.positions.len()];
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

/// The material all light shares: unlit, adding its colour to what lies
/// under it.
///
/// Transparent meshes are drawn back to front by their mesh's centre, and
/// this one mesh holds every light at once, so its centre is the middle of
/// whatever is lit this frame — it wandered as bursts and motes came and
/// went, and where light crossed an arrow the two swapped draw order from
/// frame to frame. The bias sorts the light after every other transparent
/// thing, always: light added over an arrow, never under it one frame and
/// over it the next. (As a depth offset it is a few ULPs of depth — nothing
/// against the card a glow lies under.)
fn glow_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        depth_bias: GLOW_SORT_BIAS,
        ..default()
    }
}

/// See [`glow_material`]: more than the depth of the whole table, so no
/// other transparent mesh sorts after the light.
const GLOW_SORT_BIAS: f32 = 100.0;

/// The one entity the light is drawn on, and its mesh.
pub struct GlowMesh {
    entity: Entity,
    mesh: Handle<Mesh>,
    /// Whether last frame drew anything, so an empty frame after an empty
    /// frame needn't rebuild the mesh.
    lit: bool,
}

/// Bevy system (`PostUpdate`): draw this frame's [`Glow`] and empty it.
#[allow(clippy::too_many_arguments)]
pub fn render_glow(
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut queue: ResMut<Glow>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<&GlobalTransform, With<MainCamera>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
    mut drawn: Local<Option<GlowMesh>>,
    mut motion: ResMut<crate::systems::frame_pacing::LightMotion>,
) {
    // Out of a game, nothing is drawn and nothing lingers into the next.
    if *state.get() != AppState::InGame {
        queue.0.clear();
        if let Some(d) = drawn.take() {
            commands.entity(d.entity).try_despawn();
            meshes.remove(&d.mesh);
        }
        return;
    }
    let Ok(eye) = camera.single() else {
        queue.0.clear();
        return;
    };
    if queue.0.is_empty() && drawn.as_ref().is_none_or(|d| !d.lit) {
        return;
    }
    let d = drawn.get_or_insert_with(|| {
        let material = material.get_or_insert_with(|| materials.add(glow_material())).clone();
        let mesh = meshes.add(Geometry::default().into_mesh());
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::IDENTITY,
                // The vertices move every frame; the bounds a mesh gets when
                // it spawns would cull it.
                NoFrustumCulling,
                NotShadowCaster,
                NotShadowReceiver,
                crate::systems::game_ui::InGameRoot,
            ))
            .id();
        GlowMesh { entity, mesh, lit: false }
    });
    let mut geometry = Geometry::default();
    let basis = (eye.right().as_vec3(), eye.up().as_vec3());
    for light in queue.0.drain(..) {
        geometry.add(&light, eye.translation(), basis);
    }
    d.lit = !geometry.indices.is_empty();
    // Every light here moves on its own (the seat glow breathes, bursts and
    // motes run their course); the ones with a clock of their own — impacts,
    // motes — also keep the frame loop at full rate through their components.
    motion.ambient |= d.lit;
    let _ = meshes.insert(&d.mesh, geometry.into_mesh());
}

#[cfg(test)]
mod tests {
    use super::*;

    const EYE: Vec3 = Vec3::new(0.0, 30.0, 25.0);
    const FLAT: (Vec3, Vec3) = (Vec3::X, Vec3::Z);

    fn built(light: Light) -> Geometry {
        let mut g = Geometry::default();
        g.add(&light, EYE, FLAT);
        g
    }

    fn brightness(c: [f32; 4]) -> f32 {
        c[0] + c[1] + c[2]
    }

    #[test]
    fn a_wave_is_brightest_on_its_front_and_dark_at_its_edges() {
        let g = built(Light::Wave { at: Vec3::ZERO, radius: 2.0, width: 0.8, colour: LinearRgba::RED });
        assert!(g.indices.iter().all(|&i| (i as usize) < g.positions.len()));
        for (p, c) in g.positions.iter().zip(&g.colours) {
            let r = Vec2::new(p[0], p[2]).length();
            assert!(p[1].abs() < 1e-6, "flat on the table");
            if (r - 2.0).abs() < 1e-4 {
                assert_eq!(brightness(*c), 1.0);
            }
            if !(1.2 + 1e-4..=2.2).contains(&r) {
                assert_eq!(brightness(*c), 0.0, "at {r}");
            }
        }
    }

    #[test]
    fn an_orb_faces_the_camera_and_glows_from_its_centre() {
        let (right, up) = (Vec3::X, Vec3::new(0.0, 0.64, -0.77).normalize());
        let mut g = Geometry::default();
        g.add(&Light::Orb { at: Vec3::ZERO, radius: 1.0, colour: LinearRgba::WHITE }, EYE, (right, up));
        let facing = right.cross(up);
        for (p, c) in g.positions.iter().zip(&g.colours) {
            let p = Vec3::from(*p);
            assert!(p.dot(facing).abs() < 1e-5, "in the camera's plane");
            // Brighter nearer the middle; nothing at the rim.
            let r = p.length();
            if r > 1.0 - 1e-4 {
                assert_eq!(brightness(*c), 0.0);
            }
            if r < 1e-4 {
                assert_eq!(brightness(*c), 3.0);
            }
        }
    }

    #[test]
    fn a_trail_narrows_and_fades_to_its_tail() {
        let points: Vec<Vec3> = (0..5).map(|i| Vec3::new(-(i as f32), 0.0, 0.0)).collect();
        let g = built(Light::Trail { points, half: 0.2, colour: LinearRgba::WHITE });
        // Three vertices a row: the lit middle between two dark edges.
        let middle = |row: usize| brightness(g.colours[row * 3 + 1]);
        let width = |row: usize| Vec3::from(g.positions[row * 3]).distance(Vec3::from(g.positions[row * 3 + 2]));
        assert_eq!(middle(0), 3.0);
        assert_eq!(middle(4), 0.0);
        assert!(middle(1) > middle(2) && width(0) > width(3));
        assert_eq!(brightness(g.colours[0]), 0.0);
        assert!((width(0) - 0.4).abs() < 1e-4);
        assert_eq!(g.indices.len(), 4 * 2 * 6);
    }

    #[test]
    fn a_frame_lights_its_rectangle_edge() {
        let (min, max) = (Vec2::new(-4.0, -3.0), Vec2::new(4.0, 3.0));
        let g = built(Light::Frame { min, max, y: 0.01, half: 1.0, colour: LinearRgba::WHITE });
        // Four corners round, the band closed on itself.
        assert_eq!(g.positions.len(), 4 * 4);
        assert_eq!(g.indices.len(), 4 * 3 * 6);
        for (p, c) in g.positions.iter().zip(&g.colours) {
            let lit = brightness(*c) > 0.0;
            let off_edge = (p[0].abs() - 4.0).abs();
            assert_eq!(lit, off_edge < 0.1, "{p:?}");
        }
    }
}
