//! Seat tints: the table under each player's area is shaded in that
//! player's seat colour (`player_stats::seat_color`, the same colour as
//! their HUD avatar and attack arrows), so whose board is whose reads from
//! the table itself — a quadrant each in a 4-player pod, a half each in 1v1.
//! The regions come from `card::layout::seat_region`, which leaves a seam of
//! plain table between neighbours.
//!
//! In a pod each seat's deck also carries a name plate: a tint tells four
//! boards apart, not whose each one is.

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

/// A seat's name plate: its name and life, on its deck pile.
#[derive(Component)]
pub struct SeatNamePlate(pub usize);

/// The plate's text: the seat's name ("You" for the viewer) and life, or
/// the name under a skull once the seat is out. Pure helper.
fn plate_label(cv: &crabomination::net::ClientView, p: &crabomination::net::PlayerView) -> String {
    let name = crate::systems::game_ui::table_awareness::seat_label(&cv.players, cv.your_seat, p.seat);
    if p.eliminated { format!("\u{2620} {name}") } else { format!("{name}  \u{2665} {}", p.life) }
}

/// Keep a name plate on every seat's deck pile in a pod (3+ seats; a duel's
/// far board needs no name), centred on the top of the pile and moved with
/// the camera every frame.
pub fn sync_seat_name_plates(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<crate::theme::UiFonts>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    ui_scale: Res<UiScale>,
    mut plates: Query<(Entity, &SeatNamePlate, &mut Node, &mut Text, &bevy::ui::ComputedNode)>,
) {
    let pod = view.0.as_ref().filter(|cv| cv.players.len() > 2);
    let camera = camera_q.single().ok();
    let (Some(cv), Some((camera, cam_xform))) = (pod, camera) else {
        for (e, ..) in &plates {
            commands.entity(e).despawn();
        }
        return;
    };
    let n = cv.players.len();
    // Centre of the top of the seat's deck pile, in UI px.
    let pile_top = |p: &crabomination::net::PlayerView| {
        let base = crate::card::layout::deck_position(p.seat, cv.your_seat, n);
        let top = base + Vec3::Y * crate::card::pile_height(p.library.size);
        crate::theme::project_to_ui(camera, cam_xform, &ui_scale, top)
    };
    let mut placed = vec![false; n];
    for (e, plate, mut node, mut text, computed) in &mut plates {
        let Some(p) = cv.players.iter().find(|p| p.seat == plate.0) else {
            commands.entity(e).despawn();
            continue;
        };
        placed[p.seat.min(n - 1)] = true;
        let label = plate_label(cv, p);
        if text.0 != label {
            text.0 = label;
        }
        match pile_top(p) {
            Some(at) => {
                let half = computed.size() * computed.inverse_scale_factor() * 0.5;
                node.display = Display::Flex;
                node.left = Val::Px(at.x - half.x);
                node.top = Val::Px(at.y - half.y);
            }
            None => node.display = Display::None,
        }
    }
    for p in &cv.players {
        if placed.get(p.seat).copied().unwrap_or(true) {
            continue;
        }
        // Parked off screen for the frame before layout has sized it.
        commands.spawn((
            SeatNamePlate(p.seat),
            Text::new(plate_label(cv, p)),
            ui_fonts.tf(13.0),
            TextColor(crate::theme::TEXT_PRIMARY),
            BackgroundColor(crate::theme::HUD_BG),
            BorderColor::all(crate::systems::game_ui::table_awareness::seat_color(p.seat)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-1000.0),
                top: Val::Px(-1000.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                border: UiRect::left(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(crate::theme::layer::CARD_OVERLAY),
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
