//! The key light's shadows: fitted to the table the camera is looking at,
//! and cast only by what stands on it.
//!
//! **Cascades.** Bevy's default is four cascades out to a fixed distance,
//! split geometrically from 0.1 — tuned for a scene that runs from the
//! player's feet to the horizon. This one is a table seen from ~35 units
//! away: at the duel's home pose the whole of it lies between view depths of
//! roughly 30 and 43, so the default spent its first two cascades on the
//! empty air in front of the camera, the third on almost nothing, and cut the
//! shadows off at 39.2 — short of the opponent's side, and short of a whole
//! pod table. [`fit_shadow_cascades`] instead fits two cascades to the view
//! depths the table actually spans, from the live camera, so every texel of
//! the map lands on the table and the far edge is shadowed too.
//!
//! **Casters.** Only things that stand above the table cast: cards, piles,
//! hands. The felt and what is printed on it (seat tints, zone outlines, pile
//! slots) lie flat on the table — as casters they only shadowed themselves,
//! which is what forced Bevy's default depth and normal biases on the scene,
//! and those biases lifted the shadow test above a resting card (a card lies
//! a few hundredths above the felt). With nothing flat left in the map the
//! biases can be small and a resting card's shadow meets its edge.
//! Highlight rings, counter chips and the eliminated seat's shroud cast
//! nothing a player needs (a ring's halo round a card's shadow, a chip's on
//! the unlit face it sits on) and are dropped too, as are all but every
//! fourth card of a library pile — the rest only thicken a silhouette the
//! fourth already draws. The stack lane's cards hang in front of the table,
//! toward the camera, and their shadows fell on the board below as stray
//! rectangles; they cast none while they are there.

use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder, NotShadowCaster};
use bevy::prelude::*;

use crate::card::{CardHighlightAssets, DeckPile, StackCard};
use crate::net_plugin::CurrentView;

/// Marker on the directional light that casts the table's shadows.
#[derive(Component)]
pub struct KeyLight;

/// How far outside the seats' boards the shadowed area reaches: the decks,
/// graveyards and exile beside the boards, and the viewer's hand fan.
const TABLE_MARGIN: f32 = 6.0;
/// The highest a card rises off the table outside an animation (hover lift,
/// the tallest pile), for the depth span.
const TABLE_HEIGHT: f32 = 2.0;
/// Slack on either end of the fitted depth span.
const DEPTH_SLACK: f32 = 1.0;
/// Where the two cascades meet, as a fraction of the table's depth span from
/// its near edge. The near half — the viewer's side — gets the finer texels.
const SPLIT: f32 = 0.45;

pub struct ShadowsPlugin;

impl Plugin for ShadowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (fit_shadow_cascades, quiet_flat_casters, stack_lane_casts_no_shadow));
    }
}

/// The view-depth span `(near, far)` of the table seen from `camera`: the
/// play area (every seat's board, plus [`TABLE_MARGIN`]) from the felt up to
/// [`TABLE_HEIGHT`].
pub(crate) fn table_depth_span(camera: &GlobalTransform, area: Rect) -> (f32, f32) {
    let view = camera.affine().inverse();
    let area = area.inflate(TABLE_MARGIN);
    let mut near = f32::INFINITY;
    let mut far = 0.0f32;
    for x in [area.min.x, area.max.x] {
        for z in [area.min.y, area.max.y] {
            for y in [0.0, TABLE_HEIGHT] {
                let depth = -view.transform_point3(Vec3::new(x, y, z)).z;
                near = near.min(depth);
                far = far.max(depth);
            }
        }
    }
    (near, far)
}

/// The two-cascade config for a table spanning view depths `near..far`.
pub(crate) fn fitted_cascades(near: f32, far: f32) -> CascadeShadowConfig {
    let minimum = (near - DEPTH_SLACK).max(0.1);
    let maximum = (far + DEPTH_SLACK).max(minimum + 1.0);
    CascadeShadowConfigBuilder {
        num_cascades: 2,
        minimum_distance: minimum,
        first_cascade_far_bound: minimum + (maximum - minimum) * SPLIT,
        maximum_distance: maximum,
        overlap_proportion: 0.2,
    }
    .build()
}

/// Bevy system (`PostUpdate`): refit the key light's cascades to the table
/// whenever the camera or the table moves. Written only when the fit
/// changes, so a still camera leaves it alone (and the frame loop idle).
pub fn fit_shadow_cascades(
    view: Res<CurrentView>,
    camera: Query<&GlobalTransform, With<crate::MainCamera>>,
    mut lights: Query<&mut CascadeShadowConfig, With<KeyLight>>,
) {
    let Ok(camera) = camera.single() else { return };
    let (viewer, seats) = view.0.as_ref().map(|cv| (cv.your_seat, cv.players.len())).unwrap_or((0, 2));
    let area = crate::systems::table_cloth::play_area(viewer, seats);
    let (near, far) = table_depth_span(camera, area);
    let want = fitted_cascades(near, far);
    for mut config in &mut lights {
        if config.bounds != want.bounds
            || config.minimum_distance != want.minimum_distance
            || config.overlap_proportion != want.overlap_proportion
        {
            *config = want.clone();
        }
    }
}

/// Bevy system (`PostUpdate`): drop the casters that only lie flat on the
/// table or cast nothing a player reads (see the module docs), as they spawn.
#[allow(clippy::type_complexity)]
fn quiet_flat_casters(
    mut commands: Commands,
    highlight: Option<Res<CardHighlightAssets>>,
    meshes: Query<(Entity, &Mesh3d), Added<Mesh3d>>,
    flat: Query<
        Entity,
        Or<(
            Added<crate::systems::table_tint::SeatTint>,
            Added<crate::systems::eliminated::EliminatedShroud>,
            Added<crate::systems::counter_coins::CounterCoin>,
        )>,
    >,
    piles: Query<(Entity, &DeckPile), Added<DeckPile>>,
) {
    for e in &flat {
        commands.entity(e).try_insert(NotShadowCaster);
    }
    if let Some(highlight) = highlight {
        for (e, mesh) in &meshes {
            if mesh.0 == highlight.border_mesh {
                commands.entity(e).try_insert(NotShadowCaster);
            }
        }
    }
    for (e, pile) in &piles {
        if pile.index % 4 != 0 {
            commands.entity(e).try_insert(NotShadowCaster);
        }
    }
}

/// Bevy system (`PostUpdate`): a card casts no shadow while it hangs in the
/// stack lane, and casts again when it leaves it.
fn stack_lane_casts_no_shadow(
    mut commands: Commands,
    entered: Query<&Children, Added<StackCard>>,
    mut left: RemovedComponents<StackCard>,
    children: Query<&Children>,
) {
    for faces in &entered {
        for face in faces.iter() {
            commands.entity(face).try_insert(NotShadowCaster);
        }
    }
    for card in left.read() {
        if let Ok(faces) = children.get(card) {
            for face in faces.iter() {
                commands.entity(face).try_remove::<NotShadowCaster>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The duel's home pose sees the whole table inside the fitted span, and
    /// the far edge — which Bevy's default four cascades cut off at 39.2 —
    /// is inside it.
    #[test]
    fn the_fitted_cascades_cover_the_whole_table() {
        let pose = crate::card::framing::home_pose(2, Vec2::new(1920.0, 1080.0), 1.0);
        let camera = GlobalTransform::from(pose);
        let area = crate::systems::table_cloth::play_area(0, 2);
        let (near, far) = table_depth_span(&camera, area);
        assert!(near > 1.0 && far > near, "span {near}..{far}");
        let config = fitted_cascades(near, far);
        assert_eq!(config.bounds.len(), 2);
        assert!(config.minimum_distance <= near);
        assert!(*config.bounds.last().unwrap() >= far, "{:?} ends before {far}", config.bounds);
    }

    /// A pod's camera sits further back; the fit follows it.
    #[test]
    fn a_pod_table_is_covered_too() {
        let pose = crate::card::framing::home_pose(4, Vec2::new(1920.0, 1080.0), 1.0);
        let camera = GlobalTransform::from(pose);
        let area = crate::systems::table_cloth::play_area(0, 4);
        let (near, far) = table_depth_span(&camera, area);
        let config = fitted_cascades(near, far);
        assert!(*config.bounds.last().unwrap() >= far);
        assert!(config.minimum_distance < config.bounds[0]);
    }
}
