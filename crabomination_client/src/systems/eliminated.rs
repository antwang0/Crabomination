//! Shroud over an eliminated player's board area (CR 104.3): a dark
//! translucent quad hovering above their column, so a dead seat in a
//! multiplayer pod reads as *out* at a glance instead of an oddly quiet
//! board. The HUD roster greys the eliminated row; this mirrors the same
//! state onto the 3-D table.

use bevy::prelude::*;

use crate::net_plugin::CurrentView;

/// Marker on the shroud quad; payload is the shrouded seat.
#[derive(Component)]
pub struct EliminatedShroud(pub usize);

/// Height above the table for the shroud quad — above resting cards and
/// their stack cascades, below hovering stack spells.
const SHROUD_Y: f32 = 1.0;

pub fn sync_eliminated_shrouds(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<(Entity, &EliminatedShroud)>,
) {
    let Some(cv) = &view.0 else {
        for (e, _) in &existing {
            commands.entity(e).despawn();
        }
        return;
    };
    let n = cv.players.len();
    let viewer = cv.your_seat;

    let mut missing: Vec<usize> =
        cv.players.iter().filter(|p| p.eliminated).map(|p| p.seat).collect();
    for (e, shroud) in &existing {
        match missing.iter().position(|&s| s == shroud.0) {
            Some(i) => {
                missing.swap_remove(i); // already shrouded
            }
            None => {
                commands.entity(e).despawn(); // seat revived / new game
            }
        }
    }

    for seat in missing {
        let (min, max) = crate::card::layout::seat_board_outline(seat, viewer, n);
        let center = (min + max) / 2.0;
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(max.x - min.x, max.z - min.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.02, 0.02, 0.05, 0.55),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(center.x, SHROUD_Y, center.z),
            EliminatedShroud(seat),
        ));
    }
}
