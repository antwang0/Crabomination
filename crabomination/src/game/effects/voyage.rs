//! Cosima, God of the Voyage's voyage: exiled by its own upkeep ability, the
//! card "gains" a landfall that functions from exile (`TriggerZone::
//! WhileSelfExiled`) and either adds a voyage counter or brings it home.

use super::EffectContext;
use crate::card::CounterType;
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::Voyage` — a voyage counter on the exiled source, or (`home`)
    /// return it with X +1/+1 counters and its owner draws X, X being its
    /// voyage counters. The counters are put on as it lands rather than as
    /// it enters. A source no longer on its voyage (CR 400.7) does nothing.
    pub(super) fn voyage(
        &mut self,
        home: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(src) = ctx.source else { return Ok(()) };
        let locked = self.counters_locked();
        let Some(card) = self.exile.iter_mut().find(|c| c.id == src && c.exiled_with == Some(src)) else {
            return Ok(());
        };
        if !home {
            if !locked {
                card.add_counters(CounterType::Voyage, 1);
                events.push(GameEvent::CounterAdded {
                    card_id: src,
                    counter_type: CounterType::Voyage,
                    count: 1,
                    placer: self.resolution_causer,
                });
            }
            return Ok(());
        }
        let x = card.counter_count(CounterType::Voyage);
        let owner = card.owner;
        // The voyage is over: a later exile by anything else must not find it.
        card.exiled_with = None;
        let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(owner), tapped: false };
        self.move_card_to(src, &dest, ctx, events);
        if x == 0 {
            return Ok(());
        }
        if self.battlefield_find(src).is_some() {
            let grow = Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(x as i32) };
            self.run_effect(&grow, ctx, events)?;
        }
        let draw = Effect::Draw { who: Selector::Player(PlayerRef::Seat(owner)), amount: Value::Const(x as i32) };
        self.run_effect(&draw, ctx, events)
    }
}
