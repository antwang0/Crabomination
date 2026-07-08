//! Per-match stat tracking from the server event stream — turns played and
//! per-seat cards drawn / spells cast / damage taken — surfaced as a summary
//! block on the game-over modal.

use std::collections::HashMap;

use bevy::prelude::*;

use crabomination::net::GameEventWire;

use crate::net_plugin::LatestServerEvents;

#[derive(Resource, Default)]
pub struct MatchStats {
    pub turns: u32,
    pub drawn: HashMap<usize, u32>,
    pub spells: HashMap<usize, u32>,
    pub damage_taken: HashMap<usize, u32>,
}

impl MatchStats {
    /// One compact line per seat: "Alice — 12 drawn · 9 spells · 15 dmg taken".
    pub fn seat_line(&self, seat: usize, label: &str) -> String {
        format!(
            "{label} — {} drawn · {} spells · {} dmg taken",
            self.drawn.get(&seat).copied().unwrap_or(0),
            self.spells.get(&seat).copied().unwrap_or(0),
            self.damage_taken.get(&seat).copied().unwrap_or(0),
        )
    }
}

/// Fold the latest event batch into the running stats. `LatestServerEvents`
/// holds each batch for exactly one tick (cleared by `poll_net`), so a
/// per-frame reader never double-counts. A `TurnStarted { turn: 1 }` after a
/// running match resets the counters for the rematch.
pub fn track_match_stats(events: Res<LatestServerEvents>, mut stats: ResMut<MatchStats>) {
    for ev in &events.0 {
        match ev {
            GameEventWire::TurnStarted { turn, .. } => {
                if *turn <= 1 && stats.turns > 1 {
                    *stats = MatchStats::default();
                }
                stats.turns = stats.turns.max(*turn);
            }
            GameEventWire::CardDrawn { player, .. } => {
                *stats.drawn.entry(*player).or_default() += 1;
            }
            GameEventWire::SpellCast { player, .. } => {
                *stats.spells.entry(*player).or_default() += 1;
            }
            GameEventWire::DamageDealt { amount, to_player: Some(p), .. } => {
                *stats.damage_taken.entry(*p).or_default() += amount;
            }
            _ => {}
        }
    }
}
