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

/// Seconds a seat has to have been out, with the game still going, before
/// the "You're out" panel shows. A local pod's bots finish the rest of the
/// game in about a second once nobody is waiting on you, and the game-over
/// screen says it all then; the panel is for a game that goes on (a network
/// pod with people still playing).
const OUT_PANEL_DELAY: f64 = 2.5;

/// The viewer's out-of-the-game state: when they went out (app seconds),
/// and whether they chose to keep watching.
#[derive(Resource, Default)]
pub struct OutPanelState {
    since: Option<f64>,
    dismissed: bool,
}

#[derive(Component)]
pub struct OutPanel;

#[derive(Component)]
pub struct KeepWatchingButton;

#[derive(Component)]
pub struct LeaveOutGameButton;

/// "You finished 3rd of 4" — the viewer's placing, when they have one.
pub fn placing_line(cv: &crabomination::net::ClientView) -> Option<String> {
    let me = cv.players.iter().find(|p| p.seat == cv.your_seat)?;
    let place = me.placement?;
    Some(format!("You finished {} of {}", ordinal(place), cv.players.len()))
}

/// 1st, 2nd, 3rd, 4th … 11th, 12th, 13th, 21st.
pub fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// How a seat went out, for a sentence: the view's short loss label read
/// as a phrase ("commander" → "commander damage").
pub fn out_phrase(label: &str) -> &str {
    match label {
        "life" => "life total",
        "commander" => "commander damage",
        "decked" => "drew from an empty library",
        "lose effect" => "a lose-the-game effect",
        other => other,
    }
}

/// "Out on turn 9 · commander damage" for a seat that has left the game.
pub fn out_detail(p: &crabomination::net::PlayerView) -> Option<String> {
    let turn = p.out_on_turn?;
    Some(match p.loss_reason.as_deref() {
        Some(r) => format!("out on turn {turn} · {}", out_phrase(r)),
        None => format!("out on turn {turn}"),
    })
}

/// Show the "You're out" panel while the viewer is out of a game that goes
/// on: their placing and how they went out, with Keep watching (dismiss)
/// and Leave game. Gone when the game ends (the game-over screen takes
/// over) or a new one starts.
pub fn sync_out_panel(
    mut commands: Commands,
    view: Res<CurrentView>,
    time: Res<Time>,
    ui_fonts: Res<crate::theme::UiFonts>,
    mut state: ResMut<OutPanelState>,
    existing: Query<Entity, With<OutPanel>>,
) {
    use crate::theme::{self, HoverTint, RADIUS_BUTTON, RADIUS_PANEL};
    let me = view.0.as_ref().and_then(|cv| {
        cv.players.iter().find(|p| p.seat == cv.your_seat).filter(|p| p.eliminated && cv.game_over.is_none())
    });
    let Some(me) = me else {
        *state = OutPanelState::default();
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    };
    let now = time.elapsed_secs_f64();
    let since = *state.since.get_or_insert(now);
    if state.dismissed || now - since < OUT_PANEL_DELAY {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }
    if !existing.is_empty() {
        return;
    }
    let Some(cv) = view.0.as_ref() else { return };
    let title = match placing_line(cv) {
        Some(line) => format!("You're out — {}", line.trim_start_matches("You finished ")),
        None => "You're out".to_string(),
    };
    let detail = out_detail(me).unwrap_or_default();
    let tf = |size: f32| ui_fonts.tf(size);
    let button = |label: &'static str, bg: Color| {
        (
            Button,
            Node {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(9.0)),
                border_radius: BorderRadius::all(RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(bg),
            HoverTint::new(bg),
            children![(Text::new(label), tf(16.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE)],
        )
    };
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(90.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Pickable::IGNORE,
        GlobalZIndex(theme::layer::MODAL),
        OutPanel,
        crate::systems::game_ui::InGameRoot,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(22.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
            BorderColor::all(theme::ACCENT_ORANGE),
            children![
                (Text::new(title), tf(24.0), TextColor(theme::ACCENT_ORANGE)),
                (Text::new(detail), tf(14.0), TextColor(theme::TEXT_BODY)),
                (
                    Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() },
                    children![
                        (button("Keep watching", theme::BUTTON_INFO_BG), KeepWatchingButton),
                        (button("Leave game", theme::BUTTON_TERTIARY_BG), LeaveOutGameButton),
                    ],
                ),
            ],
        )],
    ));
}

/// The panel's buttons: Keep watching hides it for the rest of the game;
/// Leave game goes back to the menu, as the Esc menu's Leave Game does.
pub fn handle_out_panel_buttons(
    keep: Query<&Interaction, (Changed<Interaction>, With<KeepWatchingButton>)>,
    leave: Query<&Interaction, (Changed<Interaction>, With<LeaveOutGameButton>)>,
    mut state: ResMut<OutPanelState>,
    mut pending: ResMut<crate::menu::PendingNetMode>,
    mut next_state: ResMut<NextState<crate::menu::AppState>>,
) {
    if keep.iter().any(|i| *i == Interaction::Pressed) {
        state.dismissed = true;
    }
    if leave.iter().any(|i| *i == Interaction::Pressed) {
        pending.0 = None;
        next_state.set(crate::menu::AppState::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::shroud_rect;

    #[test]
    fn placings_read_as_ordinals_with_how_the_seat_went_out() {
        use super::{ordinal, out_detail};
        let names: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 22, 101].iter().map(|&n| ordinal(n)).collect();
        assert_eq!(names, ["1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "101st"]);
        let mut g = crabomination::game::multi_player_game(4);
        g.turn_number = 9;
        g.commander_damage.insert((2, crabomination::card::CardId(77)), 21);
        g.check_state_based_actions();
        let cv = crabomination::server::view::project(&g, 2);
        let me = &cv.players[2];
        assert_eq!(me.placement, Some(4));
        assert_eq!(out_detail(me).as_deref(), Some("out on turn 9 · commander damage"));
        assert_eq!(super::placing_line(&cv).as_deref(), Some("You finished 4th of 4"));

        // Two more go out together; seat 0 wins. The game-over standings
        // rank the table best first, ties sharing a placing.
        g.turn_number = 12;
        g.players[1].life = 0;
        g.players[3].life = -2;
        g.check_state_based_actions();
        let cv = crabomination::server::view::project(&g, 2);
        let lines: Vec<String> =
            crate::systems::game_over::pod_standings(&cv).into_iter().map(|(l, _)| l).collect();
        assert_eq!(
            lines,
            [
                "1st  P0 — 20 life",
                "2nd  P1 — out on turn 12 · life total",
                "2nd  P3 — out on turn 12 · life total",
                "4th  You — out on turn 9 · commander damage",
            ],
        );
    }

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
