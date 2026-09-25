//! Turf War: "for each player, put a contested counter on target land that
//! player controls", and "whenever a creature deals combat damage to a
//! player, if that player controls one or more lands with contested counters
//! on them, that creature's controller gains control of one of those lands of
//! their choice and untaps it."

use crate::card::{CardId, CounterType};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::{GameError, GameState};

impl GameState {
    /// A land `seat` controls for Turf War's counter: an opponent's most
    /// valuable one (nonbasic, then highest mana value), your own least.
    fn contest_pick(&self, seat: usize, controller: usize) -> Option<CardId> {
        let lands = self.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land());
        let key = |c: &&crate::card::CardInstance| (!c.definition.supertypes.contains(&crate::card::Supertype::Basic), c.definition.cost.cmc());
        if self.same_team(seat, controller) { lands.min_by_key(key) } else { lands.max_by_key(key) }.map(|c| c.id)
    }

    /// `Effect::ContestOneLandPerPlayer`.
    pub(super) fn contest_one_land_per_player(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let n = self.players.len();
        for i in 0..n {
            let seat = (ctx.controller + i) % n;
            if !self.players[seat].is_alive() {
                continue;
            }
            if let Some(id) = self.contest_pick(seat, ctx.controller) {
                let c = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
                self.run_effect(
                    &Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Contested,
                        amount: crate::effect::Value::ONE,
                    },
                    &c,
                    events,
                )?;
            }
        }
        Ok(())
    }

    /// `Effect::TakeContestedLand` — the damaged player is the trigger's
    /// event player; the taker is the damaging creature's controller.
    pub(super) fn take_contested_land(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(damaged) = self.resolve_player(&PlayerRef::TriggerEventPlayer, ctx) else { return Ok(()) };
        let Some(taker) = ctx
            .trigger_source
            .and_then(|s| match s {
                crate::game::effects::EntityRef::Permanent(id) | crate::game::effects::EntityRef::Card(id) => Some(id),
                _ => None,
            })
            .and_then(|id| self.battlefield_find(id))
            .map(|c| c.controller)
        else {
            return Ok(());
        };
        if taker == damaged {
            return Ok(());
        }
        let Some(land) = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == damaged && c.definition.is_land() && c.counter_count(CounterType::Contested) > 0
            })
            .max_by_key(|c| (!c.definition.supertypes.contains(&crate::card::Supertype::Basic), c.definition.cost.cmc()))
            .map(|c| c.id)
        else {
            return Ok(());
        };
        let c = EffectContext { targets: vec![Target::Permanent(land)], ..ctx.clone() };
        self.run_effect(
            &Effect::Seq(vec![
                Effect::GainControl {
                    what: Selector::Target(0),
                    to: Some(PlayerRef::Seat(taker)),
                    duration: Duration::Permanent,
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
            ]),
            &c,
            events,
        )
    }
}
