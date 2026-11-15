//! The game log, the stack panel and the board point at each other.
//!
//! * Clicking a log line that names a card **pings** it on the table: a gold
//!   ring that swells and fades around the permanent for
//!   [`PING_SECS`], so "Llanowar Elves deals 1 damage" finds its elf in a
//!   twelve-creature row. (Hovering the line already previews the card.)
//! * Clicking a stack tile **scrolls the log** to the newest line about that
//!   object and flashes the line, so "where did this trigger come from"
//!   is one click.

use bevy::prelude::*;
use bevy::ui::{ComputedNode, ScrollPosition, UiGlobalTransform};
use crabomination::card::CardId;

use crate::game::GameLog;
use crate::systems::arrows::{ArrowKey, Arrows, Cue, End};
use crate::systems::game_ui::{GameLogOuterPanel, LogRow};
use crate::systems::gizmos::BoardSpots;
use crate::systems::ui_card_hover::UiCardHover;
use crate::theme;
use crate::ui_button::Interaction;

/// How long a ping and a log-line flash last.
pub const PING_SECS: f32 = 1.4;

/// The permanent a log click is pinging, and when it started.
#[derive(Resource, Default)]
pub struct BoardPing(Option<(CardId, f32)>);

impl BoardPing {
    /// Ring `card` on the board from `now` (a log click, a hint).
    pub fn ping(&mut self, card: CardId, now: f32) {
        self.0 = Some((card, now));
    }
}

/// The log line a stack-tile click is bringing into view, and when.
#[derive(Resource, Default)]
pub struct LogFocus {
    target: Option<(u64, f32)>,
    /// The line has been scrolled to (its position is only known once it
    /// has been laid out, so the scroll may wait a frame).
    scrolled: bool,
}

/// Marks a stack-panel tile: clicking it focuses its log line.
#[derive(Component)]
pub struct StackTile;

/// A log line flashing after a stack-tile click; its background before.
#[derive(Component)]
pub struct LogRowFlash {
    started: f32,
    base: Color,
}

/// Click a log row that names a card → ping that card on the board.
pub fn ping_from_log_click(
    time: Res<Time>,
    rows: Query<(&Interaction, &UiCardHover), (Changed<Interaction>, With<LogRow>)>,
    mut ping: ResMut<BoardPing>,
) {
    for (interaction, card) in &rows {
        if *interaction == Interaction::Pressed
            && let Some(id) = card.id
        {
            ping.0 = Some((id, time.elapsed_secs()));
        }
    }
}

/// The ping's ring at `t` seconds in: radius factor and alpha. Two quick
/// swells, then a fade, so it reads as "here" rather than as a selection.
fn ping_shape(t: f32) -> Option<(f32, f32)> {
    if !(0.0..PING_SECS).contains(&t) {
        return None;
    }
    let swell = (t * std::f32::consts::TAU * 1.4).sin().abs();
    let fade = 1.0 - t / PING_SECS;
    Some((1.25 + 0.25 * swell, 0.35 + 0.6 * fade))
}

pub fn draw_board_ping(time: Res<Time>, board: BoardSpots, mut ping: ResMut<BoardPing>, mut arrows: ResMut<Arrows>) {
    let Some((id, started)) = ping.0 else { return };
    let Some((radius, alpha)) = ping_shape(time.elapsed_secs() - started) else {
        ping.0 = None;
        return;
    };
    // A card no longer on the battlefield has nothing to ring; the hover
    // preview already showed it.
    let Some((at, normal, scale)) = board.spots().face(id, 0.25) else { return };
    let key = ArrowKey::new(Cue::Ping, End::Card(id), End::Card(id));
    let gold = theme::ACCENT_GOLD.to_srgba();
    arrows.ring(key, at, radius * scale, normal, Color::srgba(gold.red, gold.green, gold.blue, alpha));
}

/// Click a stack tile → focus the newest log line about its object.
pub fn focus_log_from_stack_click(
    time: Res<Time>,
    tiles: Query<(&Interaction, &UiCardHover), (Changed<Interaction>, With<StackTile>)>,
    log: Res<GameLog>,
    mut focus: ResMut<LogFocus>,
) {
    for (interaction, card) in &tiles {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(id) = card.id else { continue };
        if let Some(seq) = newest_line_about(&log, id) {
            focus.target = Some((seq, time.elapsed_secs()));
            focus.scrolled = false;
        }
    }
}

/// The newest log entry whose card is `id`.
fn newest_line_about(log: &GameLog, id: CardId) -> Option<u64> {
    log.entries.iter().rev().find(|e| e.card.as_ref().is_some_and(|c| c.id == Some(id))).map(|e| e.seq())
}

/// The scroll offset (logical px) that puts a row `row_top` px below the
/// panel's visible top a little under that top, given the current offset.
fn scroll_to_show(current: f32, row_top: f32) -> f32 {
    const MARGIN: f32 = 24.0;
    (current + row_top - MARGIN).max(0.0)
}

pub fn scroll_log_to_focus(
    mut commands: Commands,
    time: Res<Time>,
    mut focus: ResMut<LogFocus>,
    rows: Query<(Entity, &LogRow, &UiGlobalTransform, &ComputedNode, Option<&BackgroundColor>)>,
    mut panel: Query<(&mut ScrollPosition, &UiGlobalTransform, &ComputedNode), With<GameLogOuterPanel>>,
) {
    let Some((seq, started)) = focus.target else { return };
    if time.elapsed_secs() - started > PING_SECS {
        focus.target = None;
        return;
    }
    if focus.scrolled {
        return;
    }
    let Some((row, _, row_at, row_node, bg)) = rows.iter().find(|(_, r, ..)| r.0 == seq) else { return };
    let Ok((mut scroll, panel_at, panel_node)) = panel.single_mut() else { return };
    if row_node.size().y <= 0.0 {
        return; // not laid out yet
    }
    let inv = panel_node.inverse_scale_factor();
    let panel_top = panel_at.translation.y - panel_node.size().y / 2.0;
    let row_top = row_at.translation.y - row_node.size().y / 2.0;
    scroll.y = scroll_to_show(scroll.y, (row_top - panel_top) * inv);
    focus.scrolled = true;
    let base = bg.map_or(Color::NONE, |b| b.0);
    commands.entity(row).insert((LogRowFlash { started: time.elapsed_secs(), base }, BackgroundColor(base)));
}

/// Fade a flashed log line's background back to what it was.
pub fn fade_log_row_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut rows: Query<(Entity, &LogRowFlash, &mut BackgroundColor)>,
) {
    for (entity, flash, mut bg) in &mut rows {
        let t = time.elapsed_secs() - flash.started;
        if t >= PING_SECS {
            bg.0 = flash.base;
            commands.entity(entity).remove::<LogRowFlash>();
            continue;
        }
        let gold = theme::ACCENT_GOLD.to_srgba();
        bg.0 = Color::srgba(gold.red, gold.green, gold.blue, 0.45 * (1.0 - t / PING_SECS));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameLog;

    #[test]
    fn a_ping_swells_then_ends() {
        let (r0, a0) = ping_shape(0.0).unwrap();
        let (_, a_late) = ping_shape(PING_SECS * 0.9).unwrap();
        assert!(r0 >= 1.25 && a0 > a_late, "starts bright and fades");
        assert!(ping_shape(PING_SECS).is_none() && ping_shape(-0.1).is_none());
    }

    #[test]
    fn the_newest_line_about_a_card_is_focused() {
        let mut log = GameLog::default();
        let elf = Some(CardId(4));
        log.push_event_with_card("Elf enters", Color::WHITE, Some(UiCardHover::card("Llanowar Elves", elf)));
        log.push_event_with_card("Bolt resolves", Color::WHITE, Some(UiCardHover::card("Lightning Bolt", Some(CardId(9)))));
        log.push_event_with_card("Elf taps", Color::WHITE, Some(UiCardHover::card("Llanowar Elves", elf)));
        let newest = log.entries.back().unwrap().seq();
        assert_eq!(newest_line_about(&log, CardId(4)), Some(newest));
        assert_eq!(newest_line_about(&log, CardId(77)), None);
    }

    #[test]
    fn scrolling_lands_the_row_near_the_top_and_never_negative() {
        assert_eq!(scroll_to_show(100.0, 300.0), 376.0);
        assert_eq!(scroll_to_show(0.0, 10.0), 0.0);
    }
}
