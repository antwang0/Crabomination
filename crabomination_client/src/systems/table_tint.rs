//! Seat tints: the table under each player's area is shaded in that
//! player's seat colour (`player_stats::seat_color`, the same colour as
//! their HUD avatar and attack arrows), so whose board is whose reads from
//! the table itself — a quadrant each in a 4-player pod, a half each in 1v1.
//! The regions come from `card::layout::seat_region`, which leaves a seam of
//! plain table between neighbours.

use bevy::prelude::*;

use crate::net_plugin::CurrentView;

/// Marker on a tint quad.
#[derive(Component)]
pub struct SeatTint;

/// Height of the tint above the ground plane: clear of it (no z-fighting),
/// under every card (battlefield cards rest at y = 0.02).
const TINT_Y: f32 = 0.004;

/// The table's own colour (`main::setup`'s ground plane).
const TABLE: Color = Color::srgb(0.20, 0.20, 0.23);

/// Share of the seat colour in a region's tint: enough to tell four seats
/// apart at a glance, little enough that cards stay the brightest thing on
/// the table.
const TINT_STRENGTH: f32 = 0.16;

/// A seat's tint: the table colour moved [`TINT_STRENGTH`] of the way toward
/// its seat colour. Pure helper.
pub fn tint_color(seat: usize) -> Color {
    let base = TABLE.to_srgba();
    let seat = crate::systems::game_ui::table_awareness::seat_color(seat).to_srgba();
    let mix = |a: f32, b: f32| a + (b - a) * TINT_STRENGTH;
    Color::srgb(mix(base.red, seat.red), mix(base.green, seat.green), mix(base.blue, seat.blue))
}

/// Keep one tint quad per seat, rebuilt when the seat count or the viewer's
/// seat changes (the regions are viewer-relative); none without a view.
pub fn sync_seat_tints(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<Entity, With<SeatTint>>,
    mut built_for: Local<Option<(usize, usize)>>,
) {
    let key = view.0.as_ref().map(|cv| (cv.players.len(), cv.your_seat));
    if *built_for == key {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    *built_for = key;
    let Some((n, viewer)) = key else { return };
    for seat in 0..n {
        let r = crate::card::layout::seat_region(seat, viewer, n);
        let c = r.center();
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(r.width(), r.height()))),
            MeshMaterial3d(materials.add(tint_color(seat))),
            Transform::from_xyz(c.x, TINT_Y, c.y),
            SeatTint,
            // Despawned with the rest of the match on leaving it.
            crate::systems::game_ui::InGameRoot,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every seat's tint is distinct, and each stays close to the table
    /// colour (a shade, not a paint job).
    #[test]
    fn tints_are_distinct_shades_of_the_table() {
        let table = TABLE.to_srgba();
        let tints: Vec<Srgba> = (0..6).map(|s| tint_color(s).to_srgba()).collect();
        for (i, a) in tints.iter().enumerate() {
            let off = (a.red - table.red).abs() + (a.green - table.green).abs() + (a.blue - table.blue).abs();
            assert!(off > 0.02 && off < 0.25, "seat {i}'s tint is {off:.3} off the table colour");
            for b in tints.iter().skip(i + 1) {
                let d = (a.red - b.red).abs() + (a.green - b.green).abs() + (a.blue - b.blue).abs();
                assert!(d > 0.02, "two seats' tints are indistinguishable");
            }
        }
    }
}
