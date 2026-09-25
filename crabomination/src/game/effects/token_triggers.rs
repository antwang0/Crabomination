//! "Create tokens that are copies of X, except they have 'When this token
//! enters, [body]'" (Aggressive Biomancy). The copy's added ability has
//! already missed its own enters event by the time a grant could reach it,
//! so the resolving spell puts one trigger per new token on the stack with
//! that token as its source — each waits for priority and picks its own
//! targets as it triggers (CR 603.3d, 603.6a).

use super::EffectContext;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};
use crate::game::effects::EntityRef;

impl GameState {
    /// `Effect::EachPushesTrigger { what, body }` — one `body` trigger per
    /// permanent `what` resolves to, sourced from that permanent.
    pub(super) fn each_pushes_trigger(
        &mut self,
        what: &Selector,
        body: &Effect,
        ctx: &EffectContext,
        _events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ids: Vec<_> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .collect();
        for id in ids {
            let Some(controller) = self.battlefield_find(id).map(|c| c.controller) else { continue };
            let (slot0, additional) = self.auto_targets_for_effect_all_slots(body, controller, None);
            self.stack.push(
                crate::game::TriggerPush::new(id, controller, body.clone())
                    .target(slot0)
                    .additional_targets(additional)
                    .trigger_source(Some(EntityRef::Permanent(id)))
                    .build(),
            );
        }
        Ok(())
    }
}
