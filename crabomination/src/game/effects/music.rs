//! Musician — "{T}: Put a music counter on target creature. If it doesn't
//! have 'At the beginning of your upkeep, destroy this creature unless you
//! pay {1} for each music counter on it,' it gains that ability."
//! (`Effect::AddMusicCounterGrantUpkeepToll`.) The grant is permanent while
//! the creature stays (baked, CR 400.7) and never stacks: a second counter
//! raises the toll of the one ability.

use super::EffectContext;
use crate::card::{CounterType, EventKind, EventScope, EventSpec, TriggeredAbility};
use crate::effect::{Effect, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, TurnStep};

/// The granted upkeep toll.
pub(crate) fn music_toll() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect: Effect::DestroySourceUnlessPayValue {
            generic: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Music },
        },
    }
}

impl GameState {
    pub(super) fn add_music_counter_grant_upkeep_toll(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ids: Vec<_> = self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_permanent_id()).collect();
        let toll = music_toll();
        for cid in ids {
            self.run_effect(
                &Effect::AddCounter { what: Selector::ExactObjects(vec![cid]), kind: CounterType::Music, amount: Value::ONE },
                ctx,
                events,
            )?;
            if let Some(c) = self.battlefield_find_mut(cid)
                && !c.definition.triggered_abilities.contains(&toll)
            {
                c.bake_grant().triggered_abilities.push(toll.clone());
            }
        }
        Ok(())
    }
}
