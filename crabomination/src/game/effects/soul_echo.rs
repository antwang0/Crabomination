//! Soul Echo's upkeep choice and replacement. `Effect::PlayerMayStampSourceTurn`
//! — "[that opponent] may choose that …" — stamps the source with this turn
//! (`chosen_number`), or clears the stamp; `StaticEffect::
//! ReplaceDamageToYouRemovingStampedCounters` replaces all damage to the
//! controller with removing that many `kind` counters while the stamp holds,
//! "until your next upkeep" (CR 614.1a): the stamped turn, and every later
//! turn until the controller's own turn begins.

use crate::card::{CardId, StaticEffect};
use crate::effect::{Effect, Selector};
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

impl GameState {
    pub(super) fn player_may_stamp_source_turn(
        &mut self,
        who: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let _ = events;
        let Some(source) = ctx.source else { return Ok(()) };
        let seat = self.resolve_selector(who, ctx).into_iter().find_map(|e| match e {
            crate::game::effects::EntityRef::Player(p) => Some(p),
            _ => None,
        });
        let mut cursor = 0usize;
        let yes = match seat {
            Some(seat) => {
                let Some(b) = self.ask_seat_bool(
                    &mut cursor,
                    seat,
                    "Have damage to its controller remove its counters until their next upkeep?".into(),
                    source,
                    effect,
                    crate::decision::OptionalKind::MayBody,
                ) else {
                    return Ok(());
                };
                b
            }
            None => false,
        };
        self.clear_answer_log();
        let turn = self.turn_number;
        if let Some(c) = self.battlefield_find_mut(source) {
            c.chosen_number = yes.then_some(turn);
        }
        Ok(())
    }

    /// The Soul Echo `seat` controls whose stamp is live, and its counter kind.
    pub(crate) fn stamped_damage_replacement(&self, seat: usize) -> Option<(CardId, crate::card::CounterType)> {
        self.battlefield.iter().find_map(|c| {
            let stamped = c.chosen_number?;
            let live = self.turn_number == stamped || (self.turn_number > stamped && self.active_player_idx != seat);
            if c.controller != seat || !live {
                return None;
            }
            c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                StaticEffect::ReplaceDamageToYouRemovingStampedCounters { kind } => Some((c.id, kind)),
                _ => None,
            })
        })
    }
}
