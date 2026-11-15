//! "Ana is thinking…" — a chip at the top of the table while another seat
//! has been deciding for a moment.
//!
//! A bot's search pauses the table for a second or more (the client pilot's
//! 256-rollout menu search), and a remote player can take as long as they
//! like. With nothing moving that read as a frozen client. The chip names
//! the seat the game is waiting on once the view has sat unchanged for
//! [`THINKING_AFTER_SECS`], so a quick pass never flickers it on.

use bevy::prelude::*;

use crate::net_plugin::CurrentView;
use crate::theme::{self, UiFonts};

/// How long the view must sit still, on someone else's decision, before the
/// chip shows.
pub const THINKING_AFTER_SECS: f32 = 0.6;

#[derive(Component)]
pub struct ThinkingChip;

#[derive(Component)]
pub struct ThinkingChipText;

/// The seat the game is waiting on, if it is not the viewer: the player a
/// pending decision names, else the priority holder. `None` once the game
/// is over.
pub fn waiting_on(cv: &crabomination::net::ClientView) -> Option<usize> {
    if cv.game_over.is_some() {
        return None;
    }
    let seat = cv.pending_decision.as_ref().map_or(cv.priority, |pd| pd.acting_player);
    (seat != cv.your_seat).then_some(seat)
}

/// The chip's text `secs` into the wait, dots cycling so it reads as live.
fn chip_text(name: &str, secs: f32) -> String {
    let dots = 1 + (secs * 2.0) as usize % 3;
    format!("{name} is thinking{}", ".".repeat(dots))
}

pub fn sync_thinking_chip(
    mut commands: Commands,
    time: Res<Time>,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    mut since: Local<f32>,
    chip: Query<Entity, With<ThinkingChip>>,
    mut text: Query<&mut Text, With<ThinkingChipText>>,
) {
    let now = time.elapsed_secs();
    if view.is_changed() {
        *since = now;
    }
    let waiting = view.0.as_ref().and_then(|cv| {
        let seat = waiting_on(cv)?;
        Some(cv.players.get(seat).map_or_else(|| format!("Seat {}", seat + 1), |p| p.name.clone()))
    });
    let label = match waiting {
        Some(name) if now - *since >= THINKING_AFTER_SECS => Some(chip_text(&name, now - *since)),
        _ => None,
    };
    match (label, chip.iter().next()) {
        (None, Some(e)) => commands.entity(e).despawn(),
        (None, None) => {}
        (Some(label), Some(_)) => {
            for mut t in &mut text {
                if t.0 != label {
                    t.0 = label.clone();
                }
            }
        }
        (Some(label), None) => {
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        // The pending-cast banner's slot: that banner is up
                        // only while the viewer holds priority, this chip
                        // only while someone else does.
                        top: Val::Px(64.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ThinkingChip,
                    crate::systems::game_ui::InGameRoot,
                    Pickable::IGNORE,
                    GlobalZIndex(theme::layer::BANNER),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(theme::HUD_BG),
                        Pickable::IGNORE,
                    ))
                    .with_children(|chip| {
                        chip.spawn((
                            Text::new(label),
                            ui_fonts.tf(14.0),
                            TextColor(theme::TEXT_SECONDARY),
                            ThinkingChipText,
                            Pickable::IGNORE,
                        ));
                    });
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The chip waits on whoever must act: the priority holder, or the seat
    /// a pending decision names; never the viewer, never after the game.
    #[test]
    fn it_waits_on_the_seat_that_must_act() {
        let state = crabomination::demo::build_demo_state_seeded(7);
        let mut cv = crabomination::server::view::project(&state, 0);
        cv.pending_decision = None;
        cv.priority = 0;
        assert_eq!(waiting_on(&cv), None, "the viewer's own priority");
        cv.priority = 1;
        assert_eq!(waiting_on(&cv), Some(1));
        cv.game_over = Some(Some(0));
        assert_eq!(waiting_on(&cv), None, "nobody thinks after the game");
    }

    #[test]
    fn the_dots_cycle() {
        assert_eq!(chip_text("Ana", 0.0), "Ana is thinking.");
        assert_eq!(chip_text("Ana", 0.6), "Ana is thinking..");
        assert_eq!(chip_text("Ana", 1.1), "Ana is thinking...");
        assert_eq!(chip_text("Ana", 1.6), "Ana is thinking.");
    }
}
