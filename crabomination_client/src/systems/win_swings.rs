//! Post-game review: the moments your win chance fell furthest.
//!
//! Every view of a duel carries the value net's read of the viewer's win
//! chance (`ClientView::win_estimate`). This keeps the series, each reading
//! with the game-log line that came just before it, and the game-over
//! screen lists the [`SHOWN`] biggest drops — "Turn 7: 62 % → 35 % after
//! Bot 1 cast Wrath of God" — the turns worth replaying in your head.
//! Nothing is kept where there is no estimate (a pod, no net loaded).

use bevy::prelude::*;

use crate::game::GameLog;
use crate::net_plugin::CurrentView;

/// Drops shown on the game-over screen.
pub const SHOWN: usize = 3;
/// A drop smaller than this (win-chance points) is noise, not a moment.
const MIN_DROP: f32 = 0.08;

/// One reading: the turn, the estimate, and the log line just before it.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub turn: u32,
    pub estimate: f32,
    pub after: String,
}

/// This game's readings, in order. Cleared when a new game starts.
#[derive(Resource, Default, Debug)]
pub struct WinSwings(pub Vec<Reading>);

impl WinSwings {
    /// The biggest falls between consecutive readings, worst first: `(turn,
    /// from, to, what came before it)`.
    pub fn biggest_drops(&self, n: usize) -> Vec<(u32, f32, f32, String)> {
        let mut drops: Vec<(u32, f32, f32, String)> = self
            .0
            .windows(2)
            .filter(|w| w[0].estimate - w[1].estimate >= MIN_DROP)
            .map(|w| (w[1].turn, w[0].estimate, w[1].estimate, w[1].after.clone()))
            .collect();
        drops.sort_by(|a, b| (b.1 - b.2).total_cmp(&(a.1 - a.2)));
        drops.truncate(n);
        drops
    }

    /// The game-over lines: "Turn 7: 62 % → 35 % after Bot 1 cast Wrath".
    pub fn review_lines(&self) -> Vec<String> {
        self.biggest_drops(SHOWN)
            .into_iter()
            .map(|(turn, from, to, after)| {
                let pct = |p: f32| (p * 100.0).round() as u32;
                let after = if after.is_empty() { String::new() } else { format!(" after {after}") };
                format!("Turn {turn}: {} % → {} %{after}", pct(from), pct(to))
            })
            .collect()
    }
}

/// Record each new estimate with the log line before it; start over when a
/// new game begins (the turn counter goes back).
pub fn record_win_swings(view: Res<CurrentView>, log: Res<GameLog>, mut swings: ResMut<WinSwings>) {
    if !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    if swings.0.last().is_some_and(|r| cv.turn < r.turn) {
        swings.0.clear();
    }
    let Some(estimate) = cv.win_estimate else { return };
    if swings.0.last().is_some_and(|r| (r.estimate - estimate).abs() < 0.005) {
        return;
    }
    let after = log.entries.iter().rev().find(|e| e.event && !e.divider).map(|e| e.text.clone()).unwrap_or_default();
    swings.0.push(Reading { turn: cv.turn, estimate, after });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(turn: u32, estimate: f32, after: &str) -> Reading {
        Reading { turn, estimate, after: after.into() }
    }

    #[test]
    fn the_review_names_the_worst_falls() {
        let s = WinSwings(vec![
            r(1, 0.50, ""),
            r(3, 0.62, "You cast Llanowar Elves"),
            r(5, 0.35, "Bot 1 cast Wrath of God"),
            r(6, 0.31, "Bot 1 played Swamp"),
            r(7, 0.15, "Bot 1 attacks with Serra Angel"),
        ]);
        assert_eq!(
            s.review_lines(),
            vec![
                "Turn 5: 62 % → 35 % after Bot 1 cast Wrath of God".to_string(),
                "Turn 7: 31 % → 15 % after Bot 1 attacks with Serra Angel".to_string(),
            ],
            "the 4-point dip is noise"
        );
    }
}
