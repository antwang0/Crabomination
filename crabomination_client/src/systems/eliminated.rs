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

/// Table-level rectangle the shroud covers: the seat's board outline grown
/// to take in its pile strip — graveyard, library and command zone — so any
/// card still drawn at a dead seat (a commander the command zone hasn't
/// cleared yet, a lingering pile) sits under the dimming too, not just the
/// battlefield rows.
pub fn shroud_rect(seat: usize, viewer: usize, n_seats: usize) -> (Vec3, Vec3) {
    use crate::card::{CARD_HEIGHT, CARD_WIDTH};
    let (mut min, mut max) = crate::card::layout::seat_board_outline(seat, viewer, n_seats);
    let piles = [
        crate::card::graveyard_position(seat, viewer, n_seats),
        crate::card::deck_position(seat, viewer, n_seats),
        crate::card::command_zone_card_transform(seat, viewer, n_seats, 0).translation,
    ];
    // Cards lie flat, so a pile spans half a card either way in X and Z
    // (the far-edge command zone draws at 1.3x — cover that too).
    let half = Vec3::new(CARD_WIDTH * 0.7, 0.0, CARD_HEIGHT * 0.7);
    for p in piles {
        min = min.min(p - half);
        max = max.max(p + half);
    }
    (Vec3::new(min.x, 0.0, min.z), Vec3::new(max.x, 0.0, max.z))
}

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
        let (min, max) = shroud_rect(seat, viewer, n);
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

#[cfg(test)]
mod tests {
    use super::shroud_rect;

    fn inside(p: bevy::prelude::Vec3, (min, max): (bevy::prelude::Vec3, bevy::prelude::Vec3)) -> bool {
        p.x >= min.x && p.x <= max.x && p.z >= min.z && p.z <= max.z
    }

    #[test]
    fn shroud_covers_board_and_every_pile_of_a_four_seat_pod() {
        for viewer in 0..4 {
            for seat in (0..4).filter(|s| *s != viewer) {
                let rect = shroud_rect(seat, viewer, 4);
                let (bmin, bmax) = crate::card::layout::seat_board_outline(seat, viewer, 4);
                assert!(inside(bmin, rect) && inside(bmax, rect), "board of {seat}");
                for p in [
                    crate::card::graveyard_position(seat, viewer, 4),
                    crate::card::deck_position(seat, viewer, 4),
                    crate::card::command_zone_card_transform(seat, viewer, 4, 0).translation,
                ] {
                    assert!(inside(p, rect), "pile {p:?} of seat {seat} (viewer {viewer})");
                }
                // A different opponent's command zone is not shrouded.
                let other = (0..4).find(|s| *s != viewer && *s != seat).unwrap();
                let cz = crate::card::command_zone_card_transform(other, viewer, 4, 0).translation;
                assert!(!inside(cz, rect), "seat {seat}'s shroud spills onto seat {other}");
            }
        }
    }
}
