//! Arrows and rings for combat, targeting and the stack, drawn as shaded
//! geometry rather than gizmo lines.
//!
//! These cues were 3-5 px gizmo lines. Next to the lit cards, counter chips
//! and badges they read as debug output; a straight line across a busy board
//! was hard to follow back to its source, and only a small tick said which
//! end it pointed from. Here an arrow is a ribbon that arcs over the table
//! from its source to its target, turned to face the camera so it never goes
//! edge-on. It has a dark rim, so it reads over bright card art; a bright
//! core, along which bands of light run toward the target; a tail that fades
//! out of its source; and a broad head. It grows out of its source when it
//! first appears and fades when it goes, so a declaration reads as an event
//! rather than a flicker.
//!
//! Drawing is immediate-mode, like gizmos: a system pushes this frame's marks
//! into [`Arrows`], and [`render_arrows`] turns them into meshes in
//! `PostUpdate`, keeping an entity per [`ArrowKey`] alive across frames so
//! the grow and the fade have something to track.

use std::collections::HashMap;
use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use crabomination::card::CardId;

use crate::MainCamera;
use crate::menu::AppState;

/// What a mark says. Part of its [`ArrowKey`], so two cues between the same
/// pair of cards are two arrows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cue {
    /// The live arrow from a spell or ability being targeted to the cursor.
    TargetDrag,
    /// A spell or ability on the stack to its first target.
    Stack,
    /// ... and to each further target.
    StackExtra,
    /// A declared block.
    Block,
    /// The selected blocker to an attacker it could block.
    BlockCandidate,
    /// A block in the viewer's plan, not yet declared.
    BlockPlan,
    /// An attacker in the viewer's plan to what it will attack.
    AttackPlan,
    /// A declared attacker to what it attacks.
    Attack,
    /// The ring on a player an attack plan aims at.
    Defender,
    /// The ring on each legal target of the spell or ability being aimed.
    LegalTarget,
    /// The ring on the permanent whose decision is waiting on the viewer.
    DecisionSource,
    /// A creature being dragged onto what it attacks or blocks, to the
    /// pointer.
    Drag,
    /// The cord from an Aura, Equipment or Fortification to what it's
    /// attached to.
    Tether,
}

/// One end of a mark, for its key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum End {
    Card(CardId),
    Seat(usize),
    /// Wherever the pointer is.
    Cursor,
}

/// Which mark this is from frame to frame: what it says and what it joins.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ArrowKey {
    pub cue: Cue,
    pub from: End,
    pub to: End,
}

impl ArrowKey {
    pub fn new(cue: Cue, from: End, to: End) -> Self {
        Self { cue, from, to }
    }
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    Arrow { from: Vec3, to: Vec3, weight: f32, trim: [f32; 2] },
    Ring { centre: Vec3, radius: f32, normal: Vec3 },
    Cord { from: Vec3, to: Vec3, weight: f32 },
}

#[derive(Clone, Copy, Debug)]
struct Mark {
    shape: Shape,
    colour: Color,
}

/// This frame's arrows and rings. Push with [`Arrows::arrow`] and
/// [`Arrows::ring`]; [`render_arrows`] draws and empties it.
#[derive(Resource, Default)]
pub struct Arrows(Vec<(ArrowKey, Mark)>);

impl Arrows {
    /// An arrow from `from` to `to`, `weight` times the usual thickness.
    /// `colour`'s alpha fades the whole arrow.
    pub fn arrow(&mut self, key: ArrowKey, from: Vec3, to: Vec3, colour: Color, weight: f32) {
        self.arrow_trimmed(key, from, to, colour, weight, [0.0; 2]);
    }

    /// [`arrow`](Self::arrow), leaving `trim[0]` of its curve bare at the
    /// source and `trim[1]` at the target: an arrow between two combat
    /// chips meets their rims rather than running under them.
    pub fn arrow_trimmed(&mut self, key: ArrowKey, from: Vec3, to: Vec3, colour: Color, weight: f32, trim: [f32; 2]) {
        self.0.push((key, Mark { shape: Shape::Arrow { from, to, weight, trim }, colour }));
    }

    /// A cord from `from` to `to`: an arrow's arc and colours, without the
    /// head — a link rather than an action.
    pub fn cord(&mut self, key: ArrowKey, from: Vec3, to: Vec3, colour: Color, weight: f32) {
        self.0.push((key, Mark { shape: Shape::Cord { from, to, weight }, colour }));
    }

    /// A ring around `centre`, square to `normal`: flat on the table for
    /// `Vec3::Y`, or on a card's face for the card's normal.
    pub fn ring(&mut self, key: ArrowKey, centre: Vec3, radius: f32, normal: Vec3, colour: Color) {
        self.0.push((key, Mark { shape: Shape::Ring { centre, radius, normal }, colour }));
    }
}

/// How high an arrow arcs over the midpoint of its span, as a share of the
/// span, and the bounds on that height.
const ARC_RISE: f32 = 0.16;
const ARC_MIN: f32 = 0.35;
const ARC_MAX: f32 = 2.4;
/// Points the curve is sampled at before it is cut into rows.
const CURVE_SAMPLES: usize = 96;
/// Spacing of the shaft's rows along the arrow: fine enough that the light
/// bands running along it stay smooth.
const ROW_STEP: f32 = 0.12;
/// Half-width of the shaft and of the head's base, and the head's length,
/// for an arrow of weight 1.
const SHAFT_HALF: f32 = 0.15;
const HEAD_HALF: f32 = 0.5;
const HEAD_LEN: f32 = 1.05;
const HEAD_ROWS: usize = 6;
/// Half-width of a cord of weight 1, and how far in from each end it
/// narrows.
const CORD_HALF: f32 = 0.1;
const CORD_TAPER: f32 = 0.5;
/// Over how much of its length the tail narrows and fades out of its source.
const TAIL: f32 = 1.1;
/// A ring's half-thickness.
const RING_HALF: f32 = 0.1;
const RING_SEGMENTS: usize = 72;

/// Seconds a new mark takes to grow out of its source, and a vanished one
/// to fade away.
const GROW_SECS: f32 = 0.24;
const FADE_SECS: f32 = 0.2;
/// The light bands on the core: how fast they run toward the target, and
/// how far apart they are.
const FLOW_SPEED: f32 = 2.6;
const FLOW_WAVE: f32 = 1.7;

/// Brightness of the rim, the body and the core against the mark's colour.
/// Past 1 is HDR: on tiers with bloom the core glows. The body stays under
/// it, where the tonemapper leaves a colour's hue alone — pushed over, a
/// yellow washed out to cream.
const RIM: f32 = 0.14;
const BODY: f32 = 0.95;
const CORE: f32 = 1.6;

/// A row of the ribbon: a point on the curve, the way the curve runs there,
/// the ribbon's half-width and how far along the drawn arrow the row is.
#[derive(Clone, Copy, Debug)]
struct Row {
    at: Vec3,
    along: Vec3,
    half: f32,
    arc: f32,
}

/// The arrow's curve: a quadratic Bézier that peaks over the midpoint of the
/// span, higher for a longer span.
fn curve(from: Vec3, to: Vec3) -> impl Fn(f32) -> Vec3 {
    let peak = (from.distance(to) * ARC_RISE).clamp(ARC_MIN, ARC_MAX);
    let control = (from + to) / 2.0 + Vec3::Y * 2.0 * peak;
    move |s| from * (1.0 - s) * (1.0 - s) + control * 2.0 * s * (1.0 - s) + to * s * s
}

/// How much of the curve to leave bare so `trim` shows on screen at each
/// end: an arrow coming in along the line of sight is foreshortened, and
/// needs more of its curve left bare to clear a chip of the same size.
fn seen_trim(from: Vec3, to: Vec3, trim: [f32; 2], eye: Vec3) -> [f32; 2] {
    let point = curve(from, to);
    let stretch = |at: Vec3, along: Vec3| {
        let sight = (eye - at).normalize_or(Vec3::Y);
        let along = along.normalize_or(Vec3::X);
        // The share of a step along the curve that shows across the view.
        1.0 / (along - sight * along.dot(sight)).length().max(0.35)
    };
    let d = 1e-3;
    [
        trim[0] * stretch(from, point(d) - from),
        trim[1] * stretch(to, to - point(1.0 - d)),
    ]
}

/// The arrow's curve from `from` to `to`, sampled and measured along its
/// length.
struct Sampled {
    samples: Vec<Vec3>,
    lengths: Vec<f32>,
}

impl Sampled {
    fn new(from: Vec3, to: Vec3) -> Self {
        let point = curve(from, to);
        let samples: Vec<Vec3> = (0..=CURVE_SAMPLES).map(|i| point(i as f32 / CURVE_SAMPLES as f32)).collect();
        let mut lengths = vec![0.0];
        for pair in samples.windows(2) {
            lengths.push(lengths.last().copied().unwrap_or(0.0) + pair[0].distance(pair[1]));
        }
        Self { samples, lengths }
    }

    fn total(&self) -> f32 {
        self.lengths.last().copied().unwrap_or(0.0)
    }

    /// The point `arc` along the curve, and the way it runs there.
    fn at(&self, arc: f32) -> (Vec3, Vec3) {
        let (samples, lengths) = (&self.samples, &self.lengths);
        let i = lengths.partition_point(|&l| l < arc).clamp(1, CURVE_SAMPLES);
        let (a, b) = (samples[i - 1], samples[i]);
        let span = (lengths[i] - lengths[i - 1]).max(1e-6);
        (a.lerp(b, ((arc - lengths[i - 1]) / span).clamp(0.0, 1.0)), (b - a).normalize_or(Vec3::X))
    }
}

/// The rows of an arrow from `from` to `to`, tail to tip, grown `grow` of
/// the way (0-1) along its curve, with `trim` of the curve left bare at
/// each end (never more than a third of it).
fn arrow_rows(from: Vec3, to: Vec3, weight: f32, grow: f32, trim: [f32; 2]) -> Vec<Row> {
    let curve = Sampled::new(from, to);
    let total = curve.total();
    let start = trim[0].min(total / 3.0);
    let shown = (total - start - trim[1].min(total / 3.0)) * grow.clamp(0.0, 1.0);
    if shown < 0.05 {
        return Vec::new();
    }

    // A short arrow (or one still growing) gets a head in proportion.
    let head = (HEAD_LEN * weight).min(shown * 0.45);
    let head_half = HEAD_HALF * weight * head / (HEAD_LEN * weight);
    let shaft_half = SHAFT_HALF * weight;
    let shaft_end = shown - head;
    let tail = TAIL.min(shown * 0.3);
    let row = |arc: f32, half: f32| {
        let (at, along) = curve.at(start + arc);
        Row { at, along, half, arc }
    };

    let mut rows = Vec::new();
    let mut arc = 0.0;
    while arc < shaft_end {
        let taper = 0.4 + 0.6 * smoothstep(arc / tail.max(1e-3));
        rows.push(row(arc, shaft_half * taper));
        arc += ROW_STEP;
    }
    // The head's base: the shaft's last row, then the same point at the
    // head's full width.
    rows.push(row(shaft_end, shaft_half));
    rows.push(row(shaft_end, head_half));
    for k in 1..=HEAD_ROWS {
        let t = k as f32 / HEAD_ROWS as f32;
        rows.push(row(shaft_end + head * t, head_half * (1.0 - t)));
    }
    rows
}

/// The rows of a cord from `from` to `to`, grown `grow` of the way along
/// its curve: even along its length, narrowing into each end.
fn cord_rows(from: Vec3, to: Vec3, weight: f32, grow: f32) -> Vec<Row> {
    let curve = Sampled::new(from, to);
    let shown = curve.total() * grow.clamp(0.0, 1.0);
    if shown < 0.05 {
        return Vec::new();
    }
    let taper = CORD_TAPER.min(shown / 3.0).max(1e-3);
    let steps = (shown / ROW_STEP).ceil().max(1.0) as usize;
    (0..=steps)
        .map(|k| {
            let arc = shown * k as f32 / steps as f32;
            let narrow = 0.35 + 0.65 * smoothstep(arc.min(shown - arc) / taper);
            let (at, along) = curve.at(arc);
            Row { at, along, half: CORD_HALF * weight * narrow, arc }
        })
        .collect()
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

/// A light band's brightness (0-1) at `x` wavelengths along: narrow peaks.
fn band(x: f32) -> f32 {
    (0.5 + 0.5 * (TAU * x).cos()).powi(6)
}

/// Rim, body and core colours for a mark whose colour is `colour`, faded to
/// `alpha`, with a light band of strength `flow` passing.
fn tones(colour: LinearRgba, alpha: f32, flow: f32) -> [[f32; 4]; 3] {
    let scale = |k: f32, a: f32| [colour.red * k, colour.green * k, colour.blue * k, a * alpha];
    // The core runs toward white, so a dark colour still has a bright line
    // (only a little: more, and the tonemapper and bloom turn a thin yellow
    // arrow to cream).
    let hot = |k: f32| {
        let white = 0.2;
        [
            (colour.red + white) * k,
            (colour.green + white) * k,
            (colour.blue + white) * k,
            alpha,
        ]
    };
    [scale(RIM, 0.85), scale(BODY * (1.0 + 0.3 * flow), 1.0), hot(CORE * (1.0 + 0.7 * flow))]
}

/// Vertices of a mark, ready to become a mesh.
#[derive(Default)]
struct Geometry {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

/// Across the ribbon: the rim, the body, the core, the body, the rim, as a
/// share of the half-width, and which tone each takes.
///
/// The body is flat across most of the width and the core a thin line down
/// the middle: blended across the whole width, a thin arrow's core washed
/// its colour out.
const COLUMNS: [(f32, usize); 7] =
    [(-1.0, 0), (-0.78, 1), (-0.22, 1), (0.0, 2), (0.22, 1), (0.78, 1), (1.0, 0)];

impl Geometry {
    /// Stitch each pair of neighbouring rows of `columns`-wide strips into
    /// quads.
    fn stitch(&mut self, first: u32, rows: usize, columns: usize, wrap: bool) {
        let rows_joined = if wrap { rows } else { rows.saturating_sub(1) };
        for r in 0..rows_joined {
            let (a, b) = (r, (r + 1) % rows);
            for c in 0..columns - 1 {
                let v = |row: usize, col: usize| first + (row * columns + col) as u32;
                self.indices.extend([v(a, c), v(a, c + 1), v(b, c), v(a, c + 1), v(b, c + 1), v(b, c)]);
            }
        }
    }

    fn into_mesh(self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

/// The ribbon through `rows`, turned to face `eye`, with the light bands at
/// `phase` (world units along the arrow).
fn ribbon(rows: &[Row], eye: Vec3, colour: LinearRgba, alpha: f32, phase: f32) -> Geometry {
    let mut g = Geometry::default();
    let tail = rows.last().map_or(0.0, |r| r.arc * 0.3).min(TAIL);
    for row in rows {
        let to_eye = (eye - row.at).normalize_or(Vec3::Y);
        let side = row.along.cross(to_eye).normalize_or(row.along.cross(Vec3::Y).normalize_or(Vec3::X));
        let fade = alpha * smoothstep(row.arc / tail.max(1e-3));
        let tone = tones(colour, fade, band((row.arc - phase) / FLOW_WAVE));
        for (offset, which) in COLUMNS {
            g.positions.push((row.at + side * row.half * offset).to_array());
            g.normals.push(to_eye.to_array());
            g.colours.push(tone[which]);
        }
    }
    g.stitch(0, rows.len(), COLUMNS.len(), false);
    g
}

/// A ring around `centre`, square to `normal`, grown `grow` of the way out,
/// with its bright arcs turned `phase` turns.
fn ring(centre: Vec3, radius: f32, normal: Vec3, colour: LinearRgba, alpha: f32, grow: f32, phase: f32) -> Geometry {
    let mut g = Geometry::default();
    let radius = radius * (0.7 + 0.3 * grow);
    let alpha = alpha * grow;
    let normal = normal.normalize_or(Vec3::Y);
    let (u, v) = normal.any_orthonormal_pair();
    for i in 0..RING_SEGMENTS {
        let angle = TAU * i as f32 / RING_SEGMENTS as f32;
        let out = u * angle.cos() + v * angle.sin();
        // Four bright arcs, circling.
        let tone = tones(colour, alpha, band(angle * 4.0 / TAU - phase));
        for (offset, which) in COLUMNS {
            g.positions.push((centre + out * (radius + RING_HALF * offset)).to_array());
            g.normals.push(normal.to_array());
            g.colours.push(tone[which]);
        }
    }
    g.stitch(0, RING_SEGMENTS, COLUMNS.len(), true);
    g
}

/// The material every mark shares: unlit and blended, so the vertex colours
/// are the mark's colours, glow included.
fn arrow_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    }
}

/// A mark on screen: its entity, its mesh and how long it has been up (or,
/// once it stops being pushed, gone).
pub struct Live {
    entity: Entity,
    mesh: Handle<Mesh>,
    mark: Mark,
    age: f32,
    gone: f32,
    seen: bool,
}

/// Bevy system (`PostUpdate`): draw this frame's [`Arrows`], grow the new
/// ones, fade the vanished ones, and empty the queue.
#[allow(clippy::too_many_arguments)]
pub fn render_arrows(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<AppState>>,
    mut queue: ResMut<Arrows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<&GlobalTransform, With<MainCamera>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
    mut live: Local<HashMap<ArrowKey, Live>>,
) {
    // Out of a game, nothing is drawn and nothing lingers into the next.
    if *state.get() != AppState::InGame {
        queue.0.clear();
        for (_, l) in live.drain() {
            commands.entity(l.entity).try_despawn();
            meshes.remove(&l.mesh);
        }
        return;
    }
    let dt = time.delta_secs();
    let Ok(eye) = camera.single().map(GlobalTransform::translation) else { return };
    let material = material.get_or_insert_with(|| materials.add(arrow_material())).clone();

    for l in live.values_mut() {
        l.seen = false;
    }
    for (key, mark) in queue.0.drain(..) {
        if let Some(l) = live.get_mut(&key) {
            l.mark = mark;
            l.seen = true;
            continue;
        }
        let mesh = meshes.add(Geometry::default().into_mesh());
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::IDENTITY,
                // The vertices move every frame; the bounds a mesh gets
                // when it spawns would cull it.
                NoFrustumCulling,
                NotShadowCaster,
                NotShadowReceiver,
                crate::systems::game_ui::InGameRoot,
            ))
            .id();
        live.insert(key, Live { entity, mesh, mark, age: -dt, gone: 0.0, seen: true });
    }

    let phase = time.elapsed_secs() * FLOW_SPEED;
    live.retain(|_, l| {
        if l.seen {
            l.age += dt;
            l.gone = 0.0;
        } else {
            l.gone += dt;
        }
        if l.gone >= FADE_SECS {
            commands.entity(l.entity).try_despawn();
            meshes.remove(&l.mesh);
            return false;
        }
        let grow = ease_out(l.age / GROW_SECS);
        let colour = l.mark.colour.to_linear();
        let alpha = colour.alpha * (1.0 - l.gone / FADE_SECS);
        let geometry = match l.mark.shape {
            Shape::Arrow { from, to, weight, trim } => {
                let trim = seen_trim(from, to, trim, eye);
                ribbon(&arrow_rows(from, to, weight, grow, trim), eye, colour, alpha, phase)
            }
            Shape::Ring { centre, radius, normal } => {
                ring(centre, radius, normal, colour, alpha, grow, phase * 0.25)
            }
            // Its light drifts from the attachment into its host, slowly.
            Shape::Cord { from, to, weight } => {
                ribbon(&cord_rows(from, to, weight, grow), eye, colour, alpha, phase * 0.3)
            }
        };
        let _ = meshes.insert(&l.mesh, geometry.into_mesh());
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM: Vec3 = Vec3::new(-4.0, 0.2, 3.0);
    const TO: Vec3 = Vec3::new(5.0, 0.2, -6.0);

    #[test]
    fn an_arrow_runs_from_its_source_to_a_point_at_its_target() {
        let rows = arrow_rows(FROM, TO, 1.0, 1.0, [0.0; 2]);
        let (first, last) = (rows[0], rows[rows.len() - 1]);
        assert!(first.at.distance(FROM) < 1e-3, "{first:?}");
        assert!(last.at.distance(TO) < 0.02, "{last:?}");
        assert!(last.half.abs() < 1e-6, "the tip is a point");
        // The head is broader than the shaft, and the tail narrower.
        let widest = rows.iter().map(|r| r.half).fold(0.0, f32::max);
        assert_eq!(widest, HEAD_HALF);
        assert!(first.half < SHAFT_HALF);
        // It arcs over the table rather than cutting across it.
        let top = rows.iter().map(|r| r.at.y).fold(f32::MIN, f32::max);
        assert!(top > FROM.y + ARC_MIN, "peaks at {top}");
    }

    #[test]
    fn a_growing_arrow_ends_part_way_along_its_curve() {
        let full = arrow_rows(FROM, TO, 1.0, 1.0, [0.0; 2]);
        let half = arrow_rows(FROM, TO, 1.0, 0.5, [0.0; 2]);
        let (full_len, half_len) = (full.last().unwrap().arc, half.last().unwrap().arc);
        assert!((half_len - full_len / 2.0).abs() < 1e-3, "{half_len} of {full_len}");
        assert!(arrow_rows(FROM, TO, 1.0, 0.0, [0.0; 2]).is_empty());
    }

    #[test]
    fn a_trimmed_arrow_leaves_its_ends_bare() {
        let full = arrow_rows(FROM, TO, 1.0, 1.0, [0.0; 2]);
        let trimmed = arrow_rows(FROM, TO, 1.0, 1.0, [0.5, 0.8]);
        let (first, last) = (trimmed[0], trimmed[trimmed.len() - 1]);
        // Near its ends the curve is all but straight, so the bare stretch
        // is about as long as the chord.
        assert!((first.at.distance(FROM) - 0.5).abs() < 0.02, "{first:?}");
        assert!((last.at.distance(TO) - 0.8).abs() < 0.02, "{last:?}");
        // The drawn arrow's length is counted from where it starts.
        assert_eq!(first.arc, 0.0);
        assert!((last.arc - (full.last().unwrap().arc - 1.3)).abs() < 1e-3, "{last:?}");
    }

    #[test]
    fn an_end_seen_head_on_is_left_barer() {
        // Seen square on, the arc lies across the line of sight and a trim
        // is its own length...
        let eye = Vec3::new(0.0, 0.0, 100.0);
        let (from, to) = (Vec3::new(-5.0, 0.0, 0.0), Vec3::new(5.0, 0.0, 0.0));
        let [_, across] = seen_trim(from, to, [0.5; 2], eye);
        assert!((across - 0.5).abs() < 1e-3, "{across}");
        // ...and seen from beyond its target, where it comes in toward the
        // eye (at the slant it comes down onto the table), it stretches.
        let eye = Vec3::new(100.0, 0.0, 0.0);
        let [_, head_on] = seen_trim(from, to, [0.5; 2], eye);
        assert!(head_on > 0.9, "{head_on}");
    }

    #[test]
    fn the_ribbon_faces_the_camera() {
        let eye = Vec3::new(0.0, 30.0, 25.0);
        let rows = arrow_rows(FROM, TO, 1.0, 1.0, [0.0; 2]);
        let g = ribbon(&rows, eye, LinearRgba::RED, 1.0, 0.0);
        let n = COLUMNS.len();
        for (i, row) in rows.iter().enumerate().filter(|(_, r)| r.half > 0.0) {
            let (left, right) = (Vec3::from(g.positions[i * n]), Vec3::from(g.positions[i * n + n - 1]));
            let across = (right - left).normalize();
            assert!(across.dot((eye - row.at).normalize()).abs() < 1e-3, "row {i} tilts away from the eye");
            assert!(across.dot(row.along).abs() < 1e-3, "row {i} is not square to the curve");
        }
        assert_eq!(g.indices.len(), (rows.len() - 1) * (n - 1) * 6);
    }

    #[test]
    fn the_rim_is_dark_and_the_core_glows() {
        let [rim, body, core] = tones(LinearRgba::rgb(0.2, 0.8, 1.0), 1.0, 0.0);
        // The body stays in range, keeping its hue; the core glows.
        assert!(rim[2] < 0.2 && body[2] <= 1.0 && core[2] > 1.0);
        // A light band brightens the core.
        let [_, _, lit] = tones(LinearRgba::rgb(0.2, 0.8, 1.0), 1.0, 1.0);
        assert!(lit[2] > core[2]);
    }

    #[test]
    fn a_cord_joins_its_ends_without_a_head() {
        let rows = cord_rows(FROM, TO, 1.0, 1.0);
        let (first, last) = (rows[0], rows[rows.len() - 1]);
        assert!(first.at.distance(FROM) < 1e-3 && last.at.distance(TO) < 1e-3);
        // Even along its middle, narrower at its ends, never wider.
        let middle = rows[rows.len() / 2].half;
        assert!((middle - CORD_HALF).abs() < 1e-6, "{middle}");
        assert!(first.half < middle && last.half < middle);
        assert!(rows.iter().all(|r| r.half <= CORD_HALF + 1e-6));
        // It grows out of the attachment like an arrow.
        let half_grown = cord_rows(FROM, TO, 1.0, 0.5);
        assert!((half_grown.last().unwrap().arc - last.arc / 2.0).abs() < 1e-3);
    }

    #[test]
    fn a_ring_closes_on_itself() {
        let g = ring(Vec3::ZERO, 1.0, Vec3::Y, LinearRgba::RED, 1.0, 1.0, 0.0);
        assert_eq!(g.indices.len(), RING_SEGMENTS * (COLUMNS.len() - 1) * 6);
        assert!(g.indices.iter().all(|&i| (i as usize) < g.positions.len()));
        for p in &g.positions {
            let r = Vec2::new(p[0], p[2]).length();
            assert!((r - 1.0).abs() <= RING_HALF + 1e-5, "{r}");
            assert!(p[1].abs() < 1e-6, "flat on the table");
        }
        // Square to a card standing up to face the camera, it lies in the
        // card's face.
        let facing = Vec3::new(0.0, 0.5, 1.0).normalize();
        let g = ring(Vec3::ZERO, 1.0, facing, LinearRgba::RED, 1.0, 1.0, 0.0);
        assert!(g.positions.iter().all(|p| Vec3::from(*p).dot(facing).abs() < 1e-5));
    }
}
