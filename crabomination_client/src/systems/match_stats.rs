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
    pub life_gained: HashMap<usize, u32>,
    pub lands_played: HashMap<usize, u32>,
    pub discarded: HashMap<usize, u32>,
    pub milled: HashMap<usize, u32>,
    pub sacrificed: HashMap<usize, u32>,
    pub poison: HashMap<usize, u32>,
}

impl MatchStats {
    /// One compact line per seat, e.g.
    /// "Alice — 12 drawn · 9 spells · 6 lands · 15 dmg taken · 4 life gained".
    /// The five core counters always render; situational ones (discards,
    /// mills, sacrifices, poison) are appended only when nonzero so the
    /// common case stays one readable line.
    pub fn seat_line(&self, seat: usize, label: &str) -> String {
        let get = |m: &HashMap<usize, u32>| m.get(&seat).copied().unwrap_or(0);
        let mut line = format!(
            "{label} — {} drawn · {} spells · {} lands · {} dmg taken · {} life gained",
            get(&self.drawn),
            get(&self.spells),
            get(&self.lands_played),
            get(&self.damage_taken),
            get(&self.life_gained),
        );
        for (map, noun) in [
            (&self.discarded, "discarded"),
            (&self.milled, "milled"),
            (&self.sacrificed, "sacrificed"),
            (&self.poison, "poison"),
        ] {
            let n = get(map);
            if n > 0 {
                line.push_str(&format!(" · {n} {noun}"));
            }
        }
        line
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
            GameEventWire::LifeGained { player, amount } => {
                *stats.life_gained.entry(*player).or_default() += amount;
            }
            GameEventWire::LandPlayed { player, .. } => {
                *stats.lands_played.entry(*player).or_default() += 1;
            }
            GameEventWire::CardDiscarded { player, .. } => {
                *stats.discarded.entry(*player).or_default() += 1;
            }
            GameEventWire::CardMilled { player, .. } => {
                *stats.milled.entry(*player).or_default() += 1;
            }
            GameEventWire::CreatureSacrificed { who, .. }
            | GameEventWire::PermanentSacrificed { who, .. } => {
                *stats.sacrificed.entry(*who).or_default() += 1;
            }
            GameEventWire::PoisonAdded { player, amount } => {
                *stats.poison.entry(*player).or_default() += amount;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seat_line_folds_life_and_lands() {
        let mut s = MatchStats::default();
        *s.drawn.entry(0).or_default() += 3;
        *s.spells.entry(0).or_default() += 2;
        *s.lands_played.entry(0).or_default() += 4;
        *s.damage_taken.entry(0).or_default() += 5;
        *s.life_gained.entry(0).or_default() += 6;
        assert_eq!(
            s.seat_line(0, "P0"),
            "P0 — 3 drawn · 2 spells · 4 lands · 5 dmg taken · 6 life gained",
        );
    }

    #[test]
    fn situational_counters_append_only_when_nonzero() {
        let mut s = MatchStats::default();
        *s.discarded.entry(1).or_default() += 2;
        *s.poison.entry(1).or_default() += 3;
        assert_eq!(
            s.seat_line(1, "P1"),
            "P1 — 0 drawn · 0 spells · 0 lands · 0 dmg taken · 0 life gained · 2 discarded · 3 poison",
        );
    }
}
