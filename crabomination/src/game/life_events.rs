//! A life change through the CR 119 funnel (`adjust_life_applied`: Tainted
//! Remedy, the doublers and bonuses, can't-gain / can't-lose, the 2HG pool,
//! the per-turn tallies) with the event its APPLIED sign earns. The funnel
//! emits nothing, and a site that pushed `LifeGained` only on `applied > 0`
//! dropped the `LifeLost` a Tainted Remedy conversion makes (CR 119.9: the
//! player loses that life, and "whenever a player loses life" sees it).

use super::GameState;
use super::types::GameEvent;

impl GameState {
    /// `delta` life for `seat` through the funnel; pushes `LifeGained` or
    /// `LifeLost` for what actually happened and returns the applied delta.
    pub(crate) fn adjust_life_emit(&mut self, seat: usize, delta: i32, events: &mut Vec<GameEvent>) -> i32 {
        let before = self.effective_life(seat);
        let applied = self.adjust_life_applied(seat, delta);
        if applied > 0 {
            events.push(GameEvent::LifeGained { player: seat, amount: applied as u32 });
            // False Cure — the funnel docks N per point gained right after the
            // gain (`adjust_life` reports the total before it); that loss is
            // its own "loses life" event.
            let docked = before + applied - self.effective_life(seat);
            if docked > 0 {
                events.push(GameEvent::LifeLost { player: seat, amount: docked as u32 });
            }
        } else if applied < 0 {
            events.push(GameEvent::LifeLost { player: seat, amount: applied.unsigned_abs() });
        }
        applied
    }
}
