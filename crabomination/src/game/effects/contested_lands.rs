//! Turf War's second ability (its entry is `ForEachPlayerTarget`): "whenever a creature deals combat damage to a
//! player, if that player controls one or more lands with contested counters
//! on them, that creature's controller gains control of one of those lands of
//! their choice and untaps it."

use crate::card::{CardId, CounterType};
use crate::decision::PickValue;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::TakeContestedLand` — the damaged player is the trigger's
    /// event player; the taker is the damaging creature's controller, who
    /// picks the land (headless: nonbasic first, then the highest mana value).
    pub(super) fn take_contested_land(
        &mut self,
        effect: &Effect,
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
        let mut lands: Vec<(bool, u32, CardId, String)> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == damaged && c.definition.is_land() && c.counter_count(CounterType::Contested) > 0
            })
            .map(|c| {
                (
                    c.definition.supertypes.contains(&crate::card::Supertype::Basic),
                    u32::MAX - c.definition.cost.cmc(),
                    c.id,
                    c.definition.name.to_string(),
                )
            })
            .collect();
        if lands.is_empty() {
            return Ok(());
        }
        lands.sort();
        let auto = vec![lands[0].2];
        let candidates: Vec<(CardId, String)> = lands.into_iter().map(|(_, _, id, n)| (id, n)).collect();
        let mut cursor = 0;
        let Some(picked) = self.ask_seat_cards_logged(
            &mut cursor,
            taker,
            "Gain control of which contested land?".into(),
            ctx.source.unwrap_or(CardId(0)),
            candidates,
            1,
            1,
            PickValue::Gain,
            effect,
            auto.clone(),
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let land = picked.first().copied().unwrap_or(auto[0]);
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
