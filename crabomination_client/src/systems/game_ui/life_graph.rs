//! Per-turn life-history sparkline. The floating `±N` numerals
//! ([`super::player_stats::trigger_life_flash`]) show the *last* swing; this
//! shows the shape of the whole game — one column per turn per seat, so a
//! slow drain reads differently from a single burn spell.
//!
//! Toggled with `I`; hidden by default so it never competes with the board.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::net_plugin::CurrentView;
use crate::theme::{self, UiFonts};

use super::InGameRoot;

/// How many turns of history the sparkline shows.
const WINDOW: usize = 20;
/// Pixel height of the tallest bar.
const BAR_MAX_H: f32 = 54.0;
const BAR_W: f32 = 7.0;
const BAR_GAP: f32 = 2.0;

/// One life sample per (seat, turn) — the seat's life at the *end* of the
/// last view seen for that turn, so a turn with several swings records its
/// net result.
#[derive(Resource, Default)]
pub struct LifeHistory {
    /// seat → turn-ordered `(turn, life)` samples.
    pub samples: HashMap<usize, Vec<(u32, i32)>>,
    /// Set while the panel is visible (`I`).
    pub shown: bool,
    /// Bumped whenever a sample changes, so the panel only respawns on real
    /// movement rather than every view.
    pub revision: u64,
}

impl LifeHistory {
    /// Record `life` for `seat` on `turn`, overwriting the turn's existing
    /// sample. Returns true when something actually changed.
    fn record(&mut self, seat: usize, turn: u32, life: i32) -> bool {
        let row = self.samples.entry(seat).or_default();
        match row.last_mut() {
            Some((t, l)) if *t == turn => {
                if *l == life {
                    return false;
                }
                *l = life;
            }
            _ => row.push((turn, life)),
        }
        true
    }

    /// The last `WINDOW` samples for `seat`.
    fn window(&self, seat: usize) -> &[(u32, i32)] {
        let row = self.samples.get(&seat).map(Vec::as_slice).unwrap_or(&[]);
        &row[row.len().saturating_sub(WINDOW)..]
    }
}

/// Marker for the sparkline panel so it can be despawned and rebuilt.
#[derive(Component)]
pub struct LifeGraphPanel;

/// Sample every seat's life once per view.
pub fn record_life_history(view: Res<CurrentView>, mut history: ResMut<LifeHistory>) {
    if !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    let turn = cv.turn;
    let mut changed = false;
    for p in &cv.players {
        changed |= history.record(p.seat, turn, p.life);
    }
    if changed {
        history.revision = history.revision.wrapping_add(1);
    }
}

/// `I` toggles the panel.
pub fn toggle_life_graph(keys: Res<ButtonInput<KeyCode>>, mut history: ResMut<LifeHistory>) {
    if keys.just_pressed(KeyCode::KeyI) {
        history.shown = !history.shown;
        history.revision = history.revision.wrapping_add(1);
    }
}

/// Rebuild the panel whenever the history moves or the toggle flips.
pub fn sync_life_graph(
    mut commands: Commands,
    view: Res<CurrentView>,
    history: Res<LifeHistory>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<LifeGraphPanel>>,
) {
    if !history.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    if !history.shown {
        return;
    }
    let Some(cv) = &view.0 else { return };

    // Seats in a stable order, viewer first.
    let mut seats: Vec<usize> = cv.players.iter().map(|p| p.seat).collect();
    seats.sort_unstable();
    seats.sort_by_key(|s| *s != cv.your_seat);

    // A shared ceiling keeps the seats visually comparable.
    let ceiling = seats
        .iter()
        .flat_map(|s| history.window(*s))
        .map(|(_, l)| *l)
        .chain(std::iter::once(1))
        .max()
        .unwrap_or(1)
        .max(1) as f32;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                bottom: Val::Px(120.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
            Pickable::IGNORE,
            InGameRoot,
            LifeGraphPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Life history (I)"),
                ui_fonts.tf(12.0),
                TextColor(theme::TEXT_MUTED),
                Pickable::IGNORE,
            ));
            for seat in seats {
                let samples = history.window(seat);
                let label = cv
                    .players
                    .iter()
                    .find(|p| p.seat == seat)
                    .map(|p| format!("{} — {}", p.name, p.life))
                    .unwrap_or_else(|| format!("Seat {seat}"));
                let is_you = seat == cv.your_seat;
                panel.spawn((
                    Text::new(label),
                    ui_fonts.tf(12.0),
                    TextColor(if is_you { theme::TEXT_INFO } else { theme::TEXT_SECONDARY }),
                    Pickable::IGNORE,
                ));
                panel
                    .spawn((
                        Node {
                            height: Val::Px(BAR_MAX_H),
                            align_items: AlignItems::FlexEnd,
                            column_gap: Val::Px(BAR_GAP),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|row| {
                        for (_, life) in samples {
                            let frac = (*life).max(0) as f32 / ceiling;
                            row.spawn((
                                Node {
                                    width: Val::Px(BAR_W),
                                    height: Val::Px((frac * BAR_MAX_H).max(1.0)),
                                    ..default()
                                },
                                BackgroundColor(bar_color(*life, is_you)),
                                Pickable::IGNORE,
                            ));
                        }
                    });
            }
        });
}

/// Green while healthy, amber under 10, red under 5 — the same thresholds the
/// life badge uses. The viewer's own bars are brighter.
fn bar_color(life: i32, is_you: bool) -> Color {
    let base = match life {
        l if l <= 4 => Color::srgb(0.85, 0.25, 0.25),
        l if l <= 9 => Color::srgb(0.85, 0.62, 0.22),
        _ => Color::srgb(0.35, 0.70, 0.42),
    };
    if is_you { base } else { base.with_alpha(0.55) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A turn's sample is overwritten in place, so a turn with several swings
    /// records one column at its net result.
    #[test]
    fn one_sample_per_turn_per_seat() {
        let mut h = LifeHistory::default();
        assert!(h.record(0, 1, 20));
        assert!(h.record(0, 1, 17), "same turn, new value");
        assert!(!h.record(0, 1, 17), "no movement");
        assert!(h.record(0, 2, 17), "new turn");
        assert_eq!(h.samples[&0], vec![(1, 17), (2, 17)]);
    }

    /// The window keeps only the most recent `WINDOW` turns.
    #[test]
    fn window_is_capped() {
        let mut h = LifeHistory::default();
        for t in 0..(WINDOW as u32 + 5) {
            h.record(0, t, 20 - t as i32);
        }
        let w = h.window(0);
        assert_eq!(w.len(), WINDOW);
        assert_eq!(w[0].0, 5, "the oldest five turns scrolled off");
    }

    /// A seat with no samples yet renders an empty row rather than panicking.
    #[test]
    fn unknown_seat_has_an_empty_window() {
        assert!(LifeHistory::default().window(3).is_empty());
    }
}
