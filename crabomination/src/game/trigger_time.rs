//! Values a triggered ability's target filter reads as it is put on the stack
//! (CR 603.3d — targets are chosen then), re-read the same way as it resolves
//! (CR 608.2b): the X a `WithX` body names, and the object it fired on.

use super::GameState;
use super::effects::{EffectContext, EntityRef};
use super::types::PendingTriggerPush;
use crate::card::{CardId, SelectionRequirement};
use crate::effect::Effect;

impl GameState {
    /// Stamp `ResolutionScratch::trigger_subject` for a filter that reads it
    /// (`OtherThanTriggerSubject`); a guarded write, so every other trigger
    /// leaves the CoW group shared.
    pub(crate) fn stamp_trigger_subject(&mut self, filter: Option<&SelectionRequirement>, subject: Option<EntityRef>) {
        if !filter.is_some_and(SelectionRequirement::mentions_trigger_subject) {
            return;
        }
        let id: Option<CardId> = match subject {
            Some(EntityRef::Card(c) | EntityRef::Permanent(c)) => Some(c),
            _ => None,
        };
        if self.scratch.trigger_subject != id {
            self.scratch.trigger_subject = id;
        }
    }

    /// A trigger whose body is `WithX { x, .. }` (Puca's Covenant's "mana value
    /// less than or equal to the number of counters on that creature") reads
    /// its X as it triggers, so an X-relative target filter is concretized at
    /// the X the body will use. Stored as the pending trigger's `x_value`.
    pub(crate) fn settle_trigger_time_x(&self, pending: &mut PendingTriggerPush) {
        let Effect::WithX { x, .. } = &pending.effect else { return };
        if pending.x_value != 0 {
            return;
        }
        let mut ctx = EffectContext::for_trigger(pending.source, pending.controller, None, 0);
        ctx.trigger_source = pending.subject;
        ctx.event_amount = pending.event_amount;
        pending.x_value = self.evaluate_value(x, &ctx).max(0) as u32;
    }
}
