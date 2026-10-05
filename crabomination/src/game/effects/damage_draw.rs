//! "Each player draws cards equal to the amount of damage dealt to [this]
//! this turn by sources they controlled." — Grothama, All-Devouring's
//! leaves-the-battlefield trigger.

use super::EffectContext;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// Reads the source's per-source damage tally
    /// (`CardData::damage_by_source_this_turn`, cleared only at end of turn,
    /// so it survives the zone change that fired the trigger). A damaging
    /// source is credited to its controller now, or its owner once it has
    /// left the battlefield — the last-known controller isn't recorded.
    pub(super) fn each_player_draws_damage_they_dealt_to_source(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(src) = ctx.source else { return Ok(()) };
        let Some(tally) = self.find_card_anywhere(src).map(|c| c.damage_by_source_this_turn.clone()) else {
            return Ok(());
        };
        let mut per_seat = vec![0u32; self.players.len()];
        for (dealer, n) in tally.iter() {
            let seat = match self.battlefield_find(*dealer) {
                Some(c) => Some(c.controller),
                None => self.find_card_anywhere(*dealer).map(|c| c.owner),
            };
            if let Some(s) = seat.filter(|s| *s < per_seat.len()) {
                per_seat[s] += n;
            }
        }
        // CR 101.4 — "each player" acts in APNAP order.
        for seat in self.apnap_sort((0..per_seat.len()).collect()) {
            let n = per_seat[seat];
            if n > 0 && !self.players[seat].eliminated {
                self.run_effect(
                    &crate::effect::Effect::Draw {
                        who: crate::effect::Selector::Player(crate::effect::PlayerRef::Seat(seat)),
                        amount: crate::effect::Value::Const(n as i32),
                    },
                    ctx,
                    events,
                )?;
            }
        }
        Ok(())
    }
}
