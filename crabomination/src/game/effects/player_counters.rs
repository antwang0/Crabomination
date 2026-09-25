//! Aetheric Amplifier — "double the number of each kind of counter you have".

use super::EffectContext;
use crate::effect::PlayerRef;
use crate::game::GameState;
use crate::game::types::GameEvent;

impl GameState {
    /// `Effect::DoublePlayerCounters`: energy (CR 107.16), experience and
    /// poison (CR 122.1f) each double. Energy gained this way is an energy
    /// gain for "whenever you get {E}" watchers.
    pub(super) fn double_player_counters(&mut self, who: &PlayerRef, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let Some(p) = self.resolve_player(who, ctx) else { return };
        let energy = self.players[p].energy;
        if energy > 0 {
            self.players[p].energy = energy.saturating_mul(2);
            events.push(GameEvent::EnergyGained { player: p, amount: energy });
        }
        let experience = self.players[p].experience;
        if experience > 0 {
            self.players[p].experience = experience.saturating_mul(2);
        }
        let poison = self.players[p].poison_counters;
        if poison > 0 {
            self.players[p].poison_counters = poison.saturating_mul(2);
        }
    }
}
